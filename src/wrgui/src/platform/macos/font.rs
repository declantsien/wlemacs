use core_foundation::base::TCFType;
use core_text::font::{CTFont, CTFontRef};
use webrender_api::{FontTemplate, NativeFontHandle};

use crate::font::wr_font_metrics_impl;
use crate::types::{font, macfont_info, ExternalPtr, WrFontMetrics};
pub type MacfontInfoRef = ExternalPtr<macfont_info>;

pub fn macfont_font_tpl(f: *mut font) -> FontTemplate {
    let f = MacfontInfoRef::new(f as *mut macfont_info);

    let ct_font = unsafe { CTFont::wrap_under_get_rule(f.macfont) };
    let point_size = ct_font.pt_size();
    let descriptor = ct_font.copy_descriptor();
    let font_path = descriptor.font_path();
    let mut font_metrics = WrFontMetrics::default();
    FontTemplate::Native(NativeFontHandle {
        name: ct_font.postscript_name(),
        path: font_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("".to_string()),
    })
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_add_font(font: *mut font) {
    let f = MacfontInfoRef::new(font as *mut macfont_info);
    let ct_font = unsafe { CTFont::wrap_under_get_rule(f.macfont) };
    let point_size = ct_font.pt_size();
    let descriptor = ct_font.copy_descriptor();
    let font_path = descriptor.font_path();
    // println!("cf_name: {:?}, path: {:?}, size: {:?}",
    //     ct_font.postscript_name(), font_path, point_size);
    let mut font_metrics = WrFontMetrics::default();
    let font_tpl = FontTemplate::Native(NativeFontHandle {
        name: ct_font.postscript_name(),
        path: font_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("".to_string()),
    });
    wr_font_metrics_impl(
        font_tpl,
        point_size as ::libc::c_int,
        1.0,
        &mut font_metrics,
    );
}
