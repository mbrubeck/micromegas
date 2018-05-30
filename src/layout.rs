use font::{self, FakedFont, FontCollection, FontStyle, Typeface};
use harfbuzz;
use script::script_runs;
use unicode_bidi as bidi;
use word_break;

pub fn layout_line<'a, T>(text: &str, style: FontStyle, fonts: &'a FontCollection<T>) -> Layout<'a, T>
    where T: Typeface
{
    let mut layout = Layout::new();

    // Expect a single paragraph and treat it as one line.
    let bidi_info = bidi::BidiInfo::new(text, None);
    assert_eq!(bidi_info.paragraphs.len(), 1);
    let paragraph = &bidi_info.paragraphs[0];
    let line = paragraph.range.clone();

    // Iterate over bidi runs in visual order.
    let (bidi_levels, bidi_runs) = bidi_info.visual_runs(paragraph, line);
    for bidi_run in bidi_runs {
        let bidi_level = bidi_levels[bidi_run.start];

        // Split each bidi run into "words" for caching purposes. If the same word occurs
        // frequently, we can cache its layout rather than re-shaping it every time.
        for word in word_break::simple(&text[bidi_run]) {
            layout.layout_word(word, style, fonts, bidi_level);
        }
    }

    layout
}


#[derive(Debug, Clone)]
pub struct Layout<'a, T: 'a> {
    advances: Vec<f32>,
    glyphs: Vec<LayoutGlyph<'a, T>>,
}

impl<'a, T> Layout<'a, T> where T: Typeface {
    fn new() -> Self {
        Layout {
            advances: Vec::new(),
            glyphs: Vec::new(),
        }
    }

    pub fn glyphs(&self) -> &[LayoutGlyph<T>] {
        &self.glyphs
    }

    // TODO: caching
    pub fn layout_word(
        &mut self,
        word: &str,
        style: FontStyle,
        fonts: &FontCollection<T>,
        _bidi_level: bidi::Level,
    ) {
        // Iterate over same-font runs within the word.
        for (font, range) in font::itemize(word, style, fonts) {
            let hb_font = font.to_hb_font(); // TODO: cache the hb_font
            let font_run = &word[range];

            // Iterate over same-srcipt runs within the font run.
            for (script, script_run) in script_runs(font_run) {
                // TODO: Re-use the harfbuzz buffer.
                let mut buf = harfbuzz::Buffer::with(script_run);
                buf.set_script(script.to_hb_script());
                let glyphs = buf.shape(&hb_font, &harfbuzz::Features::default());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutGlyph<'a, T: 'a> {
    pub x: f32,
    pub y: f32,
    pub glyph_id: u32,

    // TODO: Move font info to Layout, to avoid storing it for every glyph.
    pub font: FakedFont<'a, T>,
}
