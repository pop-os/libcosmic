pub mod parser;

use std::sync::{Mutex, OnceLock};

use crate::{Element, Renderer};
use cosmic_text::{
    Attrs, Buffer, Color, Edit, Editor, Family, FontSystem, Metrics, Shaping, SwashCache, Weight,
};
use iced::{Length, Rectangle, Size};
use iced_core::{
    image, layout,
    widget::{self, tree},
    Widget,
};
use markdown::tokenize;

static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

pub struct Markdown {
    syntax_editor: Mutex<Editor<'static>>,
    swash_cache: Mutex<SwashCache>,
    margin: f32,
}

impl Markdown {
    pub fn new(content: &str) -> Self {
        FONT_SYSTEM.get_or_init(|| Mutex::new(FontSystem::new()));

        let metrics = metrics(14.0);
        let buffer = Buffer::new_empty(metrics);

        let mut editor = Editor::new(buffer);
        let mut parser = parser::Parser::new();
        let blocks = tokenize(content);

        parser.run(Box::leak(Box::new(blocks)));

        editor.with_buffer_mut(|buffer| {
            set_buffer_text(
                &mut FONT_SYSTEM.get().unwrap().lock().unwrap(),
                &mut parser.get_spans(),
                buffer,
            )
        });

        Self {
            syntax_editor: Mutex::new(editor),
            swash_cache: Mutex::new(SwashCache::new()),
            margin: 0.0,
        }
    }

    pub fn margin(&mut self, margin: f32) {
        self.margin = margin;
    }
}

pub struct State {
    handle_opt: Mutex<Option<image::Handle>>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State {
            handle_opt: Mutex::new(None),
        }
    }
}

impl<Message> Widget<Message, crate::Theme, Renderer> for Markdown {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut font_system = FONT_SYSTEM.get().unwrap().lock().unwrap();
        let max_width = limits.max().width - self.margin;

        let mut editor = self.syntax_editor.lock().unwrap();
        editor.borrow_with(&mut font_system).shape_as_needed(true);

        editor.with_buffer_mut(|buffer| {
            let mut layout_lines = 0;
            let mut width = 0.0;
            let mut height = 0.0;

            buffer.set_size(
                &mut font_system,
                Some(max_width),
                Some(buffer.metrics().line_height),
            );

            buffer.set_wrap(&mut font_system, cosmic_text::Wrap::Word);

            for line in buffer.lines.iter() {
                match line.layout_opt() {
                    Some(layout) => {
                        layout_lines += 1;

                        for l in layout.iter() {
                            if layout_lines > 1 {
                                width = max_width;

                                break;
                            }
                            width = l.w;
                        }

                        for l in layout.iter() {
                            if let Some(line_height) = l.line_height_opt {
                                height += line_height;
                            } else {
                                height += buffer.metrics().line_height;
                            }
                        }
                    }
                    None => (),
                }
            }

            buffer.set_size(&mut font_system, Some(max_width), Some(height));

            let size = Size::new(width, height);

            layout::Node::new(size)
        })
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let mut swash_cache = self.swash_cache.lock().unwrap();
        let mut font_system = FONT_SYSTEM.get().unwrap().lock().unwrap();
        let mut editor = self.syntax_editor.lock().unwrap();

        let scale_factor = style.scale_factor as f32;

        let view_w = layout.bounds().width as i32;
        let view_h = layout.bounds().height as i32;

        let calculate_image_scaled = |view: i32| -> (i32, f32) {
            // Get smallest set of physical pixels that fit inside the logical pixels
            let image = ((view as f32) * scale_factor).floor() as i32;
            // Convert that back into logical pixels
            let scaled = (image as f32) / scale_factor;
            (image, scaled)
        };
        let calculate_ideal = |view_start: i32| -> (i32, f32) {
            // Search for a perfect match within 16 pixels
            for i in 0..16 {
                let view = view_start - i;
                let (image, scaled) = calculate_image_scaled(view);
                if view == scaled as i32 {
                    return (image, scaled);
                }
            }
            let (image, scaled) = calculate_image_scaled(view_start);
            (image, scaled)
        };

        let (image_w, _scaled_w) = calculate_ideal(view_w);
        let (image_h, _scaled_h) = calculate_ideal(view_h);

        editor.shape_as_needed(&mut font_system, true);

        let mut pixels_u8 = vec![0; image_w as usize * image_h as usize * 4];

        let pixels = unsafe {
            std::slice::from_raw_parts_mut(pixels_u8.as_mut_ptr() as *mut u32, pixels_u8.len() / 4)
        };

        let mut handle_opt = state.handle_opt.lock().unwrap();

        if editor.redraw() || handle_opt.is_none() {
            editor.with_buffer(|buffer| {
                buffer.draw(
                    &mut font_system,
                    &mut swash_cache,
                    cosmic_text::Color(0xFFFFFF),
                    |x, y, w, h, color| {
                        draw_rect(
                            pixels,
                            Canvas {
                                w: image_w,
                                h: image_h,
                            },
                            Canvas {
                                w: w as i32,
                                h: h as i32,
                            },
                            Offset { x, y },
                            color,
                        );
                    },
                );
            });
        }

        *handle_opt = Some(image::Handle::from_pixels(
            image_w as u32,
            image_h as u32,
            pixels_u8,
        ));

        if let Some(ref handle) = *handle_opt {
            image::Renderer::draw(
                renderer,
                handle.clone(),
                image::FilterMethod::Nearest,
                Rectangle {
                    x: layout.position().x,
                    y: layout.position().y,
                    width: image_w as f32,
                    height: image_h as f32,
                },
                [0.0; 4],
            );
        }
    }
}

impl<'a, Message> From<Markdown> for Element<'a, Message> {
    fn from(value: Markdown) -> Self {
        Self::new(value)
    }
}

struct Canvas {
    w: i32,
    h: i32,
}

struct Offset {
    x: i32,
    y: i32,
}

fn draw_rect(
    buffer: &mut [u32],
    canvas: Canvas,
    offset: Canvas,
    screen: Offset,
    cosmic_color: cosmic_text::Color,
) {
    // Grab alpha channel and green channel
    let mut color = cosmic_color.0 & 0xFF00FF00;
    // Shift red channel
    color |= (cosmic_color.0 & 0x00FF0000) >> 16;
    // Shift blue channel
    color |= (cosmic_color.0 & 0x000000FF) << 16;

    let alpha = (color >> 24) & 0xFF;
    match alpha {
        0 => {
            // Do not draw if alpha is zero.
        }
        255 => {
            // Handle overwrite
            for x in screen.x..screen.x + offset.w {
                if x < 0 || x >= canvas.w {
                    // Skip if y out of bounds
                    continue;
                }

                for y in screen.y..screen.y + offset.h {
                    if y < 0 || y >= canvas.h {
                        // Skip if x out of bounds
                        continue;
                    }

                    let line_offset = y as usize * canvas.w as usize;
                    let offset = line_offset + x as usize;
                    buffer[offset] = color;
                }
            }
        }
        _ => {
            let n_alpha = 255 - alpha;
            for y in screen.y..screen.y + offset.h {
                if y < 0 || y >= canvas.h {
                    // Skip if y out of bounds
                    continue;
                }

                let line_offset = y as usize * canvas.w as usize;
                for x in screen.x..screen.x + offset.w {
                    if x < 0 || x >= canvas.w {
                        // Skip if x out of bounds
                        continue;
                    }

                    // Alpha blend with current value
                    let offset = line_offset + x as usize;
                    let current = buffer[offset];
                    if current & 0xFF000000 == 0 {
                        // Overwrite if buffer empty
                        buffer[offset] = color;
                    } else {
                        let rb = ((n_alpha * (current & 0x00FF00FF))
                            + (alpha * (color & 0x00FF00FF)))
                            >> 8;
                        let ag = (n_alpha * ((current & 0xFF00FF00) >> 8))
                            + (alpha * (0x01000000 | ((color & 0x0000FF00) >> 8)));
                        buffer[offset] = (rb & 0x00FF00FF) | (ag & 0xFF00FF00);
                    }
                }
            }
        }
    }
}

fn metrics(font_size: f32) -> Metrics {
    let line_height = (font_size * 1.4).ceil();
    Metrics::new(font_size, line_height)
}

fn set_buffer_text(
    font_system: &mut FontSystem,
    collect_spans: &mut [(&'static str, Attrs)],
    buffer: &mut Buffer,
) {
    let attrs = Attrs::new();
    attrs.family(Family::SansSerif);

    buffer.set_rich_text(
        font_system,
        collect_spans.iter().copied(),
        attrs,
        Shaping::Advanced,
    )
}
