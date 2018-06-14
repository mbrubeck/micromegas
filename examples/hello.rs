extern crate harfbuzz;
extern crate micromegas;
extern crate pathfinder_font_renderer;

use {
    harfbuzz::Features,
    micromegas::{Font, FontCollection, FontFamily, FontStyle, Layout, Options},
    pathfinder_font_renderer::freetype::FontContext,
    std::sync::Arc,
};

const OPEN_SANS: &[u8] = include_bytes!("../resources/open-sans/OpenSans-Regular.ttf");

fn main() {
    let mut font_ctx = FontContext::new().unwrap();
    font_ctx.add_font_from_memory(&0, Arc::new(OPEN_SANS.to_vec()), 0).unwrap();

    let typeface = font_ctx.get_font(&0).unwrap();
    let style = FontStyle { weight: 400, variant: 0, italic: false };
    let font = Font { typeface, style };
    let family = FontFamily::new(vec![font]);
    let fonts = FontCollection { families: vec![family] };
    let options = Options { size: 12.0, features: Features::default() };

    let mut layout = Layout::new();
    layout.push("hello world", style, &fonts, &options);

    for glyph in layout.glyphs() {
        println!("{}: ({}, {})", glyph.id, glyph.x, glyph.y);
    }
}

