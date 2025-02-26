#![allow(dead_code)]

use crate::types::{EmacsIntPoint, EmacsIntRect, EmacsIntSize, EmacsRect};

pub trait HandyDandyRectBuilder {
    fn to(&self, x2: i32, y2: i32) -> EmacsRect;
    fn by(&self, w: i32, h: i32) -> EmacsRect;
}
// Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// values to build a f32 LayoutRect
impl HandyDandyRectBuilder for (i32, i32) {
    fn to(&self, x2: i32, y2: i32) -> EmacsRect {
        EmacsIntRect::from_origin_and_size(
            EmacsIntPoint::new(self.0, self.1),
            EmacsIntSize::new(x2 - self.0, y2 - self.1),
        )
        .to_f32()
    }

    fn by(&self, w: i32, h: i32) -> EmacsRect {
        EmacsIntRect::from_origin_and_size(
            EmacsIntPoint::new(self.0, self.1),
            EmacsIntSize::new(w, h),
        )
        .to_f32()
    }
}
