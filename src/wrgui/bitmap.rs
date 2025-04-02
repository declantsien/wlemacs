use parking_lot::Mutex;
use webrender_api::units::{DeviceIntSize, TexelRect};
use webrender_api::{ExternalImage, ExternalImageHandler, ExternalImageId, ExternalImageSource, ImageDescriptor, ImageDescriptorFlags, ImageFormat};
use std::os::raw::c_void;
use std::sync::LazyLock;
use webrender::FastHashMap;

use webrender::api::ImageKey;

use crate::types::face;
use crate::util::make_slice;

static BITMAPS: LazyLock<Mutex<FastHashMap<::libc::c_int, Vec<u8>>>> =
    LazyLock::new(Default::default);

/// Used to indicate if an image is opaque, or has an alpha channel.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpacityType {
    Opaque = 0,
    HasAlphaChannel = 1,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WrImageDescriptor {
    pub format: ImageFormat,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub opacity: OpacityType,
    // TODO(gw): Remove this flag (use prim flags instead).
    pub prefer_compositor_surface: bool,
}

impl<'a> From<&'a WrImageDescriptor> for ImageDescriptor {
    fn from(desc: &'a WrImageDescriptor) -> ImageDescriptor {
        let mut flags = ImageDescriptorFlags::empty();

        if desc.opacity == OpacityType::Opaque {
            flags |= ImageDescriptorFlags::IS_OPAQUE;
        }

        ImageDescriptor {
            size: DeviceIntSize::new(desc.width, desc.height),
            stride: if desc.stride != 0 { Some(desc.stride) } else { None },
            format: desc.format,
            offset: 0,
            flags,
        }
    }
}

#[repr(u32)]
#[allow(dead_code)]
enum WrExternalImageType {
    RawData,
    NativeTexture,
    Invalid,
}

#[repr(C)]
struct WrExternalImage {
    image_type: WrExternalImageType,

    // external texture handle
    handle: u32,
    // external texture coordinate
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,

    // external image buffer
    buff: *const u8,
    size: usize,
}

extern "C" {
    fn wr_renderer_lock_external_image(
        renderer: *mut c_void,
        external_image_id: ExternalImageId,
        channel_index: u8,
    ) -> WrExternalImage;
    fn wr_renderer_unlock_external_image(renderer: *mut c_void, external_image_id: ExternalImageId, channel_index: u8);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WrExternalImageHandler {
    external_image_obj: *mut c_void,
}

impl ExternalImageHandler for WrExternalImageHandler {
    fn lock(&mut self, id: ExternalImageId, channel_index: u8) -> ExternalImage {
        let image = unsafe { wr_renderer_lock_external_image(self.external_image_obj, id, channel_index) };
        ExternalImage {
            uv: TexelRect::new(image.u0, image.v0, image.u1, image.v1),
            source: match image.image_type {
                WrExternalImageType::NativeTexture => ExternalImageSource::NativeTexture(image.handle),
                WrExternalImageType::RawData => {
                    ExternalImageSource::RawData(unsafe { make_slice(image.buff, image.size) })
                },
                WrExternalImageType::Invalid => ExternalImageSource::Invalid,
            },
        }
    }

    fn unlock(&mut self, id: ExternalImageId, channel_index: u8) {
        unsafe {
            wr_renderer_unlock_external_image(self.external_image_obj, id, channel_index);
        }
    }
}

impl face {
    pub fn stipple_bitmap(&self) -> ImageKey {
        todo!()
        // BITMAPS.lock().get(&(self.stipple as i32)).unwrap().clone()
    }
}

// #[derive(Clone)]
// pub struct FringeBitmap {
//     pub image_key: ImageKey,

//     pub width: u32,
//     pub height: u32,
// }

// pub fn get_or_create_fringe_bitmap(
//     frame: FrameRef,
//     which: i32,
//     p: *mut draw_fringe_bitmap_params,
// ) -> Option<FringeBitmap> {
//     if which <= 0 {
//         return None;
//     }

//     let mut display_info = frame.display_info().gl_renderer_data();

//     if let Some(bitmap) = display_info.fringe_bitmap_caches.get(&which) {
//         return Some(bitmap.clone());
//     }

//     let bitmap = create_fringe_bitmap(frame.gl_renderer(), p);

//     // add bitmap to cache
//     display_info
//         .fringe_bitmap_caches
//         .insert(which, bitmap.clone());

//     return Some(bitmap);
// }

// fn create_fringe_bitmap(
//     mut canvas: GlRendererRef,
//     p: *mut draw_fringe_bitmap_params,
// ) -> FringeBitmap {
//     let image_buffer = create_fringe_bitmap_image_buffer(p);

//     let (width, height) = image_buffer.dimensions();
//     let descriptor = ImageDescriptor::new(
//         width as i32,
//         height as i32,
//         ImageFormat::RGBA8,
//         ImageDescriptorFlags::empty(),
//     );

//     let data = ImageData::Raw(Arc::new(image_buffer.to_rgba8().to_vec()));

//     let image_key = canvas.add_image(descriptor, data);

//     FringeBitmap {
//         image_key,
//         width,
//         height,
//     }
// }

// fn create_fringe_bitmap_image_buffer(p: *mut draw_fringe_bitmap_params) -> DynamicImage {
//     let height = unsafe { (*p).h };

//     let bitmap_width = 8 as u32;
//     let bitmap_height = (height + unsafe { (*p).dh }) as u32;

//     let bits = unsafe { std::slice::from_raw_parts((*p).bits, (8 * bitmap_height) as usize) };

//     // convert unsigned short array into u8 array
//     let bits: Vec<u8> = bits.iter().map(|v| *v as u8).collect();

//     let bits = BitVec::from_bytes(&bits);

//     let white_pixel = Rgba([255, 255, 255, 255]);
//     let transparent_pixel = Rgba([0, 0, 0, 0]);

//     let image_buffer = RgbaImage::from_fn(bitmap_width, bitmap_height, |x, y| {
//         let index = (y * bitmap_width + x) as usize;

//         if bits
//             .get(index)
//             .expect("RgbaImage construction: out of index.")
//             == true
//         {
//             white_pixel
//         } else {
//             transparent_pixel
//         }
//     });

//     DynamicImage::ImageRgba8(image_buffer)
// }
