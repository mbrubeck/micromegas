extern crate freetype;

use euclid::Rect;
use font::{Options, Typeface};
use harfbuzz;
use harfbuzz_sys::{
    hb_blob_create,
    hb_blob_t,
    hb_face_create,
    hb_face_create_for_tables,
    hb_face_destroy,
    hb_face_get_upem,
    hb_face_t,
    hb_font_create,
    hb_font_create_sub_font,
    hb_font_destroy,
    hb_font_set_scale,
    hb_ot_font_set_funcs,
    HB_MEMORY_MODE_READONLY,
};
use pathfinder_font_renderer::freetype::{Face, ToFtF26Dot6};
use self::freetype::succeeded;
use self::freetype::freetype::{FT_Get_Char_Index, FT_ULong, FT_Face, FT_Load_Sfnt_Table, FT_Set_Char_Size};
use std::ptr;
use std::os::raw::{c_char, c_void};

impl<'a> Typeface for &'a Face {
    fn h_advance(&self, _glyph: u32, _: &Options) -> f32 {
        unimplemented!()
    }

    fn bounds(&self, _glyph: u32, _: &Options) -> Rect<f32> {
        unimplemented!()
    }

    fn has_glyph(&self, c: char) -> bool {
        unsafe { FT_Get_Char_Index(self.as_native(), c as FT_ULong) != 0 }
    }

    fn to_hb_font(&self, options: &Options) -> harfbuzz::Font {
        unsafe {
            FT_Set_Char_Size(self.as_native(), options.size.to_ft_f26dot6(), 0, 72, 0);
            let face = if let Some(bytes) = self.as_bytes() {
                let blob = harfbuzz::Blob::new_read_only(&bytes[..]);
                hb_face_create(blob.as_raw(), self.font_index())
            } else {
                hb_face_create_for_tables(Some(get_table), self.as_native() as *mut c_void, None)
            };
            let parent_font = hb_font_create(face);
            hb_ot_font_set_funcs(parent_font);

            let upem = hb_face_get_upem(face);
            hb_font_set_scale(parent_font, upem as i32, upem as i32);

            let font = hb_font_create_sub_font(parent_font);
            hb_font_destroy(parent_font);
            hb_face_destroy(face);
            harfbuzz::Font::from_raw(font)
        }
    }
}

unsafe extern "C" fn get_table(_: *mut hb_face_t, tag: u32, user_data: *mut c_void) -> *mut hb_blob_t {
    let face = user_data as FT_Face;
    let tag = tag as u64;

    // Get the length.
    let mut len = 0;
    if !succeeded(FT_Load_Sfnt_Table(face, tag, 0, ptr::null_mut(), &mut len)) {
        return ptr::null_mut();
    }
    // Get the data.
    let mut buf = Box::new(vec![0; len as usize]);
    if !succeeded(FT_Load_Sfnt_Table(face, tag, 0, buf.as_mut_ptr(), &mut len)) {
        return ptr::null_mut();
    }

    let data_ptr = buf.as_ptr() as *const c_char;
    let user_data = Box::into_raw(buf) as *mut c_void;
    unsafe extern "C" fn destroy(user_data: *mut c_void) {
         Box::from_raw(user_data as *mut Vec<u8>);
    }
    hb_blob_create(data_ptr, len as u32, HB_MEMORY_MODE_READONLY, user_data, Some(destroy))
}
