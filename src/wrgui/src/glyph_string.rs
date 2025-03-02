use std::cmp::{max, min};
use std::slice;

use webrender_api::GlyphInstance;

use crate::types::{
    composition, composition_gstring_from_id, face, face_box_type, font, frame, glyph,
    glyph_string, glyph_type, lglyph_indices, EmacsIntPoint, EmacsLength, EmacsToLayoutScale,
    Lisp_Object, AREF, NILP,
};

impl glyph_string {
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

    pub fn glyph_instances_(&self) -> Option<Vec<GlyphInstance>> {
        use glyph_type::*;
        let scale = self.f().renderer().unwrap().emacs_to_layout_scale();
        let is_overstrike = self.face().map(|f| f.overstrike()).unwrap_or(false);

        match self.glyph_type() {
            CHAR_GLYPH => {
                let x = self.x();
                let y = self.y();

                let origin = EmacsIntPoint::new(x, y).to_f32() * scale;

                let from = 0 as usize;
                let to = self.nchars as usize;
                let indices: Vec<u32> = self.glyph_indices()[from..to].iter().map(|c| *c).collect();
                let dimensions = self.font().unwrap().glyph_dimensions(indices.clone());
                let (instances, _) = indices.into_iter().zip(dimensions.into_iter()).fold(
                    (Vec::new(), origin),
                    |(mut instances, mut point), (index, dimension)| {
                        instances.push(GlyphInstance { index, point });
                        if is_overstrike {
                            let mut instance = GlyphInstance { index, point };
                            overstrike_glyph_instance(&mut instance, scale);
                            instances.push(instance);
                        }

                        if let Some(d) = dimension {
                            point.x += d.advance;
                        }

                        (instances, point)
                    },
                );
                return Some(instances);
            }
            COMPOSITE_GLYPH => {
                if !unsafe { self.first_glyph().unwrap().u.cmp.automatic() } {
                    let from = self.cmp_from as usize;
                    let to = min(self.nchars, self.cmp_to) as usize;
                    let cmp = self.cmp().unwrap();
                    let instances = self.glyph_indices()[from..to]
                        .into_iter()
                        .enumerate()
                        .filter_map(|(n, glyph)| {
                            if cmp.is_tab(n) {
                                return None;
                            }
                            return Some((n, glyph));
                        })
                        .fold(Vec::new(), |mut instances, (n, index)| {
                            let xx = self.x() + *cmp.offsets((n * 2) as isize) as i32;
                            let yy = self.y() - *cmp.offsets((n * 2 + 1) as isize) as i32;
                            let point = EmacsIntPoint::new(xx, yy).to_f32() * scale;
                            instances.push(GlyphInstance {
                                index: *index,
                                point,
                            });

                            if is_overstrike {
                                let mut instance = GlyphInstance {
                                    index: *index,
                                    point,
                                };
                                overstrike_glyph_instance(&mut instance, scale);
                                instances.push(instance);
                            }

                            instances
                        });

                    return Some(instances);
                } else {
                    use lglyph_indices::*;
                    let lgs = self.lgstring();
                    let mut width: i32 = 0;
                    for i in self.cmp_from..self.cmp_to {
                        let lglyph = unsafe { AREF(lgs, (i + 2) as isize) };
                        let adjustment = unsafe { AREF(lglyph, LGLYPH_IX_ADJUSTMENT as isize) };
                        if NILP(adjustment) {
                            // width += LGLYPH_WIDTH (lglyph);
                        }
                    }
                    todo!()
                }
            }
            _ => None,
        }
    }
}

fn overstrike_glyph_instance(instance: &mut GlyphInstance, scale: EmacsToLayoutScale) {
    instance.point.x += (EmacsLength::new(1.0) * scale).get();
}
