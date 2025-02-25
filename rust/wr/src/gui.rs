use crate::emacs::{
    draw_fringe_bitmap_params, frame, frame_parm_handler, glyph_row, glyph_string,
    gui_clear_end_of_line, gui_clear_window_mouse_face, gui_fix_overlapping_area,
    gui_get_glyph_overhangs, gui_insert_glyphs, gui_produce_glyphs, gui_write_glyphs,
    ns_frame_parm_handlers, redisplay_interface, run, terminal, text_cursor_kinds, window,
};
use crate::platform::gui::{default_font_parameter, define_frame_cursor};
use std::sync::LazyLock;

unsafe impl Sync for redisplay_interface {}
unsafe impl Send for redisplay_interface {}

/// cbindgen:ignore
#[no_mangle]
pub static mut wr_redisplay_interface: redisplay_interface = redisplay_interface {
    frame_parm_handlers: unsafe { &mut ns_frame_parm_handlers } as *mut Option<_>, //(Box::into_raw(frame_parm_handlers)) as *mut Option<_>,
    produce_glyphs: Some(gui_produce_glyphs),
    write_glyphs: Some(gui_write_glyphs),
    insert_glyphs: Some(gui_insert_glyphs),
    clear_end_of_line: Some(gui_clear_end_of_line),
    scroll_run_hook: Some(scroll_run),
    after_update_window_line_hook: Some(after_update_window_line),
    update_window_begin_hook: Some(update_window_begin),
    update_window_end_hook: Some(update_window_end),
    flush_display: Some(flush_display),
    clear_window_mouse_face: Some(gui_clear_window_mouse_face),
    get_glyph_overhangs: Some(gui_get_glyph_overhangs),
    fix_overlapping_area: Some(gui_fix_overlapping_area),
    draw_fringe_bitmap: Some(draw_fringe_bitmap),
    define_fringe_bitmap: Some(define_fringe_bitmap),
    destroy_fringe_bitmap: Some(destroy_fringe_bitmap),
    compute_glyph_string_overhangs: Some(compute_glyph_string_overhangs),
    draw_glyph_string: Some(draw_glyph_string),
    define_frame_cursor: Some(define_frame_cursor),
    clear_frame_area: Some(clear_frame_area),
    clear_under_internal_border: Some(clear_under_internal_border),
    draw_window_cursor: Some(draw_window_cursor),
    draw_vertical_window_border: Some(draw_vertical_window_border),
    draw_window_divider: Some(draw_window_divider),
    /* Never called; see comment in xterm.c.  */
    shift_glyphs_for_insert: Some(shift_glyphs_for_insert),
    show_hourglass: Some(show_hourglass),
    hide_hourglass: Some(hide_hourglass),
    default_font_parameter: Some(default_font_parameter),
};

extern "C" fn scroll_run(w: *mut window, run: *mut run) {
    println!("wr_scroll_run");
}

extern "C" fn after_update_window_line(w: *mut window, desired_row: *mut glyph_row) {
    println!("after_update_window_line");
}

extern "C" fn update_window_begin(w: *mut window) {
    println!("update_window_begin");
}

extern "C" fn update_window_end(w: *mut window, cursor_on_p: bool, mouse_face_overwritten_p: bool) {
}

extern "C" fn draw_glyph_string(s: *mut glyph_string) {}

extern "C" fn draw_fringe_bitmap(
    w: *mut window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
}

extern "C" fn flush_display(f: *mut frame) {}

extern "C" fn clear_frame_area(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
) {
}

extern "C" fn draw_window_cursor(
    w: *mut window,
    glyph_row: *mut glyph_row,
    x: ::libc::c_int,
    y: ::libc::c_int,
    cursor_type: text_cursor_kinds,
    cursor_width: ::libc::c_int,
    on_p: bool,
    active_p: bool,
) {
}

extern "C" fn draw_vertical_window_border(
    w: *mut window,
    x: ::libc::c_int,
    y_0: ::libc::c_int,
    y_1: ::libc::c_int,
) {
}

extern "C" fn draw_window_divider(
    w: *mut window,
    x_0: ::libc::c_int,
    x_1: ::libc::c_int,
    y_0: ::libc::c_int,
    y_1: ::libc::c_int,
) {
}

extern "C" fn define_fringe_bitmap(
    which: ::libc::c_int,
    bits: *mut ::libc::c_ushort,
    h: ::libc::c_int,
    wd: ::libc::c_int,
) {
}

extern "C" fn destroy_fringe_bitmap(which: ::libc::c_int) {}

extern "C" fn compute_glyph_string_overhangs(s: *mut glyph_string) {}

extern "C" fn clear_under_internal_border(f: *mut frame) {}

extern "C" fn shift_glyphs_for_insert(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    shift_by: ::libc::c_int,
) {
}

extern "C" fn show_hourglass(f: *mut frame) {}

extern "C" fn hide_hourglass(f: *mut frame) {}
