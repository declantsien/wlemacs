use crate::gfx::context_impl::ContextImpl;
use webrender_api::units::LayoutIntSize;

use gleam::gl::Gl;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::rc::Rc;

pub type GLContext = ContextImpl;

pub trait GLContextTrait {
    fn build(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        size: LayoutIntSize,
    ) -> Self;

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>);

    fn swap_buffers(&self);

    fn load_gl(&self) -> Rc<dyn Gl>;

    fn resize(&self, size: &LayoutIntSize);

    fn ensure_is_current(&mut self);
}
