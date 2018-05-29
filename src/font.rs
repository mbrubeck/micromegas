use euclid::Rect;
use harfbuzz;

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

impl Fakery {
    fn new<T>(wanted: FontStyle, actual: &Font<T>) -> Self {
        Fakery {
            fake_bold: wanted.weight >= 6 && (wanted.weight - actual.style.weight) >= 2,
            fake_italic: wanted.italic && !actual.style.italic,
        }
    }
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

    // TODO: No longer needed when cached coverage is implemented.
    fn has_glyph(&self, c: char) -> bool;
}

pub struct Font<T> {
    pub typeface: T,
    pub style: FontStyle,
}

pub struct FakedFont<'a, T: 'a> {
    pub font: &'a Font<T>,
    pub fakery: Fakery,
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

    pub fn has_glyph(&self, c: char, /* _variation: u32 */) -> bool {
        self.closest_match(FontStyle::default()).font.typeface.has_glyph(c)
        // TODO: Use cached coverage tables instead of querying the typeface each time.
        // TODO: Support variation selectors.
    }

    fn closest_match(&self, style: FontStyle) -> FakedFont<T> {
        let font = self.fonts.iter().min_by_key(|f| f.style.difference(&style)).unwrap();
        let fakery = Fakery::new(style, &font);
        FakedFont { font, fakery }
    }
}

pub struct FontCollection<T> {
    pub families: Vec<FontFamily<T>>,
}

/// Break the text into runs of characters that can each be rendered with a single font from the
/// given FontCollection.
pub fn itemize<'a, T>(
    _text: &str,
    _style: FontStyle,
    _fonts: &'a FontCollection<T>,
) -> Vec<(FakedFont<'a, T>, &'a str)>
    where T: Typeface
{
    Vec::new()
}
