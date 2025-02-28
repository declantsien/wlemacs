use crate::canvas::WrCanvas;
use crate::platform::pixel_to_color;
use crate::types::{
    block_input, face, face_id, frame, unblock_input, EmacsIntRect, FACE_FROM_ID,
    FACE_FROM_ID_OR_NULL,
};
use crate::util::HandyDandyRectBuilder;

impl<'a> frame {
    pub fn from_ptr(f: *mut frame) -> Option<&'a mut frame> {
        unsafe { f.as_mut() }
    }

    pub fn renderer(&mut self) -> Option<&mut WrCanvas> {
        unsafe { self.output_data().and_then(|d| d.wr_data.as_mut()) }
    }

    pub fn default_face(&mut self) -> Option<&mut face> {
        self.face_from_id_or_null(face_id::DEFAULT_FACE_ID)
    }

    pub fn face_from_id(&mut self, face: face_id) -> Option<&mut face> {
        let f = unsafe { FACE_FROM_ID(self, face as libc::c_int) };
        unsafe { f.as_mut() }
    }

    pub fn face_from_id_or_null(&mut self, face: face_id) -> Option<&mut face> {
        let f = unsafe { FACE_FROM_ID_OR_NULL(self, face as libc::c_int) };
        unsafe { f.as_mut() }
    }

    pub fn clear_area(&mut self, r: EmacsIntRect) {
        if self.default_face().is_none() || self.renderer().is_none() {
            return;
        }

        unsafe { block_input() };
        let clip = (0, 0).by(self.pixel_width, self.pixel_height);
        let clear_color = pixel_to_color(self.default_face().unwrap().background);
        self.renderer().unwrap().dp_push_rect(
            r.to_f32(),
            Some(clip),
            false,
            false,
            false,
            clear_color,
        );
        unsafe { unblock_input() };
    }
}
