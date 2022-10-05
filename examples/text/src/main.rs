use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::{
    cmp,
    env,
    fs,
    time::Instant,
};

struct Font<'a> {
    data: &'a [u8],
    pub rustybuzz: rustybuzz::Face<'a>,
    pub rusttype: rusttype::Font<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8], index: u32) -> Option<Self> {
        Some(Self {
            data,
            rustybuzz: rustybuzz::Face::from_slice(data, index)?,
            rusttype: rusttype::Font::try_from_bytes_and_index(data, index)?,
        })
    }
}

struct FontLayoutGlyph<'a> {
    start: usize,
    end: usize,
    x: f32,
    w: f32,
    inner: rusttype::PositionedGlyph<'a>,
}

struct FontLayoutLine<'a> {
    glyphs: Vec<FontLayoutGlyph<'a>>,
}

impl<'a> FontLayoutLine<'a> {
    pub fn draw<R: Renderer>(
        &self,
        r: &mut R,
        line_x: i32,
        line_y: i32,
        color: Color,
    ) {
        for glyph in self.glyphs.iter() {
            if let Some(bb) = glyph.inner.pixel_bounding_box() {
                let x = line_x + bb.min.x;
                let y = line_y + bb.min.y;
                glyph.inner.draw(|off_x, off_y, v| {
                    let c = (v * 255.0) as u32;
                    r.pixel(x + off_x as i32, y + off_y as i32, Color{
                        data: c << 24 | (color.data & 0x00FF_FFFF)
                    });
                });
            }
        }
    }
}

struct FontShapeGlyph<'a> {
    start: usize,
    end: usize,
    x_advance: f32,
    y_advance: f32,
    x_offset: f32,
    y_offset: f32,
    inner: rusttype::Glyph<'a>,
}

struct FontShapeSpan<'a> {
    direction: rustybuzz::Direction,
    glyphs: Vec<FontShapeGlyph<'a>>,
}

struct FontShapeLine<'a> {
    rtl: bool,
    spans: Vec<FontShapeSpan<'a>>,
}

impl<'a> FontShapeLine<'a> {
    pub fn layout(&self, font_size: i32, line_width: i32, lines: &mut Vec<FontLayoutLine<'a>>) {
        let mut push_line = true;
        let mut glyphs = Vec::new();

        let mut x = 0.0;
        let mut y = 0.0;
        for span in self.spans.iter() {
            let mut span_width = 0.0;
            for glyph in span.glyphs.iter() {
                let x_advance = font_size as f32 * glyph.x_advance;
                span_width += x_advance;
            }

            if self.rtl {
                if glyphs.is_empty() {
                    x = line_width as f32;
                }
                x -= span_width;
            }

            for glyph in span.glyphs.iter() {
                let x_advance = font_size as f32 * glyph.x_advance;
                let y_advance = font_size as f32 * glyph.y_advance;
                let x_offset = font_size as f32 * glyph.x_offset;
                let y_offset = font_size as f32 * glyph.y_offset;

                //TODO: make wrapping optional
                if self.rtl {
                    if x < 0.0 {
                        let mut glyphs_swap = Vec::new();
                        std::mem::swap(&mut glyphs, &mut glyphs_swap);
                        lines.push(FontLayoutLine { glyphs: glyphs_swap });

                        x = line_width as f32 - x_advance;
                        y = 0.0;
                    }
                } else {
                    if x + x_advance > line_width as f32 {
                        let mut glyphs_swap = Vec::new();
                        std::mem::swap(&mut glyphs, &mut glyphs_swap);
                        lines.push(FontLayoutLine { glyphs: glyphs_swap });
                        push_line = false;

                        x = 0.0;
                        y = 0.0;
                    }
                }

                let inner = glyph.inner.clone()
                    .scaled(rusttype::Scale::uniform(font_size as f32))
                    .positioned(rusttype::point(
                        x + x_offset,
                        y + y_offset,
                    ));

                glyphs.push(FontLayoutGlyph {
                    start: glyph.start,
                    end: glyph.end,
                    x,
                    w: x_advance,
                    inner,
                });
                push_line = true;

                x += x_advance;
                y += y_advance;
            }

            if self.rtl {
                x -= span_width;
            }
        }

        if push_line {
            lines.push(FontLayoutLine { glyphs });
        }
    }
}

struct FontMatches<'a> {
    fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    fn shape_span(&self, string: &str, start_span: usize, end_span: usize) -> FontShapeSpan {
        let span = &string[start_span..end_span];

        let mut spans_by_font = Vec::with_capacity(self.fonts.len());
        for (font_i, font) in self.fonts.iter().enumerate() {
            let font_scale = font.rustybuzz.units_per_em() as f32;

            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(span);
            buffer.guess_segment_properties();
            let script = buffer.script();
            let direction = buffer.direction();
            if font_i == 0 {
                println!("{:?}, {:?}: '{}'", script, direction, span);
            }

            let glyph_buffer = rustybuzz::shape(&font.rustybuzz, &[], buffer);
            let glyph_infos = glyph_buffer.glyph_infos();
            let glyph_positions = glyph_buffer.glyph_positions();

            let mut misses = 0;
            let mut glyphs = Vec::with_capacity(glyph_infos.len());
            for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
                let x_advance = pos.x_advance as f32 / font_scale;
                let y_advance = pos.y_advance as f32 / font_scale;
                let x_offset = pos.x_offset as f32 / font_scale;
                let y_offset = pos.y_offset as f32 / font_scale;

                //println!("  {:?} {:?}", info, pos);
                if info.glyph_id == 0 {
                    misses += 1;
                }

                let inner = font.rusttype.glyph(rusttype::GlyphId(info.glyph_id as u16));
                glyphs.push(FontShapeGlyph {
                    start: start_span + info.cluster as usize,
                    end: end_span, // Set later
                    x_advance,
                    y_advance,
                    x_offset,
                    y_offset,
                    inner,
                });
            }

            // Adjust end of glyphs
            match direction {
                rustybuzz::Direction::LeftToRight => {
                    for i in (1..glyphs.len()).rev() {
                        let next_start = glyphs[i].start;
                        let next_end = glyphs[i].end;
                        let prev = &mut glyphs[i - 1];
                        if prev.start == next_start {
                            prev.end = next_end;
                        } else {
                            prev.end = next_start;
                        }
                    }
                },
                rustybuzz::Direction::RightToLeft => {
                    for i in 1..glyphs.len() {
                        let next_start = glyphs[i - 1].start;
                        let next_end = glyphs[i - 1].end;
                        let prev = &mut glyphs[i];
                        if prev.start == next_start {
                            prev.end = next_end;
                        } else {
                            prev.end = next_start;
                        }
                    }
                },
                //TODO: other directions
                _ => (),
            }

            let span = FontShapeSpan {
                direction,
                glyphs
            };
            if misses == 0 {
                return span;
            } else {
                spans_by_font.push((misses, span));
            }
        }

        let mut least_misses_i = 0;
        let mut least_misses = usize::MAX;
        for (i, (misses, _)) in spans_by_font.iter().enumerate() {
            if *misses < least_misses {
                least_misses_i = i;
                least_misses = *misses;
            }
        }

        if least_misses_i > 0 {
            println!("MISSES {}, {}", least_misses_i, least_misses);
        }

        spans_by_font.remove(least_misses_i).1
    }

    fn shape_line(&self, string: &str, start_line: usize, end_line: usize) -> FontShapeLine {
        use unicode_script::{Script, UnicodeScript};

        let line = &string[start_line..end_line];

        //TODO: more special handling of characters
        let mut spans = Vec::new();

        let mut start = 0;
        let mut prev = Script::Unknown;
        for (i, c) in line.char_indices() {
            if ! line.is_char_boundary(i) {
                continue;
            }

            let cur = c.script();
            if prev != cur && prev != Script::Unknown {
                // No combination, start new span
                spans.push(self.shape_span(string, start_line + start, start_line + i));
                start = i;
                prev = Script::Unknown;
            } else {
                prev = cur;
            }
        }

        spans.push(self.shape_span(string, start_line + start, start_line + line.len()));

        let bidi = unicode_bidi::BidiInfo::new(line, None);
        let rtl = if bidi.paragraphs.is_empty() {
            false
        } else {
            assert_eq!(bidi.paragraphs.len(), 1);
            bidi.paragraphs[0].level.is_rtl()
        };

        FontShapeLine {
            rtl,
            spans,
        }
    }

    pub fn shape(&self, string: &str) -> Vec<FontShapeLine> {
        let mut lines = Vec::new();

        let mut start = 0;
        for (i, c) in string.char_indices() {
            if ! string.is_char_boundary(i) {
                continue;
            }

            if c == '\n' {
                lines.push(self.shape_line(string, start, i));
                start = i + 1;
            }
        }

        lines.push(self.shape_line(string, start, string.len()));

        lines
    }
}

struct FontSystem<'a> {
    fonts: Vec<Font<'a>>,
}

impl<'a> FontSystem<'a> {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
        }
    }

    pub fn add(&mut self, font: Font<'a>) {
        self.fonts.push(font);
    }

    pub fn matches(&'a self, patterns: &[&str]) -> Option<FontMatches<'a>> {
        let mut fonts = Vec::new();
        for font in self.fonts.iter() {
            for rec in font.rustybuzz.names() {
                if rec.name_id == 4 && rec.is_unicode() {
                    let mut words: Vec<u16> = Vec::new();

                    let mut i = 0;
                    while i + 1 < rec.name.len() {
                        words.push(
                            (rec.name[i + 1] as u16) |
                            ((rec.name[i] as u16) << 8)
                        );
                        i += 2;
                    }

                    match String::from_utf16(&words) {
                        Ok(name) => {
                            let mut matched = false;
                            for pattern in patterns.iter() {
                                println!("Matching font name '{}' with pattern '{}'", name, pattern);
                                if name.contains(pattern) {
                                    matched = true;
                                }
                            }
                            if matched {
                                println!("Matched font name '{}'", name);
                                fonts.push(font);
                            } else {
                                println!("Did not match font name '{}'", name);
                            }
                        },
                        Err(_) => ()
                    }
                }
            }
        }
        if ! fonts.is_empty() {
            Some(FontMatches { fonts })
        } else {
            None
        }
    }
}

fn main() {
    let display_scale = match orbclient::get_display_size() {
        Ok((w, h)) => {
            println!("Display size: {}, {}", w, h);
            (h as i32 / 1600) + 1
        },
        Err(err) => {
            println!("Failed to get display size: {}", err);
            1
        }
    };

    let mut window = Window::new_flags(
        -1,
        -1,
        1024 * display_scale as u32,
        768 * display_scale as u32,
        "COSMIC TEXT",
        &[WindowFlag::Resizable]
    ).unwrap();

    #[cfg(feature = "mono")]
    let default_text = include_str!("../res/mono.txt");
    #[cfg(not(feature = "mono"))]
    let default_text = include_str!("../res/proportional.txt");

    let mut text = if let Some(arg) = env::args().nth(1) {
        fs::read_to_string(&arg).expect("failed to open file")
    } else {
        default_text.to_string()
    };

    let bg_color = Color::rgb(0x34, 0x34, 0x34);
    let font_color = Color::rgb(0xFF, 0xFF, 0xFF);
    let font_sizes = [
        (10, 14), // Caption
        (14, 20), // Body
        (20, 28), // Title 4
        (24, 32), // Title 3
        (28, 36), // Title 2
        (32, 44), // Title 1
    ];
    let font_size_default = 2; // Title 4
    let mut font_size_i = font_size_default;

    let mut font_system = FontSystem::new();
    font_system.add(
        Font::new(include_bytes!("../../../res/Fira/FiraSans-Regular.otf"), 0).unwrap()
    );
    font_system.add(
        Font::new(include_bytes!("../../../res/Fira/FiraMono-Regular.otf"), 0).unwrap()
    );
    font_system.add(
        Font::new(include_bytes!("../../../res/FreeFont/FreeSans.ttf"), 0).unwrap()
    );
    font_system.add(
        Font::new(include_bytes!("../../../res/FreeFont/FreeSerif.ttf"), 0).unwrap()
    );
    font_system.add(
        Font::new(include_bytes!("../../../res/FreeFont/FreeMono.ttf"), 0).unwrap()
    );

    #[cfg(feature = "mono")]
    let font_pattern = &["Mono"];
    #[cfg(not(feature = "mono"))]
    let font_pattern = &["Sans", "Serif"];

    let font_matches = font_system.matches(font_pattern).unwrap();

    let mut shape_lines = Vec::new();
    let mut layout_lines = Vec::new();
    let mut cursor_line = 0;
    let mut cursor_glyph = 0;
    let mut mouse_x = -1;
    let mut mouse_y = -1;
    let mut mouse_left = false;
    let mut redraw = true;
    let mut relayout = true;
    let mut reshape = true;
    let mut scroll = 0;
    loop {
        let (mut font_size, mut line_height) = font_sizes[font_size_i];
        font_size *= display_scale;
        line_height *= display_scale;

        if reshape {
            let instant = Instant::now();

            shape_lines = font_matches.shape(&text);

            reshape = false;
            relayout = true;

            let duration = instant.elapsed();
            println!("reshape: {:?}", duration);
        }

        if relayout {
            let instant = Instant::now();

            layout_lines.clear();
            for line in shape_lines.iter() {
                let line_width = window.width() as i32 - 16 * display_scale;
                line.layout(font_size, line_width, &mut layout_lines);
            }

            relayout = false;
            redraw = true;

            let duration = instant.elapsed();
            println!("relayout: {:?}", duration);
        }

        if redraw {
            let instant = Instant::now();

            window.set(bg_color);

            let window_lines = (window.height() as i32 + line_height - 1) / line_height;
            scroll = cmp::max(0, cmp::min(
                layout_lines.len() as i32 - (window_lines - 1),
                scroll
            ));

            for &hitbox in &[true, false] {
                let line_x = 8 * display_scale;
                let mut line_y = line_height;
                for (line_i, line) in layout_lines.iter().skip(scroll as usize).enumerate() {
                    if line_y >= window.height() as i32 {
                        break;
                    }

                    if hitbox {
                        if mouse_left
                        && mouse_y >= line_y - font_size
                        && mouse_y < line_y - font_size + line_height
                        {
                            cursor_line = line_i + scroll as usize;
                            cursor_glyph = line.glyphs.len();
                            for (glyph_i, glyph) in line.glyphs.iter().enumerate() {
                                if mouse_x >= line_x + glyph.x as i32
                                && mouse_x <= line_x + (glyph.x + glyph.w) as i32
                                {
                                    cursor_glyph = glyph_i;
                                }
                            }
                        }
                    } else {
                        if cursor_line == line_i + scroll as usize {
                            if cursor_glyph >= line.glyphs.len() {
                                let x = match line.glyphs.last() {
                                    Some(glyph) => glyph.x + glyph.w,
                                    None => 0.0
                                };
                                window.rect(
                                    line_x + x as i32,
                                    line_y - font_size,
                                    (font_size / 2) as u32,
                                    line_height as u32,
                                    Color::rgba(0xFF, 0xFF, 0xFF, 0x20)
                                );
                            } else {
                                let glyph = &line.glyphs[cursor_glyph];
                                window.rect(
                                    line_x + glyph.x as i32,
                                    line_y - font_size,
                                    glyph.w as u32,
                                    line_height as u32,
                                    Color::rgba(0xFF, 0xFF, 0xFF, 0x20)
                                );
                                println!("{}, {}: '{}'", glyph.start, glyph.end, &text[glyph.start..glyph.end]);
                            }
                        }

                        line.draw(
                            &mut window,
                            line_x,
                            line_y,
                            font_color
                        );
                    }

                    line_y += line_height;
                }
            }

            window.sync();

            redraw = false;

            let duration = instant.elapsed();
            println!("redraw: {:?}", duration);
        }

        for event in window.events() {
            match event.to_option() {
                EventOption::Key(event) => if event.pressed {
                    match event.scancode {
                        orbclient::K_UP => if cursor_line > 0 {
                            cursor_line -= 1;
                            redraw = true;
                        },
                        orbclient::K_DOWN => if cursor_line + 1 < layout_lines.len() {
                            cursor_line += 1;
                            redraw = true;
                        },
                        orbclient::K_LEFT => {
                            let line = &layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                redraw = true;
                            }
                            if cursor_glyph > 0 {
                                cursor_glyph -= 1;
                                redraw = true;
                            }
                        },
                        orbclient::K_RIGHT => {
                            let line = &layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                redraw = true;
                            }
                            if cursor_glyph < line.glyphs.len() {
                                cursor_glyph += 1;
                                redraw = true;
                            }
                        },
                        orbclient::K_BKSP => {
                            let line = &layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                redraw = true;
                            }
                            if cursor_glyph > 0 {
                                cursor_glyph -= 1;
                                text.remove(line.glyphs[cursor_glyph].start);
                                reshape = true;
                            }
                        },
                        orbclient::K_DEL => {
                            let line = &layout_lines[cursor_line];
                            if cursor_glyph < line.glyphs.len() {
                                text.remove(line.glyphs[cursor_glyph].start);
                                reshape = true;
                            }
                        },
                        orbclient::K_0 => {
                            font_size_i = font_size_default;
                            relayout = true;
                        },
                        orbclient::K_MINUS => if font_size_i > 0 {
                            font_size_i -= 1;
                            relayout = true;
                        },
                        orbclient::K_EQUALS => if font_size_i + 1 < font_sizes.len() {
                            font_size_i += 1;
                            relayout = true;
                        },
                        _ => (),
                    }
                },
                EventOption::TextInput(event) => {
                    let line = &layout_lines[cursor_line];
                    if cursor_glyph >= line.glyphs.len() {
                        match line.glyphs.last() {
                            Some(glyph) => {
                                text.insert(glyph.end, event.character);
                                cursor_glyph += 1;
                                reshape = true;
                            },
                            None => () // TODO
                        }
                    } else {
                        let glyph = &line.glyphs[cursor_glyph];
                        text.insert(glyph.start, event.character);
                        cursor_glyph += 1;
                        reshape = true;
                    }
                },
                EventOption::Mouse(event) => {
                    mouse_x = event.x;
                    mouse_y = event.y;
                    if mouse_left {
                        redraw = true;
                    }
                },
                EventOption::Button(event) => {
                    if event.left != mouse_left {
                        mouse_left = event.left;
                        if mouse_left {
                            redraw = true;
                        }
                    }
                }
                EventOption::Resize(_) => {
                    relayout = true;
                },
                EventOption::Scroll(event) => {
                    scroll -= event.y * 3;
                    redraw = true;
                },
                EventOption::Quit(_) => return,
                _ => (),
            }
        }
    }
}
