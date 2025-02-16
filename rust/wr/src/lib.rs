#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs_sys;

pub mod frame;

pub mod bindings;
pub mod color;
pub mod image;
pub mod output;

mod capi;
mod face;
pub mod font;
mod glyph;
mod texture;
mod util;
