use crate::canvas::WrCanvas;
use crate::platform::font::FontInfo;
use crate::types::{
    font, frame, EmacsLength, EmacsRect, EmacsToLayoutScale,
    ExternalPtr, GlyphSize, LayoutLength,
};
use webrender::api::{
    FontInstanceKey, FontInstanceOptions, FontInstancePlatformOptions, FontKey, FontTemplate,
    FontVariation, IdNamespace,
};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutSize};
use webrender_api::GlyphDimensions;
use wr_glyph_rasterizer::{BaseFontInstance, FontInstance, GlyphRasterizer};

impl<'a> font {
    pub fn from_ptr(f: *mut font) -> Option<&'a font> {
        unsafe { f.as_ref() }
    }

    pub fn from_ptr_mut(f: *mut font) -> Option<&'a mut font> {
        unsafe { f.as_mut() }
    }
}

static WR_GLYPH_RASTERIZER: LazyLock<Mutex<GlyphRasterizer>> = LazyLock::new(|| {
    let worker = rayon::ThreadPoolBuilder::new()
        .thread_name(|idx| format!("WRWorker#{}", idx))
        .build();
    let workers = std::sync::Arc::new(worker.unwrap());
    let rasterizer = GlyphRasterizer::new(workers, None, true);
    Mutex::new(rasterizer)
});

static CACHE: LazyLock<Mutex<HashMap<FontTemplate, FontKey>>> = LazyLock::new(Default::default);
static INSTANCES_CACHE: LazyLock<
    Mutex<
        HashMap<
            (
                FontKey,
                GlyphSize, // scaled glyph size
                Option<FontInstanceOptions>,
                Option<FontInstancePlatformOptions>,
                Vec<FontVariation>,
            ),
            FontInstance,
        >,
    >,
> = LazyLock::new(Default::default);

pub type FontRef = ExternalPtr<font>;
static PENDING_FONTS: LazyLock<Mutex<Vec<FontRef>>> = LazyLock::new(Default::default);
const namespace: IdNamespace = IdNamespace(1);

fn wr_font_key(font_tpl: FontTemplate, rasterizer: Option<&mut GlyphRasterizer>) -> FontKey {
    let mut cache = CACHE.lock();
    let font_key = || {
        if let Some(key) = cache.get(&font_tpl) {
            return *key;
        }
        FontKey::new(namespace, cache.len() as u32)
    };
    let key = font_key();
    if let Some(rasterizer) = rasterizer {
        rasterizer.add_font(key, font_tpl.clone());
    }
    cache.insert(font_tpl, key);
    key
}

fn wr_font_instance(
    key: FontKey,
    glyph_size: GlyphSize,
    rasterizer: Option<&mut GlyphRasterizer>,
) -> FontInstance {
    let mut instances_cache = INSTANCES_CACHE.lock();
    let options = FontInstanceOptions::default();
    let platform_options = FontInstancePlatformOptions::default();
    let variations: Vec<FontVariation> = Vec::new();
    let hash_key = (
        key,
        glyph_size,
        Some(options),
        Some(platform_options),
        variations,
    );
    let font_instance = || {
        if let Some(key) = instances_cache.get(&hash_key) {
            return key.clone();
        }
        let instance_key = FontInstanceKey::new(namespace, instances_cache.len() as u32);

        let base = BaseFontInstance::new(
            instance_key,
            key,
            glyph_size.to_layout_length().get(),
            Some(options),
            Some(platform_options),
            Vec::new(),
        );
        FontInstance::from_base(std::sync::Arc::new(base))
    };
    let mut instance = font_instance();
    if let Some(rasterizer) = rasterizer {
        rasterizer.prepare_font(&mut instance);
    }
    instances_cache.insert(hash_key, instance.clone());
    instance.clone()
}

pub fn with_get_glyph_dimension<F>(
    font_info: &mut FontInfo,
    glyph_size: i32,
    scale_factor: f32,
    fr: Option<*mut frame>,
    f: &mut F,
) where
    F: FnMut(&mut dyn FnMut(char) -> Option<GlyphDimensions>, bool),
{
    if let Some(wr) = fr
        .and_then(|f| frame::from_ptr(f))
        .and_then(|f| f.renderer_mut())
    {
        let font_tpl = font_info.font_template();
        let key = wr.wr_add_font(font_tpl);
        let glyph_size = EmacsLength::new(glyph_size as f32);
        let instance_key = wr.wr_add_font_instance(
            key,
            glyph_size,
            Some(FontInstanceOptions::default()),
            Some(FontInstancePlatformOptions::default()),
            Vec::new(),
        );
        font_info.f = fr.unwrap();
        font_info.key = key;
        font_info.instance_key = instance_key;

        let mut get_glyph_dimension = Box::new(|ch: char| -> Option<GlyphDimensions> {
            let str = ch.to_string();
            let index = wr
                .glyph_indices(key, str.as_str())
                .get(0)
                .map(|i| *i)
                .and_then(|i| i)
                .unwrap_or(0);
            let indices = vec![index];
            let d = wr
                .glyph_dimensions(instance_key, indices)
                .get(0)
                .map(|i| *i)
                .and_then(|i| i);
            return d;
        });
        f(&mut get_glyph_dimension, false);
    } else {
        let font_tpl = font_info.font_template();
        let mut rasterizer = WR_GLYPH_RASTERIZER.lock();

        let key = wr_font_key(font_tpl, Some(&mut rasterizer));
        let scale_factor = EmacsToLayoutScale::new(scale_factor);

        let glyph_size: EmacsLength = EmacsLength::new(glyph_size as f32);
        let glyph_size = GlyphSize::from_layout_length(glyph_size * scale_factor);
        let instance = wr_font_instance(key, glyph_size, Some(&mut rasterizer));

        let get_glyph_dimension = Box::new(|ch: char| -> Option<GlyphDimensions> {
            let index = rasterizer.get_glyph_index(key, ch);
            /* In order to simulate the Xft behavior, we use metrics of
            glyph ID 0 if there is no glyph for an ASCII printable.  */
            let index = index.unwrap_or(0);
            rasterizer.get_glyph_dimensions(&instance, index)
        });
        font_info.f = fr.unwrap();
        f(&mut Box::new(get_glyph_dimension), true);
    };
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_prepare_font(f: *mut frame, font: *mut font) {
    let ft = font::from_ptr_mut(font).unwrap();
    let fr = frame::from_ptr(f).unwrap();
    with_get_glyph_dimension(
        font::from_ptr_mut(font)
            .and_then(|ft| ft.font_info_mut())
            .unwrap(),
        ft.pixel_size,
        fr.scale_factor() as f32,
        Some(f),
        &mut |get_glyph_dimension, is_pending| {
            let mut n = 0;
            (32..127).for_each(|c| {
                let ch = char::from_u32(c).unwrap();
                // None when FT_Glyph_Format is not FT_GLYPH_FORMAT_OUTLINE and FT_GLYPH_FORMAT_BITMAP
                if let Some(dimensions) = get_glyph_dimension(ch) {
                    let this_width = dimensions.advance as i32;
                    if this_width > 0 {
                        if ft.min_width == 0 || ft.min_width > this_width {
                            ft.min_width = this_width;
                        }
                        if this_width > ft.max_width {
                            ft.max_width = this_width;
                        }
                        if c == 32 {
                            ft.space_width = this_width;
                        }
                        ft.average_width += this_width;
                    }
                    n += 1;
                    // println!("ch: {ch:?}, dimensions: {dimensions:?}");
                }
            });

            ft.average_width /= n;

            if is_pending {
                PENDING_FONTS.lock().push(FontRef::new(font));
            }
        },
    )
}

//TODO
// font-driver has_char/encode_char may needs to use wr api
// so that  glyphdimensions

impl font {
    #[inline(always)]
    pub fn base(&self) -> i32 {
        self.ascent
    }

    #[inline(always)]
    pub fn vcenter_baseline_offset(&self, f: &frame) -> i32 {
        // check C macro VCENTER_BASELINE_OFFSET
        let x = if f.line_height > self.height { 1 } else { 0 };
        self.descent + (f.line_height - self.height + x) / 2
            - f.font().map(|ft| ft.descent).unwrap_or(0)
            - f.baseline_offset()
    }

    pub fn font_key(&self, f: &mut frame) -> FontKey {
        let wr = f.renderer_mut().unwrap();
        let font_tpl = self.font_template();
        wr.wr_add_font(font_tpl)
    }

    pub fn font_instance_key(&self, f: &mut frame) -> FontInstanceKey {
        let font_key = self.font_key(f);
        let glyph_size = EmacsLength::new(self.pixel_size as f32);
        let wr = f.renderer_mut().unwrap();
        wr.wr_add_font_instance(
            font_key,
            glyph_size,
            Some(FontInstanceOptions::default()),
            Some(FontInstancePlatformOptions::default()),
            Vec::new(),
        )
    }

    pub fn glyph_dimensions(
        &self,
        f: &mut frame,
        indices: Vec<u32>,
    ) -> Vec<Option<GlyphDimensions>> {
        let instance_key = self.font_instance_key(f);
        let wr = f.renderer_mut().unwrap();
        wr.glyph_dimensions(instance_key, indices)
    }
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wrfont_get_advance_width_for_glyph(
    font_info: *mut FontInfo,
    glyph: u32,
    prev_advance: f64,
) -> libc::c_double {
    let font = font::from_ptr(font_info as *mut font).unwrap();
     let font_info = FontInfo::from_ptr_mut(font_info).unwrap();
      let wr = frame::from_ptr(font_info.f)
        .and_then(|f| f.renderer_mut())
        .unwrap();
    let GlyphDimensions { advance, .. } = wr
        .glyph_dimensions(font_info.instance_key, vec![glyph])
        .get(0)
        .map(|i| *i)
        .and_then(|i| i)
        .unwrap();
    let advance = LayoutLength::new(advance) / wr.emacs_to_layout_scale();
    return advance.get() as f64;
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wrfont_get_bounding_rect_for_glyph(
    font_info: *mut FontInfo,
    glyph: u32,
    prev_advance: f64,
) -> EmacsRect {
    let font = font::from_ptr(font_info as *mut font).unwrap();
    let font_info = FontInfo::from_ptr_mut(font_info).unwrap();
    let wr = frame::from_ptr(font_info.f)
        .and_then(|f| f.renderer_mut())
        .unwrap();
    let GlyphDimensions {
        left,
        top,
        width,
        height,
        ..
    } = wr
        .glyph_dimensions(font_info.instance_key, vec![glyph])
        .get(0)
        .map(|i| *i)
        .and_then(|i| i)
        .unwrap();
    let bounds = LayoutRect::from_origin_and_size(
        LayoutPoint::new(left as f32, top as f32),
        LayoutSize::new(width as f32, height as f32),
    ) / wr.emacs_to_layout_scale();
    return bounds;
}

pub fn flush_pendings_fonts_to_wr(wr: &mut WrCanvas) {
    let mut pendings_fonts = PENDING_FONTS.lock();
    pendings_fonts.iter().map(|ft| *ft).for_each(|mut ft| {
        let tpl = ft.font_template();
        let key = wr.wr_add_font(tpl);
        let glyph_size = ft.pixel_size;
        let glyph_size = EmacsLength::new(glyph_size as f32);
        let instance_key = wr.wr_add_font_instance(
            key,
            glyph_size,
            Some(FontInstanceOptions::default()),
            Some(FontInstancePlatformOptions::default()),
            Vec::new(),
        );
        let font_info = FontInfo::from_ptr_mut(ft.as_mut() as *mut FontInfo).unwrap();
        font_info.key = key;
        font_info.instance_key = instance_key;
    });
    pendings_fonts.clear();
}
