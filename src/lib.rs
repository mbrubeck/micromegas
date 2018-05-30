extern crate euclid;
extern crate harfbuzz;
extern crate unic_ucd_category;
extern crate unic_emoji_char;
extern crate unicode_bidi as bidi;
extern crate unicode_script;

mod font;
mod word_break;
mod script;

pub use font::{FontCollection, FontFamily, FontStyle, Typeface};

use script::script_runs;

pub fn layout_line<T>(text: &str, style: FontStyle, fonts: &FontCollection<T>)
    where T: Typeface
{
    let bidi_info = bidi::BidiInfo::new(text, None);

    // Expect a single paragraph and treat it as one line.
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
            layout_word(word, style, fonts, bidi_level);
        }
    }
}

// TODO: caching
pub fn layout_word<T>(
    word: &str,
    style: FontStyle,
    fonts: &FontCollection<T>,
    _bidi_level: bidi::Level,
)
where T: Typeface
{
    for (font, range) in font::itemize(word, style, fonts) {
        let hb_font = font.to_hb_font(); // TODO: cache
        let font_run = &word[range];
        for (script, script_run) in script_runs(font_run) {
            let mut buf = harfbuzz::Buffer::with(script_run);
            buf.set_script(script.to_hb_script());
            let glyphs = buf.shape(&hb_font, &harfbuzz::Features::default());
        }
    }
}
