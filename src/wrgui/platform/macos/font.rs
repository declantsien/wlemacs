use core_foundation::base::TCFType;
use core_text::font::CTFont;
use webrender_api::{FontTemplate, NativeFontHandle};

use crate::types::{font, macfont_info};
pub type FontInfo = macfont_info;

impl<'a> macfont_info {
    pub fn from_font(f: &font) -> Option<&macfont_info> {
        let font_info = f as *const font as *const macfont_info;
        unsafe { font_info.as_ref() }
    }

    pub fn from_ptr_mut(f: *mut macfont_info) -> Option<&'a mut macfont_info> {
        unsafe { f.as_mut() }
    }

    pub fn from_ptr(f: *mut macfont_info) -> Option<&'a macfont_info> {
        unsafe { f.as_ref() }
    }

    pub fn font_template(&self) -> FontTemplate {
        let ct_font = unsafe { CTFont::wrap_under_get_rule(self.macfont) };
        let descriptor = ct_font.copy_descriptor();
        let font_path = descriptor.font_path();
        FontTemplate::Native(NativeFontHandle {
            name: ct_font.postscript_name(),
            path: font_path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("".to_string()),
        })
    }
}

impl font {
    pub fn font_info(&self) -> Option<&FontInfo> {
        let font_info = self as *const font as *const macfont_info;
        unsafe { font_info.as_ref() }
    }
    pub fn font_info_mut(&mut self) -> Option<&mut FontInfo> {
        let font_info = self as *mut font as *mut macfont_info;
        unsafe { font_info.as_mut() }
    }

    pub fn font_template(&self) -> FontTemplate {
        let font_info = self.font_info().unwrap();
        font_info.font_template()
    }
}
