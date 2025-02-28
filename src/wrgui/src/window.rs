use std::cmp::max;

use crate::types::{
    frame, glyph_row, glyph_row_area, window, window_box, window_to_frame_pixel_y, EmacsIntRect,
    XFRAME,
};
use crate::util::HandyDandyRectBuilder;

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
}
