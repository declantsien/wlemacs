#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs_sys;

pub mod frame;

pub mod color;
pub mod fns;
pub mod image;
pub mod output;

mod face;
pub mod font;
mod fringe;
mod glyph;
mod texture;
mod util;
