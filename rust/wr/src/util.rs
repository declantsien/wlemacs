use webrender::api::units::*;

pub trait HandyDandyRectBuilder {
    fn to(&self, x2: i32, y2: i32) -> DeviceRect;
    fn by(&self, w: i32, h: i32) -> DeviceRect;
}
// Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// values to build a f32 LayoutRect
impl HandyDandyRectBuilder for (i32, i32) {
    fn to(&self, x2: i32, y2: i32) -> DeviceRect {
        DeviceIntRect::from_origin_and_size(
            DeviceIntPoint::new(self.0, self.1),
            DeviceIntSize::new(x2 - self.0, y2 - self.1),
        )
        .to_f32()
    }

    fn by(&self, w: i32, h: i32) -> DeviceRect {
        DeviceIntRect::from_origin_and_size(
            DeviceIntPoint::new(self.0, self.1),
            DeviceIntSize::new(w, h),
        )
        .to_f32()
    }
}
