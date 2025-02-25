#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

// pub mod frame;

// pub mod bindings;
pub mod color;
// pub mod image;
// pub mod output;

/// cbindgen:ignore
mod emacs;
// mod face;
mod canvas;
pub mod font;
pub mod gui;
pub mod types;
// mod glyph;
mod texture;
mod util;

#[no_mangle]
pub extern "C" fn test() {
    todo!()
}

pub mod platform {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::color::*;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::font;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::gui;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::types;

    // #[cfg(any(
    //     target_os = "android",
    //     all(unix, not(any(target_os = "ios", target_os = "macos")))
    // ))]
    // pub use super::platform::unix::font;
    // #[cfg(target_os = "windows")]
    // pub use super::platform::windows::font;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub mod macos {
        pub mod color;
        pub mod font;
        pub mod gui;
        pub mod types;
    }
    // #[cfg(any(
    //     target_os = "android",
    //     all(unix, not(any(target_os = "macos", target_os = "ios")))
    // ))]
    // pub mod unix {
    //     pub mod font;
    // }
    // #[cfg(target_os = "windows")]
    // pub mod windows {
    //     pub mod font;
    // }
}

// #[cfg(any(glutin, surfman, gtk3))]
pub mod gfx {
    pub mod context;

    pub mod context_impl {
        // #[cfg(glutin)]
        // pub use crate::gfx::context_impl::glutin::*;
        // #[cfg(gtk3)]
        // pub use crate::gfx::context_impl::gtk3::*;
        // #[cfg(surfman)]
        pub use crate::gfx::context_impl::surfman::*;

        // #[cfg(glutin)]
        // pub mod glutin;
        // #[cfg(gtk3)]
        // pub mod gtk3;
        // #[cfg(surfman)]
        pub mod surfman;
    }
}

use crate::canvas::WrCanvas;
use crate::emacs::Emacs_Rectangle;
use crate::font::macfont_font_tpl;
use crate::types::WrVecU32;
use crate::util::HandyDandyRectBuilder;
use core_text::font::CTFontRef;
use platform::pixel_to_color;
use types::{EmacsIntPoint, EmacsIntSize, EmacsLength, EmacsPoint, EmacsRect, LayoutLength};
use webrender_api::{AlphaType, CommonItemProperties, FontTemplate, ImageRendering};

#[no_mangle]
pub extern "C" fn wr_clear_area(
    canvas: &mut WrCanvas,
    color: ::std::os::raw::c_ulong,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    canvas.push_rect(color, (x, y).by(width, height), None);
}

#[no_mangle]
pub extern "C" fn wr_flush(canvas: &mut WrCanvas) {
    canvas.flush();
}

// #[no_mangle]
// pub extern "C" fn wr_dp_push_rect(
//     canvas: &mut WrCanvas,
//     rect: &Emacs_Rectangle,
//     clip: &Emacs_Rectangle,
//     is_backface_visible: bool,
//     force_antialiasing: bool,
//     is_checkerboard: bool,
//     color_pixel: ::libc::c_ulong,
// ) {
//     // debug_assert!(unsafe { !is_in_render_thread() });
//     canvas.dp_push_rect(
//         rect.into(),
//         clip.into(),
//         is_backface_visible,
//         force_antialiasing,
//         is_checkerboard,
//         color_pixel,
//     );
// }
#[no_mangle]
pub extern "C" fn wr_vec_u32_free(v: WrVecU32) {
    v.into_vec();
}

#[no_mangle]
pub extern "C" fn wr_macfont_draw(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    ct_font_ref: CTFontRef,
    char2b: &mut WrVecU32,
    from: ::libc::c_int,
    to: ::libc::c_int,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    glyph_size: ::libc::c_int,
    padding_p: bool,
) {
    let font_tpl = macfont_font_tpl(ct_font_ref);
    wr_font_draw(
        canvas,
        color_pixel,
        font_tpl,
        char2b,
        from,
        to,
        x,
        y,
        width,
        height,
        glyph_size,
        padding_p,
    );
}

#[no_mangle]
fn wr_font_draw(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    font_tpl: FontTemplate,
    char2b: &mut WrVecU32,
    from: ::libc::c_int,
    to: ::libc::c_int,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    glyph_size: ::libc::c_int,
    padding_p: bool,
) {
    use crate::color::pixel_to_color;
    use webrender_api::{
        CommonItemProperties, FontInstanceOptions, FontInstancePlatformOptions, GlyphInstance,
    };

    println!("{:?}", font_tpl);
    let font_key = canvas.wr_add_font(font_tpl);
    let font_instance_key = canvas.wr_add_font_instance(
        font_key,
        EmacsLength::new(glyph_size as f32),
        Some(FontInstanceOptions::default()),
        Some(FontInstancePlatformOptions::default()),
        Vec::new(),
    );
    let glyph_indices = char2b.as_slice();
    // println!("{:?}", glyph_indices);
    let glyph_dimensions = canvas.glyph_dimensions(font_instance_key, glyph_indices.to_vec());
    // println!("{:?}", glyph_dimensions);
    let glyph_dimensions = glyph_dimensions.as_slice();
    let scale_factor = canvas.emacs_to_layout_scale();
    let visible_rect = (x, y).by(width, height);
    println!("visible_rect {visible_rect:?}");
    // FIXME visible_rect set above is not correct here, hard code it for now
    let visible_rect = (0, 0).by(10000, 10000);
    let mut x = EmacsLength::new(x as f32);
    let mut glyph_instances: Vec<GlyphInstance> = vec![];
    (from..to).for_each(|i| {
        let index = glyph_indices[i as usize];
        // wr get_glyph_dimensions return none for ‘empty’ textures (height or width = 0)
        // spaces (’ ’) will mostly be None
        // glyphinstance type is using layoutpixel
        let glyph_instance = GlyphInstance {
            index,
            point: EmacsPoint::new(x.get(), y as f32) * scale_factor,
        };
        // scale back to device pixel
        let advance_width: LayoutLength = glyph_dimensions[i as usize]
            .map(|d| LayoutLength::new(d.advance))
            .unwrap_or(LayoutLength::new(0.0));

        if padding_p {
            x += EmacsLength::new(1.0);
        } else {
            x += advance_width / scale_factor;
        }
        glyph_instances.push(glyph_instance);
    });

    canvas.display(|builder, space_and_clip, scale| {
        let foreground_color = pixel_to_color(color_pixel);

        // draw foreground
        if !glyph_instances.is_empty() {
            let visible_rect = visible_rect;

            builder.push_text(
                &CommonItemProperties::new(visible_rect * scale, space_and_clip),
                visible_rect * scale,
                &glyph_instances,
                font_instance_key,
                foreground_color,
                None,
            );
        }
    });
}

#[no_mangle]
pub extern "C" fn wr_scroll_run(
    canvas: &mut WrCanvas,
    viewport: &Emacs_Rectangle,
    new_frame_position: &Emacs_Rectangle,
) {
    use webrender_api::{AlphaType, ColorF, CommonItemProperties, ImageRendering};

    let viewport: EmacsRect = viewport.into();
    let new_frame_position: EmacsRect = new_frame_position.into();
    if let Some(image_key) = canvas.get_previous_frame() {
        canvas.display(|builder, space_and_clip, scale| {
            builder.push_image(
                &CommonItemProperties::new(viewport * scale, space_and_clip),
                new_frame_position * scale,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        });
    }
}
