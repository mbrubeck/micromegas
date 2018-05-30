use euclid::Rect;
use harfbuzz;
use std::ops::Range;
use unic_emoji_char::{is_emoji_modifier, is_emoji_modifier_base};
use unic_ucd_category::GeneralCategory;

// TODO: Use bit-packing to reduce size.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FontStyle {
    pub weight: u32,
    pub variant: u32,
    pub italic: bool,
    // TODO: language list ID
}

impl FontStyle {
    /// Compute a matching metric between two styles. 0 is an exact match.
    fn difference(&self, other: &Self) -> u32 {
        self.weight - other.weight + if self.italic == other.italic { 0 } else { 2 }
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle { weight: 4, variant: 0, italic: false }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Fakery {
    pub fake_bold: bool,
    pub fake_italic: bool,
}

#[derive(Clone)]
pub struct Options {
    // font: &Font,
    pub size: f32,
    pub scale_x: f32,
    pub skew_x: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    // paintFlags
    pub fakery: Fakery,
    // hyphenEdit
    pub features: harfbuzz::Features,
}

pub trait Typeface {
    fn h_advance(&self, glyph: u32, options: Options) -> f32;
    fn bounds(&self, glyph: u32, options: Options) -> Rect<f32>;
    fn to_hb_font(&self) -> harfbuzz::Font;

    // TODO: No longer needed when cached coverage is implemented.
    fn has_glyph(&self, c: char) -> bool;
}

#[derive(Debug)]
pub struct Font<T> {
    pub typeface: T,
    pub style: FontStyle,
}

#[derive(Debug)]
pub struct FakedFont<'a, T: 'a> {
    pub font: &'a Font<T>,
    pub fakery: Fakery,
}

impl<'a, T> Copy for FakedFont<'a, T> {}

impl<'a, T> Clone for FakedFont<'a, T> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T> FakedFont<'a, T> {
    fn new(wanted: FontStyle, font: &'a Font<T>) -> Self {
        let fakery = Fakery {
            fake_bold: wanted.weight >= 6 && (wanted.weight - font.style.weight) >= 2,
            fake_italic: wanted.italic && !font.style.italic,
        };
        FakedFont { font, fakery }
    }
}

pub struct FontFamily<T> {
    fonts: Vec<Font<T>>,
    // lang_id
    // variant
    // supported_axes
    // coverage (cached bit set)
    // cmap_fmt14_coverage
}

impl<T> FontFamily<T> where T: Typeface {
    pub fn new(fonts: Vec<Font<T>>) -> Self {
        assert!(fonts.len() > 1, "FontFamily must contain at least one font");
        FontFamily { fonts }
    }

    pub fn has_glyph(&self, c: char) -> bool {
        self.closest_match(FontStyle::default()).font.typeface.has_glyph(c)
        // TODO: Use cached coverage tables instead of querying the typeface each time.
        // TODO: Support variation selectors.
    }

    fn closest_match(&self, style: FontStyle) -> FakedFont<T> {
        let font = self.fonts.iter().min_by_key(|f| f.style.difference(&style)).unwrap();
        FakedFont::new(style, font)
    }
}

pub struct FontCollection<T> {
    pub families: Vec<FontFamily<T>>,
}

impl<T> FontCollection<T> where T: Typeface {
    // TODO: lang list id, variant
    fn family_for_char(&self, c: char) -> &FontFamily<T> {
        // TODO: Calculate a score for each family based on variation, language, coverage.
        self.families.iter().find(|f| f.has_glyph(c)).unwrap_or(&self.families[0])
    }
}

/// Break the text into runs of characters that can each be rendered with a single font from the
/// given FontCollection.
pub fn itemize<'a, T>(
    text: &str,
    style: FontStyle,
    fonts: &'a FontCollection<T>,
) -> Vec<(FakedFont<'a, T>, Range<usize>)>
    where T: Typeface
{
    let mut result: Vec<(FakedFont<T>, Range<usize>)> = Vec::new();
    let mut last_family: Option<&FontFamily<T>> = None;
    let mut prev = '\0';

    for (i, c) in text.char_indices() {
        let mut should_continue_run = false;
        // Continue current run if c is a format character, or is whitelisted and has coverage.
        if does_not_need_font_support(c) {
            should_continue_run = true;
        }
        if let Some(last_family) = last_family {
            if (is_sticky_whitelisted(c) || is_combining(c)) && last_family.has_glyph(c) {
                should_continue_run = true;
            }
        }

        if !should_continue_run {
            let family = fonts.family_for_char(c);
            // Start a new run if a new font is found.
            if i == 0 || last_family.map(|f| f as *const _) != Some(family as *const _) {
                let mut start = i;
                // If a combining mark or emoji modifier is found in a different font that also
                // supports the previous character, move the previous character to the new run.
                if i != 0 && (is_combining(c) || is_emoji_modifier(c) || is_emoji_modifier_base(prev))
                    && family.has_glyph(c)
                {
                    let prev_len = prev.len_utf8();
                    start -= prev_len;
                    let mut is_empty = false;
                    if let Some((_, range)) = result.last_mut() {
                        range.end -= prev_len;
                        is_empty = range.start == range.end;
                    }
                    if is_empty {
                        result.pop();
                    }
                }
                if last_family.is_none() {
                    start = 0;
                }
                result.push((family.closest_match(style), start..start));
                last_family = Some(family);
            }
        }
        prev = c;
        if let Some((_, range)) = result.last_mut() {
            range.end = i + c.len_utf8();
        }
    }
    result
}

/// Characters where we want to continue using existing font run for (or stick to the next run if
/// they start a string), even if the font does not support them explicitly. These are handled
/// properly by Minikin or HarfBuzz even if the font does not explicitly support them and it's
/// usually meaningless to switch to a different font to display them.
fn does_not_need_font_support(c: char) -> bool {
    match c {
        '\u{00AD}' // SOFT HYPHEN
        | '\u{034F}' // COMBINING GRAPHEME JOINER
        | '\u{061C}' // ARABIC LETTER MARK
        | '\u{200C}'...'\u{200F}' // ZERO WIDTH NON-JOINER..RIGHT-TO-LEFT MARK
        | '\u{202A}'...'\u{202E}' // LEFT-TO-RIGHT EMBEDDING..RIGHT-TO-LEFT OVERRIDE
        | '\u{2066}'...'\u{2069}' // LEFT-TO-RIGHT ISOLATE..POP DIRECTIONAL ISOLATE
        | '\u{FEFF}' // BYTE ORDER MARK
        => true,
        _ => is_variation_selector(c)
    }
}

fn is_variation_selector(c: char) -> bool {
    is_bmp_variation_selector(c) || is_variation_selector_supplement(c)
}

const VS1: char = '\u{FE00}';
const VS16: char = '\u{FE0F}';
const VS17: char = '\u{E0100}';
const VS256: char = '\u{E01EF}';

fn is_bmp_variation_selector(c: char) -> bool {
    match c {
        VS1...VS16 => true,
        _ => false,
    }
}

fn is_variation_selector_supplement(c: char) -> bool {
    match c {
        VS17...VS256 => true,
        _ => false,
    }
}

/// Characters where we want to continue using existing font run instead of
/// recomputing the best match in the fallback list.
const STICKY_WHITELIST: &[char] = &[
    '!',
    ',',
    '-',
    '.',
    ':',
    ';',
    '?',
    '\u{00A0}', // NBSP
    '\u{2010}', // HYPHEN
    '\u{2011}', // NB_HYPHEN
    '\u{202F}', // NNBSP
    '\u{2640}', // FEMALE_SIGN,
    '\u{2642}', // MALE_SIGN,
    '\u{2695}', // STAFF_OF_AESCULAPIUS
];

fn is_sticky_whitelisted(c: char) -> bool {
    STICKY_WHITELIST.contains(&c)
}

fn is_combining(c: char) -> bool {
    GeneralCategory::of(c).is_mark()
}
