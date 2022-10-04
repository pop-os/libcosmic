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
    info: rustybuzz::GlyphInfo,
    pos: rustybuzz::GlyphPosition,
    font: &'a Font<'a>
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
        let mut glyphs = Vec::new();

        let mut x = 0.0;
        let mut y = 0.0;
        for span in self.spans.iter() {
            let mut span_width = 0.0;
            for glyph in span.glyphs.iter() {
                let font_scale = glyph.font.rustybuzz.units_per_em() as f32;
                let x_advance = (font_size * glyph.pos.x_advance) as f32 / font_scale;
                span_width += x_advance;
            }

            if self.rtl {
                if glyphs.is_empty() {
                    x = line_width as f32;
                }
                x -= span_width;
            }

            for glyph in span.glyphs.iter() {
                let font_scale = glyph.font.rustybuzz.units_per_em() as f32;
                let x_advance = (font_size * glyph.pos.x_advance) as f32 / font_scale;
                let y_advance = (font_size * glyph.pos.y_advance) as f32 / font_scale;
                let x_offset = (font_size * glyph.pos.x_offset) as f32 / font_scale;
                let y_offset = (font_size * glyph.pos.y_offset) as f32 / font_scale;

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

                        x = 0.0;
                        y = 0.0;
                    }
                }

                let inner = glyph.font.rusttype.glyph(rusttype::GlyphId(glyph.info.glyph_id as u16))
                    .scaled(rusttype::Scale::uniform(font_size as f32))
                    .positioned(rusttype::point(
                        x + x_offset,
                        y + y_offset,
                    ));

                glyphs.push(FontLayoutGlyph {
                    start: glyph.start,
                    end: glyph.end,
                    inner,
                });

                x += x_advance;
                y += y_advance;
            }

            if self.rtl {
                x -= span_width;
            }
        }

        if ! glyphs.is_empty() {
            lines.push(FontLayoutLine { glyphs });
        }
    }
}

struct FontMatches<'a> {
    fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    fn shape_span(&self, string: &'a str, start_span: usize, end_span: usize) -> FontShapeSpan {
        let span = &string[start_span..end_span];

        let mut spans_by_font = Vec::with_capacity(self.fonts.len());
        for (font_i, font) in self.fonts.iter().enumerate() {
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
                //println!("  {:?} {:?}", info, pos);
                if info.glyph_id == 0 {
                    misses += 1;
                }
                glyphs.push(FontShapeGlyph {
                    start: start_span + info.cluster as usize,
                    end: end_span, // Set later
                    info: *info,
                    pos: *pos,
                    font,
                });
            }

            // Adjust end of glyphs
            match direction {
                rustybuzz::Direction::LeftToRight => {
                    for i in 1..glyphs.len() {
                        glyphs[i - 1].end = glyphs[i].start;
                    }
                },
                rustybuzz::Direction::RightToLeft => {
                    for i in 1..glyphs.len() {
                        glyphs[i].end = glyphs[i - 1].start;
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

    fn shape_line(&self, string: &'a str, start_line: usize, end_line: usize) -> FontShapeLine {
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

    pub fn shape(&self, string: &'a str) -> Vec<FontShapeLine> {
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
    let mut window = Window::new_flags(
        -1, -1,
        1024, 768,
        "COSMIC TEXT",
        &[WindowFlag::Resizable]
    ).unwrap();

    #[cfg(feature = "mono")]
    let default_text = include_str!("../res/mono.txt");
    #[cfg(not(feature = "mono"))]
    let default_text = include_str!("../res/proportional.txt");

    let text = if let Some(arg) = env::args().nth(1) {
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

    let shape_lines = font_matches.shape(&text);

    let mut layout_lines = Vec::new();
    let mut mouse_x = -1;
    let mut mouse_y = -1;
    let mut mouse_left = false;
    let mut redraw = true;
    let mut relayout = true;
    let mut scroll = 0;
    loop {
        let (font_size, line_height) = font_sizes[font_size_i];

        if relayout {
            let instant = Instant::now();

            layout_lines.clear();
            for line in shape_lines.iter() {
                let line_width = window.width() as i32 - 16;
                line.layout(font_size, line_width, &mut layout_lines);
            }

            redraw = true;
            relayout = false;

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

            let line_x = 8;
            let mut line_y = line_height;
            for line in layout_lines.iter().skip(scroll as usize) {
                if line_y >= window.height() as i32 {
                    break;
                }

                if mouse_y >= line_y - font_size
                && mouse_y < line_y - font_size + line_height
                {
                    let mut i = 0;
                    while i < line.glyphs.len() {
                        let glyph = &line.glyphs[i];
                        i += 1;

                        if let Some(bb) = glyph.inner.pixel_bounding_box() {
                            if mouse_x >= line_x + bb.min.x && mouse_x <= line_x + bb.max.x {
                                window.rect(
                                    line_x + bb.min.x,
                                    line_y - font_size,
                                    bb.width() as u32,
                                    line_height as u32,
                                    Color::rgba(0xFF, 0xFF, 0xFF, 0x20)
                                );

                                println!("{}, {}: '{}'", glyph.start, glyph.end, &text[glyph.start..glyph.end]);
                            }
                        }
                    }
                }

                line.draw(
                    &mut window,
                    line_x,
                    line_y,
                    font_color
                );

                line_y += line_height;
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
