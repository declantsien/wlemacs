use crate::types::{font, frame, EmacsLength, EmacsToLayoutScale, GlyphSize};
use webrender::api::{
    FontInstanceKey, FontInstanceOptions, FontInstancePlatformOptions, FontKey, FontTemplate,
    FontVariation, IdNamespace,
};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;
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

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_prepare_font(f: *mut frame, font: *mut font) {
    let font = font::from_ptr_mut(font).unwrap();
    let f = frame::from_ptr(f).unwrap();
    let font_tpl = font.font_template();
    let mut rasterizer = WR_GLYPH_RASTERIZER.lock();

    let key = wr_font_key(font_tpl, Some(&mut rasterizer));
    let scale_factor = EmacsToLayoutScale::new(f.scale_factor() as f32);

    let glyph_size: EmacsLength = EmacsLength::new(font.pixel_size as f32);
    let glyph_size = GlyphSize::from_layout_length(glyph_size * scale_factor);
    let instance = wr_font_instance(key, glyph_size, Some(&mut rasterizer));

    let mut n = 0;
    (32..127).for_each(|c| {
        let ch = char::from_u32(c).unwrap();
        /* In order to simulate the Xft behavior, we use metrics of
        glyph ID 0 if there is no glyph for an ASCII printable.  */
        let glyph_index = rasterizer.get_glyph_index(key, ch).unwrap_or(0);
        // None when FT_Glyph_Format is not FT_GLYPH_FORMAT_OUTLINE and FT_GLYPH_FORMAT_BITMAP
        if let Some(dimensions) = rasterizer.get_glyph_dimensions(&instance, glyph_index) {
            let this_width = dimensions.advance as i32;
            if this_width > 0 {
                if font.min_width == 0 || font.min_width > this_width {
                    font.min_width = this_width;
                }
                if this_width > font.max_width {
                    font.max_width = this_width;
                }
                if c == 32 {
                    font.space_width = this_width;
                }
                font.average_width += this_width;
            }
            n += 1;
            // println!("ch: {ch:?}, dimensions: {dimensions:?}");
        }
    });

    font.average_width /= n;
}
