use unicode_script::Script;

use super::Font;

#[cfg(not(any(macos, unix, windows)))]
use self::other::*;
#[cfg(not(any(macos, unix, windows)))]
mod other;

#[cfg(macos)]
use self::macos::*;
#[cfg(macos)]
mod macos;

#[cfg(unix)]
use self::unix::*;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
use self::windows::*;
#[cfg(windows)]
mod windows;

pub struct FontFallbackIter<'a> {
    fonts: &'a [Font<'a>],
    locale: &'a str,
    scripts: Vec<Script>,
    script_i: (usize, usize),
    common_i: usize,
    other_i: usize,
}

impl<'a> FontFallbackIter<'a> {
    pub fn new(fonts: &'a [Font<'a>], scripts: Vec<Script>, locale: &'a str) -> Self {
        Self {
            fonts,
            locale,
            scripts,
            script_i: (0, 0),
            common_i: 0,
            other_i: 0,
        }
    }
}

impl<'a> Iterator for FontFallbackIter<'a> {
    type Item = &'a Font<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.script_i.0 < self.scripts.len() {
            let script_families = script_fallback(&self.scripts[self.script_i.0], self.locale);
            while self.script_i.1 < script_families.len() {
                let script_family = script_families[self.script_i.1];
                self.script_i.1 += 1;
                for font in self.fonts.iter() {
                    if font.info.family == script_family {
                        return Some(font);
                    }
                }
            }
            self.script_i.0 += 1;
            self.script_i.1 = 0;
        }

        let common_families = common_fallback();
        while self.common_i < common_families.len() {
            let common_family = common_families[self.common_i];
            self.common_i += 1;
            for font in self.fonts.iter() {
                if font.info.family == common_family {
                    return Some(font);
                }
            }
        }

        //TODO: do we need to do this?
        //TODO: do not evaluate fonts more than once!
        while self.other_i < self.fonts.len() {
            let font = &self.fonts[self.other_i];
            self.other_i += 1;
            return Some(font);
        }

        None
    }
}
