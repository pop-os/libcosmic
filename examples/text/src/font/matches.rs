use super::{Font, FontLineIndex, FontShapeGlyph, FontShapeWord, FontShapeLine, FontShapeSpan};

pub struct FontMatches<'a> {
    pub fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    fn shape_word(&self, font_i: usize, line: &str, start_word: usize, end_word: usize) -> FontShapeWord {
        let word = &line[start_word..end_word];

        let font_scale = self.fonts[font_i].rustybuzz.units_per_em() as f32;

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(word);
        buffer.guess_segment_properties();
        let direction = buffer.direction();

        let glyph_buffer = rustybuzz::shape(&self.fonts[font_i].rustybuzz, &[], buffer);
        let glyph_infos = glyph_buffer.glyph_infos();
        let glyph_positions = glyph_buffer.glyph_positions();

        let mut missing = Vec::new();
        let mut glyphs = Vec::with_capacity(glyph_infos.len());
        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            let x_advance = pos.x_advance as f32 / font_scale;
            let y_advance = pos.y_advance as f32 / font_scale;
            let x_offset = pos.x_offset as f32 / font_scale;
            let y_offset = pos.y_offset as f32 / font_scale;

            //println!("  {:?} {:?}", info, pos);
            if info.glyph_id == 0 {
                missing.push(start_word + info.cluster as usize);
            }

            #[cfg(feature = "ab_glyph")]
            let inner = ab_glyph::GlyphId(info.glyph_id as u16);

            #[cfg(feature = "rusttype")]
            let inner = font.rusttype.glyph(rusttype::GlyphId(info.glyph_id as u16));

            #[cfg(feature = "swash")]
            let inner = info.glyph_id as swash::GlyphId;

            glyphs.push(FontShapeGlyph {
                start: start_word + info.cluster as usize,
                end: end_word, // Set later
                x_advance,
                y_advance,
                x_offset,
                y_offset,
                #[cfg(feature = "ab_glyph")]
                font: &self.fonts[font_i].ab_glyph,
                #[cfg(feature = "swash")]
                font: &self.fonts[font_i].swash,
                inner,
            });
        }

        // Adjust end of glyphs
        match direction {
            rustybuzz::Direction::LeftToRight => {
                for i in (1..glyphs.len()).rev() {
                    let next_start = glyphs[i].start;
                    let next_end = glyphs[i].end;
                    let prev = &mut glyphs[i - 1];
                    if prev.start == next_start {
                        prev.end = next_end;
                    } else {
                        prev.end = next_start;
                    }
                }
            },
            rustybuzz::Direction::RightToLeft => {
                for i in 1..glyphs.len() {
                    let next_start = glyphs[i - 1].start;
                    let next_end = glyphs[i - 1].end;
                    let prev = &mut glyphs[i];
                    if prev.start == next_start {
                        prev.end = next_end;
                    } else {
                        prev.end = next_start;
                    }
                }
            },
            //TODO: other directions
            _ => (),
        }

        //TODO: improve performance!
        if !missing.is_empty() && font_i + 1 < self.fonts.len() {
            let mut fb_word = self.shape_word(font_i + 1, line, start_word, end_word);

            for start_glyph in missing {
                // Find beginning of glyphs to replace
                let mut i = 0;
                while i < glyphs.len() {
                    if glyphs[i].start == start_glyph {
                        break;
                    }
                    i += 1;
                }

                // Remove all matching glyphs and find end
                let mut end_glyph = start_glyph;
                while i < glyphs.len() {
                    if glyphs[i].start == start_glyph {
                        let glyph = glyphs.remove(i);
                        if glyph.end > end_glyph {
                            end_glyph = glyph.end;
                        }
                    } else {
                        break;
                    }
                }

                // Insert all matching glyphs
                let mut j = 0;
                while j < fb_word.glyphs.len() {
                    if fb_word.glyphs[j].start >= start_glyph && fb_word.glyphs[j].end <= end_glyph {
                        glyphs.insert(i, fb_word.glyphs.remove(j));
                        i += 1;
                    } else {
                        j += 1;
                    }
                }
            }
        }

        /*
        for glyph in glyphs.iter() {
            println!("'{}': {}, {}, {}, {}", &line[glyph.start..glyph.end], glyph.x_advance, glyph.y_advance, glyph.x_offset, glyph.y_offset);
        }
        */

        FontShapeWord { glyphs }
    }

    fn shape_span(&self, line: &str, start_span: usize, end_span: usize, para_rtl: bool, span_rtl: bool) -> FontShapeSpan {
        use unicode_script::{Script, UnicodeScript};

        let span = &line[start_span..end_span];

        println!("Span {}: '{}'", if span_rtl { "RTL" } else { "LTR" }, span);

        let mut words = vec![
            self.shape_word(0, line, start_span, end_span),
        ];

        if span_rtl {
            for word in words.iter_mut() {
                word.glyphs.reverse();
            }
        }

        //TODO: improve performance
        for (linebreak, _) in unicode_linebreak::linebreaks(span) {
            let mut glyphs_opt = None;
            'words: for word_i in 0..words.len() {
                for glyph_i in 0..words[word_i].glyphs.len() {
                    if words[word_i].glyphs[glyph_i].start == start_span + linebreak {
                        glyphs_opt = Some(words[word_i].glyphs.split_off(glyph_i));
                        break 'words;
                    }
                }
            }
            if let Some(glyphs) = glyphs_opt {
                words.push(FontShapeWord { glyphs });
            }
        }

        if span_rtl {
            for word in words.iter_mut() {
                word.glyphs.reverse();
            }
        }

        if para_rtl != span_rtl {
            words.reverse();
        }

        /*
        let mut word_script = Script::Unknown;
        for (i, c) in span.char_indices() {
            let next_script = c.script();
            if word_script != next_script && word_script != Script::Unknown {
                // No combination, start new span
                words.push(self.shape_word(line, start_span + start, start_span + i));
                start = i;
                word_script = Script::Unknown;
            } else {
                word_script = next_script;
            }
        }
        words.push(self.shape_word(line, start_span + start, end_span));
        */

        FontShapeSpan {
            rtl: span_rtl,
            words,
        }
    }

    pub fn shape_line(&self, line_i: FontLineIndex, line: &str) -> FontShapeLine {
        let mut spans = Vec::new();

        let bidi = unicode_bidi::BidiInfo::new(line, None);
        let rtl = if bidi.paragraphs.is_empty() {
            false
        } else {
            assert_eq!(bidi.paragraphs.len(), 1);
            let para_info = &bidi.paragraphs[0];
            let para_rtl = para_info.level.is_rtl();

            let paragraph = unicode_bidi::Paragraph::new(&bidi, &para_info);

            let mut start = 0;
            let mut span_rtl = para_rtl;
            for i in paragraph.para.range.clone() {
                let next_rtl = paragraph.info.levels[i].is_rtl();
                if span_rtl != next_rtl {
                    spans.push(self.shape_span(line, start, i, para_rtl, span_rtl));
                    span_rtl = next_rtl;
                    start = i;
                }
            }
            spans.push(self.shape_span(line, start, line.len(), para_rtl, span_rtl));

            para_rtl
        };

        FontShapeLine {
            line_i,
            rtl,
            spans,
        }
    }
}
