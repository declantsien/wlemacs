use crate::bindings::{face as Face, face_underline_type};
use crate::font::FontRef;
use crate::lisp::ExternalPtr;

pub type FaceRef = ExternalPtr<Face>;

impl FaceRef {
    pub fn font(&self) -> FontRef {
        FontRef::new(self.font)
    }

    pub fn underline_type(&self) -> face_underline_type::Type {
        self.underline()
    }

    pub fn bg_color(&self) -> ::libc::c_ulong {
        self.background
    }

    pub fn fg_color(&self) -> ::libc::c_ulong {
        self.foreground
    }
}
