use core_foundation::base::TCFType;
use core_text::font::CTFont;
use webrender_api::{FontTemplate, NativeFontHandle};

use crate::types::{macfont_info, ExternalPtr, FontRef};
pub type MacfontInfoRef = ExternalPtr<macfont_info>;

impl FontRef {
    pub fn font_template(&mut self) -> FontTemplate {
        let font_info = MacfontInfoRef::new(self.as_mut() as *mut macfont_info);
        let ct_font = unsafe { CTFont::wrap_under_get_rule(font_info.macfont) };
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
