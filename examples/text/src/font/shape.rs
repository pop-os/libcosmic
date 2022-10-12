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
    #[cfg(feature = "swash")]
    pub font: &'a swash::FontRef<'a>,
    #[cfg(feature = "swash")]
    pub inner: swash::GlyphId,
}

impl<'a> FontShapeGlyph<'a> {
    fn layout(&self, font_size: i32, x: f32, y: f32) -> FontLayoutGlyph<'a, ()> {
        let x_offset = font_size as f32 * self.x_offset;
        let y_offset = font_size as f32 * self.y_offset;
        let x_advance = font_size as f32 * self.x_advance;

        #[cfg(feature = "ab_glyph")]
        let inner = self.font.outline_glyph(
            self.inner.with_scale_and_position(
                font_size as f32,
                ab_glyph::point(
                    x + x_offset,
                    y - y_offset,
                )
            )
        );

        #[cfg(feature = "rusttype")]
        let inner = self.inner.clone()
            .scaled(rusttype::Scale::uniform(font_size as f32))
            .positioned(rusttype::point(
                x + x_offset,
                y - y_offset,
            ));

        #[cfg(feature = "swash")]
        let inner = {
            use swash::scale::{Render, ScaleContext, Source, StrikeWith};
            use swash::zeno::{Format, Vector};

            //TODO: store somewhere else
            static mut CONTEXT: Option<ScaleContext> = None;

            unsafe {
                if CONTEXT.is_none() {
                    CONTEXT = Some(ScaleContext::new());
                }
            }

            // Build the scaler
            let mut scaler = unsafe { CONTEXT.as_mut().unwrap() }
                .builder(*self.font)
                .size(font_size as f32)
                .hint(true)
                .build();

            // Compute the fractional offset-- you'll likely want to quantize this
            // in a real renderer
            let offset = Vector::new((x + x_offset).fract(), (y - y_offset).fract());

            // Select our source order
            let image_opt = Render::new(&[
                // Color outline with the first palette
                Source::ColorOutline(0),
                // Color bitmap with best fit selection mode
                Source::ColorBitmap(StrikeWith::BestFit),
                // Standard scalable outline
                Source::Outline,
            ])
                // Select a subpixel format
                .format(Format::Alpha)
                // Apply the fractional offset
                .offset(offset)
                // Render the image
                .render(&mut scaler, self.inner);
            ((x + x_offset).trunc() as i32, (y - y_offset).trunc() as i32, image_opt)
        };

        FontLayoutGlyph {
            start: self.start,
            end: self.end,
            x: x,
            w: x_advance,
            inner,
            phantom: PhantomData,
        }
    }
}

pub struct FontShapeWord<'a> {
    pub blank: bool,
    pub glyphs: Vec<FontShapeGlyph<'a>>,
}

pub struct FontShapeSpan<'a> {
    pub rtl: bool,
    pub words: Vec<FontShapeWord<'a>>,
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

        let start_x = if self.rtl { line_width as f32 } else { 0.0 };
        let end_x = if self.rtl { 0.0 } else { line_width as f32 };
        let mut x = start_x;
        let mut y = 0.0;
        for span in self.spans.iter() {
            //TODO: improve performance!
            let mut word_ranges = Vec::new();
            if self.rtl != span.rtl {
                let mut fit_x = x;
                let mut fitting_end = span.words.len();
                for i in (0..span.words.len()).rev() {
                    let word = &span.words[i];

                    let mut word_size = 0.0;
                    for glyph in word.glyphs.iter() {
                        word_size += font_size as f32 * glyph.x_advance;
                    }

                    let wrap = if self.rtl {
                        fit_x - word_size < end_x
                    } else {
                        fit_x + word_size > end_x
                    };

                    if wrap {
                        let mut fitting_start = i + 1;
                        while fitting_start < fitting_end {
                            if span.words[fitting_start].blank {
                                fitting_start += 1;
                            } else {
                                break;
                            }
                        }
                        word_ranges.push((fitting_start..fitting_end, true));
                        fitting_end = i + 1;

                        fit_x = start_x;
                    }

                    if self.rtl {
                        fit_x -= word_size;
                    } else {
                        fit_x += word_size;
                    }
                }
                if ! word_ranges.is_empty() {
                    while fitting_end > 0 {
                        if span.words[fitting_end - 1].blank {
                            fitting_end -= 1;
                        } else {
                            break;
                        }
                    }
                }
                word_ranges.push((0..fitting_end, false));
            } else {
                let mut fit_x = x;
                let mut fitting_start = 0;
                for i in 0..span.words.len() {
                    let word = &span.words[i];

                    let mut word_size = 0.0;
                    for glyph in word.glyphs.iter() {
                        word_size += font_size as f32 * glyph.x_advance;
                    }

                    let wrap = if self.rtl {
                        fit_x - word_size < end_x
                    } else {
                        fit_x + word_size > end_x
                    };

                    if wrap {
                        //TODO: skip blanks
                        word_ranges.push((fitting_start..i, true));
                        fitting_start = i;

                        fit_x = start_x;
                    }

                    if self.rtl {
                        fit_x -= word_size;
                    } else {
                        fit_x += word_size;
                    }
                }
                word_ranges.push((fitting_start..span.words.len(), false));
            }

            for (range, wrap) in word_ranges {
                for word in span.words[range].iter() {
                    for glyph in word.glyphs.iter() {
                        let x_advance = font_size as f32 * glyph.x_advance;
                        let y_advance = font_size as f32 * glyph.y_advance;

                        if self.rtl {
                            x -= x_advance
                        }

                        glyphs.push(glyph.layout(font_size, x, y));
                        push_line = true;

                        if ! self.rtl {
                            x += x_advance;
                        }
                        y += y_advance;
                    }
                }

                if wrap {
                    let mut glyphs_swap = Vec::new();
                    std::mem::swap(&mut glyphs, &mut glyphs_swap);
                    layout_lines.insert(layout_i, FontLayoutLine {
                        line_i: self.line_i,
                        glyphs: glyphs_swap
                    });
                    layout_i += 1;

                    x = start_x;
                    y = 0.0;
                }
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
