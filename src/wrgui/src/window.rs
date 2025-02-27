use std::cmp::max;

use crate::types::{
    glyph_row, glyph_row_area, window_box, window_to_frame_pixel_y, EmacsIntRect, FrameRef,
    WindowRef, XFRAME,
};
use crate::util::HandyDandyRectBuilder;

impl WindowRef {
    pub fn x_frame(&mut self) -> FrameRef {
        let f = unsafe { XFRAME(self.frame) };
        FrameRef::new(f)
    }

    pub fn to_frame_pixel_y(&mut self, y: ::libc::c_int) -> ::libc::c_int {
        unsafe { window_to_frame_pixel_y(self.as_mut(), y) }
    }

    pub fn window_box(&mut self, area: impl Into<glyph_row_area>) -> EmacsIntRect {
        let mut r = EmacsIntRect::zero().to_rect();

        unsafe {
            window_box(
                self.as_mut(),
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
                self.as_mut(),
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
