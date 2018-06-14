extern crate micromegas;
extern crate pathfinder_font_renderer;

use {
    micromegas::{Font, FontCollection, FontFamily, FontStyle, Layout},
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

    let mut layout = Layout::new();
    layout.push("hello", style, &fonts);

    for glyph in layout.glyphs() {
        println!("{}: ({}, {})", glyph.id, glyph.x, glyph.y);
    }
}

