use super::util::HandyDandyRectBuilder;
use crate::color::pixel_to_color;
use crate::face::WrFace;
use crate::glyph::{GlyphStringExtWr, WrGlyph};
use crate::image::{ImageExt, ImageRef};
use crate::output::{CanvasRef, FringeBitmap, WrCanvas};
use emacs_sys::bindings::glyph_type;
use emacs_sys::display_traits::{DrawGlyphsFace, GlyphStringRef};
use emacs_sys::frame::FrameRef;
use std::cmp::min;
use webrender::api::units::*;
use webrender::api::*;

pub trait FrameExtWrCommon {
    fn is_wr_data_initialized(&self) -> bool;
    fn wr(&self) -> CanvasRef;
    fn fg_color_f(&self) -> ColorF;
    fn cursor_color_f(&self) -> ColorF;
    fn cursor_foreground_color_f(&self) -> ColorF;
    fn logical_size(&self) -> LayoutSize;
    fn physical_size(&self) -> DeviceIntSize;
    fn draw_glyph_string(&mut self, s: GlyphStringRef);

    fn draw_glyph_string_background(&mut self, s: GlyphStringRef, force_p: bool);

    fn draw_char_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_stretch_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_image_glyph(&mut self, s: GlyphStringRef);

    fn draw_image(
        &mut self,
        image_key: ImageKey,
        bounds: DeviceRect,
        clip_bounds: Option<DeviceRect>,
    );

    fn draw_composite_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_fringe_bitmap(
        &mut self,
        pos: DevicePoint,
        image: Option<FringeBitmap>,
        bitmap_color: ColorF,
        background_color: ColorF,
        image_clip_rect: DeviceRect,
        clear_rect: DeviceRect,
        row_rect: DeviceRect,
    );
}

impl FrameExtWrCommon for FrameRef {
    fn is_wr_data_initialized(&self) -> bool {
        !self.output().wr_data.is_null()
    }

    fn wr(&self) -> CanvasRef {
        if !self.is_wr_data_initialized() {
            log::debug!("gl renderer data empty");
            let data = Box::new(WrCanvas::build(self.clone()));
            self.output().wr_data = Box::into_raw(data) as *mut emacs_sys::bindings::wr_canvas;
        }

        CanvasRef::new(self.output().wr_data as *mut WrCanvas)
    }

    fn fg_color_f(&self) -> ColorF {
        pixel_to_color(self.fg_color())
    }

    fn cursor_color_f(&self) -> ColorF {
        pixel_to_color(self.cursor_color())
    }

    fn cursor_foreground_color_f(&self) -> ColorF {
        pixel_to_color(self.cursor_foreground_color())
    }

    fn logical_size(&self) -> LayoutSize {
        LayoutSize::new(self.pixel_width as f32, self.pixel_height as f32)
    }

    fn physical_size(&self) -> DeviceIntSize {
        let size = self.logical_size() * euclid::Scale::new(self.scale_factor() as f32);
        size.to_i32()
    }

    fn draw_glyph_string(&mut self, mut s: GlyphStringRef) {
        // wip
        s.set_gc();

        match s.glyph_type() {
            glyph_type::CHAR_GLYPH => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, false);
                }
                self.draw_char_glyph_string_foreground(s)
            }
            glyph_type::STRETCH_GLYPH => self.draw_stretch_glyph_string_foreground(s),
            glyph_type::IMAGE_GLYPH => self.draw_image_glyph(s),
            glyph_type::COMPOSITE_GLYPH => {
                if s.for_overlaps() != 0 || s.cmp_from > 0 && s.automatic_composite_p() {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }
                self.draw_composite_glyph_string_foreground(s)
            }
            glyph_type::XWIDGET_GLYPH => {
                log::warn!("TODO unimplemented! GlyphType::XWIDGET_GLYPH\n")
            }
            glyph_type::GLYPHLESS_GLYPH => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }

                self.draw_glyphless_glyph_string_foreground(s);
            }
            _ => {}
        }

        if !s.is_for_overlaps() {
            // Draw underline
            s.draw_underline();

            // Draw overline
            if s.face().overline_p() {
                s.draw_overline();
            }

            /* Draw strike-through.  */
            if s.face().strike_through_p() {
                s.draw_strike_through();
            }
        }
    }

    // Draw the background of glyph_string S.  If S->background_filled_p
    // is non-zero don't draw it.  FORCE_P non-zero means draw the
    // background even if it wouldn't be drawn normally.  This is used
    // when a string preceding S draws into the background of S, or S
    // contains the first component of a composition.
    fn draw_glyph_string_background(&mut self, mut s: GlyphStringRef, force_p: bool) {
        // Nothing to do if background has already been drawn or if it
        // shouldn't be drawn in the first place.
        if s.background_filled_p() {
            return;
        }
        let box_line_width = std::cmp::max(s.face().box_horizontal_line_width, 0);

        if s.stippled_p() {
            // Fill background with a stipple pattern.
            // fill_background (s, s.x, s.y + box_line_width,
            //     s.background_width,
            //     s.height - 2 * box_line_width);
            s.set_background_filled_p(true);
        } else if s.font().height < s.height - 2 * box_line_width
	    /* When xdisp.c ignores FONT_HEIGHT, we cannot trust
	    font dimensions, since the actual glyphs might be
	    much smaller.  So in that case we always clear the
	    rectangle with background color.  */
	    || s.font().too_high_p()
            || s.font_not_found_p()
            || s.extends_to_end_of_line_p() || force_p
        {
            let background_color = s.bg_color();
            self.wr().push_rect(
                background_color,
                (s.x, s.y + box_line_width).by(s.background_width, s.height - 2 * box_line_width),
                None,
            );

            s.set_background_filled_p(true);
        }
    }

    fn draw_char_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        let x = s.x;
        let y = s.y;

        let visible_height = s.visible_height();

        // draw background
        let background_color = s.bg_color();
        self.wr().push_rect(
            background_color,
            (x, y).by(s.background_width, visible_height),
            None,
        );

        self.wr().display(|builder, space_and_clip, scale| {
            let foreground_color = s.fg_color_f();

            let glyph_instances = s.glyph_instances(scale);
            // draw foreground
            if !glyph_instances.is_empty() {
                let font_instance_key = s.font_instance_key();
                let visible_rect = (x, y).by(s.width as i32, visible_height) / scale;

                builder.push_text(
                    &CommonItemProperties::new(visible_rect, space_and_clip),
                    visible_rect,
                    &glyph_instances,
                    font_instance_key,
                    foreground_color,
                    None,
                );
            }
        });
    }

    fn draw_stretch_glyph_string_foreground(&mut self, mut s: GlyphStringRef) {
        if s.background_filled_p() {
            return;
        }

        let visible_height = s.visible_height();
        let background_width = if s.hl() == DrawGlyphsFace::Cursor {
            let frame: FrameRef = s.f.into();

            min(frame.column_width, s.background_width)
        } else {
            s.background_width
        };

        let background_color = s.bg_color();
        self.wr().push_rect(
            background_color,
            (s.x, s.y).by(background_width, visible_height),
            None,
        );

        s.set_background_filled_p(true);
    }

    fn draw_image_glyph(&mut self, mut s: GlyphStringRef) {
        // clear area
        let x = s.x;
        let y = s.y;
        let visible_height = s.visible_height();
        let background_color = s.bg_color();
        self.wr().push_rect(
            background_color,
            (x, y).by(s.background_width, visible_height),
            None,
        );
        let clip_rect = s.clip_rect();

        let background_color = s.face().bg_color_f();
        let clip_bounds =
            (clip_rect.x, clip_rect.y).by(clip_rect.width as i32, clip_rect.height as i32);
        let bounds = (s.x, s.y).by(s.slice.width() as i32, s.slice.height() as i32);

        // render background
        let background_rect = bounds.intersection(&clip_bounds);
        if let Some(background_rect) = background_rect {
            self.wr().draw_rectangle(background_color, background_rect);
        }

        let image: ImageRef = s.img.into();
        let frame: FrameRef = s.f.into();
        if let Some((image_key, descriptor)) = image.meta(frame) {
            //s.img viewbox is the layout size we draw to
            //we don't crop image to fix into viewbox
            //So the width of image uploaded to WebRender could be bigger than viewbox
            //We do scaling here to avoid stretching
            let dwidth = s.slice.height() as f32 / descriptor.size.height as f32
                * descriptor.size.width as f32;
            let bounds = (s.x, s.y).by(dwidth as i32, s.slice.height() as i32);
            self.draw_image(image_key, bounds, Some(clip_bounds));
        }
    }

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        let _x = s.x();
        println!("draw glyphless glyph string forground");
        //TODO
    }

    fn draw_image(
        &mut self,
        image_key: ImageKey,
        bounds: DeviceRect,
        clip_bounds: Option<DeviceRect>,
    ) {
        self.wr().display(|builder, space_and_clip, scale_factor| {
            // render image
            builder.push_image(
                &CommonItemProperties::new(
                    clip_bounds.unwrap_or(bounds) / scale_factor,
                    space_and_clip,
                ),
                bounds / scale_factor,
                ImageRendering::Auto,
                AlphaType::Alpha,
                image_key,
                ColorF::WHITE,
            );
        });
    }

    fn draw_composite_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        // S is a glyph string for a composition.  S->cmp_from is the index
        // of the first character drawn for glyphs of this composition.
        // S->cmp_from == 0 means we are drawing the very first character of
        // this composition

        // Draw a rectangle for the composition if the font for the very
        // first character of the composition could not be loaded.
        if s.font_not_found_p() {
            if s.cmp_from == 0 {
                self.wr()
                    .push_rect(self.cursor_color(), (s.x, s.y).by(s.width, s.height), None);
            }
        } else {
            let visible_height = s.visible_height();

            let x = s.x;
            let y = s.y;
            let background_color = s.bg_color();
            self.wr().push_rect(
                background_color,
                (x, y).by(s.background_width, visible_height),
                None,
            );
            self.wr().display(|builder, space_and_clip, scale| {
                let s = s.clone();

                let foreground_color = s.fg_color_f();

                let visible_rect = (x, y).by(s.width, visible_height) / scale;

                let glyph_instances = s.glyph_instances(scale);
                // draw foreground
                if !glyph_instances.is_empty() {
                    let font_instance_key = s.font_instance_key();
                    builder.push_text(
                        &CommonItemProperties::new(visible_rect, space_and_clip),
                        visible_rect,
                        &glyph_instances,
                        font_instance_key,
                        foreground_color,
                        None,
                    );
                }
            });
        }
    }

    fn draw_fringe_bitmap(
        &mut self,
        pos: DevicePoint,
        image: Option<FringeBitmap>,
        bitmap_color: ColorF,
        background_color: ColorF,
        image_clip_rect: DeviceRect,
        clear_rect: DeviceRect,
        row_rect: DeviceRect,
    ) {
        // Fixed clear_rect
        let clear_rect = clear_rect
            .union(&image_clip_rect)
            .intersection(&row_rect)
            .unwrap_or_else(|| DeviceRect::zero());

        // Fixed image_clip_rect
        let image_clip_rect = image_clip_rect
            .intersection(&row_rect)
            .unwrap_or_else(|| DeviceRect::zero());

        // clear area
        self.wr().draw_rectangle(background_color, clear_rect);

        self.wr().display(|builder, space_and_clip, scale| {
            if let Some(image) = &image {
                let image_display_rect = DeviceRect::new(
                    pos,
                    DevicePoint::new(image.width as f32, image.height as f32),
                ) / scale;
                // render image
                builder.push_image(
                    &CommonItemProperties::new(image_clip_rect / scale, space_and_clip),
                    image_display_rect,
                    ImageRendering::Auto,
                    AlphaType::Alpha,
                    image.image_key,
                    bitmap_color,
                );
            }
        });
    }
}
