use std::{
    collections::HashMap,
    sync::Mutex,
};

pub use self::cache::*;
mod cache;

pub use self::layout::*;
mod layout;

pub use self::matches::*;
mod matches;

pub use self::shape::*;
mod shape;

pub use self::system::*;
mod system;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FontCacheKey {
    glyph_id: u16,

}

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
    pub name: &'a str,
    pub data: &'a [u8],
    pub index: u32,
    pub rustybuzz: rustybuzz::Face<'a>,
    #[cfg(feature = "ab_glyph")]
    pub ab_glyph: ab_glyph::FontRef<'a>,
    #[cfg(feature = "rusttype")]
    pub rusttype: rusttype::Font<'a>,
    #[cfg(feature = "swash")]
    pub swash: swash::FontRef<'a>,
    #[cfg(feature = "swash")]
    pub scale_context: Mutex<swash::scale::ScaleContext>,
    pub cache: Mutex<HashMap<CacheKey, CacheItem>>,
}

impl<'a> Font<'a> {
    pub fn new(name: &'a str, data: &'a [u8], index: u32) -> Option<Self> {
        Some(Self {
            name,
            data,
            index,
            rustybuzz: rustybuzz::Face::from_slice(data, index)?,
            #[cfg(feature = "ab_glyph")]
            ab_glyph: ab_glyph::FontRef::try_from_slice_and_index(data, index).ok()?,
            #[cfg(feature = "rusttype")]
            rusttype: rusttype::Font::try_from_bytes_and_index(data, index)?,
            #[cfg(feature = "swash")]
            swash: swash::FontRef::from_index(data, index as usize)?,
            #[cfg(feature = "swash")]
            scale_context: Mutex::new(swash::scale::ScaleContext::new()),
            cache: Mutex::new(HashMap::new()),
        })
    }
}
