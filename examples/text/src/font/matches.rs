use super::{Font, FontShapeGlyph, FontShapeLine, FontShapeSpan};

pub struct FontMatches<'a> {
    pub fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    fn shape_span(&self, string: &str, start_span: usize, end_span: usize) -> FontShapeSpan {
        let span = &string[start_span..end_span];

        let mut spans_by_font = Vec::with_capacity(self.fonts.len());
        for (font_i, font) in self.fonts.iter().enumerate() {
            let font_scale = font.rustybuzz.units_per_em() as f32;

            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.push_str(span);
            buffer.guess_segment_properties();
            let direction = buffer.direction();
            if font_i == 0 {
                //println!("{:?}, {:?}: '{}'", script, direction, span);
            }

            let glyph_buffer = rustybuzz::shape(&font.rustybuzz, &[], buffer);
            let glyph_infos = glyph_buffer.glyph_infos();
            let glyph_positions = glyph_buffer.glyph_positions();

            let mut misses = 0;
            let mut glyphs = Vec::with_capacity(glyph_infos.len());
            for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
                let x_advance = pos.x_advance as f32 / font_scale;
                let y_advance = pos.y_advance as f32 / font_scale;
                let x_offset = pos.x_offset as f32 / font_scale;
                let y_offset = pos.y_offset as f32 / font_scale;

                //println!("  {:?} {:?}", info, pos);
                if info.glyph_id == 0 {
                    misses += 1;
                }

                #[cfg(feature = "ab_glyph")]
                let inner = ab_glyph::GlyphId(info.glyph_id as u16);

                #[cfg(feature = "rusttype")]
                let inner = font.rusttype.glyph(rusttype::GlyphId(info.glyph_id as u16));

                glyphs.push(FontShapeGlyph {
                    start: start_span + info.cluster as usize,
                    end: end_span, // Set later
                    x_advance,
                    y_advance,
                    x_offset,
                    y_offset,
                    #[cfg(feature = "ab_glyph")]
                    font: &font.ab_glyph,
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

            let span = FontShapeSpan {
                direction,
                glyphs
            };
            if misses == 0 {
                return span;
            } else {
                spans_by_font.push((misses, span));
            }
        }

        let mut least_misses_i = 0;
        let mut least_misses = usize::MAX;
        for (i, (misses, _)) in spans_by_font.iter().enumerate() {
            if *misses < least_misses {
                least_misses_i = i;
                least_misses = *misses;
            }
        }

        if least_misses_i > 0 {
            //println!("MISSES {}, {}", least_misses_i, least_misses);
        }

        spans_by_font.remove(least_misses_i).1
    }

    fn shape_line(&self, string: &str, start_line: usize, end_line: usize) -> FontShapeLine {
        use unicode_script::{Script, UnicodeScript};

        let line = &string[start_line..end_line];

        //TODO: more special handling of characters
        let mut spans = Vec::new();

        let mut start = 0;
        let mut prev = Script::Unknown;
        for (i, c) in line.char_indices() {
            if ! line.is_char_boundary(i) {
                continue;
            }

            let cur = c.script();
            if prev != cur && prev != Script::Unknown {
                // No combination, start new span
                spans.push(self.shape_span(string, start_line + start, start_line + i));
                start = i;
                prev = Script::Unknown;
            } else {
                prev = cur;
            }
        }

        spans.push(self.shape_span(string, start_line + start, start_line + line.len()));

        let bidi = unicode_bidi::BidiInfo::new(line, None);
        let rtl = if bidi.paragraphs.is_empty() {
            false
        } else {
            assert_eq!(bidi.paragraphs.len(), 1);
            bidi.paragraphs[0].level.is_rtl()
        };

        FontShapeLine {
            rtl,
            spans,
        }
    }

    pub fn shape(&self, string: &str) -> Vec<FontShapeLine> {
        let mut lines = Vec::new();

        let mut start = 0;
        for (i, c) in string.char_indices() {
            if ! string.is_char_boundary(i) {
                continue;
            }

            if c == '\n' {
                lines.push(self.shape_line(string, start, i));
                start = i + 1;
            }
        }

        lines.push(self.shape_line(string, start, string.len()));

        lines
    }
}
