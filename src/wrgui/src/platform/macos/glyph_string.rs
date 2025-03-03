use webrender_api::ColorF;

use crate::types::{draw_glyphs_face, glyph_string, prepare_face_for_display};

impl glyph_string {
    /* Transfer glyph string parameters from S's face to S itself.
    Set S->stipple_p as appropriate, taking the draw type into
    account.  */
    /// ns_set_glyph_string_gc in rust
    pub fn set_gc(&mut self) {
        unsafe { prepare_face_for_display(self.f, self.face) };

        use draw_glyphs_face::*;
        match self.hl {
            DRAW_NORMAL_TEXT | DRAW_INVERSE_VIDEO | DRAW_MOUSE_FACE | DRAW_IMAGE_RAISED
            | DRAW_IMAGE_SUNKEN => self.set_stippled_p(self.face().unwrap().stipple != 0),
            DRAW_CURSOR => {
                self.set_stippled_p(false);
            }
            _ => unreachable!(),
        }
    }

    pub fn fg_color(&self) -> ColorF {
        self.f().cursor_color()
    }

    pub fn bg_color(&self) -> ColorF {
        // FIXME something here aren't right
        // use in wr_font_draw with_background

        // if self.hl == draw_glyphs_face::DRAW_CURSOR {
        //     if self
        //         .face()
        //         .and_then(|face| face.bg_color())
        //         .map(|color| color == self.f().cursor_color())
        //         .unwrap_or(false)
        //     {
        //         self.face().unwrap().fg_color().unwrap()
        //     } else {
        //         self.f().cursor_color()
        //     }
        // } else {
        //     self.face().unwrap().bg_color().unwrap()
        // }
        ColorF::WHITE
    }
}
