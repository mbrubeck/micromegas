extern crate harfbuzz;
extern crate unicode_bidi as bidi;

mod font;
mod word_break;

pub fn layout_line(text: &str) {
    let bidi_info = bidi::BidiInfo::new(text, None);

    // Expect a single paragraph and treat it as one line.
    assert_eq!(bidi_info.paragraphs.len(), 1);
    let paragraph = &bidi_info.paragraphs[0];
    let line = paragraph.range.clone();

    let (bidi_levels, bidi_runs) = bidi_info.visual_runs(paragraph, line);
    for bidi_run in bidi_runs {
        let bidi_level = bidi_levels[bidi_run.start];
        for word in word_break::simple(&text[bidi_run]) {
            layout_word(word, bidi_level);
        }
    }
}

pub fn layout_word(word: &str, bidi_level: bidi::Level) -> f32 {
    let mut advance = 0.0;

    // for font_run in font_collection.itemize(word, style)

    advance
}
