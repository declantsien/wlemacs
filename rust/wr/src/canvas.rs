use crate::capi::Emacs_Pixmap;
use crate::color::pixel_to_color;
use crate::gfx::context::GLContext;
use crate::types::AntialiasBorder;
use crate::util::HandyDandyRectBuilder;
use image::GenericImageView;
use webrender::api::euclid::Length;

use super::types::ImageHash;
use crate::gfx::context::GLContextTrait;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use gleam::gl;
use webrender::FastHashMap;

use webrender::api::units::*;
use webrender::api::*;
use webrender::{
    RenderApi, Renderer, Transaction, {self},
};

pub type LayoutLength = Length<f32, LayoutPixel>;
pub type DeviceLength = Length<f32, DevicePixel>;

use super::texture::TextureResourceManager;

#[derive(Clone)]
pub struct FringeBitmap {
    pub image_key: ImageKey,

    pub width: u32,
    pub height: u32,
}

pub struct WrCanvas {
    device_size: DeviceIntSize,
    scale_factor: LayoutToDeviceScale,
    fonts: FastHashMap<FontTemplate, FontKey>,
    fringe_bitmaps: FastHashMap<i32, FringeBitmap>,
    font_instances: FastHashMap<
        (
            FontKey,
            FontSize,
            Option<FontInstanceOptions>,
            Option<FontInstancePlatformOptions>,
            Vec<FontVariation>,
        ),
        FontInstanceKey,
    >,
    images: FastHashMap<ImageHash, (ImageKey, ImageDescriptor)>,
    allow_mipmaps: bool,
    pub render_api: RenderApi,
    pub document_id: DocumentId,
    pipeline_id: PipelineId,
    root_space_and_clip: SpaceAndClipInfo,
    // When is in between of define clipchain_id
    is_defining_clipinfo: bool,
    // state of defined clip_ids
    clip_ids: Vec<ClipId>,
    epoch: Epoch,
    display_list_builder: Option<DisplayListBuilder>,
    previous_frame_image: Option<ImageKey>,
    texture_resources: Rc<RefCell<TextureResourceManager>>,
    renderer: Renderer,
    gl_context: GLContext,
    gl: Rc<dyn gl::Gl>,
}

impl fmt::Debug for WrCanvas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "gl renderer data")
    }
}

impl WrCanvas {
    pub fn build(
        mut gl_context: GLContext,
        device_size: DeviceIntSize,
        device_pixel_ratio: libc::c_double,
    ) -> Self {
        let gl = gl_context.load_gl();

        let version = gl.get_string(gl::VERSION);
        println!("WebRender - OpenGL version new {}", version);
        println!("Device size {:?}", device_size);

        gl_context.ensure_is_current();

        // webrender
        let webrender_opts = webrender::WebRenderOptions {
            clear_color: ColorF::new(1.0, 1.0, 1.0, 1.0),
            ..webrender::WebRenderOptions::default()
        };

        let notifier = Box::new(Notifier::new());
        let (mut renderer, sender) =
            webrender::create_webrender_instance(gl.clone(), notifier, webrender_opts, None)
                .unwrap();

        let texture_resources = Rc::new(RefCell::new(TextureResourceManager::new(
            gl.clone(),
            sender.create_api(),
        )));
        let mut api = sender.create_api();
        let document_id = api.add_document(device_size);

        let external_image_handler = texture_resources.borrow_mut().new_external_image_handler();
        renderer.set_external_image_handler(external_image_handler);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);
        let mut txn = Transaction::new();

        let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
        txn.set_root_pipeline(pipeline_id);

        let scale_factor = LayoutToDeviceScale::new(device_pixel_ratio as f32);
        gl_context.resize(&device_size);

        api.send_transaction(document_id, txn);

        Self {
            device_size,
            scale_factor,
            fonts: FastHashMap::default(),
            font_instances: FastHashMap::default(),
            images: FastHashMap::default(),
            fringe_bitmaps: FastHashMap::default(),
            allow_mipmaps: false,
            render_api: api,
            document_id,
            pipeline_id,
            root_space_and_clip,
            is_defining_clipinfo: false,
            clip_ids: Vec::new(),
            epoch,
            display_list_builder: None,
            previous_frame_image: None,
            renderer,
            gl_context,
            gl,
            texture_resources,
        }
    }

    fn copy_framebuffer_to_texture(&self, device_rect: DeviceIntRect) -> ImageKey {
        let mut origin = device_rect.min;

        let device_size = self.device_size();

        if !self.renderer.device.surface_origin_is_top_left() {
            origin.y = device_size.height - origin.y - device_rect.height();
        }

        let fb_rect = FramebufferIntRect::from_origin_and_size(
            FramebufferIntPoint::from_untyped(origin.to_untyped()),
            FramebufferIntSize::from_untyped(device_rect.size().to_untyped()),
        );

        let need_flip = !self.renderer.device.surface_origin_is_top_left();

        let (image_key, texture_id) = self.texture_resources.borrow_mut().new_image(
            self.document_id,
            fb_rect.size(),
            need_flip,
        );

        let gl = &self.gl;
        gl.bind_texture(gl::TEXTURE_2D, texture_id);

        gl.copy_tex_sub_image_2d(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            fb_rect.min.x,
            fb_rect.min.y,
            fb_rect.size().width,
            fb_rect.size().height,
        );

        gl.bind_texture(gl::TEXTURE_2D, 0);

        image_key
    }

    // pub fn scale(&self) -> f32 {
    //     self.frame.scale_factor() as f32
    // }

    pub fn layout_to_device_scale_factor(&self) -> LayoutToDeviceScale {
        self.scale_factor
    }

    pub fn layout_size(&self) -> LayoutSize {
        let device_size = self.device_size();
        device_size.to_f32().cast_unit::<LayoutPixel>()
    }

    fn new_builder(&mut self, image: Option<(ImageKey, LayoutRect)>) -> DisplayListBuilder {
        let pipeline_id = self.pipeline_id;

        let layout_size = self.layout_size();
        let mut builder = DisplayListBuilder::new(pipeline_id);
        builder.begin();

        if let Some((image_key, image_rect)) = image {
            let bounds = LayoutRect::from_size(layout_size);

            builder.push_image(
                &CommonItemProperties::new(bounds, self.root_space_and_clip),
                image_rect,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        }

        builder
    }

    pub fn device_size(&self) -> DeviceIntSize {
        self.device_size
    }

    pub fn with_display_list_builder<F, T>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&mut DisplayListBuilder) -> T,
    {
        if self.display_list_builder.is_none() {
            let layout_size = self.layout_size();

            let image_and_pos = self
                .previous_frame_image
                .map(|image_key| (image_key, LayoutRect::from_size(layout_size)));

            self.display_list_builder = Some(self.new_builder(image_and_pos));
        }

        if let Some(builder) = &mut self.display_list_builder {
            Some(f(builder));
        }
        None
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo, LayoutToDeviceScale),
    {
        let scale_factor = self.layout_to_device_scale_factor();
        let spatial_id = self.root_space_and_clip.spatial_id;
        // TBD where these clones
        let mut clip_chain_id = None;
        let clip_ids = self.clip_ids.clone();
        if !clip_ids.is_empty() {
            clip_chain_id = self.with_display_list_builder(|builder| {
                builder.define_clip_chain(None, clip_ids.clone())
            });
        }

        let space_and_clip = if let Some(clip_chain_id) = clip_chain_id {
            SpaceAndClipInfo {
                spatial_id,
                clip_chain_id,
            }
        } else {
            self.root_space_and_clip
        };

        self.with_display_list_builder(|builder| {
            f(builder, space_and_clip, scale_factor);
        });

        self.assert_no_gl_error();
    }

    fn ensure_context_is_current(&mut self) {
        self.gl_context.ensure_is_current();
        self.assert_no_gl_error();
    }

    #[track_caller]
    fn assert_no_gl_error(&self) {
        debug_assert_eq!(self.gl.get_error(), gleam::gl::NO_ERROR);
    }

    pub fn flush(&mut self) {
        self.assert_no_gl_error();
        self.ensure_context_is_current();

        let builder = std::mem::replace(&mut self.display_list_builder, None);

        if let Some(mut builder) = builder {
            let epoch = self.epoch;
            let mut txn = Transaction::new();

            txn.set_display_list(epoch, builder.end());
            txn.set_root_pipeline(self.pipeline_id);
            txn.generate_frame(0, true, RenderReasons::NONE);

            self.display_list_builder = None;

            self.render_api.send_transaction(self.document_id, txn);

            self.render_api.flush_scene_builder();

            let device_size = self.device_size();

            self.gl_context.bind_framebuffer(&mut self.gl);

            self.renderer.update();

            self.assert_no_gl_error();

            self.renderer.render(device_size, 0).unwrap();
            let _ = self.renderer.flush_pipeline_info();

            self.texture_resources.borrow_mut().clear();

            let image_key = self.copy_framebuffer_to_texture(DeviceIntRect::from_size(device_size));
            self.previous_frame_image = Some(image_key);

            self.gl_context.swap_buffers();
        }
    }

    pub fn get_previous_frame(&self) -> Option<ImageKey> {
        self.previous_frame_image
    }

    pub fn clear_display_list_builder(&mut self) {
        let _ = std::mem::replace(&mut self.display_list_builder, None);
    }
    pub fn wr_add_font_instance(
        &mut self,
        font_key: FontKey,
        glyph_size: LayoutLength,
        options: Option<FontInstanceOptions>,
        platform_options: Option<FontInstancePlatformOptions>,
        variations: Vec<FontVariation>,
    ) -> FontInstanceKey {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();
        let glyph_size = glyph_size * self.layout_to_device_scale_factor();
        let glyph_size = glyph_size.get();
        let hash_map_key = (
            font_key,
            FontSize::from_f32_px(glyph_size),
            options,
            platform_options,
            variations.clone(),
        );
        if let Some(font_instance_key) = self.font_instances.get(&hash_map_key) {
            return *font_instance_key;
        };

        let key = self.render_api.generate_font_instance_key();
        let mut txn = Transaction::new();
        txn.add_font_instance(
            key,
            font_key,
            glyph_size,
            options,
            platform_options,
            variations,
        );
        self.render_api.send_transaction(self.document_id, txn);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("wr add font instance in {:?}", elapsed);
        }
        key
    }

    pub fn wr_delete_font_instance(&mut self, key: FontInstanceKey) {
        let mut txn = Transaction::new();
        txn.delete_font_instance(key);
        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn wr_add_font(&mut self, data: FontTemplate) -> FontKey {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        if let Some(key) = self.fonts.get(&data) {
            return *key;
        }

        let font_key = self.render_api.generate_font_key();
        let mut txn = Transaction::new();
        match data {
            FontTemplate::Raw(ref bytes, index) => {
                txn.add_raw_font(font_key, bytes.to_vec(), index)
            }
            FontTemplate::Native(ref native_font) => {
                txn.add_native_font(font_key, native_font.clone())
            }
        }

        self.render_api.send_transaction(self.document_id, txn);

        self.fonts.insert(data, font_key);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("wr add font in {:?}", elapsed);
        }
        font_key
    }

    pub fn wr_delete_font(&mut self, key: FontKey) {
        todo!()
    }

    pub fn glyph_indices(&self, key: FontKey, text: &str) -> Vec<Option<GlyphIndex>> {
        self.render_api.get_glyph_indices(key, text)
    }

    /// Gets the dimensions for the supplied glyph keys
    ///
    /// Note: Internally, the internal texture cache doesn't store
    /// 'empty' textures (height or width = 0)
    /// This means that glyph dimensions e.g. for spaces (' ') will mostly be None.
    pub fn glyph_dimensions(
        &self,
        key: FontInstanceKey,
        glyph_indices: Vec<GlyphIndex>,
    ) -> Vec<Option<GlyphDimensions>> {
        self.render_api.get_glyph_dimensions(key, glyph_indices)
    }

    pub fn allow_mipmaps(&mut self, allow_mipmaps: bool) {
        self.allow_mipmaps = allow_mipmaps;
    }

    pub fn add_image(&mut self, descriptor: ImageDescriptor, data: ImageData) -> ImageKey {
        let image_key = self.render_api.generate_image_key();
        let mut txn = Transaction::new();

        txn.add_image(image_key, descriptor, data, None);

        self.render_api.send_transaction(self.document_id, txn);

        image_key
    }

    pub fn update_image(&mut self, key: ImageKey, descriptor: ImageDescriptor, data: ImageData) {
        let mut txn = Transaction::new();

        txn.update_image(key, descriptor, data, &DirtyRect::All);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image(&mut self, image_key: ImageKey) {
        let mut txn = Transaction::new();

        txn.delete_image(image_key);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image_by_pixmap(&mut self, _pixmap: Emacs_Pixmap) {
        // We cache image by source from image_cache.rs
        // transform(rotate, resize(scale)) on the fly
        // loop images, compare pixmap,find image_key
        log::warn!("TODO free pixmap");
    }

    // Create glyph raster image instance with scaled size
    pub fn add_or_update_image(
        &mut self,
        hash: &ImageHash,
        descriptor: ImageDescriptor,
        data: ImageData,
    ) -> ImageKey {
        let image_key = self.image_key(&hash).map(|c| c.0);

        if let Some(key) = image_key {
            self.update_image(key, descriptor, data);
            return key;
        }

        let key = self.add_image(descriptor, data);
        self.images.insert(*hash, (key, descriptor));
        key
    }

    pub fn image_key(&self, hash: &ImageHash) -> Option<(ImageKey, ImageDescriptor)> {
        self.images.get(hash).copied()
    }

    pub fn get_or_create_fringe_bitmap(
        &mut self,
        which: i32,
        bitmap_width: u32,
        bitmap_height: u32,
        bits: *mut ::libc::c_ushort,
    ) -> Option<FringeBitmap> {
        if which <= 0 {
            return None;
        }

        if let Some(bitmap) = self.fringe_bitmaps.get(&which) {
            return Some(bitmap.clone());
        }

        let bitmap = self.create_fringe_bitmap(bitmap_width, bitmap_height, bits);

        // add bitmap to cache
        self.fringe_bitmaps.insert(which, bitmap.clone());

        return Some(bitmap);
    }

    pub fn begin_define_clip(&mut self) {
        assert_eq!(self.is_defining_clipinfo, false);
        assert_eq!(self.clip_ids.is_empty(), true);
        self.is_defining_clipinfo = true;
    }

    pub fn end_define_clip(&mut self) {
        assert_eq!(self.is_defining_clipinfo, true);
        self.is_defining_clipinfo = false;
        self.clip_ids = Vec::new();
    }

    pub fn define_clip_rect(&mut self, rect: LayoutRect) {
        assert_eq!(self.is_defining_clipinfo, true);
        let spatial_id = self.root_space_and_clip.spatial_id;
        let clip_id =
            self.with_display_list_builder(|builder| builder.define_clip_rect(spatial_id, rect));
        if let Some(clip_id) = clip_id {
            self.clip_ids.push(clip_id);
        }
    }

    fn create_fringe_bitmap(
        &mut self,
        bitmap_width: u32,
        bitmap_height: u32,
        bits: *mut ::libc::c_ushort,
    ) -> FringeBitmap {
        let image_buffer = create_fringe_bitmap_image_buffer(bitmap_width, bitmap_height, bits);

        let (width, height) = image_buffer.dimensions();
        let descriptor = ImageDescriptor::new(
            width as i32,
            height as i32,
            ImageFormat::RGBA8,
            ImageDescriptorFlags::empty(),
        );

        let data = ImageData::Raw(Arc::new(image_buffer.to_rgba8().to_vec()));

        let image_key = self.add_image(descriptor, data);

        FringeBitmap {
            image_key,
            width,
            height,
        }
    }

    pub fn update(&mut self) {
        let size = self.device_size();
        let device_rect =
            DeviceIntRect::from_origin_and_size(DeviceIntPoint::new(0, 0), size.clone());
        log::debug!("resize {size:?} rect {device_rect:?}");
        let mut txn = Transaction::new();
        txn.set_document_view(device_rect);
        self.render_api.send_transaction(self.document_id, txn);

        self.gl_context.resize(&size);
    }

    pub fn draw_rectangle(&mut self, clear_color: ColorF, rect: LayoutRect) {
        self.display(|builder, space_and_clip, scale_factor| {
            builder.push_rect(
                &CommonItemProperties::new(rect, space_and_clip),
                rect,
                clear_color,
            );
        });
    }

    pub fn push_rect(
        &mut self,
        color_pixel: ::libc::c_ulong,
        rect: LayoutRect,
        clip_rect: Option<LayoutRect>,
    ) {
        println!("color pixel: {color_pixel:?}");
        let clear_color = crate::platform::pixel_to_color(color_pixel);
        println!("colorf: {clear_color:?}");
        let scale = self.layout_to_device_scale_factor();
        self.display(|builder, space_and_clip, scale_factor| {
            builder.push_rect(
                &CommonItemProperties::new(
                    (clip_rect.unwrap_or(rect) * scale).cast_unit::<LayoutPixel>(),
                    space_and_clip,
                ),
                (rect * scale).cast_unit::<LayoutPixel>(),
                clear_color,
            );
        });
    }

    pub fn dp_push_rect(
        &mut self,
        rect: LayoutRect,
        clip: LayoutRect,
        is_backface_visible: bool,
        force_antialiasing: bool,
        is_checkerboard: bool,
        color: libc::c_ulong,
    ) {
        // debug_assert!(unsafe { !is_in_render_thread() });
        let rect = (rect * self.layout_to_device_scale_factor()).cast_unit::<LayoutPixel>();
        let clip = (clip * self.layout_to_device_scale_factor()).cast_unit::<LayoutPixel>();
        let color = crate::platform::pixel_to_color(color);

        self.display(|dl_builder, space_and_clip, scale_factor| {
            let mut prim_info =
                common_item_properties_for_rect(clip, is_backface_visible, &space_and_clip);
            if force_antialiasing {
                prim_info.flags |= PrimitiveFlags::ANTIALISED;
            }
            if is_checkerboard {
                prim_info.flags |= PrimitiveFlags::CHECKERBOARD_BACKGROUND;
            }

            dl_builder.push_rect(&prim_info, rect, color);
        });
    }

    pub fn push_border(
        &mut self,
        color_pixel: ::libc::c_ulong,
        rect: LayoutRect,
        clip_rect: Option<LayoutRect>,
    ) {
        let color = pixel_to_color(color_pixel);
        let border_widths = LayoutSideOffsets::new_all_same(1.0);

        let border_side = BorderSide {
            color,
            style: BorderStyle::Solid,
        };

        let border_details = BorderDetails::Normal(NormalBorder {
            top: border_side,
            right: border_side,
            bottom: border_side,
            left: border_side,
            radius: BorderRadius::uniform(0.0),
            do_aa: true,
        });

        self.display(|builder, space_and_clip, scale_factor| {
            builder.push_border(
                &CommonItemProperties::new(clip_rect.unwrap_or(rect), space_and_clip),
                rect,
                border_widths,
                border_details,
            );
        });
    }

    pub fn dp_push_border(
        &mut self,
        rect: LayoutRect,
        clip: LayoutRect,
        is_backface_visible: bool,
        do_aa: AntialiasBorder,
        widths: DeviceIntSideOffsets,
        top: BorderSide,
        right: BorderSide,
        bottom: BorderSide,
        left: BorderSide,
        radius: BorderRadius,
    ) {
        // debug_assert!(unsafe { is_in_main_thread() });

        let border_details = BorderDetails::Normal(NormalBorder {
            left,
            right,
            top,
            bottom,
            radius,
            do_aa: do_aa == AntialiasBorder::Yes,
        });

        self.display(|dl_builder, space_and_clip, scale_factor| {
            let prim_info = CommonItemProperties {
                clip_rect: clip,
                clip_chain_id: space_and_clip.clip_chain_id,
                spatial_id: space_and_clip.spatial_id,
                flags: prim_flags(
                    is_backface_visible,
                    /* prefer_compositor_surface */ false,
                ),
            };

            dl_builder.push_border(
                &prim_info,
                rect,
                device_int_to_layout_side_offsets(widths, scale_factor),
                border_details,
            );
        });
    }

    pub fn dp_push_line(
        &mut self,
        bounds: LayoutRect,
        clip: LayoutRect,
        color: &ColorF,
        style: LineStyle,
        wavy_line_thickness: LayoutLength,
        is_backface_visible: bool,
        orientation: LineOrientation,
    ) {
        // debug_assert!(unsafe { is_in_main_thread() });

        self.display(|dl_builder, space_and_clip, scale_factor| {
            let prim_info = CommonItemProperties {
                clip_rect: clip,
                clip_chain_id: space_and_clip.clip_chain_id,
                spatial_id: space_and_clip.spatial_id,
                flags: prim_flags(
                    is_backface_visible,
                    /* prefer_compositor_surface */ false,
                ),
            };

            dl_builder.push_line(
                &prim_info,
                &bounds,
                wavy_line_thickness.get(),
                orientation,
                color,
                style,
            );
        });
    }

    pub fn deinit(mut self) {
        self.ensure_context_is_current();
        self.renderer.deinit();
    }
}

struct Notifier {}

impl Notifier {
    fn new() -> Notifier {
        Notifier {}
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier {})
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        _: DocumentId,
        _scrolled: bool,
        composite_needed: bool,
        _frame_publish_id: FramePublishId,
    ) {
        self.wake_up(composite_needed);
    }
}

fn create_fringe_bitmap_image_buffer(
    bitmap_width: u32,
    bitmap_height: u32,
    bits: *mut ::libc::c_ushort,
) -> image::DynamicImage {
    use image::{Rgba, RgbaImage};

    // convert unsigned short array into u8 array
    let bits: Vec<u8> = if bits.is_null() {
        // `len` is assumed to be 0.
        Vec::new()
    } else {
        let bits = unsafe { std::slice::from_raw_parts(bits, (8 * bitmap_height) as usize) };
        bits.iter().map(|v| *v as u8).collect()
    };

    let bits = bit_vec::BitVec::from_bytes(&bits);

    let white_pixel = Rgba([255, 255, 255, 255]);
    let transparent_pixel = Rgba([0, 0, 0, 0]);

    let image_buffer = RgbaImage::from_fn(bitmap_width, bitmap_height, |x, y| {
        let index = (y * bitmap_width + x) as usize;

        if bits
            .get(index)
            .expect("RgbaImage construction: out of index.")
            == true
        {
            white_pixel
        } else {
            transparent_pixel
        }
    });

    image::DynamicImage::ImageRgba8(image_buffer)
}

// A helper fn to construct a PrimitiveFlags
fn prim_flags(is_backface_visible: bool, prefer_compositor_surface: bool) -> PrimitiveFlags {
    let mut flags = PrimitiveFlags::empty();

    if is_backface_visible {
        flags |= PrimitiveFlags::IS_BACKFACE_VISIBLE;
    }

    if prefer_compositor_surface {
        flags |= PrimitiveFlags::PREFER_COMPOSITOR_SURFACE;
    }

    flags
}

fn prim_flags2(
    is_backface_visible: bool,
    prefer_compositor_surface: bool,
    supports_external_compositing: bool,
) -> PrimitiveFlags {
    let mut flags = PrimitiveFlags::empty();

    if supports_external_compositing {
        flags |= PrimitiveFlags::SUPPORTS_EXTERNAL_COMPOSITOR_SURFACE;
    }

    flags | prim_flags(is_backface_visible, prefer_compositor_surface)
}

fn common_item_properties_for_rect(
    clip_rect: LayoutRect,
    is_backface_visible: bool,
    space_and_clip: &SpaceAndClipInfo,
) -> CommonItemProperties {
    CommonItemProperties {
        // NB: the damp-e10s talos-test will frequently crash on startup if we
        // early-return here for empty rects. I couldn't figure out why, but
        // it's pretty harmless to feed these through, so, uh, we do?
        clip_rect,
        clip_chain_id: space_and_clip.clip_chain_id,
        spatial_id: space_and_clip.spatial_id,
        flags: prim_flags(
            is_backface_visible,
            /* prefer_compositor_surface */ false,
        ),
    }
}

fn device_int_to_layout_side_offsets(
    offsets: DeviceIntSideOffsets,
    scale_factor: LayoutToDeviceScale,
) -> LayoutSideOffsets {
    LayoutSideOffsets::new(
        offsets.top as f32 / scale_factor.get(),
        offsets.right as f32 / scale_factor.get(),
        offsets.bottom as f32 / scale_factor.get(),
        offsets.left as f32 / scale_factor.get(),
    )
}

fn device_int_to_layout_length(
    length: DeviceIntLength,
    scale_factor: LayoutToDeviceScale,
) -> LayoutLength {
    LayoutLength::new(length.get() as f32 / scale_factor.get())
}
