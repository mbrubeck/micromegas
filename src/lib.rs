extern crate euclid;
extern crate harfbuzz;
extern crate unic_ucd_category;
extern crate unic_emoji_char;
extern crate unicode_bidi as bidi;

pub use font::{FontCollection, FontFamily, FontStyle, Typeface};

mod font;
mod word_break;

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

pub fn layout_word<T>(
    word: &str,
    style: FontStyle,
    fonts: &FontCollection<T>,
    _bidi_level: bidi::Level,
) -> f32
    where T: Typeface
{
    let mut advance = 0.0;

    for (_font, _run) in font::itemize(word, style, fonts) {
    }

    advance
}
