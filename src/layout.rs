use font::{self, FakedFont, FontCollection, FontStyle, Options, Typeface};
use harfbuzz::{self, Direction::*};
use script::script_runs;
use unicode_bidi as bidi;
use word_break;

#[derive(Debug, Clone)]
pub struct LayoutGlyph<'a, T: 'a> {
    pub x: f32,
    pub y: f32,
    pub id: u32,

    // TODO: Move font info to Layout, to avoid storing it for every glyph.
    pub font: FakedFont<'a, T>,
}

#[derive(Debug, Clone)]
pub struct Layout<'a, T: 'a> {
    advance: f32,
    advances: Vec<f32>,
    glyphs: Vec<LayoutGlyph<'a, T>>,
}

impl<'a, T> Layout<'a, T> where T: Typeface {
    pub fn new() -> Self {
        Layout {
            advance: 0.,
            advances: Vec::new(),
            glyphs: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        text: &str,
        style: FontStyle,
        fonts: &'a FontCollection<T>,
        options: &Options,
    ) {
        // Expect a single paragraph and treat it as one line.
        // TODO: Implement line breaking.
        let bidi_info = bidi::BidiInfo::new(text, None);
        assert_eq!(bidi_info.paragraphs.len(), 1);
        let paragraph = &bidi_info.paragraphs[0];
        let line = paragraph.range.clone();

        // Iterate over bidi runs in visual order.
        let (bidi_levels, bidi_runs) = bidi_info.visual_runs(paragraph, line);
        for bidi_run in bidi_runs {
            println!("bidi run {}", &text[bidi_run.clone()]);
            let bidi_level = bidi_levels[bidi_run.start];

            // Split each bidi run into "words" for caching purposes. If the same word occurs
            // frequently, we can cache its layout rather than re-shaping it every time.
            for word in word_break::simple(&text[bidi_run]) {
                println!("  word {}", word);
                self.push_word(word, style, fonts, options, bidi_level);
            }
        }
    }

    pub fn glyphs(&self) -> &[LayoutGlyph<T>] {
        &self.glyphs
    }

    // TODO: caching
    fn push_word(
        &mut self,
        word: &str,
        style: FontStyle,
        fonts: &'a FontCollection<T>,
        options: &Options,
        bidi_level: bidi::Level,
    ) {
        // Iterate over same-font runs within the word.
        let mut font_runs = font::itemize(word, style, fonts);
        if bidi_level.is_rtl() {
            font_runs.reverse();
        }

        for (font, range) in font_runs {
            let hb_font = font.font.typeface.to_hb_font(&options); // TODO: cache the hb_font
            let font_run = &word[range];

            // Iterate over same-script runs within the font run.
            // TODO: Reverse order if RTL. Minikin does not do this yet because it is "unlikely"
            // with the current font stack to have multiple script runs within an RTL font run.
            for (script, script_run) in script_runs(font_run) {
                println!("    script_run {}", script_run);
                // TODO: Re-use the harfbuzz buffer.
                let mut buf = harfbuzz::Buffer::with(script_run);
                buf.set_script(script.to_hb_script());
                buf.set_direction(if bidi_level.is_rtl() { RTL } else { LTR });

                // Get glyph info from the shaper and append it to the Layout.
                let glyphs = buf.shape(&hb_font, &options.features);
                self.glyphs.reserve(glyphs.len());
                self.advances.reserve(glyphs.len());

                for glyph in glyphs {
                    self.glyphs.push(LayoutGlyph {
                        x: glyph.x_offset() as f32 + self.advance,
                        y: glyph.y_offset() as f32,
                        id: glyph.id(),
                        font,
                    });
                    let advance = glyph.x_advance() as f32;
                    self.advances.push(advance);
                    self.advance += advance;
                    // TODO: letter-spacing.
                    // TODO: Record glyph bounding boxes in the Layout.
                }
            }
        }
    }

    pub fn append(&mut self, other: Self) {
        self.glyphs.reserve(other.glyphs.len());
        for mut glyph in other.glyphs {
            glyph.x += self.advance;
            self.glyphs.push(glyph);
        }
        self.advances.extend(other.advances);
        self.advance += other.advance;
    }
}
