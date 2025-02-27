use crate::canvas::WrCanvas;
use crate::types::{face_id, FaceRef, FrameRef, FACE_FROM_ID, FACE_FROM_ID_OR_NULL};

impl FrameRef {
    pub fn renderer(&mut self) -> &mut WrCanvas {
        unsafe { self.output_data().wr_data.as_mut().unwrap() }
    }

    pub fn default_face(&mut self) -> FaceRef {
        self.face_from_id_or_null(face_id::DEFAULT_FACE_ID)
    }

    pub fn face_from_id(&mut self, face: face_id) -> FaceRef {
        let f = unsafe { FACE_FROM_ID(self.as_mut(), face as libc::c_int) };
        FaceRef::new(f)
    }

    pub fn face_from_id_or_null(&mut self, face: face_id) -> FaceRef {
        let f = unsafe { FACE_FROM_ID_OR_NULL(self.as_mut(), face as libc::c_int) };
        FaceRef::new(f)
    }
}
