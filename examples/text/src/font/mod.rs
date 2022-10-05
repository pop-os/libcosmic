pub use self::layout::*;
mod layout;

pub use self::matches::*;
mod matches;

pub use self::shape::*;
mod shape;

pub use self::system::*;
mod system;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct FontLineIndex(usize);

impl FontLineIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

pub struct Font<'a> {
    data: &'a [u8],
    pub rustybuzz: rustybuzz::Face<'a>,
    #[cfg(feature = "ab_glyph")]
    pub ab_glyph: ab_glyph::FontRef<'a>,
    #[cfg(feature = "rusttype")]
    pub rusttype: rusttype::Font<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8], index: u32) -> Option<Self> {
        Some(Self {
            data,
            rustybuzz: rustybuzz::Face::from_slice(data, index)?,
            #[cfg(feature = "ab_glyph")]
            ab_glyph: ab_glyph::FontRef::try_from_slice_and_index(data, index).ok()?,
            #[cfg(feature = "rusttype")]
            rusttype: rusttype::Font::try_from_bytes_and_index(data, index)?,
        })
    }
}
