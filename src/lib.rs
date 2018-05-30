extern crate euclid;
extern crate harfbuzz;
extern crate unic_ucd_category;
extern crate unic_emoji_char;
extern crate unicode_bidi as bidi;
extern crate unicode_script;

pub use font::{FontCollection, FontFamily, FontStyle, Typeface};

use unicode_script::Script;

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
)
where T: Typeface
{
    for (_font, range) in font::itemize(word, style, fonts) {
        let font_run = &word[range];
        for (_script, _script_run) in script_runs(font_run) {
            // ....
        }
    }
}

struct ScriptRuns<'a> {
    text: &'a str,
    script: Script,
    pos: usize,
}

impl<'a> Iterator for ScriptRuns<'a> {
    type Item = (Script, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        for (i, c) in self.text[self.pos..].char_indices() {
            let script = unicode_script::get_script(c);
            if script != self.script {
                match self.script {
                    Script::Unknown | Script::Inherited | Script::Common => {
                        self.script = script;
                        continue;
                    }
                    _ => {}
                }
                match script {
                    Script::Inherited | Script::Common => continue,
                    _ => {}
                }
                let start = self.pos;
                self.pos = i;
                self.script = script;
                return Some((script, &self.text[start..i]));
            }
        }
        if self.pos < self.text.len() {
            return Some((self.script, &self.text[self.pos..]))
        }
        None
    }
}

fn script_runs(text: &str) -> ScriptRuns {
    ScriptRuns { text, script: Script::Unknown, pos: 0 }
}
