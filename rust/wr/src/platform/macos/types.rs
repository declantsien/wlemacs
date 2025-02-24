use objc2_foundation::NSRect;

use crate::types::{EmacsPoint, EmacsRect, EmacsSize};

pub fn ns_rect_to_emacs(r: &NSRect) -> EmacsRect {
    EmacsRect::from_origin_and_size(
        EmacsPoint::new(r.origin.x as f32, r.origin.y as f32),
        EmacsSize::new(r.size.width as f32, r.size.height as f32),
    )
}
