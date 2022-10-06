#[cfg(feature = "ab_glyph")]
use ab_glyph::Font;
use core::marker::PhantomData;

use super::{FontLayoutGlyph, FontLayoutLine, FontLineIndex};

pub struct FontShapeGlyph<'a> {
    pub start: usize,
    pub end: usize,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    #[cfg(feature = "ab_glyph")]
    pub font: &'a ab_glyph::FontRef<'a>,
    #[cfg(feature = "ab_glyph")]
    pub inner: ab_glyph::GlyphId,
    #[cfg(feature = "rusttype")]
    pub inner: rusttype::Glyph<'a>,
}

pub struct FontShapeSpan<'a> {
    pub direction: rustybuzz::Direction,
    pub glyphs: Vec<FontShapeGlyph<'a>>,
}

pub struct FontShapeLine<'a> {
    pub line_i: FontLineIndex,
    pub rtl: bool,
    pub spans: Vec<FontShapeSpan<'a>>,
}

impl<'a> FontShapeLine<'a> {
    pub fn layout(&self, font_size: i32, line_width: i32, layout_lines: &mut Vec<FontLayoutLine<'a>>, mut layout_i: usize) {
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
                        layout_lines.push(FontLayoutLine {
                            line_i: self.line_i,
                            glyphs: glyphs_swap
                        });

                        x = line_width as f32 - x_advance;
                        y = 0.0;
                    }
                } else {
                    if x + x_advance > line_width as f32 {
                        let mut glyphs_swap = Vec::new();
                        std::mem::swap(&mut glyphs, &mut glyphs_swap);
                        layout_lines.insert(layout_i, FontLayoutLine {
                            line_i: self.line_i,
                            glyphs: glyphs_swap
                        });
                        layout_i += 1;

                        x = 0.0;
                        y = 0.0;
                    }
                }

                #[cfg(feature = "ab_glyph")]
                let inner = glyph.font.outline_glyph(
                    glyph.inner.with_scale_and_position(
                        font_size as f32,
                        ab_glyph::point(
                            x + x_offset,
                            y + y_offset,
                        )
                    )
                );

                #[cfg(feature = "rusttype")]
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
                    phantom: PhantomData,
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
            layout_lines.insert(layout_i, FontLayoutLine {
                line_i: self.line_i,
                glyphs
            });
        }
    }
}
