use crate::emacs::{Emacs_Color, Emacs_GC, Emacs_Pixmap, Emacs_Rectangle};
use crate::color::{color_to_xcolor, lookup_color_by_name_or_hex, pixel_to_color};
use crate::frame::FrameExtWrCommon;
use crate::image::{ImageExt, ImageRef, WrPixmap};
use crate::output::{DeviceLength, WrCanvas};
use crate::util::HandyDandyRectBuilder;
use emacs_sys::bindings::{
    block_input, face_id, globals, image, lookup_basic_face, unblock_input, FACE_FROM_ID_OR_NULL,
};
use emacs_sys::frame::{Frame, FrameRef};
use emacs_sys::lisp::LispObject;
use std::slice;
use webrender::api::{
    AlphaType, BorderRadius, BorderSide, BorderStyle, ColorF, CommonItemProperties,
    FontInstanceOptions, FontInstancePlatformOptions, FontTemplate, GlyphInstance, ImageRendering,
    LineOrientation, LineStyle, NativeFontHandle,
};

use std::{mem, ptr};
use webrender::api::units::{DeviceIntLength, DeviceIntPoint, DeviceIntSideOffsets, DeviceRect};

/// Whether a border should be antialiased.
#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum AntialiasBorder {
    No = 0,
    Yes,
}

#[repr(C)]
pub struct WrVecU32 {
    /// `data` must always be valid for passing to Vec::from_raw_parts.
    /// In particular, it must be non-null even if capacity is zero.
    data: *mut u32,
    length: usize,
    capacity: usize,
}

impl WrVecU32 {
    fn into_vec(mut self) -> Vec<u32> {
        // Clear self and then drop self.
        self.flush_into_vec()
    }

    // Clears self without consuming self.
    fn flush_into_vec(&mut self) -> Vec<u32> {
        // Create a Vec using Vec::from_raw_parts.
        //
        // Here are the safety requirements, verbatim from the documentation of `from_raw_parts`:
        //
        // > * `ptr` must have been allocated using the global allocator, such as via
        // >   the [`alloc::alloc`] function.
        // > * `T` needs to have the same alignment as what `ptr` was allocated with.
        // >   (`T` having a less strict alignment is not sufficient, the alignment really
        // >   needs to be equal to satisfy the [`dealloc`] requirement that memory must be
        // >   allocated and deallocated with the same layout.)
        // > * The size of `T` times the `capacity` (ie. the allocated size in bytes) needs
        // >   to be the same size as the pointer was allocated with. (Because similar to
        // >   alignment, [`dealloc`] must be called with the same layout `size`.)
        // > * `length` needs to be less than or equal to `capacity`.
        // > * The first `length` values must be properly initialized values of type `T`.
        // > * `capacity` needs to be the capacity that the pointer was allocated with.
        // > * The allocated size in bytes must be no larger than `isize::MAX`.
        // >   See the safety documentation of [`pointer::offset`].
        //
        // These comments don't say what to do for zero-capacity vecs which don't have
        // an allocation. In particular, the requirement "`ptr` must have been allocated"
        // is not met for such vecs.
        //
        // However, the safety requirements of `slice::from_raw_parts` are more explicit
        // about the empty case:
        //
        // > * `data` must be non-null and aligned even for zero-length slices. One
        // >   reason for this is that enum layout optimizations may rely on references
        // >   (including slices of any length) being aligned and non-null to distinguish
        // >   them from other data. You can obtain a pointer that is usable as `data`
        // >   for zero-length slices using [`NonNull::dangling()`].
        //
        // For the empty case we follow this requirement rather than the more stringent
        // requirement from the `Vec::from_raw_parts` docs.
        let vec = unsafe { Vec::from_raw_parts(self.data, self.length, self.capacity) };
        self.data = ptr::NonNull::dangling().as_ptr();
        self.length = 0;
        self.capacity = 0;
        vec
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.data, self.length) }
    }
}

#[repr(C)]
pub struct WrVecU8 {
    /// `data` must always be valid for passing to Vec::from_raw_parts.
    /// In particular, it must be non-null even if capacity is zero.
    data: *mut u8,
    length: usize,
    capacity: usize,
}

impl WrVecU8 {
    fn into_vec(mut self) -> Vec<u8> {
        // Clear self and then drop self.
        self.flush_into_vec()
    }

    // Clears self without consuming self.
    fn flush_into_vec(&mut self) -> Vec<u8> {
        // Create a Vec using Vec::from_raw_parts.
        //
        // Here are the safety requirements, verbatim from the documentation of `from_raw_parts`:
        //
        // > * `ptr` must have been allocated using the global allocator, such as via
        // >   the [`alloc::alloc`] function.
        // > * `T` needs to have the same alignment as what `ptr` was allocated with.
        // >   (`T` having a less strict alignment is not sufficient, the alignment really
        // >   needs to be equal to satisfy the [`dealloc`] requirement that memory must be
        // >   allocated and deallocated with the same layout.)
        // > * The size of `T` times the `capacity` (ie. the allocated size in bytes) needs
        // >   to be the same size as the pointer was allocated with. (Because similar to
        // >   alignment, [`dealloc`] must be called with the same layout `size`.)
        // > * `length` needs to be less than or equal to `capacity`.
        // > * The first `length` values must be properly initialized values of type `T`.
        // > * `capacity` needs to be the capacity that the pointer was allocated with.
        // > * The allocated size in bytes must be no larger than `isize::MAX`.
        // >   See the safety documentation of [`pointer::offset`].
        //
        // These comments don't say what to do for zero-capacity vecs which don't have
        // an allocation. In particular, the requirement "`ptr` must have been allocated"
        // is not met for such vecs.
        //
        // However, the safety requirements of `slice::from_raw_parts` are more explicit
        // about the empty case:
        //
        // > * `data` must be non-null and aligned even for zero-length slices. One
        // >   reason for this is that enum layout optimizations may rely on references
        // >   (including slices of any length) being aligned and non-null to distinguish
        // >   them from other data. You can obtain a pointer that is usable as `data`
        // >   for zero-length slices using [`NonNull::dangling()`].
        //
        // For the empty case we follow this requirement rather than the more stringent
        // requirement from the `Vec::from_raw_parts` docs.
        let vec = unsafe { Vec::from_raw_parts(self.data, self.length, self.capacity) };
        self.data = ptr::NonNull::dangling().as_ptr();
        self.length = 0;
        self.capacity = 0;
        vec
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data, self.length) }
    }

    fn from_vec(mut v: Vec<u8>) -> WrVecU8 {
        let w = WrVecU8 {
            data: v.as_mut_ptr(),
            length: v.len(),
            capacity: v.capacity(),
        };
        mem::forget(v);
        w
    }

    fn reserve(&mut self, len: usize) {
        let mut vec = self.flush_into_vec();
        vec.reserve(len);
        *self = Self::from_vec(vec);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        let mut vec = self.flush_into_vec();
        vec.extend_from_slice(bytes);
        *self = Self::from_vec(vec);
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WrFontTemplate {
    Raw = 0,
    Native,
}

#[cfg(target_os = "windows")]
fn read_font_descriptor(bytes: &mut WrVecU8, index: u32) -> NativeFontHandle {
    let wchars: Vec<u16> = bytes
        .as_slice()
        .chunks_exact(2)
        .map(|c| u16::from_ne_bytes([c[0], c[1]]))
        .collect();
    NativeFontHandle {
        path: PathBuf::from(OsString::from_wide(&wchars)),
        index,
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn read_font_descriptor(bytes: &mut WrVecU8, index: u32) -> NativeFontHandle {
    // On macOS, the descriptor string is a concatenation of the PostScript name
    // and the font file path (to disambiguate cases where there are multiple
    // faces with the same psname present). The index is the length of the psname
    // portion of the descriptor (= starting offset of the path).
    // Here, we split the descriptor into its two components for further use.
    let chars = bytes.flush_into_vec();
    NativeFontHandle {
        name: String::from_utf8(chars[..index as usize].to_vec()).unwrap_or("".to_string()),
        path: String::from_utf8(chars[index as usize..].to_vec()).unwrap_or("".to_string()),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
fn read_font_descriptor(bytes: &mut WrVecU8, index: u32) -> NativeFontHandle {
    let chars = bytes.flush_into_vec();
    NativeFontHandle {
        path: std::path::PathBuf::from(
            <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(chars),
        ),
        index,
    }
}

#[no_mangle]
pub extern "C" fn wr_vec_u8_free(v: WrVecU8) {
    v.into_vec();
}

#[no_mangle]
pub extern "C" fn wr_vec_u32_free(v: WrVecU32) {
    v.into_vec();
}

impl Into<DeviceRect> for &Emacs_Rectangle {
    fn into(self) -> DeviceRect {
        (self.x, self.y)
            .by(self.width as i32, self.height as i32)
            .to_f32()
    }
}

#[no_mangle]
pub extern "C" fn wr_flush(canvas: &mut WrCanvas) {
    canvas.flush();
}

#[no_mangle]
pub extern "C" fn wr_draw_fringe_bitmap(
    canvas: &mut WrCanvas,
    which: ::libc::c_int,
    pos_x: ::libc::c_int,
    pos_y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    bitmap_width: ::libc::c_int,
    bitmap_height: ::libc::c_int,
    bits: *mut ::libc::c_ushort,
    gc: &Emacs_GC,
    clip_bounds: &Emacs_Rectangle,
) {
    let clip_bounds: DeviceRect =
        (clip_bounds.x, clip_bounds.y).by(clip_bounds.width as i32, clip_bounds.height as i32);

    let pos = DeviceIntPoint::new(pos_x, pos_y).to_f32();

    let image_clip_rect: DeviceRect = {
        if which > 0 {
            (pos_x, pos_y).by(width, height)
        } else {
            DeviceRect::zero()
        }
    };

    let image =
        canvas.get_or_create_fringe_bitmap(which, bitmap_width as u32, bitmap_height as u32, bits);

    // Fixed image_clip_rect
    let image_clip_rect = image_clip_rect
        .intersection(&clip_bounds)
        .unwrap_or_else(|| DeviceRect::zero());

    canvas.display(|builder, space_and_clip, scale| {
        if let Some(image) = &image {
            let image_display_rect = DeviceRect::new(
                pos,
                webrender::api::units::DevicePoint::new(image.width as f32, image.height as f32),
            ) / scale;
            // render image
            builder.push_image(
                &CommonItemProperties::new(image_clip_rect / scale, space_and_clip),
                image_display_rect,
                ImageRendering::Auto,
                AlphaType::Alpha,
                image.image_key,
                pixel_to_color(gc.foreground),
            );
        }
    });
}

// #[no_mangle]
// pub extern "C" fn wr_clip_chain(
// ) -> WrClipChainId {
//     todo!()
// }

#[no_mangle]
pub extern "C" fn wr_push_rect(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    bounds: &Emacs_Rectangle,
    clip_bounds: Option<&Emacs_Rectangle>,
) {
    canvas.push_rect(color_pixel, bounds.into(), clip_bounds.map(|b| b.into()));
}

#[no_mangle]
pub extern "C" fn wr_dp_push_rect(
    canvas: &mut WrCanvas,
    rect: &Emacs_Rectangle,
    clip: &Emacs_Rectangle,
    is_backface_visible: bool,
    force_antialiasing: bool,
    is_checkerboard: bool,
    color_pixel: ::libc::c_ulong,
) {
    // debug_assert!(unsafe { !is_in_render_thread() });
    canvas.dp_push_rect(
        rect.into(),
        clip.into(),
        is_backface_visible,
        force_antialiasing,
        is_checkerboard,
        color_pixel,
    );
}

#[no_mangle]
pub extern "C" fn wr_draw_horizontal_wave(
    canvas: &mut WrCanvas,
    rect: &Emacs_Rectangle,
    color_pixel: ::libc::c_ulong,
    thickness: ::libc::c_int,
) {
    // debug_assert!(unsafe { !is_in_render_thread() });
    canvas.dp_push_line(
        rect.into(),
        rect.into(),
        &pixel_to_color(color_pixel),
        LineStyle::Wavy,
        DeviceIntLength::new(thickness),
        false,
        LineOrientation::Horizontal,
    );
}

#[no_mangle]
pub extern "C" fn wr_begin_define_clip(canvas: &mut WrCanvas) {
    canvas.begin_define_clip();
}

#[no_mangle]
pub extern "C" fn wr_end_define_clip(canvas: &mut WrCanvas) {
    canvas.end_define_clip();
}

#[no_mangle]
pub extern "C" fn wr_define_clip_rect(canvas: &mut WrCanvas, rect: &Emacs_Rectangle) {
    canvas.define_clip_rect(rect.into());
}

#[no_mangle]
pub extern "C" fn wr_draw_box_rect(
    canvas: &mut WrCanvas,
    color: ::libc::c_ulong,
    left_x: ::libc::c_int,
    top_y: ::libc::c_int,
    right_x: ::libc::c_int,
    bottom_y: ::libc::c_int,
    hwidth: ::libc::c_int,
    vwidth: ::libc::c_int,
    left_p: bool,
    right_p: bool,
    clip_rect: &Emacs_Rectangle,
) {
    let color = pixel_to_color(color);
    let clip_rect: DeviceRect = clip_rect.into();
    let rect = (left_x, top_y).to(right_x, bottom_y);
    let border_widths = DeviceIntSideOffsets::new(hwidth, vwidth, hwidth, vwidth);

    let default_border_side: BorderSide = BorderSide {
        color,
        style: BorderStyle::Solid,
    };
    let none_border_side: BorderSide = BorderSide {
        color,
        style: BorderStyle::None,
    };

    let left = if left_p {
        default_border_side
    } else {
        none_border_side
    };
    let right = if right_p {
        default_border_side
    } else {
        none_border_side
    };

    canvas.dp_push_border(
        rect,
        clip_rect,
        true,
        AntialiasBorder::Yes,
        border_widths,
        default_border_side,
        right,
        default_border_side,
        left,
        BorderRadius::default(),
    )
}

#[no_mangle]
pub extern "C" fn wr_draw_relief_box(
    canvas: &mut WrCanvas,
    top_left_color: ::libc::c_ulong,
    bottom_right_color: ::libc::c_ulong,
    left_x: ::libc::c_int,
    top_y: ::libc::c_int,
    right_x: ::libc::c_int,
    bottom_y: ::libc::c_int,
    hwidth: ::libc::c_int,
    vwidth: ::libc::c_int,
    top_p: bool,
    bot_p: bool,
    left_p: bool,
    right_p: bool,
    clip_rect: &Emacs_Rectangle,
) {
    let top_left_color = pixel_to_color(top_left_color);
    let bottom_right_color = pixel_to_color(bottom_right_color);
    let clip_rect: DeviceRect = clip_rect.into();
    let rect = (left_x, top_y).to(right_x, bottom_y);
    let border_widths = DeviceIntSideOffsets::new(hwidth, vwidth, hwidth, vwidth);

    let top = if top_p {
        BorderSide {
            color: top_left_color,
            style: BorderStyle::Solid,
        }
    } else {
        BorderSide {
            color: top_left_color,
            style: BorderStyle::None,
        }
    };
    let bottom = if bot_p {
        BorderSide {
            color: bottom_right_color,
            style: BorderStyle::Solid,
        }
    } else {
        BorderSide {
            color: bottom_right_color,
            style: BorderStyle::None,
        }
    };

    let left = if left_p {
        BorderSide {
            color: top_left_color,
            style: BorderStyle::Solid,
        }
    } else {
        BorderSide {
            color: top_left_color,
            style: BorderStyle::None,
        }
    };
    let right = if right_p {
        BorderSide {
            color: bottom_right_color,
            style: BorderStyle::Solid,
        }
    } else {
        BorderSide {
            color: bottom_right_color,
            style: BorderStyle::None,
        }
    };

    canvas.dp_push_border(
        rect,
        clip_rect,
        true,
        AntialiasBorder::Yes,
        border_widths,
        top,
        right,
        bottom,
        left,
        BorderRadius::default(),
    )
}

#[no_mangle]
pub extern "C" fn wr_push_border(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    bounds: &Emacs_Rectangle,
    clip_bounds: Option<&Emacs_Rectangle>,
) {
    canvas.push_border(color_pixel, bounds.into(), clip_bounds.map(|b| b.into()));
}

// pgtk_draw_rectangle
#[no_mangle]
pub extern "C" fn wr_draw_rect(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    respect_alpha_background: bool,
) {
    let color = pixel_to_color(color_pixel);
    let rect = (x, y).by(width, height);
    let border_widths = DeviceIntSideOffsets::new_all_same(1);

    let default_border_side: BorderSide = BorderSide {
        color,
        style: BorderStyle::Solid,
    };

    canvas.dp_push_border(
        rect,
        rect,
        respect_alpha_background,
        AntialiasBorder::Yes,
        border_widths,
        default_border_side,
        default_border_side,
        default_border_side,
        default_border_side,
        BorderRadius::default(),
    )
}

#[no_mangle]
pub extern "C" fn wr_clear_area(
    canvas: &mut WrCanvas,
    color: ::std::os::raw::c_ulong,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    canvas.push_rect(color, (x, y).by(width, height), None);
}

#[no_mangle]
pub extern "C" fn wr_scroll_run(
    canvas: &mut WrCanvas,
    viewport: &Emacs_Rectangle,
    new_frame_position: &Emacs_Rectangle,
) {
    let viewport: DeviceRect = viewport.into();
    let new_frame_position: DeviceRect = new_frame_position.into();
    if let Some(image_key) = canvas.get_previous_frame() {
        canvas.display(|builder, space_and_clip, scale| {
            builder.push_image(
                &CommonItemProperties::new(viewport / scale, space_and_clip),
                new_frame_position / scale,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        });
    }
}

#[no_mangle]
pub extern "C" fn wr_free_pixmap_impl(canvas: &mut WrCanvas, pixmap: Emacs_Pixmap) {
    canvas.delete_image_by_pixmap(pixmap);

    // take back ownership and RAII will drop resource.
    let _ = unsafe { Box::from_raw(pixmap as *mut WrPixmap) };
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_pixel(ximg: *mut image, x: i32, y: i32) -> i32 {
    unimplemented!();
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_put_pixel(ximg: *mut image, x: i32, y: i32, pixel: u64) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_can_use_native_image_api(image_type: LispObject) -> bool {
    crate::image::can_use_native_image_api(image_type)
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_load_image(
    frame: FrameRef,
    img: *mut image,
    _spec_file: LispObject,
    _spec_data: LispObject,
) -> bool {
    let image: ImageRef = img.into();
    image.load(frame)
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_transform_image(
    frame: FrameRef,
    img: *mut image,
    width: i32,
    height: i32,
    rotation: f64,
) {
    let image: ImageRef = img.into();
    image.transform(frame, width, height, rotation);
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn image_pixmap_draw_cross(
    _frame: FrameRef,
    _pixmap: Emacs_Pixmap,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: u32,
    _color: u64,
) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn image_sync_to_pixmaps(_frame: FrameRef, _img: *mut image) {
    unimplemented!();
}

/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_clear_under_internal_border_impl(f: *mut Frame, canvas: &mut WrCanvas) {
    let mut f = FrameRef::new(f);
    let border = f.internal_border_width();
    let width = f.pixel_width;
    let height = f.pixel_height;
    let margin = f.top_margin_height();
    let bottom_margin = f.bottom_margin_height();
    let face_id_fallback = |id: face_id| {
        if unsafe { globals.Vface_remapping_alist.is_not_nil() } {
            unsafe { lookup_basic_face(ptr::null_mut(), f.clone().as_mut(), id as i32) }
        } else {
            id as i32
        }
    };
    let face_id = match f.parent_frame() {
        Some(_) => face_id_fallback(face_id::CHILD_FRAME_BORDER_FACE_ID),
        None => face_id_fallback(face_id::INTERNAL_BORDER_FACE_ID),
    };
    let face = unsafe { FACE_FROM_ID_OR_NULL(f.as_mut(), face_id) };

    unsafe { block_input() };

    if face.is_null() {
        wr_clear_area(canvas, f.background_pixel, 0, 0, border, height);
        wr_clear_area(canvas, f.background_pixel, 0, margin, width, border);
        wr_clear_area(
            canvas,
            f.background_pixel,
            0,
            width - border,
            border,
            height,
        );
        wr_clear_area(
            canvas,
            f.background_pixel,
            0,
            height - bottom_margin - border,
            width,
            border,
        );
    } else {
        log::error!("unimplemented: clean under internal border with face");
    }

    unsafe { unblock_input() };
}

#[no_mangle]
pub extern "C" fn wr_parse_color(
    color_name: *const ::libc::c_char,
    xcolor: *mut Emacs_Color,
) -> ::libc::c_int {
    use std::ffi::CStr;
    let color_name: &CStr = unsafe { CStr::from_ptr(color_name) };
    let color_name: &str = color_name.to_str().unwrap();
    if let Some(color) = lookup_color_by_name_or_hex(&format!("{}", color_name.to_owned())) {
        color_to_xcolor(color, xcolor);
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn wr_destroy(canvas: *mut WrCanvas) {
    mem::drop(Box::from_raw(canvas));
}

// TODO
// may use wrstate from webrender_bindings

// FIXME remove frame from function args
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wr_init(f: *mut Frame) -> *mut WrCanvas {
    // assert!(unsafe { !is_in_render_thread() });
    let f: FrameRef = f.into();

    //     let state = Box::new(WrState {
    //     pipeline_id,
    //     frame_builder: WebRenderFrameBuilder::new(pipeline_id),
    // });

    // Box::into_raw(state)

    let data = Box::new(WrCanvas::build(f));
    Box::into_raw(data)
}

/// cbindgen:ignore
/// Fit GL context to frame, reflecting frame/scale factor changes
#[no_mangle]
pub extern "C" fn wr_fit_context(f: *mut Frame) {
    let frame: FrameRef = f.into();
    if frame.output().is_null() || frame.output().wr_data.is_null() {
        return;
    }
    frame.wr().update();
}

// #[no_mangle]
// pub extern "C" fn wr_resource_updates_add_raw_font(
//     txn: &mut Transaction,
//     key: WrFontKey,
//     bytes: &mut WrVecU8,
//     index: u32,
// ) {
//     txn.add_raw_font(key, bytes.flush_into_vec(), index);
// }

#[no_mangle]
pub extern "C" fn wr_macfont_draw(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    ct_font_ref: CTFontRef,
    char2b: &mut WrVecU32,
    from: ::libc::c_int,
    to: ::libc::c_int,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    glyph_size: ::libc::c_int,
    padding_p: bool,
) {
    let font_tpl = macfont_font_tpl(ct_font_ref);
    wr_font_draw(canvas, color_pixel, font_tpl, char2b, from, to, x, y, width, height, glyph_size, padding_p);
}

#[no_mangle]
pub extern "C" fn wr_ftfont_draw(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    data: &mut WrVecU8,
    index: u32,
    char2b: &mut WrVecU32,
    from: ::libc::c_int,
    to: ::libc::c_int,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    glyph_size: ::libc::c_int,
    padding_p: bool,
) {
    let font_tpl = FontTemplate::Native(read_font_descriptor(data, index));
}

#[no_mangle]
fn wr_font_draw(
    canvas: &mut WrCanvas,
    color_pixel: ::libc::c_ulong,
    font_tpl: FontTemplate,
    char2b: &mut WrVecU32,
    from: ::libc::c_int,
    to: ::libc::c_int,
    x: ::libc::c_int,
    y: ::libc::c_int,
    width: ::libc::c_int,
    height: ::libc::c_int,
    glyph_size: ::libc::c_int,
    padding_p: bool,
) {
    // println!("{:?}", font_tpl);
    let font_key = canvas.wr_add_font(font_tpl);
    let font_instance_key = canvas.wr_add_font_instance(
        font_key,
        DeviceLength::new(glyph_size as f32),
        Some(FontInstanceOptions::default()),
        Some(FontInstancePlatformOptions::default()),
        Vec::new(),
    );
    let glyph_indices = char2b.as_slice();
    // println!("{:?}", glyph_indices);
    let glyph_dimensions = canvas.glyph_dimensions(font_instance_key, glyph_indices.to_vec());
    // println!("{:?}", glyph_dimensions);
    let glyph_dimensions = glyph_dimensions.as_slice();
    let scale_factor = canvas.layout_to_device_scale_factor();
    let visible_rect = (x, y).by(width, height);
    println!("visible_rect {visible_rect:?}");
    // FIXME visible_rect set above is not correct here, hard code it for now
    let visible_rect = (0, 0).by(10000, 10000);
    let mut x = x;
    let mut glyph_instances: Vec<GlyphInstance> = vec![];
    (from..to).for_each(|i| {
        let index = glyph_indices[i as usize];
        // wr get_glyph_dimensions return none for ‘empty’ textures (height or width = 0)
        // spaces (’ ’) will mostly be None
        // glyphinstance type is using layoutpixel
        let glyph_instance = GlyphInstance {
            index,
            point: DeviceIntPoint::new(x, y).to_f32() / scale_factor,
        };
        // scale back to device pixel
        let advance_width = glyph_dimensions[i as usize]
            .map(|d| d.advance)
            .unwrap_or(0.0)
            * scale_factor.get();
        if padding_p {
            x += 1;
        } else {
            x += advance_width as i32;
        }
        glyph_instances.push(glyph_instance);
    });

    canvas.display(|builder, space_and_clip, scale| {
        let foreground_color = pixel_to_color(color_pixel);

        // draw foreground
        if !glyph_instances.is_empty() {
            let visible_rect = visible_rect / scale;

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

// /// Capture the contents of the current WebRender frame and
// /// save them to a folder relative to the current working directory.
// ///
// /// If START-SEQUENCE is not nil, start capturing each WebRender frame to disk.
// /// If there is already a sequence capture in progress, stop it and start a new
// /// one, with the new path and flags.
// #[allow(unused_variables)]
// #[lisp_fn(min = "2")]
// pub fn wr_api_capture(path: LispStringRef, bits_raw: LispObject, start_sequence: LispObject) {
//     #[cfg(not(feature = "capture"))]
//     error!("Webrender capture not avaiable");
//     #[cfg(feature = "capture")]
//     {
//         use emacs_sys::frame::window_frame_live_or_selected;
//         use std::fs::create_dir_all;
//         use std::fs::File;
//         use std::io::Write;

//         let path = std::path::PathBuf::from(path.to_utf8());
//         match create_dir_all(&path) {
//             Ok(_) => {}
//             Err(err) => {
//                 error!("Unable to create path '{:?}' for capture: {:?}", &path, err);
//             }
//         };
//         let bits_raw = unsafe {
//             emacs_sys::bindings::check_integer_range(
//                 bits_raw,
//                 webrender::CaptureBits::SCENE.bits() as i64,
//                 webrender::CaptureBits::all().bits() as i64,
//             )
//         };

//         let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
//         let canvas = frame.wr_data();
//         let bits = webrender::CaptureBits::from_bits(bits_raw as _).unwrap();
//         let revision_file_path = path.join("wr.txt");
//         message!("Trying to save webrender capture under {:?}", &path);

//         // api call here can possibly make Emacs panic. For example there isn't
//         // enough disk space left. `panic::catch_unwind` isn't support here.
//         if start_sequence.is_nil() {
//             canvas.render_api.save_capture(path, bits);
//         } else {
//             canvas.render_api.start_capture_sequence(path, bits);
//         }

//         match File::create(revision_file_path) {
//             Ok(mut file) => {
//                 if let Err(err) = write!(&mut file, "{}", "") {
//                     error!("Unable to write webrender revision: {:?}", err)
//                 }
//             }
//             Err(err) => error!(
//                 "Capture triggered, creating webrender revision info skipped: {:?}",
//                 err
//             ),
//         }
//     }
// }

// /// Stop a capture begun with `wr--capture'.
// #[lisp_fn(min = "0")]
// pub fn wr_api_stop_capture_sequence() {
//     #[cfg(not(feature = "capture"))]
//     error!("Webrender capture not avaiable");
//     #[cfg(feature = "capture")]
//     {
//         use emacs_sys::frame::window_frame_live_or_selected;

//         message!("Stop capturing WR state");
//         let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
//         let canvas = frame.wr_data();
//         canvas.render_api.stop_capture_sequence();
//     }
// }

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn wr_log_init() {
    // #[cfg(debug_assertions)]
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    // install global collector configured based on WR_LOG env var.
    // #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("WR_LOG"))
        .init();

    log::trace!("Emacs WR");
}

unsafe fn make_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() {
        &[]
    } else {
        slice::from_raw_parts(ptr, len)
    }
}

unsafe fn make_slice_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
    if ptr.is_null() {
        &mut []
    } else {
        slice::from_raw_parts_mut(ptr, len)
    }
}
