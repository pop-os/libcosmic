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

    #[cfg(feature = "mono")]
    let font = Font::new(include_bytes!("../../../res/FreeFont/FreeMono.ttf"), 0).unwrap();
    #[cfg(not(feature = "mono"))]
    let font = Font::new(include_bytes!("../../../res/FreeFont/FreeSerif.ttf"), 0).unwrap();
    let font_scale = font.rustybuzz.units_per_em();

    let mut shaped_lines = Vec::new();
    for line in text.lines() {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(line);
        buffer.guess_segment_properties();
        println!("{:?}: {}", buffer.script(), line);

        let glyph_buffer = rustybuzz::shape(&font.rustybuzz, &[], buffer);
        let glyph_infos = glyph_buffer.glyph_infos();
        let glyph_positions = glyph_buffer.glyph_positions();

        let mut shaped = Vec::new();
        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            println!("  {:?} {:?}", info, pos);

            shaped.push((*info, *pos));
        }
        shaped_lines.push(shaped);
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
                let mut glyphs = Vec::new();
                for (info, pos) in shaped.iter() {
                    let glyph = font.rusttype.glyph(rusttype::GlyphId(info.glyph_id as u16))
                        .scaled(rusttype::Scale::uniform(font_size as f32))
                        .positioned(rusttype::point(
                            (font_size * pos.x_offset) as f32 / font_scale as f32,
                            (font_size * pos.y_offset) as f32 / font_scale as f32
                        ));
                    glyphs.push((pos.x_advance, pos.y_advance, glyph));
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
                for (x_advance, y_advance, glyph) in glyph_line.iter() {
                    if let Some(bb) = glyph.pixel_bounding_box() {
                        //TODO: make wrapping optional
                        if glyph_x + bb.max.x >= window.width() as i32 {
                            line_y += line_height;

                            glyph_x = 0;
                            glyph_y = line_y;
                        }

                        let x = glyph_x + bb.min.x;
                        let y = glyph_y + bb.min.y;
                        if mouse_x >= x
                        && mouse_x < x + bb.width()
                        && mouse_y >= y
                        && mouse_y < y + bb.height() {
                            window.rect(x, y, bb.width() as u32, bb.height() as u32, hover_color);
                        }
                        glyph.draw(|off_x, off_y, v| {
                            let c = (v * 255.0) as u32;
                            window.pixel(x + off_x as i32, y + off_y as i32, Color{
                                data: c << 24 | (font_color.data & 0x00FF_FFFF)
                            });
                        });
                    }

                    glyph_x += (font_size * x_advance) / font_scale;
                    glyph_y += (font_size * y_advance) / font_scale;
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
