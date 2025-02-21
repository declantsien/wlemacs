#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

// pub mod frame;

// pub mod bindings;
// pub mod color;
// pub mod image;
// pub mod output;

// /// cbindgen:ignore
// mod capi;
// mod face;
pub mod font;
pub mod types;
// mod glyph;
// mod texture;
// mod util;

#[no_mangle]
pub extern "C" fn test() {
    todo!()
}
