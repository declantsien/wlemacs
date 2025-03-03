use crate::types::{
    block_input, frame, mark_window_cursors_off, ns_default_font_parameter, ns_define_frame_cursor,
    unblock_input, window, Emacs_Cursor, Emacs_Pixmap, Lisp_Object, XWINDOW,
};
use crate::util::HandyDandyRectBuilder;

use super::color::pixel_to_color;

pub struct EmacsView {}

/// cbindgen:ignore
#[no_mangle]
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
    f.renderer_mut()
        .unwrap()
        .dp_push_rect(rect, None, false, false, false, clear_color);
    unsafe { unblock_input() };
}

/// cbindgen:ignore
#[no_mangle]
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
