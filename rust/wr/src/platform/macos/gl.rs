use crate::canvas::WrCanvas;
use crate::gfx::context::{GLContext, GLContextTrait};
use gleam::gl;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::ptr::NonNull;
use webrender_api::units::{DeviceIntSize, LayoutToDeviceScale};

struct EmacsView {}

pub fn raw_display_handle() -> raw_window_handle::RawDisplayHandle {
    let raw = AppKitDisplayHandle::new();
    RawDisplayHandle::AppKit(raw)
}

pub fn raw_window_handle(view: &EmacsView) -> raw_window_handle::RawWindowHandle {
    let handle = AppKitWindowHandle::new(NonNull::from(view).cast());
    RawWindowHandle::AppKit(handle)
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_frame_gl_context(
    view: &EmacsView,
    width: libc::c_int,
    height: libc::c_int,
    scale_factor: libc::c_double,
)  -> *mut WrCanvas {
    let display_handle = raw_display_handle();
    let window_handle = raw_window_handle(view);
    println!("window handle: {window_handle:?}");
    let device_size = DeviceIntSize::new(width, height);
    let size = device_size.to_f32() / LayoutToDeviceScale::new(1.0 / (scale_factor as f32));
    let mut gl_context = GLContext::build(display_handle, window_handle, size.to_i32());

    let data = Box::new(WrCanvas::build(gl_context, device_size, scale_factor));
    Box::into_raw(data)
}

// #[no_mangle]
// pub extern "C" fn wr_init(f: *mut Frame) -> *mut WrCanvas {
//     // assert!(unsafe { !is_in_render_thread() });
//     let f: FrameRef = f.into();

//     //     let state = Box::new(WrState {
//     //     pipeline_id,
//     //     frame_builder: WebRenderFrameBuilder::new(pipeline_id),
//     // });

//     // Box::into_raw(state)

//     let data = Box::new(WrCanvas::build(f));
//     Box::into_raw(data)
// }
