use harfbuzz;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct FontStyle {
    weight: u32,
    variant: u32,
    italic: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct FontFakery {
    fake_bold: bool,
    fake_italic: bool,
}

#[derive(Clone)]
struct Options {
    // font: &Font,
    size: f32,
    scale_x: f32,
    skew_x: f32,
    letter_spacing: f32,
    word_spacing: f32,
    // paintFlags
    fakery: FontFakery,
    // hyphenEdit
    features: harfbuzz::Features,
}
