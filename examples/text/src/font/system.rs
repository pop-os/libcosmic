use std::ops::Deref;

use super::{Font, FontMatches};

pub struct FontSystem {
    db: fontdb::Database,
}

impl FontSystem {
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        let now = std::time::Instant::now();
        db.load_system_fonts();
        //TODO: configurable default fonts
        db.set_monospace_family("Fira Mono");
        db.set_sans_serif_family("Fira Sans");
        db.set_serif_family("DejaVu Serif");
        println!(
            "Loaded {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );

        //TODO only do this on demand!
        assert_eq!(db.len(), db.faces().len());
        for i in 0..db.len() {
            let id = db.faces()[i].id;
            unsafe {
                db.make_shared_face_data(id);
            }
        }

        Self {
            db,
        }
    }

    pub fn matches<'a, F: Fn(&fontdb::FaceInfo) -> bool>(&'a self, f: F) -> Option<FontMatches<'a>> {
        let mut fonts = Vec::new();
        for face in self.db.faces() {
            if ! f(face) {
                continue;
            }

            let font_opt = Font::new(
                match &face.source {
                    fontdb::Source::Binary(data) => {
                        data.deref().as_ref()
                    },
                    fontdb::Source::File(path) => {
                        println!("Unsupported fontdb Source::File('{}')", path.display());
                        continue;
                    },
                    fontdb::Source::SharedFile(_path, data) => {
                        data.deref().as_ref()
                    },
                },
                face.index,
            );

            match font_opt {
                Some(font) => fonts.push(font),
                None => {
                    eprintln!("failed to load font '{}'", face.post_script_name);
                }
            }
        }
        /*
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
        */
        if ! fonts.is_empty() {
            Some(FontMatches {
                fonts
            })
        } else {
            None
        }
    }
}
