use crate::canvas::WrCanvas;
use crate::types::{font, frame, EmacsLength, EmacsToLayoutScale, GlyphSize};
use webrender::api::{
    FontInstanceKey, FontInstanceOptions, FontInstancePlatformOptions, FontKey, FontTemplate,
    FontVariation, IdNamespace,
};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;
use webrender_api::{GlyphDimensions, GlyphIndex};
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

pub fn with_get_glyph_dimension<F>(font: &mut font, fr: *mut frame, f: F)
where
    F: Fn(&mut font, &mut dyn FnMut(char) -> Option<GlyphDimensions>),
{
    if let Some(wr) = frame::from_ptr(fr).and_then(|f| f.renderer_mut()) {
        let font_tpl = font.font_template();
        let key = wr.wr_add_font(font_tpl);
        let glyph_size = EmacsLength::new(font.pixel_size as f32);
        let instance_key = wr.wr_add_font_instance(
            key,
            glyph_size,
            Some(FontInstanceOptions::default()),
            Some(FontInstancePlatformOptions::default()),
            Vec::new(),
        );
        let font_info = font.font_info_mut().unwrap();
        font_info.f = fr;
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
        f(font, &mut get_glyph_dimension);
    } else {
        let font_tpl = font.font_template();
        let mut rasterizer = WR_GLYPH_RASTERIZER.lock();

        let key = wr_font_key(font_tpl, Some(&mut rasterizer));
        let scale_factor =
            EmacsToLayoutScale::new(frame::from_ptr(fr).unwrap().scale_factor() as f32);

        let glyph_size: EmacsLength = EmacsLength::new(font.pixel_size as f32);
        let glyph_size = GlyphSize::from_layout_length(glyph_size * scale_factor);
        let instance = wr_font_instance(key, glyph_size, Some(&mut rasterizer));

        let get_glyph_dimension = Box::new(|ch: char| -> Option<GlyphDimensions> {
            let index = rasterizer.get_glyph_index(key, ch);
            /* In order to simulate the Xft behavior, we use metrics of
            glyph ID 0 if there is no glyph for an ASCII printable.  */
            let index = index.unwrap_or(0);
            rasterizer.get_glyph_dimensions(&instance, index)
        });
        f(font, &mut Box::new(get_glyph_dimension));
    };
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_prepare_font(f: *mut frame, font: *mut font) {
    with_get_glyph_dimension(
        font::from_ptr_mut(font).unwrap(),
        f,
        |ft, get_glyph_dimension| {
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
