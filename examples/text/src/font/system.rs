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
                &face.post_script_name,
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

        if ! fonts.is_empty() {
            Some(FontMatches {
                fonts
            })
        } else {
            None
        }
    }
}
