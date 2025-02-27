use crate::types::{
    glyph_row_area, window_box, window_to_frame_pixel_y, EmacsIntRect, FrameRef, WindowRef, XFRAME,
};

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
}
