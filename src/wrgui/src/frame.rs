use crate::canvas::WrCanvas;
use crate::platform::pixel_to_color;
use crate::types::{
    block_input, face, face_cache, face_id, font, frame, unblock_input, EmacsIntRect, Mouse_HLInfo,
    FACE_FROM_ID, FACE_FROM_ID_OR_NULL,
};
use crate::util::HandyDandyRectBuilder;

impl<'a> frame {
    pub fn from_ptr(f: *mut frame) -> Option<&'a mut frame> {
        unsafe { f.as_mut() }
    }

    pub fn renderer_mut(&mut self) -> Option<&mut WrCanvas> {
        unsafe {
            self.output_data()
                .and_then(|d| (d.wr_data as *mut WrCanvas).as_mut())
        }
    }

    pub fn renderer(&self) -> Option<&WrCanvas> {
        unsafe {
            self.output_data()
                .and_then(|d| (d.wr_data as *const WrCanvas).as_ref())
        }
    }

    pub fn default_face(&mut self) -> Option<&mut face> {
        self.face_from_id_or_null(face_id::DEFAULT_FACE_ID)
    }

    pub fn mouse_hl_info(&self) -> &Mouse_HLInfo {
        unsafe {
            &self
                .output_data()
                .unwrap()
                .display_info
                .as_ref()
                .unwrap()
                .mouse_highlight
        }
    }

    pub fn face_cache(&self) -> &face_cache {
        unsafe { self.face_cache.as_ref().unwrap() }
    }

    pub fn face_from_id_or_none(&self, id: usize) -> Option<&mut face> {
        let cache = self.face_cache;

        let faces_map: &[*mut face] =
            unsafe { std::slice::from_raw_parts_mut((*cache).faces_by_id, (*cache).used as usize) };

        faces_map
            .get(id)
            .copied()
            .and_then(|f| unsafe { f.as_mut() })
    }

    pub fn mouse_face(&self) -> &mut face {
        let id = self.mouse_hl_info().mouse_face_face_id as usize;
        self.face_from_id_or_none(id).unwrap_or(
            self.face_from_id_or_none(face_id::MOUSE_FACE_ID as usize)
                .unwrap(),
        )
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
        if self.default_face().is_none() || self.renderer_mut().is_none() {
            return;
        }

        unsafe { block_input() };
        let clip = (0, 0).by(self.pixel_width, self.pixel_height);
        let clear_color = pixel_to_color(self.default_face().unwrap().background);
        self.renderer_mut().unwrap().dp_push_rect(
            r.to_f32(),
            Some(clip),
            false,
            false,
            false,
            clear_color,
        );
        unsafe { unblock_input() };
    }

    pub fn baseline_offset(&self) -> i32 {
        self.output_data().map(|d| d.baseline_offset).unwrap_or(0)
    }

    pub fn font(&self) -> Option<&font> {
        self.output_data().and_then(|d| unsafe { d.font.as_ref() })
    }
}
