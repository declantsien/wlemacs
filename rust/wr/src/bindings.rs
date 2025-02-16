use crate::capi::{Emacs_Color, Emacs_Pixmap, Emacs_Rectangle};
use crate::color::{color_to_xcolor, lookup_color_by_name_or_hex};
use crate::face::WrFace;
use crate::frame::FrameExtWrCommon;
use crate::image::{ImageExt, ImageRef, WrPixmap};
use crate::output::{CanvasRef, DeviceLength, WrCanvas};
use crate::util::HandyDandyRectBuilder;
use emacs_sys::bindings::{
    block_input, draw_fringe_bitmap_params, face_id, font_info, globals, glyph_string,
    gui_clear_cursor, image, lookup_basic_face, run, unblock_input, AREF, FACE_FROM_ID_OR_NULL,
};
use emacs_sys::display_traits::{FaceRef, GlyphRowArea, GlyphStringRef};
use emacs_sys::font::FontRef;
use emacs_sys::frame::{Frame, FrameRef};
use emacs_sys::lisp::LispObject;
use emacs_sys::window::{Window, WindowRef};
use webrender::api::{
    AlphaType, ColorF, CommonItemProperties, FontInstanceOptions, FontInstancePlatformOptions,
    FontKey, FontSize, FontTemplate, ImageRendering, NativeFontHandle,
};

use crate::font::FontInfoRef;
use std::ffi::CString;
use std::ptr;
use webrender::api::units::{
    DeviceIntLength, DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePoint, DeviceRect,
    DeviceSize,
};

#[repr(C)]
pub struct WrVecU8 {
    /// `data` must always be valid for passing to Vec::from_raw_parts.
    /// In particular, it must be non-null even if capacity is zero.
    data: *mut u8,
    length: usize,
    capacity: usize,
}

impl Into<DeviceRect> for &Emacs_Rectangle {
    fn into(self) -> DeviceRect {
        (self.x, self.y)
            .by(self.width as i32, self.height as i32)
            .to_f32()
    }
}

// impl From<&Emacs_Rectangle> for DeviceRect {
//     fn from(rect: &Emacs_Rectangle) -> Self {
//         (rect.x, rect.y)
//             .by(rect.width as i32, rect.height as i32)
//             .to_f32()
//     }
// }

#[repr(C)]
pub struct WrFontKey(pub u32, pub u32);
#[repr(C)]
pub struct WrFontInstanceKey(pub u32, pub u32);

#[no_mangle]
pub extern "C" fn wr_flush(canvas: &mut WrCanvas) {
    canvas.flush();
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_draw_glyph_string(s: *mut glyph_string) {
    let s: GlyphStringRef = s.into();

    let mut frame: FrameRef = s.f.into();

    frame.draw_glyph_string(s);
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_draw_fringe_bitmap(
    window: *mut Window,
    p: *mut draw_fringe_bitmap_params,
    clip_bounds: &Emacs_Rectangle,
) {
    let window: WindowRef = window.into();
    let mut frame: FrameRef = window.get_frame();

    let clip_bounds: DeviceRect =
        (clip_bounds.x, clip_bounds.y).by(clip_bounds.width as i32, clip_bounds.height as i32);

    let which = unsafe { (*p).which };

    let pos_x = unsafe { (*p).x };
    let pos_y = unsafe { (*p).y };

    let pos = DeviceIntPoint::new(pos_x, pos_y).to_f32();

    let image_clip_rect: DeviceRect = {
        let width = unsafe { (*p).wd };
        let height = unsafe { (*p).h };

        if which > 0 {
            (pos_x, pos_y).by(width, height)
        } else {
            DeviceRect::zero()
        }
    };

    let clear_rect = if unsafe { (*p).bx >= 0 && !(*p).overlay_p() } {
        unsafe { ((*p).bx, (*p).by).by((*p).nx, (*p).ny) }
    } else {
        DeviceRect::zero()
    };

    let bitmap_width = 8 as u32;
    let bitmap_height = (unsafe { (*p).h } + unsafe { (*p).dh }) as u32;
    let bits = unsafe { (*p).bits };

    let image = frame
        .wr()
        .get_or_create_fringe_bitmap(which, bitmap_width, bitmap_height, bits);

    let face = FaceRef::new(unsafe { (*p).face });

    let background_color = face.bg_color_f();

    let bitmap_color = if unsafe { (*p).cursor_p() } {
        frame.cursor_color_f()
    } else if unsafe { (*p).overlay_p() } {
        background_color
    } else {
        face.fg_color_f()
    };

    frame.draw_fringe_bitmap(
        pos,
        image,
        bitmap_color,
        background_color,
        image_clip_rect,
        clear_rect,
        clip_bounds,
    );
}

#[no_mangle]
pub extern "C" fn wr_push_rect(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    bounds: &Emacs_Rectangle,
    clip_bounds: Option<&Emacs_Rectangle>,
) {
    canvas.push_rect(color_pixel, bounds.into(), clip_bounds.map(|b| b.into()));
}

#[no_mangle]
pub extern "C" fn wr_push_border(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    bounds: &Emacs_Rectangle,
    clip_bounds: Option<&Emacs_Rectangle>,
) {
    canvas.push_border(color_pixel, bounds.into(), clip_bounds.map(|b| b.into()));
}

#[no_mangle]
pub extern "C" fn wr_clear_area(
    canvas: &mut WrCanvas,
    color: ::std::os::raw::c_ulong,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    canvas.push_rect(color, (x, y).by(width, height), None);
}

#[no_mangle]
pub extern "C" fn wr_scroll_run(
    canvas: &mut WrCanvas,
    viewport: &Emacs_Rectangle,
    new_frame_position: &Emacs_Rectangle,
) {
    let viewport: DeviceRect = viewport.into();
    let new_frame_position: DeviceRect = new_frame_position.into();
    if let Some(image_key) = canvas.get_previous_frame() {
        canvas.display(|builder, space_and_clip, scale| {
            builder.push_image(
                &CommonItemProperties::new(viewport / scale, space_and_clip),
                new_frame_position / scale,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        });
    }
}

#[no_mangle]
pub extern "C" fn wr_free_pixmap_impl(canvas: &mut WrCanvas, pixmap: Emacs_Pixmap) {
    canvas.delete_image_by_pixmap(pixmap);

    // take back ownership and RAII will drop resource.
    let _ = unsafe { Box::from_raw(pixmap as *mut WrPixmap) };
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_pixel(ximg: *mut image, x: i32, y: i32) -> i32 {
    unimplemented!();
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_put_pixel(ximg: *mut image, x: i32, y: i32, pixel: u64) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_can_use_native_image_api(image_type: LispObject) -> bool {
    crate::image::can_use_native_image_api(image_type)
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_load_image(
    frame: FrameRef,
    img: *mut image,
    _spec_file: LispObject,
    _spec_data: LispObject,
) -> bool {
    let image: ImageRef = img.into();
    image.load(frame)
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_transform_image(
    frame: FrameRef,
    img: *mut image,
    width: i32,
    height: i32,
    rotation: f64,
) {
    let image: ImageRef = img.into();
    image.transform(frame, width, height, rotation);
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_add_font(frame: *mut Frame, font_object: LispObject) {
    let f = FrameRef::new(frame);
    let filename = unsafe {
        AREF(
            font_object,
            emacs_sys::bindings::font_property_index::FONT_FILE_INDEX
                .try_into()
                .unwrap(),
        )
    };
    let path = std::path::PathBuf::from(String::from(filename));
    let mut font = FontRef::new(unsafe { emacs_sys::bindings::XFONT_OBJECT(font_object) });
    let mut font_info = FontInfoRef::new(font.as_mut() as *mut font_info);
    let index = font_info.index as u32;

    let wr_font_key = f
        .wr()
        .wr_add_font(FontTemplate::Native(NativeFontHandle { path, index }));

    let scale = f.wr().layout_to_device_scale_factor();
    let wr_font_instance_key = f.wr().wr_add_font_instance(
        wr_font_key,
        FontSize::from_f32_px((DeviceLength::new(font.pixel_size as f32) / scale).get()),
        Some(FontInstanceOptions::default()),
        Some(FontInstancePlatformOptions::default()),
        Vec::new(),
    );
    // font_info.font_key = wr_font_key;
    // font_info.font_instance_key = wr_font_instance_key;
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn image_pixmap_draw_cross(
    _frame: FrameRef,
    _pixmap: Emacs_Pixmap,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: u32,
    _color: u64,
) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn image_sync_to_pixmaps(_frame: FrameRef, _img: *mut image) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_clear_under_internal_border_impl(f: *mut Frame, canvas: &mut WrCanvas) {
    let mut f = FrameRef::new(f);
    let border = f.internal_border_width();
    let width = f.pixel_width;
    let height = f.pixel_height;
    let margin = f.top_margin_height();
    let bottom_margin = f.bottom_margin_height();
    let face_id_fallback = |id: face_id| {
        if unsafe { globals.Vface_remapping_alist.is_not_nil() } {
            unsafe { lookup_basic_face(ptr::null_mut(), f.clone().as_mut(), id as i32) }
        } else {
            id as i32
        }
    };
    let face_id = match f.parent_frame() {
        Some(_) => face_id_fallback(face_id::CHILD_FRAME_BORDER_FACE_ID),
        None => face_id_fallback(face_id::INTERNAL_BORDER_FACE_ID),
    };
    let face = unsafe { FACE_FROM_ID_OR_NULL(f.as_mut(), face_id) };

    unsafe { block_input() };

    if face.is_null() {
        wr_clear_area(canvas, f.background_pixel, 0, 0, border, height);
        wr_clear_area(canvas, f.background_pixel, 0, margin, width, border);
        wr_clear_area(
            canvas,
            f.background_pixel,
            0,
            width - border,
            border,
            height,
        );
        wr_clear_area(
            canvas,
            f.background_pixel,
            0,
            height - bottom_margin - border,
            width,
            border,
        );
    } else {
        log::error!("unimplemented: clean under internal border with face");
    }

    unsafe { unblock_input() };
}

#[no_mangle]
pub extern "C" fn wr_parse_color(
    color_name: *const ::libc::c_char,
    xcolor: *mut Emacs_Color,
) -> ::libc::c_int {
    use std::ffi::CStr;
    let color_name: &CStr = unsafe { CStr::from_ptr(color_name) };
    let color_name: &str = color_name.to_str().unwrap();
    if let Some(color) = lookup_color_by_name_or_hex(&format!("{}", color_name.to_owned())) {
        color_to_xcolor(color, xcolor);
        1
    } else {
        0
    }
}

// #[no_mangle]
// pub extern "C" fn wr_init() -> *mut libc::c_void {
//     let data = Box::new(WrData::build(self.clone()));
//     Box::into_raw(data) as *mut libc::c_void
// }

#[no_mangle]
pub extern "C" fn wr_destroy(wr_data: *mut libc::c_void) {
    let _ = unsafe { Box::from_raw(wr_data as *mut WrCanvas) };
}

/// cbindgen:ignore
/// Fit GL context to frame, reflecting frame/scale factor changes
#[no_mangle]
pub extern "C" fn wr_fit_context(f: *mut Frame) {
    let frame: FrameRef = f.into();
    if frame.output().is_null() || frame.output().wr_data.is_null() {
        return;
    }
    frame.wr().update();
}

// /// Capture the contents of the current WebRender frame and
// /// save them to a folder relative to the current working directory.
// ///
// /// If START-SEQUENCE is not nil, start capturing each WebRender frame to disk.
// /// If there is already a sequence capture in progress, stop it and start a new
// /// one, with the new path and flags.
// #[allow(unused_variables)]
// #[lisp_fn(min = "2")]
// pub fn wr_api_capture(path: LispStringRef, bits_raw: LispObject, start_sequence: LispObject) {
//     #[cfg(not(feature = "capture"))]
//     error!("Webrender capture not avaiable");
//     #[cfg(feature = "capture")]
//     {
//         use emacs_sys::frame::window_frame_live_or_selected;
//         use std::fs::create_dir_all;
//         use std::fs::File;
//         use std::io::Write;

//         let path = std::path::PathBuf::from(path.to_utf8());
//         match create_dir_all(&path) {
//             Ok(_) => {}
//             Err(err) => {
//                 error!("Unable to create path '{:?}' for capture: {:?}", &path, err);
//             }
//         };
//         let bits_raw = unsafe {
//             emacs_sys::bindings::check_integer_range(
//                 bits_raw,
//                 webrender::CaptureBits::SCENE.bits() as i64,
//                 webrender::CaptureBits::all().bits() as i64,
//             )
//         };

//         let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
//         let canvas = frame.wr_data();
//         let bits = webrender::CaptureBits::from_bits(bits_raw as _).unwrap();
//         let revision_file_path = path.join("wr.txt");
//         message!("Trying to save webrender capture under {:?}", &path);

//         // api call here can possibly make Emacs panic. For example there isn't
//         // enough disk space left. `panic::catch_unwind` isn't support here.
//         if start_sequence.is_nil() {
//             canvas.render_api.save_capture(path, bits);
//         } else {
//             canvas.render_api.start_capture_sequence(path, bits);
//         }

//         match File::create(revision_file_path) {
//             Ok(mut file) => {
//                 if let Err(err) = write!(&mut file, "{}", "") {
//                     error!("Unable to write webrender revision: {:?}", err)
//                 }
//             }
//             Err(err) => error!(
//                 "Capture triggered, creating webrender revision info skipped: {:?}",
//                 err
//             ),
//         }
//     }
// }

// /// Stop a capture begun with `wr--capture'.
// #[lisp_fn(min = "0")]
// pub fn wr_api_stop_capture_sequence() {
//     #[cfg(not(feature = "capture"))]
//     error!("Webrender capture not avaiable");
//     #[cfg(feature = "capture")]
//     {
//         use emacs_sys::frame::window_frame_live_or_selected;

//         message!("Stop capturing WR state");
//         let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
//         let canvas = frame.wr_data();
//         canvas.render_api.stop_capture_sequence();
//     }
// }

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn wr_log_init() {
    // #[cfg(debug_assertions)]
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    // install global collector configured based on WR_LOG env var.
    // #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("WR_LOG"))
        .init();

    log::trace!("Emacs WR");
}
