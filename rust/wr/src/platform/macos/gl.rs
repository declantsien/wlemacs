use crate::canvas::WrCanvas;
use crate::gfx::context::{GLContext, GLContextTrait};
use crate::types::{EmacsIntPoint, EmacsIntSize, EmacsPoint, EmacsRect};
use crate::util::HandyDandyRectBuilder;
use objc2_app_kit::NSColor;
use objc2_foundation::NSRect;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::ptr::NonNull;
use webrender_api::{AlphaType, CommonItemProperties, ImageRendering};

use super::color::ns_color_to_color_f;
use super::types::ns_rect_to_emacs;

pub struct EmacsView {}

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
) -> *mut WrCanvas {
    use crate::types::{EmacsIntSize, EmacsToDeviceScale};

    let display_handle = raw_display_handle();
    let window_handle = raw_window_handle(view);
    println!("window handle: {window_handle:?}");
    let size = EmacsIntSize::new(width, height);
    let device_size = (size.to_f32() * EmacsToDeviceScale::new(scale_factor as f32)).to_i32();
    let gl_context = GLContext::build(display_handle, window_handle, device_size.to_i32());

    let data = Box::new(WrCanvas::build(gl_context, size, scale_factor));
    Box::into_raw(data)
}

#[no_mangle]
pub extern "C" fn wr_dp_push_rect(
    canvas: &mut WrCanvas,
    rect: &NSRect,
    clip: &NSRect,
    is_backface_visible: bool,
    force_antialiasing: bool,
    is_checkerboard: bool,
    color: &NSColor,
) {
    // debug_assert!(unsafe { !is_in_render_thread() });
    canvas.dp_push_rect(
        ns_rect_to_emacs(rect),
        ns_rect_to_emacs(clip),
        is_backface_visible,
        force_antialiasing,
        is_checkerboard,
        ns_color_to_color_f(color),
    );
}

#[no_mangle]
pub extern "C" fn wr_draw_fringe_bitmap(
    canvas: &mut WrCanvas,
    which: ::libc::c_int,
    pos_x: ::libc::c_int,
    pos_y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    bitmap_width: ::libc::c_int,
    bitmap_height: ::libc::c_int,
    bits: *mut ::libc::c_ushort,
    foreground: &NSColor,
    clip_bounds: &NSRect,
) {
    println!("draw fringe");
    let clip_bounds: EmacsRect = ns_rect_to_emacs(clip_bounds);

    let pos = EmacsIntPoint::new(pos_x, pos_y).to_f32();

    let image_clip_rect: EmacsRect = {
        if which > 0 {
            (pos_x, pos_y).by(width, height)
        } else {
            EmacsRect::zero()
        }
    };

    let image = canvas.get_or_create_fringe_bitmap(
        which,
        EmacsIntSize::new(bitmap_width, bitmap_height),
        bits,
    );

    // Fixed image_clip_rect
    let image_clip_rect = image_clip_rect
        .intersection(&clip_bounds)
        .unwrap_or_else(|| EmacsRect::zero());

    canvas.display(|builder, space_and_clip, scale| {
        if let Some(image) = &image {
            println!("draw fringe has image");
            let image_display_rect = EmacsRect::new(
                pos,
                EmacsPoint::new(image.width as f32, image.height as f32),
            );
            // render image
            builder.push_image(
                &CommonItemProperties::new(image_clip_rect * scale, space_and_clip),
                image_display_rect * scale,
                ImageRendering::Auto,
                AlphaType::Alpha,
                image.image_key,
                ns_color_to_color_f(foreground),
            );
        }
    });
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
