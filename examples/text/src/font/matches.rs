use super::{Font, FontLineIndex, FontShapeGlyph, FontShapeWord, FontShapeLine, FontShapeSpan};

pub struct FontMatches<'a> {
    pub fonts: Vec<&'a Font<'a>>,
}

impl<'a> FontMatches<'a> {
    fn shape_word(&self, font_i: usize, line: &str, start_word: usize, end_word: usize, span_rtl: bool, blank: bool) -> FontShapeWord {
        let word = &line[start_word..end_word];

        let font_scale = self.fonts[font_i].rustybuzz.units_per_em() as f32;

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.set_direction(if span_rtl {
            rustybuzz::Direction::RightToLeft
        } else {
            rustybuzz::Direction::LeftToRight
        });
        buffer.push_str(word);
        buffer.guess_segment_properties();

        let rtl = match buffer.direction() {
            rustybuzz::Direction::RightToLeft => true,
            //TODO: other directions?
            _  => false,
        };
        assert_eq!(rtl, span_rtl);

        if font_i == 0 {
            println!(
                "    Word {}{}: '{}'",
                if rtl { "RTL" } else { "LTR" },
                if blank { " BLANK" } else { "" },
                word
            );
        }

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
        if rtl {
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
        } else {
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
        }

        //TODO: improve performance!
        if !missing.is_empty() && font_i + 1 < self.fonts.len() {
            let mut missing_i = 0;
            while missing_i < missing.len() {
                let mut missing_glyph = missing[missing_i];
                missing_i += 1;

                let mut start_glyph = missing_glyph;

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
                    if glyphs[i].start == missing_glyph {
                        let glyph = glyphs.remove(i);
                        if glyph.start < start_glyph {
                            start_glyph = glyph.start;
                        }
                        if glyph.end > end_glyph {
                            end_glyph = glyph.end;
                        }
                    } else if missing_i < missing.len() {
                        // Combine repeated missing items
                        if glyphs[i].start == missing[missing_i] {
                            missing_glyph = missing[missing_i];
                            missing_i += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let mut fb_word = self.shape_word(font_i + 1, line, start_glyph, end_glyph, span_rtl, blank);

                // Insert all matching glyphs
                // println!("Locate fallback for {},{} '{}' from font {} to font {}", start_glyph, end_glyph, &line[start_glyph..end_glyph], font_i + 1, font_i);
                let mut j = 0;
                while j < fb_word.glyphs.len() {
                    if fb_word.glyphs[j].start >= start_glyph && fb_word.glyphs[j].end <= end_glyph {
                        // println!("Copy fallback for {},{} '{}' from font {} cluster {} to font {}", start_glyph, end_glyph, &line[start_glyph..end_glyph], font_i + 1, fb_word.glyphs[j].start, font_i);
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

        FontShapeWord { blank, glyphs }
    }

    fn shape_span(&self, line: &str, start_span: usize, end_span: usize, line_rtl: bool, span_rtl: bool) -> FontShapeSpan {
        let span = &line[start_span..end_span];

        println!("  Span {}: '{}'", if span_rtl { "RTL" } else { "LTR" }, span);

        let mut words = Vec::new();

        let mut start_word = 0;
        for (end_lb, _) in unicode_linebreak::linebreaks(span) {
            let mut start_lb = end_lb;
            if end_lb != span.len() {
                while start_lb > 1 {
                    start_lb -= 1;
                    if span.is_char_boundary(start_lb) {
                        break;
                    }
                }
            }
            words.push(self.shape_word(0, line, start_span + start_word, start_span + start_lb, span_rtl, false));
            if start_lb < end_lb {
                words.push(self.shape_word(0, line, start_span + start_lb, start_span + end_lb, span_rtl, true));
            }
            start_word = end_lb;
        }

        // Reverse glyphs in RTL lines
        if line_rtl {
            for word in words.iter_mut() {
                word.glyphs.reverse();
            }
        }

        // Reverse words in spans that do not match line direction
        if line_rtl != span_rtl {
            words.reverse();
        }

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
            let line_rtl = para_info.level.is_rtl();

            println!("Line {}: '{}'", if line_rtl { "RTL" } else { "LTR" }, line);

            let paragraph = unicode_bidi::Paragraph::new(&bidi, &para_info);

            let mut start = 0;
            let mut span_rtl = line_rtl;
            for i in paragraph.para.range.clone() {
                let next_rtl = paragraph.info.levels[i].is_rtl();
                if span_rtl != next_rtl {
                    spans.push(self.shape_span(line, start, i, line_rtl, span_rtl));
                    span_rtl = next_rtl;
                    start = i;
                }
            }
            spans.push(self.shape_span(line, start, line.len(), line_rtl, span_rtl));

            line_rtl
        };

        FontShapeLine {
            line_i,
            rtl,
            spans,
        }
    }
}
