use emacs_sys::bindings::font_info;
use emacs_sys::lisp::ExternalPtr;
use webrender::api::FontInstanceKey;
use webrender::api::GlyphDimensions;
use webrender::api::GlyphIndex;

use emacs_sys::bindings::assq_no_quit;
use emacs_sys::bindings::AREF;
use emacs_sys::bindings::XCAR;
use emacs_sys::bindings::XCDR;
use emacs_sys::bindings::XFIXNUM;
use emacs_sys::globals::QCfont_entity;
use std::sync::LazyLock;

use std::ptr;

use emacs_sys::bindings::font;
use emacs_sys::bindings::font_driver;
use emacs_sys::bindings::font_make_object;
use emacs_sys::bindings::font_metrics;
use emacs_sys::bindings::font_property_index;
use emacs_sys::bindings::frame;
use emacs_sys::bindings::ftfont_combining_capability;
use emacs_sys::bindings::ftfont_filter_properties;
use emacs_sys::bindings::ftfont_get_cache;
use emacs_sys::bindings::ftfont_list2;
use emacs_sys::bindings::ftfont_list_family;
use emacs_sys::bindings::ftfont_match2;
use emacs_sys::bindings::glyph_string;
use emacs_sys::bindings::register_font_driver;
use emacs_sys::globals::Qftwr;
use emacs_sys::globals::Qnil;
use emacs_sys::lisp::LispObject;

// pub type FontInfoRef = ExternalPtr<font_info>;

// pub trait FontInfoWrExt {
//     fn font_instance_key() -> FontInstanceKey;
//     fn glyph_dimensions(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<GlyphDimensions>>;
//     fn get_glyph_advance_widths(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<f32>>;
// }

// impl FontInfoWrExt for FontInfoRef {
//     fn font_instance_key() -> FontInstanceKey {
//         todo!()
//     }

//     // file:///home/declan/src/webrender/target/doc/webrender/render_api/struct.RenderApi.html#method.get_glyph_dimensions
//     // Note: Internally, the internal texture cache doesn’t store ‘empty’ textures (height or width = 0) This means that glyph dimensions e.g. for spaces (’ ’) will mostly be None.
//     fn glyph_dimensions(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<GlyphDimensions>> {
//         unimplemented!();
//     }

//     fn get_glyph_advance_widths(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<f32>> {
//         self.glyph_dimensions(glyph_indices)
//             .iter()
//             .map(|i| i.map(|d| d.advance))
//             .collect()
//     }
// }

pub struct FontDriver(pub font_driver);
unsafe impl Sync for FontDriver {}

static FONT_DRIVER: LazyLock<FontDriver> = LazyLock::new(|| {
    log::trace!("FONT_DRIVER is being created...");
    let mut font_driver = font_driver::default();

    font_driver.type_ = Qftwr;
    // font_driver.case_sensitive = true;
    font_driver.get_cache = Some(ftfont_get_cache);
    font_driver.list = Some(list);
    font_driver.match_ = Some(match_);
    font_driver.list_family = Some(ftfont_list_family);
    font_driver.open_font = Some(open_font);
    font_driver.close_font = Some(close_font);
    font_driver.has_char = Some(has_char);
    font_driver.encode_char = Some(encode_char);
    font_driver.text_extents = Some(text_extents);
    font_driver.draw = Some(draw);
    // TODO
    font_driver.get_bitmap = None;
    font_driver.anchor_point = None;
    // #ifdef HAVE_LIBOTF
    //   .otf_capability = ftcrfont_otf_capability,
    // #endif
    // #if defined HAVE_M17N_FLT && defined HAVE_LIBOTF
    //   .shape = ftcrfont_shape,
    // #endif
    // #if defined HAVE_OTF_GET_VARIATION_GLYPHS || defined HAVE_FT_FACE_GETCHARVARIANTINDEX
    //   .get_variation_glyphs = ftcrfont_variation_glyphs,
    // #endif
    font_driver.filter_properties = Some(ftfont_filter_properties);
    font_driver.combining_capability = Some(ftfont_combining_capability);

    FontDriver(font_driver)
});

impl FontDriver {
    fn global() -> &'static FontDriver {
        &FONT_DRIVER
    }
}

extern "C" fn draw(
    _s: *mut glyph_string,
    _from: i32,
    _to: i32,
    _x: i32,
    _y: i32,
    _with_background: bool,
) -> i32 {
    0
}

extern "C" fn list(f: *mut frame, spec: LispObject) -> LispObject {
    return unsafe { ftfont_list2(f, spec, Qftwr) };
}

extern "C" fn match_(f: *mut frame, spec: LispObject) -> LispObject {
    return unsafe { ftfont_match2(f, spec, Qftwr) };
}

extern "C" fn open_font(frame: *mut frame, font_entity: LispObject, pixel_size: i32) -> LispObject {
    println!("open font: {:?} {:?}", pixel_size, font_entity);
    log::trace!("open font: {:?}", pixel_size);

    let extra = unsafe {
        assq_no_quit(
            QCfont_entity,
            AREF(font_entity, font_property_index::FONT_EXTRA_INDEX as isize),
        )
    };
    if !extra.is_cons() {
        return Qnil;
    }
    let extra = unsafe { XCDR(extra) };
    let font_id = unsafe { XCAR(extra) };
    let idx = unsafe { XCDR(extra) };
    println!("filename: {:?}, index {:?}", font_id, idx);
    let idx: u32 = unsafe { XFIXNUM(idx).try_into().unwrap() };
    println!("filename: {:?}, index {:?}", font_id, idx);
    let path_my_font_file: String = font_id.into();

    // let wr_font_key = frame.add_font()
    // let wr_font_instance_key = frame.add_font_instance();

    let font_bytes = std::fs::read(path_my_font_file).unwrap();
    match skrifa::FontRef::from_index(&font_bytes, idx) {
        Ok(font) => {
            use skrifa::MetadataProvider;
            // Print the font attributes (stretch, weight and style)
            println!("{:?}", font.attributes());
            // Iterate through the localized strings
            let nameid = skrifa::string::StringId::FAMILY_NAME;
            for string in font.localized_strings(nameid) {
                // Print the string identifier and the actual value
                println!("[{:?}] {}", nameid, string.to_string());
            }
        }
        Err(e) => println!("error {:?}", e),
    }

    // let pixel_size = if pixel_size == 0 {
    //     // pixel_size here reflects to DPR 1 for webrender display, we have scale_factor from winit.
    //     // while pgtk/ns/w32 reflects to actual DPR on device by setting resx/resy to display
    //     if !frame.font().is_null() {
    //         frame.font().pixel_size as i64
    //     } else {
    //         // fallback font size
    //         16
    //     }
    // } else {
    //     // prefer elisp specific font size
    //     pixel_size as i64
    // };

    // let font_object: LispFontLike =
    //     unsafe { font_make_object(vecsize!(FontInfo) as i32, font_entity, pixel_size as i32) }
    //         .into();

    // // set type
    // font_object.aset(font_property_index::FONT_TYPE_INDEX, Qftwr);

    // // set name
    // font_object.aset(
    //     font_property_index::FONT_NAME_INDEX,
    //     LispSymbolRef::from(LispFontLike(font_entity).aref(font_property_index::FONT_FAMILY_INDEX))
    //         .symbol_name(),
    // );

    // let mut font_info = FontInfoRef::new(
    //     font_object
    //         .as_lisp_object()
    //         .as_font()
    //         .unwrap()
    //         .as_font_mut() as *mut FontInfo,
    // );
    // let metrics = font.metrics(&[]).scale(pixel_size as f32);

    // font_info.font.pixel_size = pixel_size as i32;
    // font_info.font.average_width = metrics.average_width.ceil() as i32;
    // font_info.font.ascent = metrics.ascent.ceil() as i32;
    // font_info.font.descent = metrics.descent.ceil() as i32;
    // font_info.font.space_width = font
    //     .glyph_metrics(&[])
    //     .scale(pixel_size as f32)
    //     .advance_width(font.charmap().map(' '))
    //     .ceil() as i32;
    // font_info.font.max_width = metrics.max_width.ceil() as i32;
    // font_info.font.underline_thickness = metrics.stroke_size.ceil() as i32;
    // font_info.font.underline_position = metrics.underline_offset.ceil() as i32;

    // font_info.font.height = (metrics.ascent + metrics.descent + metrics.leading).ceil() as i32;
    // font_info.font.baseline_offset = 0;

    // font_info.id = font_id;
    // font_info.base = ManuallyDrop::new(font);

    // let driver = FontDriver::global();
    // font_info.font.driver = &driver.0;
    // font_info.frame = frame.as_mut();

    // log::trace!("open font done: {:?}", pixel_size);
    // font_object.as_lisp_object()
    Qnil
}

extern "C" fn close_font(f: *mut font) {
    // todo delete font/font instance from WR
}

extern "C" fn encode_char(font: *mut font, c: i32) -> u32 {
    // get indices from WR
    unimplemented!()
}

extern "C" fn has_char(_font: LispObject, _c: i32) -> i32 {
    -1
}

#[allow(unused_variables)]
extern "C" fn text_extents(
    font: *mut font,
    code: *const u32,
    nglyphs: i32,
    metrics: *mut font_metrics,
) {
    unimplemented!()
}

extern "C" fn otf_capability(_font: *mut font) -> LispObject {
    todo!()
}

/// Swash implementation of shape for font backend.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn shape(lgstring: LispObject, direction: LispObject) -> LispObject {
    Qnil
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn register_ftwrfont_driver(f: *mut frame) {
    let driver = FontDriver::global();
    unsafe {
        register_font_driver(&driver.0, f);
    }
}

// #[allow(unused_variables)]
// #[no_mangle]
// pub extern "C" fn syms_of_ftwrfont() {
//     def_lisp_sym!(Qftwr, "ftwr");
//     def_lisp_sym!(Qmonospace, "monospace");
//     def_lisp_sym!(Qfixed, "fixed");
//     def_lisp_sym!(Qzh, "zh");

//     register_ftwrfont_driver(ptr::null_mut());
// }
