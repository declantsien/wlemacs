use webrender::api::ColorF;

pub fn pixel_to_color(c: u64) -> ColorF {
    let a = (((c >> 24) & 0xff) as f64 / 255.0) as f32;
    let r = (((c >> 16) & 0xff) as f64 / 255.0) as f32;
    let g = (((c >> 8) & 0xff) as f64 / 255.0) as f32;
    let b = ((c & 0xff) as f64 / 255.0) as f32;
    ColorF::new(r, g, b, a)
}
