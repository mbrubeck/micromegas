extern crate euclid;
extern crate harfbuzz;
extern crate unic_ucd_category;
extern crate unic_emoji_char;
extern crate unicode_bidi;
extern crate unicode_script;

mod font;
pub mod platform;
mod word_break;
mod script;
mod layout;

pub use font::{FontCollection, FontFamily, FontStyle, Typeface};
pub use layout::{layout_line, Layout, LayoutGlyph};
