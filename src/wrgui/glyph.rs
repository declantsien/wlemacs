use crate::color::pixel_to_color;
use crate::util::HandyDandyRectBuilder;
use crate::types::{
    face_underline_type, glyph_type, Emacs_Rectangle as NativeRectangle,
};
use std::cmp::max;
use webrender::api::units::*;
use webrender::api::{
    FontInstanceKey, FontInstanceOptions, FontInstancePlatformOptions, FontKey, *,
};
use webrender::{self};
// TODO: maybe configurable from lisp world
const WAVY_LINE_THICKNESS: i32 = 1;

pub trait WrGlyph {
    fn fg_color_f(&self) -> ColorF;
    fn underline_color_f(&self) -> ColorF;
    fn overline_color_f(&self) -> ColorF;
    fn strike_through_color_f(&self) -> ColorF;
    fn x(&self) -> i32;
    fn underline_area(&self) -> DeviceRect;
    fn underwave_area(&self) -> DeviceRect;
    fn font_info(&self) -> FontInfoRef;
    fn font_key(&self) -> FontKey;
    fn font_instance_key(&self) -> FontInstanceKey;
    fn glyph_dimensions(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<GlyphDimensions>>;
    fn get_glyph_advance_widths(&self, glyph_indices: Vec<GlyphIndex>)
        -> Vec<Option<LayoutLength>>;
    fn composite_p(&self) -> bool;
    fn automatic_composite_p(&self) -> bool;
    fn visible_height(&self) -> i32;
    fn glyph_indices(&self) -> Vec<u32>;
    fn glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance>;
    fn char_glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance>;
    fn composite_glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance>;
    fn automatic_composite_glyph_instances(
        &self,
        scale_factor: LayoutToDeviceScale,
    ) -> Vec<GlyphInstance>;
}

impl WrGlyph for GlyphStringRef {
    fn fg_color_f(&self) -> ColorF {
        pixel_to_color(self.fg_color())
    }

    fn underline_color_f(&self) -> ColorF {
        pixel_to_color(self.underline_color())
    }

    fn overline_color_f(&self) -> ColorF {
        pixel_to_color(self.overline_color())
    }

    fn strike_through_color_f(&self) -> ColorF {
        pixel_to_color(self.strike_through_color())
    }

    // If first glyph of S has a left box line, start drawing the text
    // of S to the right of that box line.
    fn x(&self) -> i32 {
        if !self.face.is_null()
            && unsafe { (*self.face).box_() } != FACE_NO_BOX
            && unsafe { (*self.first_glyph).left_box_line_p() }
        {
            self.x + std::cmp::max(unsafe { (*self.face).box_vertical_line_width }, 0)
        } else {
            self.x
        }
    }

    fn underwave_area(&self) -> DeviceRect {
        let wave_height = 3;
        // let wave_length = 2; // Webrender internals
        (self.x, self.ybase - wave_height + 3).by(self.width as i32, wave_height)
    }

    fn underline_area(&self) -> DeviceRect {
        assert_ne!(
            self.face().underline_type(),
            face_underline_type::FACE_UNDERLINE_WAVE
        );
        let underline_size = || {
            if let Some(prev) = self.prev().filter(|s| {
                s.face().underline_type() == face_underline_type::FACE_UNDERLINE_SINGLE
                    && (s.face().underline_at_descent_line_p()
                        == self.face().underline_at_descent_line_p())
                    && (s.face().underline_pixels_above_descent_line
                        == self.face().underline_pixels_above_descent_line)
            }) {
                return (prev.underline_thickness, prev.underline_position);
            } else {
                let font = self.font_for_underline_metrics();
                /* Get the underline thickness.  Default is 1 pixel.  */
                let thickness = font
                    .filter(|font| font.underline_thickness > 0)
                    .map(|font| font.underline_thickness)
                    .unwrap_or(1);
                if unsafe { globals.x_underline_at_descent_line }
                    || self.face().underline_at_descent_line_p()
                {
                    let position = (self.height - thickness)
                        - (self.ybase - self.y)
                        - self.face().underline_pixels_above_descent_line;
                    return (thickness, position);
                } else {
                    //  Get the underline position.  This is the recommended
                    // vertical offset in pixels from the baseline to the top of
                    // the underline.  This is a signed value according to the
                    // specs, and its default is

                    // ROUND ((maximum descent) / 2), with
                    // ROUND(x) = floor (x + 0.5)
                    let position = match font {
                        Some(font)
                            if unsafe { globals.x_use_underline_position_properties }
                                && font.underline_position >= 0 =>
                        {
                            font.underline_position
                        }
                        Some(font) => (font.descent + 1) / 2,
                        _ => unsafe { globals.underline_minimum_offset.try_into().unwrap() },
                    };
                    return (thickness, position);
                };
            }
        };

        let (mut thickness, mut position) = underline_size();
        /* Ignore minimum_offset if the amount of pixels was
        explicitly specified.  */
        if self.face().underline_pixels_above_descent_line != 0 {
            position = max(position, unsafe {
                globals.underline_minimum_offset.try_into().unwrap()
            });
        }
        /* Check the sanity of thickness and position.  We should
        avoid drawing underline out of the current line area.  */
        if self.y + self.height <= self.ybase + position {
            position = (self.height - 1) - (self.ybase - self.y);
        }
        if self.y + self.height < self.ybase + position + thickness {
            thickness = (self.y + self.height) - (self.ybase + position);
        }
        self.clone().underline_thickness = thickness;
        self.clone().underline_position = position;
        let y = self.ybase + position;
        (self.x, y).by(self.width as i32, thickness)
    }

    fn font_info(&self) -> FontInfoRef {
        FontInfoRef::new(self.font as *mut font_info)
    }

    fn composite_p(&self) -> bool {
        self.glyph_type() == glyph_type::COMPOSITE_GLYPH
    }

    fn automatic_composite_p(&self) -> bool {
        self.composite_p() && unsafe { (*self.first_glyph).u.cmp.automatic() }
    }

    fn visible_height(&self) -> i32 {
        if unsafe { (*self.row).mode_line_p() } {
            unsafe { (*self.row).height }
        } else {
            unsafe { (*self.row).visible_height }
        }
    }

    fn font_key(&self) -> FontKey {
        let font_tpl = crate::font::wr_font_tpl(self.font().as_mut());
        // println!("font tpl {:?}", font_tpl);
        self.frame().wr().wr_add_font(font_tpl)
    }

    fn font_instance_key(&self) -> FontInstanceKey {
        let font_key = self.font_key();
        let f = self.frame();
        f.wr().wr_add_font_instance(
            font_key,
            DeviceLength::new(self.font().pixel_size as f32),
            Some(FontInstanceOptions::default()),
            Some(FontInstancePlatformOptions::default()),
            Vec::new(),
        )
    }

    // file:///home/declan/src/webrender/target/doc/webrender/render_api/struct.RenderApi.html#method.get_glyph_dimensions
    // Note: Internally, the internal texture cache doesn’t store ‘empty’ textures (height or width = 0) This means that glyph dimensions e.g. for spaces (’ ’) will mostly be None.
    fn glyph_dimensions(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<GlyphDimensions>> {
        let key = self.font_instance_key();
        let f = self.frame();
        f.wr().glyph_dimensions(key, glyph_indices)
    }

    fn get_glyph_advance_widths(
        &self,
        glyph_indices: Vec<GlyphIndex>,
    ) -> Vec<Option<LayoutLength>> {
        self.glyph_dimensions(glyph_indices)
            .iter()
            .map(|i| i.map(|d| LayoutLength::new(d.advance)))
            .collect()
    }

    fn glyph_indices(&self) -> Vec<u32> {
        let from = 0 as usize;
        let to = self.nchars as usize;

        self.get_chars()[from..to]
            .iter()
            .map(|c| *c as u32)
            .collect()
    }

    fn glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance> {
        let glyph_type = self.glyph_type();

        match glyph_type {
            glyph_type::CHAR_GLYPH => self.char_glyph_instances(scale_factor),
            glyph_type::COMPOSITE_GLYPH => {
                if self.automatic_composite_p() {
                    self.automatic_composite_glyph_instances(scale_factor)
                } else {
                    self.composite_glyph_instances(scale_factor)
                }
            }
            _ => vec![],
        }
    }

    fn char_glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance> {
        let font_info = self.font_info();

        let x_start = self.x();
        let y_start = self.y + (font_info.font.ascent + (self.height - font_info.font.height) / 2);

        let glyph_indices = self.glyph_indices();

        let glyph_advances = self.get_glyph_advance_widths(glyph_indices.clone());
        let mut glyph_instances: Vec<GlyphInstance> = vec![];
        // println!("indices: {:?}, dimensions: {:?}", glyph_indices.clone(), glyph_dimensions);

        let face = self.face;

        for (i, index) in glyph_indices.into_iter().enumerate() {
            let previous_char_width = if i == 0 {
                0
            } else {
                // wr get_glyph_dimensions return none for ‘empty’ textures (height or width = 0)
                // spaces (’ ’) will mostly be None
                glyph_advances[i - 1]
                    .map(|len| (len * scale_factor).get() as i32)
                    .unwrap_or(0)
            };

            let previous_char_start = if i == 0 {
                x_start
            } else {
                (glyph_instances[i - 1].point * scale_factor).to_i32().x
            };

            let mut start = previous_char_start + previous_char_width;

            if self.face().overstrike() {
                start += 1;
            }

            let glyph_instance = GlyphInstance {
                index,
                point: DeviceIntPoint::new(start, y_start).to_f32() / scale_factor,
            };

            glyph_instances.push(glyph_instance);
        }
        glyph_instances
    }

    fn composite_glyph_instances(&self, scale_factor: LayoutToDeviceScale) -> Vec<GlyphInstance> {
        let font_info = self.font_info();

        let x = self.x();

        let y_start = self.y + (font_info.font.ascent + (self.height - font_info.font.height) / 2);

        let offsets = self.composite_offsets();

        let glyph_instances: Vec<GlyphInstance> = self
            .composite_chars()
            .into_iter()
            .enumerate()
            .filter_map(|(n, glyph)| {
                // TAB in a composition means display glyphs with padding
                // space on the left or right.
                if self.composite_glyph(n as usize).is_some()
                    && self.composite_glyph(n as usize).unwrap() == <u8 as Into<i64>>::into(b'\t')
                {
                    return None;
                }

                let mut xx = x + offsets[n as usize * 2] as i32;
                let yy = y_start - offsets[n as usize * 2 + 1] as i32;
                if self.face().overstrike() {
                    xx += 1;
                }

                let glyph_instance = GlyphInstance {
                    index: *glyph,
                    point: DeviceIntPoint::new(xx, yy).to_f32() / scale_factor,
                };

                Some(glyph_instance)
            })
            .collect();
        glyph_instances
    }

    fn automatic_composite_glyph_instances(
        &self,
        scale_factor: LayoutToDeviceScale,
    ) -> Vec<GlyphInstance> {
        let mut instances: Vec<GlyphInstance> = vec![];
        let lgstring = self.get_lgstring();
        let mut composite_lglyph = |lglyph: LispObject, x: i32, y: i32| {
            let code = lglyph.lglyph_code().as_fixnum_or_error();
            let index: webrender::api::GlyphIndex = code.try_into().unwrap();
            let glyph_instance = GlyphInstance {
                index,
                point: DeviceIntPoint::new(x, y).to_f32() / scale_factor,
            };
            log::warn!("automatic composite glyph instance {glyph_instance:?}");
            instances.push(glyph_instance);
        };

        let mut x = self.x();

        let y = self.ybase;

        let cmp_from = self.cmp_from;
        let cmp_to = self.cmp_to;

        for n in cmp_from..cmp_to {
            let lglyph = lgstring.lgstring_glyph(n as u32);
            if lglyph.lglyph_adjustment().is_nil() {
                composite_lglyph(lglyph, x, y);
                if self.face().overstrike() {
                    composite_lglyph(lglyph, x + 1, y);
                }
                let glyph_pixel_width = lglyph.lglyph_width().xfixnum() as i32;
                x += glyph_pixel_width;
            } else {
                let xoff = lglyph.lglyph_xoff() as i32;
                let yoff = lglyph.lglyph_yoff() as i32;
                let wadjust = lglyph.lglyph_wadjust() as i32;
                if self.face().overstrike() {
                    composite_lglyph(lglyph, x + xoff + 1, y + yoff);
                }

                x += wadjust;
            }
        }
        instances
    }
}

pub trait GlyphStringExtWr {
    fn clip_rect(&mut self) -> NativeRectangle;
    fn draw_line(
        &self,
        style: LineStyle,
        color: ColorF,
        rect: DeviceRect,
        orientation: LineOrientation,
    );
    fn draw_underline(&self);
    fn draw_overline(&mut self);
    fn draw_strike_through(&mut self);
}

impl GlyphStringExtWr for GlyphStringRef {
    fn clip_rect(&mut self) -> NativeRectangle {
        use emacs_sys::bindings::get_glyph_string_clip_rect;
        let mut clip_rect = NativeRectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        unsafe { get_glyph_string_clip_rect(self.as_mut(), &mut clip_rect) };
        clip_rect
    }

    // underline/wave overline strike-through etc
    // webrender support 4 line styles
    fn draw_line(
        &self,
        style: LineStyle,
        color: ColorF,
        area: DeviceRect,
        orientation: LineOrientation,
    ) {
        let x = self.x;
        let y = self.y;

        let visible_height = self.visible_height();
        self.frame().wr().display(|builder, space_and_clip, scale| {
            let common = CommonItemProperties::new(
                (x, y).by(self.width as i32, visible_height) / scale,
                space_and_clip,
            );

            builder.push_line(
                &common,
                &(area / scale),
                WAVY_LINE_THICKNESS as f32,
                orientation,
                &color,
                style,
            );
        });
    }

    fn draw_underline(&self) {
        let color = self.underline_color_f();

        if let Some(style) = self.face().underline_style() {
            let area = match style {
                LineStyle::Solid | LineStyle::Dotted | LineStyle::Dashed => self.underline_area(),
                LineStyle::Wavy => self.underwave_area(),
            };
            self.draw_line(style, color, area, LineOrientation::Horizontal);
        }
    }

    fn draw_overline(&mut self) {
        assert_eq!(self.face().overline_p(), true);
        let dy = 0;
        let h = 1;
        let area = (self.x, self.y + dy).by(self.width, h);
        self.draw_line(
            LineStyle::Solid,
            self.overline_color_f(),
            area,
            LineOrientation::Horizontal,
        );
    }

    fn draw_strike_through(&mut self) {
        assert_eq!(self.face().strike_through_p(), true);
        /* Y-coordinate and height of the glyph string's first
        glyph.  We cannot use s->y and s->height because those
        could be larger if there are taller display elements
        (e.g., characters displayed with a larger font) in the
        same glyph row.  */
        let glyph_y = self.ybase - self.first_glyph().ascent as i32;
        let glyph_height = self.first_glyph().ascent + self.first_glyph().descent;
        /* Strike-through width and offset from the glyph string's
        top edge.  */
        let h = 1;
        let dy = (glyph_height - h) / 2;
        let area = (self.x, glyph_y + dy as i32).by(self.width, h as i32);
        self.draw_line(
            LineStyle::Solid,
            self.strike_through_color_f(),
            area,
            LineOrientation::Horizontal,
        );
    }
}
