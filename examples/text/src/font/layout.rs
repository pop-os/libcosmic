use core::marker::PhantomData;

use super::FontLineIndex;

pub struct FontLayoutGlyph<'a, T: 'a> {
    pub start: usize,
    pub end: usize,
    pub x: f32,
    pub w: f32,
    #[cfg(feature = "ab_glyph")]
    pub inner: Option<ab_glyph::OutlinedGlyph>,
    #[cfg(feature = "rusttype")]
    pub inner: rusttype::PositionedGlyph<'a>,
    #[cfg(feature = "swash")]
    pub inner: (i32, i32, Option<swash::scale::image::Image>),
    pub phantom: PhantomData<&'a T>,
}

pub struct FontLayoutLine<'a> {
    pub line_i: FontLineIndex,
    pub glyphs: Vec<FontLayoutGlyph<'a, ()>>,
}

impl<'a> FontLayoutLine<'a> {
    pub fn draw<F: FnMut(i32, i32, u32)>(&self, base: u32, mut f: F) {
        for glyph in self.glyphs.iter() {
            #[cfg(feature = "ab_glyph")]
            if let Some(ref outline) = glyph.inner {
                let bb = outline.px_bounds();
                let x = bb.min.x as i32;
                let y = bb.min.y as i32;
                outline.draw(|off_x, off_y, v| {
                    //TODO: ensure v * 255.0 does not overflow!
                    let color =
                        ((v * 255.0) as u32) << 24 |
                        base & 0xFFFFFF;
                    f(x + off_x as i32, y + off_y as i32, color);
                });
            }

            #[cfg(feature = "rusttype")]
            if let Some(bb) = glyph.inner.pixel_bounding_box() {
                let x = bb.min.x;
                let y = bb.min.y;
                glyph.inner.draw(|off_x, off_y, v| {
                    //TODO: ensure v * 255.0 does not overflow!
                    let color =
                        ((v * 255.0) as u32) << 24 |
                        base & 0xFFFFFF;
                    f(x + off_x as i32, y + off_y as i32, color);
                });
            }

            #[cfg(feature = "swash")]
            if let Some(ref image) = glyph.inner.2 {
                use swash::scale::image::Content;

                let x = glyph.inner.0 + image.placement.left;
                let y = glyph.inner.1 - image.placement.top;

                match image.content {
                    Content::Mask => {
                        let mut i = 0;
                        for off_y in 0..image.placement.height as i32 {
                            for off_x in 0..image.placement.width as i32 {
                                let color =
                                    (image.data[i] as u32) << 24 |
                                    base & 0xFFFFFF;
                                f(x + off_x, y + off_y, color);
                                i += 1;
                            }
                        }
                    },
                    Content::Color => {
                        let mut i = 0;
                        for off_y in 0..image.placement.height as i32 {
                            for off_x in 0..image.placement.width as i32 {
                                let color =
                                    (image.data[i + 3] as u32) << 24 |
                                    (image.data[i] as u32) << 16 |
                                    (image.data[i + 1] as u32) << 8 |
                                    (image.data[i + 2] as u32);
                                f(x + off_x, y + off_y, color);
                                i += 4;
                            }
                        }
                    },
                    Content::SubpixelMask => {
                        println!("TODO: SubpixelMask");
                    }
                }
            }
        }
    }
}
