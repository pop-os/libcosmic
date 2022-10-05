use super::{Font, FontMatches};

pub struct FontSystem<'a> {
    fonts: Vec<Font<'a>>,
}

impl<'a> FontSystem<'a> {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
        }
    }

    pub fn add(&mut self, font: Font<'a>) {
        self.fonts.push(font);
    }

    pub fn matches(&'a self, patterns: &[&str]) -> Option<FontMatches<'a>> {
        let mut fonts = Vec::new();
        for font in self.fonts.iter() {
            for rec in font.rustybuzz.names() {
                if rec.name_id == 4 && rec.is_unicode() {
                    let mut words: Vec<u16> = Vec::new();

                    let mut i = 0;
                    while i + 1 < rec.name.len() {
                        words.push(
                            (rec.name[i + 1] as u16) |
                            ((rec.name[i] as u16) << 8)
                        );
                        i += 2;
                    }

                    match String::from_utf16(&words) {
                        Ok(name) => {
                            let mut matched = false;
                            for pattern in patterns.iter() {
                                println!("Matching font name '{}' with pattern '{}'", name, pattern);
                                if name.contains(pattern) {
                                    matched = true;
                                }
                            }
                            if matched {
                                println!("Matched font name '{}'", name);
                                fonts.push(font);
                            } else {
                                println!("Did not match font name '{}'", name);
                            }
                        },
                        Err(_) => ()
                    }
                }
            }
        }
        if ! fonts.is_empty() {
            Some(FontMatches { fonts })
        } else {
            None
        }
    }
}
