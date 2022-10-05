use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::{
    cmp,
    env,
    fs,
    marker::PhantomData,
    time::Instant,
};
use text::{
    Font,
    FontSystem,
};

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
    let mut rehit = false;
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

        if rehit {
            let instant = Instant::now();

            let window_lines = (window.height() as i32 + line_height - 1) / line_height;
            scroll = cmp::max(0, cmp::min(
                layout_lines.len() as i32 - (window_lines - 1),
                scroll
            ));

            let line_x = 8 * display_scale;
            let mut line_y = line_height;
            for (line_i, line) in layout_lines.iter().skip(scroll as usize).enumerate() {
                if line_y >= window.height() as i32 {
                    break;
                }

                if mouse_left
                && mouse_y >= line_y - font_size
                && mouse_y < line_y - font_size + line_height
                {
                    let new_cursor_line = line_i + scroll as usize;
                    let mut new_cursor_glyph = line.glyphs.len();
                    for (glyph_i, glyph) in line.glyphs.iter().enumerate() {
                        if mouse_x >= line_x + glyph.x as i32
                        && mouse_x <= line_x + (glyph.x + glyph.w) as i32
                        {
                           new_cursor_glyph = glyph_i;
                        }
                    }
                    if new_cursor_line != cursor_line || new_cursor_glyph != cursor_glyph {
                        cursor_line = new_cursor_line;
                        cursor_glyph = new_cursor_glyph;
                        redraw = true;
                    }
                }

                line_y += line_height;
            }

            rehit = false;

            let duration = instant.elapsed();
            println!("rehit: {:?}", duration);
        }

        if redraw {
            let instant = Instant::now();

            window.set(bg_color);

            let window_lines = (window.height() as i32 + line_height - 1) / line_height;
            scroll = cmp::max(0, cmp::min(
                layout_lines.len() as i32 - (window_lines - 1),
                scroll
            ));

            let line_x = 8 * display_scale;
            let mut line_y = line_height;
            for (line_i, line) in layout_lines.iter().skip(scroll as usize).enumerate() {
                if line_y >= window.height() as i32 {
                    break;
                }

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

                line.draw(|x, y, v| {
                    let c = (v * 255.0) as u32;
                    window.pixel(line_x + x, line_y + y, Color{
                        data: c << 24 | (font_color.data & 0x00FF_FFFF)
                    });
                });

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
                        rehit = true;
                    }
                },
                EventOption::Button(event) => {
                    if event.left != mouse_left {
                        mouse_left = event.left;
                        if mouse_left {
                            rehit = true;
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
