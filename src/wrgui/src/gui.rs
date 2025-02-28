use euclid::{Point2D, Rect, Size2D};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use webrender_api::{AlphaType, ColorF, CommonItemProperties, ImageRendering};

use crate::canvas::WrCanvas;
use crate::gfx::context::{GLContext, GLContextTrait};
use crate::platform::gui::{default_font_parameter, define_frame_cursor};
use crate::platform::pixel_to_color;
use crate::types::{
    block_input, draw_fringe_bitmap_params, draw_glyphs_face, draw_phys_cursor_glyph, face_id,
    frame, get_phys_cursor_glyph, glyph_row, glyph_row_area, glyph_string, glyph_type,
    gui_clear_cursor, gui_clear_end_of_line, gui_clear_window_mouse_face, gui_fix_overlapping_area,
    gui_get_glyph_overhangs, gui_insert_glyphs, gui_produce_glyphs, gui_write_glyphs,
    ns_frame_parm_handlers, redisplay_interface, run, text_cursor_kinds, unblock_input, window, x_draw_xwidget_glyph_string,
};
use crate::util::HandyDandyRectBuilder;

unsafe impl Sync for redisplay_interface {}
unsafe impl Send for redisplay_interface {}

use crate::types::glyph_matrix;

impl glyph_matrix {
    // TODO needs to verify that
    pub fn row(&self, row: libc::c_int) -> Option<&mut glyph_row> {
        assert!(!self.rows.is_null());
        assert!(row >= 0 && row < self.nrows);

        unsafe { self.rows.offset(row as isize).as_mut() }
    }
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wrgui_init(f: *mut frame) {
    use crate::types::{EmacsIntSize, EmacsToDeviceScale};
    let f = frame::from_ptr(f).unwrap();
    let scale_factor = f.scale_factor();

    let display_handle = f.display_handle().unwrap().as_raw();
    let window_handle = f.window_handle().unwrap().as_raw();
    println!("window handle: {window_handle:?}");
    let size = EmacsIntSize::new(f.pixel_width, f.pixel_height);
    let device_size = (size.to_f32() * EmacsToDeviceScale::new(scale_factor as f32)).to_i32();
    let gl_context = GLContext::build(display_handle, window_handle, device_size.to_i32());

    let data = Box::new(WrCanvas::build(gl_context, size, scale_factor));
    f.output_data_mut().unwrap().wr_data = Box::into_raw(data);
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
    let win = window::from_ptr(w).unwrap();
    let win_mut = window::from_ptr_mut(w).unwrap();
    let f = win.x_frame_mut().unwrap();
    let run = unsafe { run.as_ref().unwrap() };
    let Rect {
        origin: Point2D { x, y, .. },
        size: Size2D {
            width, mut height, ..
        },
    } = win_mut.window_box(glyph_row_area::ANY_AREA).to_rect();
    let from_y = win_mut.to_frame_pixel_y(run.current_y);
    let to_y = win_mut.to_frame_pixel_y(run.desired_y);
    let bottom_y = y + height;

    if to_y < from_y {
        /* Scrolling up.  Make sure we don't copy part of the mode
        line at the bottom.  */
        if (from_y + run.height) > bottom_y {
            height = bottom_y - from_y;
        } else {
            height = run.height;
        }
    } else {
        /* Scrolling down.  Make sure we don't copy over the mode line.
        at the bottom.  */
        if (to_y + run.height) > bottom_y {
            height = bottom_y - to_y;
        } else {
            height = run.height;
        }
    }

    unsafe { block_input() };
    /* Cursor off.  Will be switched on again in gui_update_window_end.  */
    unsafe { gui_clear_cursor(w) };

    let diff_y = to_y - from_y;
    let viewport = (x, to_y).by(width, height);
    let new_frame_position = (0, 0 + diff_y).by(f.pixel_width, f.pixel_height);

    if let Some(image_key) = f.renderer().unwrap().get_previous_frame() {
        f.renderer()
            .unwrap()
            .display(|builder, space_and_clip, scale| {
                builder.push_image(
                    &CommonItemProperties::new(viewport * scale, space_and_clip),
                    new_frame_position * scale,
                    ImageRendering::Auto,
                    AlphaType::PremultipliedAlpha,
                    image_key,
                    ColorF::WHITE,
                );
            });
    }

    unsafe { unblock_input() };
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
extern "C" fn draw_glyph_string(s: *mut glyph_string) {
    use glyph_type::*;
    let relief_drawn_p = false;
    let gs = || unsafe { s.as_mut().unwrap() };

    /* If S draws into the background of its successors, draw the
    background of the successors first so that S can draw into it.
    This makes S->next use XDrawString instead of XDrawImageString.  */

    /* If S draws into the background of its successors, draw the
    background of the successors first so that S can draw into it.
    This makes S->next use XDrawString instead of XDrawImageString.  */
    if !gs().next.is_null() && gs().right_overhang != 0 && !gs().for_overlaps() != 0 {
        let width: i32;
        let next: &mut glyph_string;
        // TODO
        // for (width = 0, next = s->next;
        //      next && width < s->right_overhang;
        //      width += next->width, next = next->next)
        //   if (next->first_glyph->type != IMAGE_GLYPH)
        //     {
        //       cairo_t *cr = pgtk_begin_cr_clip (next->f);
        //       pgtk_set_glyph_string_gc (next);
        //       pgtk_set_glyph_string_clipping (next, cr);
        //       if (next->first_glyph->type == STRETCH_GLYPH)
        //         pgtk_draw_stretch_glyph_string (next);
        //       else
        //         pgtk_draw_glyph_string_background (next, true);
        //       next->num_clips = 0;
        //       pgtk_end_cr_clip (next->f);
        //     }
    }

    /* Set up S->gc, set clipping and draw S.  */
    gs().set_gc();

    // /* Draw relief (if any) in advance for char/composition so that the
    //    glyph string can be drawn over it.  */
    // if (!s->for_overlaps
    //     && s->face->box != FACE_NO_BOX
    //     && (s->first_glyph->type == CHAR_GLYPH
    //         || s->first_glyph->type == COMPOSITE_GLYPH))

    //   {
    //     pgtk_set_glyph_string_clipping (s, cr);
    //     pgtk_draw_glyph_string_background (s, true);
    //     pgtk_draw_glyph_string_box (s);
    //     pgtk_set_glyph_string_clipping (s, cr);
    //     relief_drawn_p = true;
    //   }
    // else if (!s->clip_head	/* draw_glyphs didn't specify a clip mask. */
    //          && !s->clip_tail
    //          && ((s->prev && s->prev->hl != s->hl && s->left_overhang)
    //              || (s->next && s->next->hl != s->hl && s->right_overhang)))
    //   /* We must clip just this glyph.  left_overhang part has already
    //      drawn when s->prev was drawn, and right_overhang part will be
    //      drawn later when s->next is drawn. */
    //   pgtk_set_glyph_string_clipping_exactly (s, s, cr);
    // else
    //   pgtk_set_glyph_string_clipping (s, cr);
    match glyph_type::from(unsafe { gs().first_glyph.as_ref().unwrap().type_() }) {
        CHAR_GLYPH => {}
        COMPOSITE_GLYPH => {}
        GLYPHLESS_GLYPH => {}
        IMAGE_GLYPH => {}
        STRETCH_GLYPH => {}
        XWIDGET_GLYPH => unsafe { x_draw_xwidget_glyph_string (s) }
    }
}

#[allow(unused_variables)]
extern "C" fn draw_fringe_bitmap(
    w: *mut window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
    let w = window::from_ptr_mut(w).unwrap();
    let f = window::from_ptr(w).unwrap().x_frame_mut().unwrap();
    let row = unsafe { row.as_ref().unwrap() };
    let p = unsafe { p.as_ref().unwrap() };
    let face = unsafe { p.face.as_ref().unwrap() };
    let clip = w.row_clip_bounds(row, glyph_row_area::ANY_AREA);

    if p.bx >= 0 && !p.overlay_p() {
        let rect = (p.bx, p.by).by(p.nx, p.ny);
        f.renderer().unwrap().dp_push_rect(
            rect,
            Some(clip.to_f32()),
            false,
            false,
            false,
            pixel_to_color(face.background),
        );
    }

    if p.which > 0 {
        let bitmap_width = 8;
        let bitmap_height = p.h + p.dh;

        let foreground = if p.cursor_p() {
            if p.overlay_p() {
                pixel_to_color(face.background)
            } else {
                f.cursor_color()
            }
        } else {
            pixel_to_color(face.foreground)
        };
    }

    //check wlcterm.c
    // wr_draw_fringe_bitmap(FRAME_WR_DATA(f), p->which, p->x, p->y, p->wd, p->h,
    //     bitmap_width, bitmap_height, p->bits,
    //     &gcv, &clip_bounds);
}

#[allow(unused_variables)]
extern "C" fn flush_display(f: *mut frame) {
    if let Some(r) = frame::from_ptr(f).and_then(|f| f.renderer()) {
        r.flush();
    }
}

#[allow(unused_variables)]
extern "C" fn clear_frame_area(
    f: *mut frame,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
) {
    let f = frame::from_ptr(f).unwrap();
    let r = (x, y).by(width, height);
    f.clear_area(r.to_i32());
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
    use text_cursor_kinds::*;
    let win = window::from_ptr_mut(w).unwrap();
    let f = window::from_ptr(w).and_then(|w| w.x_frame_mut()).unwrap();
    let glyph_row = unsafe { glyph_row.as_mut().unwrap() };

    if !on_p {
        return;
    }

    win.phys_cursor_type = cursor_type;
    win.set_phys_cursor_on_p(true);

    let in_fringe_p = glyph_row.exact_window_width_line_p()
        && (if glyph_row.reversed_p() {
            win.phys_cursor.hpos < 0
        } else {
            win.phys_cursor.hpos >= glyph_row.used[glyph_row_area::TEXT_AREA as usize] as i32
        });

    let mut draw_hollow_cursor = || {
        /* Get the glyph the cursor is on.  If we can't tell because
        the current matrix is invalid or such, give up.  */
        let cursor_glyph = unsafe { get_phys_cursor_glyph(w).as_mut() };
        if let Some(cursor_glyph) = cursor_glyph {
            /* Compute frame-relative coordinates for phys cursor.  */
            let Rect {
                origin: Point2D { mut x, y, .. },
                size: Size2D { height, .. },
            } = window::from_ptr_mut(w)
                .unwrap()
                .phys_cursor_geometry(glyph_row, cursor_glyph)
                .to_rect();
            let mut wd = win.phys_cursor_width - 1;

            if (cursor_glyph.resolved_level() & 1) != 0 && cursor_glyph.pixel_width > wd as i16 {
                x += (cursor_glyph.pixel_width as i32 - wd) as i32;
                if wd > 0 {
                    wd -= 1;
                }
            }

            let clip = win.row_clip_bounds(glyph_row, glyph_row_area::TEXT_AREA);

            let color = f.cursor_color();
            f.renderer().unwrap().dp_push_rect(
                (x, y).by(wd, height - 1).to_f32(),
                Some(clip.to_f32()),
                false,
                false,
                false,
                color,
            );
        }
    };

    let mut draw_bar_cursor = |kind: text_cursor_kinds| {
        let cursor_glyph = unsafe { get_phys_cursor_glyph(w).as_mut() };
        if let Some(cursor_glyph) = cursor_glyph {
            /* Experimental avoidance of cursor on xwidget.  */
            if cursor_glyph.type_() == glyph_type::XWIDGET_GLYPH as u32 {
                return;
            }

            /* If on an image, draw like a normal cursor.  That's usually better
            visible than drawing a bar, esp. if the image is large so that
            the bar might not be in the window.  */
            if cursor_glyph.type_() == glyph_type::IMAGE_GLYPH as u32 {
                let win = window::from_ptr(w).unwrap();
                let metrix = unsafe { win.current_matrix.as_ref().unwrap() };
                if let Some(r) = metrix.row(win.phys_cursor.vpos) {
                    unsafe { draw_phys_cursor_glyph(w, r, draw_glyphs_face::DRAW_CURSOR) };
                }
            } else {
                //TODO
            }
        }
    };

    if in_fringe_p {
        glyph_row.set_cursor_in_fringe_p(true);
        unsafe {
            crate::types::draw_fringe_bitmap(w, glyph_row, glyph_row.reversed_p() as ::libc::c_int)
        };
    } else {
        match cursor_type {
            FILLED_BOX_CURSOR => {
                unsafe {
                    crate::types::draw_phys_cursor_glyph(
                        w,
                        glyph_row,
                        draw_glyphs_face::DRAW_CURSOR,
                    )
                };
            }
            HOLLOW_BOX_CURSOR => {
                draw_hollow_cursor();
            }

            BAR_CURSOR => {
                draw_bar_cursor(BAR_CURSOR);
                // draw_bar_cursor(glyph_row, cursor_width, BAR_CURSOR);
            }

            HBAR_CURSOR => {
                draw_bar_cursor(HBAR_CURSOR);
                // draw_bar_cursor(glyph_row, cursor_width, HBAR_CURSOR);
            }

            NO_CURSOR => {
                win.phys_cursor_width = 0;
            }
            _ => panic!("invalid cursor type"),
        }
    }
    // TODO pgtk has this
    // if (w == XWINDOW (f->selected_window))
    // {
    //     int frame_x = (WINDOW_TO_FRAME_PIXEL_X (w, x)
    // 	+ WINDOW_LEFT_FRINGE_WIDTH (w)
    // 	+ WINDOW_LEFT_MARGIN_WIDTH (w));
    //   int frame_y = WINDOW_TO_FRAME_PIXEL_Y (w, y);
    //     pgtk_im_set_cursor_location (f, frame_x, frame_y,
    // 	w->phys_cursor_width,
    // 	w->phys_cursor_height);
    // }
}

#[allow(unused_variables)]
extern "C" fn draw_vertical_window_border(
    w: *mut window,
    x: ::libc::c_int,
    y0: ::libc::c_int,
    y1: ::libc::c_int,
) {
    let win = window::from_ptr(w);
    let f = || window::from_ptr_mut(w).unwrap().x_frame_mut().unwrap();
    let face = f().face_from_id_or_null(face_id::VERTICAL_BORDER_FACE_ID);
    if let Some(face) = face {
        f().renderer().unwrap().dp_push_rect(
            (x, y0).by(1, y1 - y0),
            None,
            false,
            false,
            false,
            pixel_to_color(face.background),
        );
    }
}

#[allow(unused_variables)]
extern "C" fn draw_window_divider(
    w: *mut window,
    x0: ::libc::c_int,
    x1: ::libc::c_int,
    y0: ::libc::c_int,
    y1: ::libc::c_int,
) {
    let win = window::from_ptr(w);
    let f = || window::from_ptr_mut(w).unwrap().x_frame_mut().unwrap();
    let get_color = |id| {
        let face = f().face_from_id_or_null(id);
        let color = face.map(|f| f.foreground).unwrap_or(f().foreground_pixel);
        pixel_to_color(color)
    };
    let id = face_id::WINDOW_DIVIDER_FACE_ID;
    let id_first = face_id::WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID;
    let id_last = face_id::WINDOW_DIVIDER_LAST_PIXEL_FACE_ID;
    let draw = |r, id| {
        let c = get_color(id);
        f().renderer()
            .unwrap()
            .dp_push_rect(r, None, false, false, false, c);
    };

    if (y1 - y0) > (x1 - x0) && (x1 - x0) >= 3 {
        /* A vertical divider, at least three pixels wide: Draw first and
        last pixels differently.  */
        draw((x0, y0).by(1, y1 - y0), id_first);
        draw((x0 + 1, y0).by(x1 - x0 - 2, y1 - y0), id);
        draw((x1 - 1, y0).by(1, y1 - y0), id_last);
    } else if (x1 - x0) > (y1 - y0) && (y1 - y0) >= 3 {
        /* A horizontal divider, at least three pixels high: Draw first and
        last pixels differently.  */
        draw((x0, y0).by(x1 - x0, 1), id_first);
        draw((x0, y0 + 1).by(x1 - x0, y1 - y0 - 2), id);
        draw((x0, y1 - 1).by(x1 - x0, 1), id_last);
    } else {
        /* In any other case do not draw the first and last pixels
        differently.  */
        draw((x0, y0).to(x1, y1), id);
    }
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
