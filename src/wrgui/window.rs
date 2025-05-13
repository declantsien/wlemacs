use std::cmp::max;

use crate::types::{
    frame, get_phys_cursor_geometry, get_phys_cursor_glyph, glyph, glyph_matrix, glyph_row,
    glyph_row_area, text_cursor_kinds, window, window_box, window_to_frame_pixel_y, EmacsIntRect,
    Lisp_Object, BUF_BEGV, BUF_PT, BUF_ZV, XBUFFER, XFRAME, XWINDOW,
};
use crate::util::HandyDandyRectBuilder;

/// cbindgen:ignore
unsafe extern "C" {
    pub fn window_box_left_offset(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn window_box_right(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn window_box_left(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn cursor_in_mouse_face_p(w: *const window) -> bool;
}

impl<'a> window {
    pub fn from_lisp(w: Lisp_Object) -> Option<&'a window> {
        let w = unsafe { XWINDOW(w) };
        unsafe { w.as_ref() }
    }

    pub fn from_ptr(f: *mut window) -> Option<&'a window> {
        unsafe { f.as_ref() }
    }

    pub fn from_ptr_mut(f: *mut window) -> Option<&'a mut window> {
        unsafe { f.as_mut() }
    }

    pub fn x_frame(&self) -> Option<&frame> {
        let f = unsafe { XFRAME(self.frame) };
        unsafe { f.as_ref() }
    }

    pub fn x_frame_mut(&self) -> Option<&mut frame> {
        let f = unsafe { XFRAME(self.frame) };
        unsafe { f.as_mut() }
    }

    pub fn to_frame_pixel_y(&mut self, y: ::libc::c_int) -> ::libc::c_int {
        unsafe { window_to_frame_pixel_y(self, y) }
    }

    pub fn box_left_offset(&self, area: impl Into<glyph_row_area>) -> ::libc::c_int {
        unsafe { window_box_left_offset(self, area.into()) }
    }

    pub fn box_right(&self, area: impl Into<glyph_row_area>) -> ::libc::c_int {
        unsafe { window_box_right(self, area.into()) }
    }

    pub fn box_left(&self, area: impl Into<glyph_row_area>) -> ::libc::c_int {
        unsafe { window_box_left(self, area.into()) }
    }

    pub fn cursor_in_mouse_face_p(&self) -> bool {
        unsafe { cursor_in_mouse_face_p(self) }
    }

    pub fn window_box(&mut self, area: impl Into<glyph_row_area>) -> EmacsIntRect {
        let mut r = EmacsIntRect::zero().to_rect();

        unsafe {
            window_box(
                self,
                area.into(),
                &mut r.origin.x,
                &mut r.origin.y,
                &mut r.size.width,
                &mut r.size.height,
            );
        }

        r.to_box2d()
    }

    pub fn row_clip_bounds(
        &mut self,
        row: &glyph_row,
        area: impl Into<glyph_row_area>,
    ) -> EmacsIntRect {
        let mut r = EmacsIntRect::zero().to_rect();

        unsafe {
            window_box(
                self,
                area.into(),
                &mut r.origin.x,
                &mut r.origin.y,
                &mut r.size.width,
                &mut (0 as i32),
            );
        }
        let x = r.origin.x;
        let y = self.to_frame_pixel_y(max(0, row.y));
        let y = max(y, r.origin.y);
        let width = r.size.width;
        let height = row.visible_height;
        (x, y).by(width, height).to_i32()
    }

    pub fn phys_cursor_glyph(&mut self) -> Option<&mut glyph> {
        unsafe { get_phys_cursor_glyph(self).as_mut() }
    }

    pub fn phys_cursor_geometry(
        &mut self,
        row: &mut glyph_row,
        cursor_glyph: &mut glyph,
    ) -> EmacsIntRect {
        let mut rect = EmacsIntRect::zero().to_rect();
        unsafe {
            get_phys_cursor_geometry(
                self,
                row,
                cursor_glyph,
                &mut rect.origin.x,
                &mut rect.origin.y,
                &mut rect.size.height,
            )
        };
        rect.to_box2d()
    }

    #[allow(unused_variables)]
    pub fn draw_bar_cursor(&self, row: &glyph_row, width: ::libc::c_int, kind: text_cursor_kinds) {}

    fn text_bottom_y(&mut self) -> i32 {
        unsafe { crate::types::window_text_bottom_y(self) }
    }

    fn scroll_bar_area_height(&mut self) -> i32 {
        unsafe { crate::types::window_scroll_bar_area_height(self) }
    }

    #[inline]
    pub fn update(&mut self) {
        let desired_matrix = unsafe { self.desired_matrix.as_ref().unwrap() };

        // #ifdef GLYPH_DEBUG
        //   /* Check that W's frame doesn't have glyph matrices.  */
        //   eassert (FRAME_WINDOW_P (XFRAME (WINDOW_FRAME (w))));
        // #endif

        let mut row = 0;
        let mut end = desired_matrix.nrows - 1;
        let yb = self.text_bottom_y();

        let tab_line_row = desired_matrix.tab_line_row();
        if tab_line_row.is_some() {
            row += 1;
        }

        let mut header_line_row: glyph_row;

        let changed_p = false;
        let mouse_face_overwritten_p = false;
        let invisible_rows_marked = false;

        /* Update the mode line, if necessary.  */
        match desired_matrix.mode_line_row() {
            Some(row) => {
                row.y = yb + self.scroll_bar_area_height();
                self.update_line(desired_matrix.nrows - 1, mouse_face_overwritten_p);
            }
            None => todo!(),
        }

        /* Find first enabled row.  Optimizations in redisplay_internal
        may lead to an update with only one row enabled.  There may
        be also completely empty matrices.  */
        // while row < end && !desired_matrix.row(row).enabled_p() {
        //     row += 1;
        // }

        //       /* Take note of the tab line, if there is one.  We will
        //       update it below, after updating all of the window's lines.  */
        //       if row.mode_line_p() && row.tab_line_p()
        //   {
        //       tab_line_row = Some(row);
        //       ++row;
        //   }
        // else
        //   tab_line_row = NULL;
    }

    pub fn update_line(&mut self, vpos: i32, mouse_face_overwritten_p: bool) {
        // bm update_window_line
    }

    pub fn build_display_list(&self) {
        let buffer = unsafe { XBUFFER(self.contents) };
        log::trace!(
            "PT = {}, BEGV = {}. ZV = {}",
            unsafe { BUF_PT(buffer) },
            unsafe { BUF_BEGV(buffer) },
            unsafe { BUF_ZV(buffer) }
        );
        log::trace!("Cursor pos {:?}", self.cursor);
    }
}

impl glyph_matrix {
    pub fn rows(&self) -> &[glyph_row] {
        unsafe { std::slice::from_raw_parts(self.rows, self.nrows as usize) }
    }

    pub fn row(&self, row: usize) -> &mut glyph_row {
        assert!(!self.rows.is_null());
        assert!(row >= 0 && row < self.nrows as usize);

        match unsafe { self.rows.offset(row as isize).as_mut() } {
            Some(row) => row,
            None => unreachable!(),
        }
    }

    pub fn tab_line_row(&self) -> Option<&mut glyph_row> {
        let row = self.row(0);
        if row.mode_line_p() && row.tab_line_p() {
            return Some(row);
        }
        None
    }

    /// Return the row reserved for the mode line in MATRIX.
    /// Row MATRIX->nrows - 1 is always reserved for the mode line.
    pub fn mode_line_row(&self) -> Option<&mut glyph_row> {
        let row = self.row((self.nrows - 1) as usize);
        if row.mode_line_p() {
            return Some(row);
        }
        None
    }

    pub fn build_display_list(&self) {
        use glyph_row_area::*;
        self.rows().iter().enumerate().for_each(|(vpos, row)| {
            for area in [LEFT_MARGIN_AREA, TEXT_AREA, RIGHT_MARGIN_AREA].iter() {
                println!("--row {vpos}---");
                let glyphs = row.glyphs(*area);
                glyphs.iter().for_each(|g| {
                    let ch = unsafe { g.u.ch };
                    let face_id = g.face_id();
                    println!("char {:?}, face_id {face_id}", char::from_u32(ch));
                });
            }
        });
    }
}

impl glyph_row {
    pub fn glyphs(&self, area: glyph_row_area) -> &[glyph] {
        let area: i32 = area.into();

        let raw_glyphs = self.glyphs[area as usize];
        let mut used = self.used[area as usize];

        /* Glyph for a line end in text.  */
        if area == glyph_row_area::TEXT_AREA.into()
            && used == 0
            && unsafe { raw_glyphs.as_ref().map(|g| g.charpos > 0).unwrap_or(false) }
        {
            used += 1;
        }
        unsafe { std::slice::from_raw_parts(raw_glyphs, used as usize) }
    }
}

// window body/modeline/echo area/cursor/frige in different layer
// body using a scroll frame
// get window content height or just
// window current matrix to display items
// update display items with desired matrix
// draw glyphs details draw_glyphs from xdisp.c
