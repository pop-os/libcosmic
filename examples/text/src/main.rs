use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::cmp;

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

fn main() {
    let mut window = Window::new_flags(
        -1, -1,
        1024, 768,
        "COSMIC TEXT",
        &[WindowFlag::Resizable]
    ).unwrap();

    let text = include_str!("../res/UTF-8-demo.txt");
    let lines: Vec<&'static str> = text.lines().collect();

    let bg_color = Color::rgb(0x34, 0x34, 0x34);
    let font_color = Color::rgb(0xFF, 0xFF, 0xFF);
    let font_sizes = [
        (10.0, 14), // Caption
        (14.0, 20), // Body
        (20.0, 28), // Title 4
        (24.0, 32), // Title 3
        (28.0, 36), // Title 2
        (32.0, 44), // Title 1
    ];
    let font_size_default = 2; // Title 4
    let mut font_size_i = font_size_default;

    let font = Font::new(include_bytes!("../../../res/FreeFont/FreeMono.ttf"), 0).unwrap();
    let font_scale = font.rustybuzz.units_per_em() as f32;

    let mut redraw = true;
    let mut scroll = 0;
    loop {
        if redraw {
            window.set(bg_color);

            let (font_size, line_height) = font_sizes[font_size_i];

            let window_lines = (window.height() as i32 + line_height - 1) / line_height;
            scroll = cmp::max(0, cmp::min(
                lines.len() as i32 - window_lines,
                scroll
            ));

            let mut line_y = 0;
            for line in lines.iter().skip(scroll as usize).take((window_lines + 1) as usize) {
                let mut buffer = rustybuzz::UnicodeBuffer::new();
                buffer.push_str(line);
                buffer.guess_segment_properties();
                //println!("{:?}: {}", buffer.script(), line);

                let glyph_buffer = rustybuzz::shape(&font.rustybuzz, &[], buffer);
                let glyph_infos = glyph_buffer.glyph_infos();
                let glyph_positions = glyph_buffer.glyph_positions();

                let mut glyph_x = 0i32;
                let mut glyph_y = line_y;
                for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
                    //println!("  {:?} {:?}", info, pos);

                    let glyph = font.rusttype.glyph(rusttype::GlyphId(info.glyph_id as u16));
                    let scaled = glyph.scaled(rusttype::Scale::uniform(font_size));
                    let positioned = scaled.positioned(rusttype::point(
                        glyph_x as f32 + font_size * pos.x_offset as f32 / font_scale,
                        glyph_y as f32 + font_size * pos.y_offset as f32 / font_scale,
                    ));
                    if let Some(bb) = positioned.pixel_bounding_box() {
                        positioned.draw(|x, y, v| {
                            let x = x as i32 + bb.min.x;
                            let y = y as i32 + bb.min.y;
                            let c = (v * 255.0) as u32;
                            window.pixel(x, y, Color{
                                data: c << 24 | (font_color.data & 0x00FF_FFFF)
                            });
                        });
                    }

                    glyph_x += (font_size * pos.x_advance as f32 / font_scale) as i32;
                    glyph_y += (font_size * pos.y_advance as f32 / font_scale) as i32;
                }

                line_y += line_height;
            }

            window.sync();

            redraw = false;
        }

        for event in window.events() {
            match event.to_option() {
                EventOption::Key(event) => if event.pressed {
                    match event.scancode {
                        orbclient::K_0 => {
                            font_size_i = font_size_default;
                            redraw = true;
                        },
                        orbclient::K_MINUS => if font_size_i > 0 {
                            font_size_i -= 1;
                            redraw = true;
                        },
                        orbclient::K_EQUALS => if font_size_i + 1 < font_sizes.len() {
                            font_size_i += 1;
                            redraw = true;
                        },
                        _ => (),
                    }
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
