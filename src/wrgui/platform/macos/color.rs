use objc2::rc::Retained;
use objc2_app_kit::NSColor;
use webrender::api::ColorF;

pub fn pixel_to_color(c: u64) -> ColorF {
    let a = (((c >> 24) & 0xff) as f64 / 255.0) as f32;
    let r = (((c >> 16) & 0xff) as f64 / 255.0) as f32;
    let g = (((c >> 8) & 0xff) as f64 / 255.0) as f32;
    let b = ((c & 0xff) as f64 / 255.0) as f32;
    ColorF::new(r, g, b, a)
}

pub fn ns_color_to_color_f(c: *mut ::libc::c_void) -> ColorF {
    let c: Retained<NSColor> = unsafe { Retained::retain(c.cast()) }.unwrap();
    let mut r: f64 = 0.0;
    let mut g: f64 = 0.0;
    let mut b: f64 = 0.0;
    let mut a: f64 = 0.0;
    unsafe { c.getRed_green_blue_alpha(&mut r, &mut g, &mut b, &mut a) };
    ColorF::new(r as f32, g as f32, b as f32, a as f32)
}
