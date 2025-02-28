use crate::canvas::WrCanvas;
use crate::types::{
    block_input, draw_glyphs_face, frame, glyph_string, mark_window_cursors_off,
    ns_default_font_parameter, ns_define_frame_cursor, prepare_face_for_display, unblock_input,
    window, EmacsIntPoint, EmacsIntSize, EmacsPoint, EmacsRect, Emacs_Cursor, Emacs_Pixmap,
    Lisp_Object, XWINDOW,
};
use crate::util::HandyDandyRectBuilder;
use objc2_app_kit::NSColor;
use objc2_foundation::NSRect;
use webrender_api::{AlphaType, CommonItemProperties, ImageRendering};

use super::color::{ns_color_to_color_f, pixel_to_color};
use super::types::ns_rect_to_emacs;

pub struct EmacsView {}

// impl FrameRef {
//     pub fn output_data(&mut self) -> OutputDataRef {
//         OutputDataRef::new(unsafe { self.output_data.ns })
//     }
//     pub fn scale_factor(&mut self) -> f64 {
//         unsafe { ns_frame_scale_factor(self.as_mut()) }
//     }
// }

// /// cbindgen:ignore
// #[no_mangle]
// pub extern "C" fn wr_draw_fringe_bitmap(
//     canvas: &mut WrCanvas,
//     which: ::libc::c_int,
//     pos_x: ::libc::c_int,
//     pos_y: ::libc::c_int,
//     width: ::libc::c_int,
//     height: ::libc::c_int,
//     bitmap_width: ::libc::c_int,
//     bitmap_height: ::libc::c_int,
//     bits: *mut ::libc::c_ushort,
//     foreground: &NSColor,
//     clip_bounds: &NSRect,
// ) {
//     println!("draw fringe");
//     let clip_bounds: EmacsRect = ns_rect_to_emacs(clip_bounds);

//     let pos = EmacsIntPoint::new(pos_x, pos_y).to_f32();

//     let image_clip_rect: EmacsRect = {
//         if which > 0 {
//             (pos_x, pos_y).by(width, height)
//         } else {
//             EmacsRect::zero()
//         }
//     };

//     let image = canvas.get_or_create_fringe_bitmap(
//         which,
//         EmacsIntSize::new(bitmap_width, bitmap_height),
//         bits,
//     );

//     // Fixed image_clip_rect
//     let image_clip_rect = image_clip_rect
//         .intersection(&clip_bounds)
//         .unwrap_or_else(|| EmacsRect::zero());

//     canvas.display(|builder, space_and_clip, scale| {
//         if let Some(image) = &image {
//             println!("draw fringe has image");
//             let image_display_rect = EmacsRect::new(
//                 pos,
//                 EmacsPoint::new(image.width as f32, image.height as f32),
//             );
//             // render image
//             builder.push_image(
//                 &CommonItemProperties::new(image_clip_rect * scale, space_and_clip),
//                 image_display_rect * scale,
//                 ImageRendering::Auto,
//                 AlphaType::Alpha,
//                 image.image_key,
//                 ns_color_to_color_f(foreground),
//             );
//         }
//     });
// }

impl glyph_string {
    pub fn set_gc(&mut self) {
        use draw_glyphs_face::*;
        let gc = unsafe { self.gc.as_mut().unwrap() };
        unsafe { prepare_face_for_display(self.f, self.face) };
        match self.hl {
            DRAW_NORMAL_TEXT | DRAW_IMAGE_RAISED | DRAW_IMAGE_SUNKEN | DRAW_INVERSE_VIDEO => {
                gc.foreground = self.face().foreground;
                gc.background = self.face().background;
                self.set_stippled_p(self.face().stipple != 0);
            }
            DRAW_CURSOR => {
                self.set_cursor_gc();
                self.set_stippled_p(false);
            }
            DRAW_MOUSE_FACE => {
                self.set_mouse_face_gc();
                self.set_stippled_p(self.face().stipple != 0);
            }
        }
    }

    pub fn set_cursor_gc(&mut self) {
        let f = unsafe { self.f.as_ref().unwrap() };
        if self.font == unsafe { f.output_data().unwrap().font }
            && self.face().background == f.background_pixel
            && self.face().foreground == f.foreground_pixel
            && !self.cmp.is_null()
        {
            // ns_output doesn't have this
            // self.gc = unsafe { f.output_data().unwrap().cursor_gc }
        } else {
            // ns todo
        }
    }
    pub fn set_mouse_face_gc(&mut self) {}
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_clear_frame(f: *mut frame) {
    let frame = frame::from_ptr(f);
    let face = frame::from_ptr(f).and_then(|f| f.default_face());

    if frame.is_none() && face.is_none() {
        return;
    }
    let f = frame.unwrap();
    let face = face.unwrap();

    unsafe { mark_window_cursors_off(XWINDOW(f.root_window)) };

    unsafe { block_input() };
    let rect = (0, 0).by(f.pixel_width, f.pixel_height);
    let clear_color = pixel_to_color(face.background);
    println!("clear frame clear_color: {clear_color:?}");
    f.renderer()
        .unwrap()
        .dp_push_rect(rect, None, false, false, false, clear_color);
    unsafe { unblock_input() };
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_clear_frame_area(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
) {
    if let Some(f) = frame::from_ptr(f) {
        let r = (x, y).by(width, height);
        f.clear_area(r.to_i32());
    }
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_condemn_scroll_bars(f: *mut frame) {
    //TODO
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_free_pixmap(f: *mut frame, pixmap: Emacs_Pixmap) {
    //TODO
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_judge_scroll_bars(f: *mut frame) {
    //TODO
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_redeem_scroll_bar(window: *mut window) {}
/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_set_vertical_scroll_bar(
    w: *mut window,
    portion: ::libc::c_int,
    whole: ::libc::c_int,
    position: ::libc::c_int,
) {
}

/// cbindgen:ignore
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn ns_set_horizontal_scroll_bar(
    w: *mut window,
    portion: ::libc::c_int,
    whole: ::libc::c_int,
    position: ::libc::c_int,
) {
}

pub extern "C" fn define_frame_cursor(f: *mut frame, cursor: Emacs_Cursor) {
    unsafe { ns_define_frame_cursor(f, cursor) };
}

pub extern "C" fn default_font_parameter(f: *mut frame, parms: Lisp_Object) {
    unsafe { ns_default_font_parameter(f, parms) };
}

// #[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn wr_new_font(
    f: *mut frame,
    font_object: Lisp_Object,
    fontset: ::libc::c_int,
) -> Lisp_Object {
    return font_object;
}
