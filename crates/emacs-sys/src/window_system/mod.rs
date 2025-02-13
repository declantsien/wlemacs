#[cfg(have_pgtk)]
mod pgtk;
#[cfg(have_pgtk)]
pub use pgtk::*;
#[cfg(have_wayland_client)]
mod wayland_client;
// #[cfg(have_wayland_client)]
// pub use wayland_client::*;

use crate::frame::FrameRef;
use euclid::*;
pub struct LayoutPixel;
pub type LayoutSize = Size2D<f32, LayoutPixel>;
pub struct DevicePixel;
pub type DeviceIntSize = Size2D<i32, DevicePixel>;

impl FrameRef {
    pub fn logical_size(&self) -> LayoutSize {
        LayoutSize::new(self.pixel_width as f32, self.pixel_height as f32)
    }

    pub fn physical_size(&self) -> DeviceIntSize {
        let size = self.logical_size() * euclid::Scale::new(self.scale_factor() as f32);
        size.to_i32()
    }
}
