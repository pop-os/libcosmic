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
    pub phantom: PhantomData<&'a T>,
}

pub struct FontLayoutLine<'a> {
    pub line_i: FontLineIndex,
    pub glyphs: Vec<FontLayoutGlyph<'a, ()>>,
}

impl<'a> FontLayoutLine<'a> {
    pub fn draw<F: FnMut(i32, i32, f32)>(&self, mut f: F) {
        for glyph in self.glyphs.iter() {
            #[cfg(feature = "ab_glyph")]
            if let Some(ref outline) = glyph.inner {
                let bb = outline.px_bounds();
                let x = bb.min.x as i32;
                let y = bb.min.y as i32;
                outline.draw(|off_x, off_y, v| {
                    f(x + off_x as i32, y + off_y as i32, v);
                });
            }

            #[cfg(feature = "rusttype")]
            if let Some(bb) = glyph.inner.pixel_bounding_box() {
                let x = bb.min.x;
                let y = bb.min.y;
                glyph.inner.draw(|off_x, off_y, v| {
                    f(x + off_x as i32, y + off_y as i32, v);
                });
            }
        }
    }
}
