use crate::canvas::WrCanvas;
use crate::gfx::context::{GLContext, GLContextTrait};
use crate::platform::gui::{default_font_parameter, define_frame_cursor};
use crate::types::{
    draw_fringe_bitmap_params, frame, glyph_row, glyph_string, gui_clear_end_of_line,
    gui_clear_window_mouse_face, gui_fix_overlapping_area, gui_get_glyph_overhangs,
    gui_insert_glyphs, gui_produce_glyphs, gui_write_glyphs, ns_frame_parm_handlers,
    redisplay_interface, run, text_cursor_kinds, window, FrameRef,
};

unsafe impl Sync for redisplay_interface {}
unsafe impl Send for redisplay_interface {}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wrgui_init(f: *mut frame) {
    use crate::types::{EmacsIntSize, EmacsToDeviceScale};
    let mut f = FrameRef::new(f);
    let scale_factor = f.scale_factor();

    let display_handle = f.raw_display_handle();
    let window_handle = f.raw_window_handle();
    println!("window handle: {window_handle:?}");
    let size = EmacsIntSize::new(f.pixel_width, f.pixel_height);
    let device_size = (size.to_f32() * EmacsToDeviceScale::new(scale_factor as f32)).to_i32();
    let gl_context = GLContext::build(display_handle, window_handle, device_size.to_i32());

    let data = Box::new(WrCanvas::build(gl_context, size, scale_factor));
    f.output_data().wr_data = Box::into_raw(data);
}

/// cbindgen:ignore
#[no_mangle]
pub static mut wr_redisplay_interface: redisplay_interface = redisplay_interface {
    frame_parm_handlers: &raw mut ns_frame_parm_handlers as *mut Option<_>,
    // frame_parm_handlers: unsafe { &mut ns_frame_parm_handlers } as *mut Option<_>,
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

#[allow(unused_variables)]
extern "C" fn scroll_run(w: *mut window, run: *mut run) {
    println!("wr_scroll_run");
}

#[allow(unused_variables)]
extern "C" fn after_update_window_line(w: *mut window, desired_row: *mut glyph_row) {
    println!("after_update_window_line");
}

#[allow(unused_variables)]
extern "C" fn update_window_begin(w: *mut window) {
    println!("update_window_begin");
}

#[allow(unused_variables)]
extern "C" fn update_window_end(w: *mut window, cursor_on_p: bool, mouse_face_overwritten_p: bool) {
}

#[allow(unused_variables)]
extern "C" fn draw_glyph_string(s: *mut glyph_string) {}

#[allow(unused_variables)]
extern "C" fn draw_fringe_bitmap(
    w: *mut window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
}

#[allow(unused_variables)]
extern "C" fn flush_display(f: *mut frame) {}

#[allow(unused_variables)]
extern "C" fn clear_frame_area(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
) {
}

#[allow(unused_variables)]
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

#[allow(unused_variables)]
extern "C" fn draw_vertical_window_border(
    w: *mut window,
    x: ::libc::c_int,
    y_0: ::libc::c_int,
    y_1: ::libc::c_int,
) {
}

#[allow(unused_variables)]
extern "C" fn draw_window_divider(
    w: *mut window,
    x_0: ::libc::c_int,
    x_1: ::libc::c_int,
    y_0: ::libc::c_int,
    y_1: ::libc::c_int,
) {
}

#[allow(unused_variables)]
extern "C" fn define_fringe_bitmap(
    which: ::libc::c_int,
    bits: *mut ::libc::c_ushort,
    h: ::libc::c_int,
    wd: ::libc::c_int,
) {
}

#[allow(unused_variables)]
extern "C" fn destroy_fringe_bitmap(which: ::libc::c_int) {}

#[allow(unused_variables)]
extern "C" fn compute_glyph_string_overhangs(s: *mut glyph_string) {}

#[allow(unused_variables)]
extern "C" fn clear_under_internal_border(f: *mut frame) {}

#[allow(unused_variables)]
extern "C" fn shift_glyphs_for_insert(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    shift_by: ::libc::c_int,
) {
}

#[allow(unused_variables)]
extern "C" fn show_hourglass(f: *mut frame) {}

#[allow(unused_variables)]
extern "C" fn hide_hourglass(f: *mut frame) {}
