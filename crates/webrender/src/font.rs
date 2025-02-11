use crate::frame::FrameExtWrCommon;
use emacs_sys::bindings::font_info;
use emacs_sys::lisp::ExternalPtr;
use webrender::api::FontInstanceKey;
use webrender::api::GlyphDimensions;
use webrender::api::GlyphIndex;

use emacs_sys::frame::FrameRef;

pub type FontInfoRef = ExternalPtr<font_info>;
