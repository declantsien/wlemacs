use std::cmp::max;
use std::slice;

use webrender_api::GlyphInstance;

use crate::types::{
    composition, composition_gstring_from_id, face, face_box_type, font, frame, glyph, glyph_row,
    glyph_string, glyph_type, window, EmacsIntPoint, EmacsLength, Lisp_Object,
};

impl<'a> glyph_string {
    pub fn from_ptr(f: *mut glyph_string) -> Option<&'a glyph_string> {
        unsafe { f.as_ref() }
    }

    pub fn from_ptr_mut(f: *mut glyph_string) -> Option<&'a mut glyph_string> {
        unsafe { f.as_mut() }
    }

    pub fn next(&self) -> Option<&mut glyph_string> {
        unsafe { self.next.as_mut() }
    }

    pub fn is_for_overlaps(&self) -> bool {
        self.for_overlaps() != 0
    }

    pub fn is_right_overhang(&self) -> bool {
        self.right_overhang != 0
    }

    pub fn row(&self) -> Option<&glyph_row> {
        unsafe { self.row.as_ref() }
    }

    pub fn row_mut(&self) -> Option<&mut glyph_row> {
        unsafe { self.row.as_mut() }
    }

    pub fn window(&self) -> Option<&window> {
        unsafe { self.w.as_ref() }
    }

    pub fn x(&self) -> i32 {
        /* If first glyph of S has a left box line, start drawing the text
        of S to the right of that box line.  */
        if let (Some(face), Some(first_glyph)) = (self.face(), self.first_glyph()) {
            if face.box_() == face_box_type::FACE_NO_BOX && first_glyph.left_box_line_p() {
                return self.x + max(face.box_vertical_line_width, 0);
            }
        }
        return self.x;
    }

    pub fn y(&self) -> i32 {
        assert!(self.font().is_some());
        let ft = self.font().unwrap();
        let mut boff = ft.baseline_offset;

        if ft.vertical_centering {
            boff = ft.vcenter_baseline_offset(self.f()) - boff;
        }

        return self.ybase - boff;
    }

    pub fn face(&self) -> Option<&face> {
        unsafe { self.face.as_ref() }
    }

    pub fn f(&self) -> &frame {
        unsafe { self.f.as_ref().unwrap() }
    }

    pub fn f_mut(&self) -> &mut frame {
        unsafe { self.f.as_mut().unwrap() }
    }

    pub fn font(&self) -> Option<&font> {
        if self.font_not_found_p() {
            return None;
        }
        unsafe { self.font.as_ref() }
    }

    pub fn font_mut(&self) -> Option<&mut font> {
        if self.font_not_found_p() {
            return None;
        }
        unsafe { self.font.as_mut() }
    }

    pub fn first_glyph(&self) -> Option<&glyph> {
        unsafe { self.first_glyph.as_ref() }
    }

    pub fn glyph_type(&self) -> glyph_type {
        glyph_type::from(self.first_glyph().unwrap().type_())
    }

    pub fn cmp(&self) -> Option<&composition> {
        unsafe { self.cmp.as_ref() }
    }

    pub fn glyph_indices(&self) -> &[u32] {
        let len = self.nchars as usize;

        unsafe { slice::from_raw_parts(self.char2b, len) }
    }

    pub fn cmp_is_automatic(&self) -> bool {
        unsafe { self.first_glyph().unwrap().u.cmp.automatic() }
    }

    pub fn lgstring(&self) -> Lisp_Object {
        assert!(self.cmp_is_automatic());
        unsafe { composition_gstring_from_id(self.cmp_id) }
    }

    pub fn glyph_instances(&self, from: usize, to: usize, x: i32, y: i32) -> Vec<GlyphInstance> {
        let scale = self.f().renderer().unwrap().emacs_to_layout_scale();
        let origin = EmacsIntPoint::new(x, y).to_f32() * scale;

        let indices: Vec<u32> = self.glyph_indices()[from..to].iter().map(|c| *c).collect();
        let dimensions = self
            .font()
            .unwrap()
            .glyph_dimensions(self.f_mut(), indices.clone());
        let (instances, _) = indices.into_iter().zip(dimensions.into_iter()).fold(
            (Vec::new(), origin),
            |(mut instances, mut point), (index, dimension)| {
                instances.push(GlyphInstance { index, point });

                if self.padding_p() {
                    point.x += (EmacsLength::new(1.0) * scale).get();
                } else {
                    // wr get_glyph_dimensions return none for ‘empty’ textures (height or width = 0)
                    // spaces (’ ’) will mostly be None
                    if let Some(d) = dimension {
                        point.x += d.advance.floor();
                    }
                }

                (instances, point)
            },
        );
        return instances;
    }
}
