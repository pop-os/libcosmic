use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use std::{
    cmp,
    env,
    fs,
    time::Instant,
};
use text::{
    Font,
    FontLayoutLine,
    FontLineIndex,
    FontMatches,
    FontShapeLine,
    FontSystem,
};

pub struct TextBuffer<'a> {
    font_matches: &'a FontMatches<'a>,
    text_lines: Vec<String>,
    shape_lines: Vec<FontShapeLine<'a>>,
    layout_lines: Vec<FontLayoutLine<'a>>,
    redraw: bool,
    font_size: i32,
    line_width: i32,
    line_height: i32,
}

impl<'a> TextBuffer<'a> {
    pub fn new(font_matches: &'a FontMatches, text: &str, font_size: i32, line_width: i32, line_height: i32) -> Self {
        let mut buffer = Self {
            font_matches,
            text_lines: text.lines().map(String::from).collect(),
            shape_lines: Vec::new(),
            layout_lines: Vec::new(),
            redraw: true,
            font_size,
            line_width,
            line_height,
        };
        buffer.reshape();
        buffer
    }

    pub fn reshape(&mut self) {
        let instant = Instant::now();

        self.shape_lines.clear();
        for (line_i, text_line) in self.text_lines.iter().enumerate() {
            self.shape_lines.push(
                self.font_matches.shape_line(FontLineIndex::new(line_i), text_line)
            );
        }

        let duration = instant.elapsed();
        eprintln!("reshape: {:?}", duration);

        self.relayout();
    }

    pub fn reshape_line(&mut self, line_i: FontLineIndex) {
        let instant = Instant::now();

        self.shape_lines[line_i.get()] = self.font_matches.shape_line(line_i, &self.text_lines[line_i.get()]);

        let duration = instant.elapsed();
        eprintln!("reshape line {}: {:?}", line_i.get(), duration);

        self.relayout_line(line_i);
    }

    pub fn relayout(&mut self) {
        let instant = Instant::now();

        self.layout_lines.clear();
        for line in self.shape_lines.iter() {
            let layout_i = self.layout_lines.len();
            line.layout(self.font_size, self.line_width, &mut self.layout_lines, layout_i);
        }

        self.redraw = true;

        let duration = instant.elapsed();
        eprintln!("relayout: {:?}", duration);
    }

    pub fn relayout_line(&mut self, line_i: FontLineIndex) {
        let instant = Instant::now();

        let mut insert_opt = None;
        let mut layout_i = 0;
        while layout_i < self.layout_lines.len() {
            let layout_line = &self.layout_lines[layout_i];
            if layout_line.line_i == line_i {
                if insert_opt.is_none() {
                    insert_opt = Some(layout_i);
                }
                self.layout_lines.remove(layout_i);
            } else {
                layout_i += 1;
            }
        }

        let shape_line = &self.shape_lines[line_i.get()];
        shape_line.layout(self.font_size, self.line_width, &mut self.layout_lines, insert_opt.unwrap());

        self.redraw = true;

        let duration = instant.elapsed();
        eprintln!("relayout line {}: {:?}", line_i.get(), duration);
    }
}

fn main() {
    let display_scale = match orbclient::get_display_size() {
        Ok((w, h)) => {
            eprintln!("Display size: {}, {}", w, h);
            (h as i32 / 1600) + 1
        },
        Err(err) => {
            eprintln!("Failed to get display size: {}", err);
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

    let mut font_system = FontSystem::new();

    font_system.add(
        Font::new(include_bytes!("../../../res/Fira/FiraSans-Regular.otf"), 0).unwrap()
    );
    font_system.add(
        Font::new(include_bytes!("../../../res/Fira/FiraMono-Regular.otf"), 0).unwrap()
    );

    let mut font_datas = Vec::new();
    for (font_path, font_index) in &[
        ("/usr/share/fonts/truetype/freefont/FreeSans.ttf", 0),
        ("/usr/share/fonts/truetype/freefont/FreeSerif.ttf", 0),
        ("/usr/share/fonts/truetype/freefont/FreeMono.ttf", 0),
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 2 /* simplified chinese */),
    ] {
        match fs::read(font_path) {
            Ok(font_data) => font_datas.push((font_path, font_data, *font_index)),
            Err(err) => {
                eprintln!("failed to read font '{}': {}", font_path, err)
            }
        }
    }

    for (font_path, font_data, font_index) in &font_datas {
        match Font::new(font_data, *font_index) {
            Some(font) => font_system.add(font),
            None => {
                eprintln!("failed to parse font '{}'", font_path)
            }
        }
    }

    #[cfg(feature = "mono")]
    let font_pattern = &["Mono"];
    #[cfg(not(feature = "mono"))]
    let font_pattern = &["Sans", "Serif"];

    let font_matches = font_system.matches(font_pattern).unwrap();

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
    let font_size_default = 5; // Title 1
    let mut font_size_i = font_size_default;

    let text = if let Some(arg) = env::args().nth(1) {
        fs::read_to_string(&arg).expect("failed to open file")
    } else {
        #[cfg(feature = "mono")]
        let default_text = include_str!("../res/mono.txt");
        #[cfg(not(feature = "mono"))]
        let default_text = include_str!("../res/proportional.txt");
        default_text.to_string()
    };

    let line_x = 8 * display_scale;
    let mut buffer = TextBuffer::new(
        &font_matches,
        &text,
        font_sizes[font_size_i].0 * display_scale,
        window.width() as i32 - line_x * 2,
        font_sizes[font_size_i].1 * display_scale,
    );

    let mut cursor_line = 0;
    let mut cursor_glyph = 0;
    let mut mouse_x = -1;
    let mut mouse_y = -1;
    let mut mouse_left = false;
    let mut rehit = false;
    let mut scroll = 0;
    loop {
        let font_size = buffer.font_size;
        let line_height = buffer.line_height;

        let window_lines = (window.height() as i32 + line_height - 1) / line_height;
        scroll = cmp::max(0, cmp::min(
            buffer.layout_lines.len() as i32 - (window_lines - 1),
            scroll
        ));

        if rehit {
            let instant = Instant::now();

            let mut line_y = line_height;
            for (line_i, line) in buffer.layout_lines.iter().skip(scroll as usize).enumerate() {
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
                        buffer.redraw = true;
                    }
                }

                line_y += line_height;
            }

            rehit = false;

            let duration = instant.elapsed();
            eprintln!("rehit: {:?}", duration);
        }

        if buffer.redraw {
            let instant = Instant::now();

            window.set(bg_color);

            let mut line_y = line_height;
            for (line_i, line) in buffer.layout_lines.iter().skip(scroll as usize).enumerate() {
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

                        let text_line = &buffer.text_lines[line.line_i.get()];
                        eprintln!("{}, {}: '{}'", glyph.start, glyph.end, &text_line[glyph.start..glyph.end]);
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

            buffer.redraw = false;

            let duration = instant.elapsed();
            eprintln!("redraw: {:?}", duration);
        }

        for event in window.events() {
            match event.to_option() {
                EventOption::Key(event) => if event.pressed {
                    match event.scancode {
                        orbclient::K_UP => if cursor_line > 0 {
                            cursor_line -= 1;
                            buffer.redraw = true;
                        },
                        orbclient::K_DOWN => if cursor_line + 1 < buffer.layout_lines.len() {
                            cursor_line += 1;
                            buffer.redraw = true;
                        },
                        orbclient::K_LEFT => {
                            let line = &buffer.layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                buffer.redraw = true;
                            }
                            if cursor_glyph > 0 {
                                cursor_glyph -= 1;
                                buffer.redraw = true;
                            }
                        },
                        orbclient::K_RIGHT => {
                            let line = &buffer.layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                buffer.redraw = true;
                            }
                            if cursor_glyph < line.glyphs.len() {
                                cursor_glyph += 1;
                                buffer.redraw = true;
                            }
                        },
                        orbclient::K_BKSP => {
                            let line = &buffer.layout_lines[cursor_line];
                            if cursor_glyph > line.glyphs.len() {
                                cursor_glyph = line.glyphs.len();
                                buffer.redraw = true;
                            }
                            if cursor_glyph > 0 {
                                cursor_glyph -= 1;
                                let glyph = &line.glyphs[cursor_glyph];
                                let text_line = &mut buffer.text_lines[line.line_i.get()];
                                text_line.remove(glyph.start);
                                buffer.reshape_line(line.line_i);
                            }
                        },
                        orbclient::K_DEL => {
                            let line = &buffer.layout_lines[cursor_line];
                            if cursor_glyph < line.glyphs.len() {
                                let glyph = &line.glyphs[cursor_glyph];
                                let text_line = &mut buffer.text_lines[line.line_i.get()];
                                text_line.remove(glyph.start);
                                buffer.reshape_line(line.line_i);
                            }
                        },
                        orbclient::K_0 => {
                            font_size_i = font_size_default;
                            buffer.font_size = font_sizes[font_size_i].0 * display_scale;
                            buffer.line_height = font_sizes[font_size_i].1 * display_scale;
                            buffer.relayout();
                        },
                        orbclient::K_MINUS => if font_size_i > 0 {
                            font_size_i -= 1;
                            buffer.font_size = font_sizes[font_size_i].0 * display_scale;
                            buffer.line_height = font_sizes[font_size_i].1 * display_scale;
                            buffer.relayout();
                        },
                        orbclient::K_EQUALS => if font_size_i + 1 < font_sizes.len() {
                            font_size_i += 1;
                            buffer.font_size = font_sizes[font_size_i].0 * display_scale;
                            buffer.line_height = font_sizes[font_size_i].1 * display_scale;
                            buffer.relayout();
                        },
                        _ => (),
                    }
                },
                EventOption::TextInput(event) => {
                    let line = &buffer.layout_lines[cursor_line];
                    if cursor_glyph >= line.glyphs.len() {
                        match line.glyphs.last() {
                            Some(glyph) => {
                                let text_line = &mut buffer.text_lines[line.line_i.get()];
                                text_line.insert(glyph.end, event.character);
                                cursor_glyph += 1;
                                buffer.reshape_line(line.line_i);
                            },
                            None => {
                                let text_line = &mut buffer.text_lines[line.line_i.get()];
                                text_line.push(event.character);
                                cursor_glyph += 1;
                                buffer.reshape_line(line.line_i);
                            }
                        }
                    } else {
                        let glyph = &line.glyphs[cursor_glyph];
                        let text_line = &mut buffer.text_lines[line.line_i.get()];
                        text_line.insert(glyph.start, event.character);
                        cursor_glyph += 1;
                        buffer.reshape_line(line.line_i);
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
                EventOption::Resize(event) => {
                    buffer.line_width = event.width as i32 - line_x * 2;
                    buffer.relayout();
                },
                EventOption::Scroll(event) => {
                    scroll -= event.y * 3;
                    buffer.redraw = true;
                },
                EventOption::Quit(_) => return,
                _ => (),
            }
        }
    }
}
