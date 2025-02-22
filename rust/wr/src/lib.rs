#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

// pub mod frame;

// pub mod bindings;
pub mod color;
// pub mod image;
// pub mod output;

// /// cbindgen:ignore
mod capi;
// mod face;
mod canvas;
pub mod font;
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
    pub use super::platform::macos::gl;
    // #[cfg(any(
    //     target_os = "android",
    //     all(unix, not(any(target_os = "ios", target_os = "macos")))
    // ))]
    // pub use super::platform::unix::font;
    // #[cfg(target_os = "windows")]
    // pub use super::platform::windows::font;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub mod macos {
        pub mod gl;
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
        #[cfg(glutin)]
        pub use crate::gfx::context_impl::glutin::*;
        #[cfg(gtk3)]
        pub use crate::gfx::context_impl::gtk3::*;
        // #[cfg(surfman)]
        pub use crate::gfx::context_impl::surfman::*;

        #[cfg(glutin)]
        pub mod glutin;
        #[cfg(gtk3)]
        pub mod gtk3;
        // #[cfg(surfman)]
        pub mod surfman;
    }
}

use crate::util::HandyDandyRectBuilder;
use crate::canvas::WrCanvas;

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

