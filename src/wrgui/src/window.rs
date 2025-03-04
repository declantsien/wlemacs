use std::cmp::max;

use crate::types::{
    frame, get_phys_cursor_geometry, get_phys_cursor_glyph, glyph, glyph_row, glyph_row_area,
    text_cursor_kinds, window, window_box, window_to_frame_pixel_y, EmacsIntRect, XFRAME,
};
use crate::util::HandyDandyRectBuilder;

// cbindgen:ignore
unsafe extern "C" {
    pub fn window_box_left_offset(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn window_box_right(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn window_box_left(arg1: *const window, arg2: glyph_row_area) -> ::libc::c_int;
    pub fn cursor_in_mouse_face_p(w: *const window) -> bool;
}

impl<'a> window {
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
}
