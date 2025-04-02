use objc2_foundation::NSRect;

use crate::types::{frame, ns_output, EmacsPoint, EmacsRect, EmacsSize, ExternalPtr};
pub type OutputData = ns_output;
pub type OutputDataRef = ExternalPtr<OutputData>;

pub fn ns_rect_to_emacs(r: &NSRect) -> EmacsRect {
    EmacsRect::from_origin_and_size(
        EmacsPoint::new(r.origin.x as f32, r.origin.y as f32),
        EmacsSize::new(r.size.width as f32, r.size.height as f32),
    )
}

pub fn frame_output_data(f: &frame) -> &OutputData {
    unsafe { f.output_data.ns.as_ref().unwrap() }
}
