#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
// #![feature(concat_idents)]
#![allow(non_upper_case_globals)]

// In Rust, a delegate is a trait that the embedder can install its own impl for.
// There are also different OpenGL versions across multiple platforms, which can be challenging to configure and link. Verso is experimenting with using Glutin for better configuration and attempting to get closer to the general Rust ecosystem.
// Off main thread HTML parsing in Servo https://servo.org/blog/2017/08/23/gsoc-parsing/
// https://servo.zulipchat.com/

mod frame;

// pub mod bindings;
mod bitmap;
mod color;
mod dispnew;
mod glyph_string;
mod window;
// mod image;
// pub mod output;

/// cbindgen:ignore
// mod emacs;
// mod face;
// mod fringe;
mod canvas;
pub mod font;
pub mod gui;
pub mod types;
// mod glyph;
mod texture;
mod util;

pub mod platform {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::color::*;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::font;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::frame;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::glyph_string;
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
        pub mod frame;
        pub mod glyph_string;
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
