use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::{
    cmp,
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

struct FontGlyph<'a> {
    x_advance: i32,
    y_advance: i32,
    inner: rusttype::PositionedGlyph<'a>,
}

struct FontShape<'a> {
    info: rustybuzz::GlyphInfo,
    pos: rustybuzz::GlyphPosition,
    font: &'a Font<'a>
}

impl<'a> FontShape<'a> {
    pub fn glyph(&self, font_size: i32) -> FontGlyph<'a> {
        let font_scale = self.font.rustybuzz.units_per_em();

        let glyph = self.font.rusttype.glyph(rusttype::GlyphId(self.info.glyph_id as u16))
            .scaled(rusttype::Scale::uniform(font_size as f32))
            .positioned(rusttype::point(
                (font_size * self.pos.x_offset) as f32 / font_scale as f32,
                (font_size * self.pos.y_offset) as f32 / font_scale as f32
            ));

        FontGlyph {
            x_advance: (font_size * self.pos.x_advance) / font_scale,
            y_advance: (font_size * self.pos.y_advance) / font_scale,
            inner: glyph
        }
    }
}

struct FontMatches<'a> {
    fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    pub fn shape(&self, line: &str) -> Vec<FontShape> {
        let mut font_shaped = Vec::with_capacity(self.fonts.len());
        for font in self.fonts.iter() {
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(line);
            buffer.guess_segment_properties();
            println!("{:?}: {}", buffer.script(), line);

            let glyph_buffer = rustybuzz::shape(&font.rustybuzz, &[], buffer);
            let glyph_infos = glyph_buffer.glyph_infos();
            let glyph_positions = glyph_buffer.glyph_positions();

            let mut misses = 0;
            let mut shaped = Vec::with_capacity(glyph_infos.len());
            for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
                //println!("  {:?} {:?}", info, pos);
                if info.glyph_id == 0 {
                    misses += 1;
                }
                shaped.push(FontShape {
                    info: *info,
                    pos: *pos,
                    font,
                });
            }
            if misses == 0 {
                return shaped;
            } else {
                font_shaped.push((misses, shaped));
            }
        }

        let mut least_i = 0;
        let mut least_misses = usize::MAX;
        for (i, (misses, _)) in font_shaped.iter().enumerate() {
            if *misses < least_misses {
                least_i = i;
                least_misses = *misses;
            }
        }

        if least_i > 0 {
            println!("MISSES {}, {}", least_i, least_misses);
        }

        font_shaped.remove(least_i).1
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
    let text = include_str!("../res/mono.txt");
    #[cfg(not(feature = "mono"))]
    let text = include_str!("../res/proportional.txt");

    let bg_color = Color::rgb(0x34, 0x34, 0x34);
    let hover_color = Color::rgb(0x80, 0x80, 0x80);
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

    let mut shaped_lines = Vec::new();
    for line in text.lines() {
        shaped_lines.push(font_matches.shape(line));
    }

    let mut glyph_lines = Vec::new();
    let mut mouse_x = -1;
    let mut mouse_y = -1;
    let mut redraw = true;
    let mut reglyph = true;
    let mut scroll = 0;
    loop {
        let (font_size, line_height) = font_sizes[font_size_i];

        if reglyph {
            let instant = Instant::now();

            glyph_lines.clear();
            for shaped in shaped_lines.iter() {
                let mut glyphs = Vec::with_capacity(shaped.len());
                for shape in shaped.iter() {
                    glyphs.push(shape.glyph(font_size));
                }
                glyph_lines.push(glyphs);
            }

            redraw = true;
            reglyph = false;

            let duration = instant.elapsed();
            println!("reglyph: {:?}", duration);
        }

        if redraw {
            let instant = Instant::now();

            window.set(bg_color);

            let window_lines = (window.height() as i32 + line_height - 1) / line_height;
            scroll = cmp::max(0, cmp::min(
                glyph_lines.len() as i32 - (window_lines - 1),
                scroll
            ));

            let mut line_y = line_height;
            for glyph_line in glyph_lines.iter().skip(scroll as usize) {
                if line_y >= window.height() as i32 {
                    break;
                }

                let mut glyph_x = 0i32;
                let mut glyph_y = line_y;
                for glyph in glyph_line.iter() {
                    if let Some(bb) = glyph.inner.pixel_bounding_box() {
                        //TODO: make wrapping optional
                        if glyph_x + bb.max.x >= window.width() as i32 {
                            line_y += line_height;

                            glyph_x = 0;
                            glyph_y = line_y;
                        }

                        if mouse_x >= glyph_x
                        && mouse_x < glyph_x + glyph.x_advance
                        && mouse_y >= line_y - font_size
                        && mouse_y < line_y - font_size + line_height
                        {
                            //TODO: this highlights only one character of combinations
                            window.rect(glyph_x, line_y - font_size, glyph.x_advance as u32, line_height as u32, hover_color);
                        }

                        let x = glyph_x + bb.min.x;
                        let y = glyph_y + bb.min.y;
                        glyph.inner.draw(|off_x, off_y, v| {
                            let c = (v * 255.0) as u32;
                            window.pixel(x + off_x as i32, y + off_y as i32, Color{
                                data: c << 24 | (font_color.data & 0x00FF_FFFF)
                            });
                        });
                    }

                    glyph_x += glyph.x_advance;
                    glyph_y += glyph.y_advance;
                }

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
                            reglyph = true;
                        },
                        orbclient::K_MINUS => if font_size_i > 0 {
                            font_size_i -= 1;
                            reglyph = true;
                        },
                        orbclient::K_EQUALS => if font_size_i + 1 < font_sizes.len() {
                            font_size_i += 1;
                            reglyph = true;
                        },
                        _ => (),
                    }
                },
                EventOption::Mouse(event) => {
                    mouse_x = event.x;
                    mouse_y = event.y;
                    redraw = true;
                },
                EventOption::Resize(_) => {
                    redraw = true;
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
