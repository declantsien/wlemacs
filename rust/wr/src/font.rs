use crate::types::{DeviceLength, GlyphSize, WrFontMetrics, WrFontTemplate, WrVecU8};
use webrender::api::units::LayoutToDeviceScale;
use webrender::api::{
    FontInstanceKey, FontInstanceOptions, FontInstancePlatformOptions, FontKey, FontTemplate,
    FontVariation, IdNamespace, NativeFontHandle,
};

use core_foundation::base::TCFType;
use core_text::font::{CTFont, CTFontRef};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;
use wr_glyph_rasterizer::{BaseFontInstance, FontInstance, GlyphRasterizer};

// pub mod platform {
//     #[cfg(any(target_os = "macos", target_os = "ios"))]
//     pub use super::platform::macos::font;
//     #[cfg(any(
//         target_os = "android",
//         all(unix, not(any(target_os = "ios", target_os = "macos")))
//     ))]
//     pub use super::platform::unix::font;
//     #[cfg(target_os = "windows")]
//     pub use super::platform::windows::font;

//     #[cfg(any(target_os = "ios", target_os = "macos"))]
//     pub mod macos {
//         pub mod font;
//     }
//     #[cfg(any(
//         target_os = "android",
//         all(unix, not(any(target_os = "macos", target_os = "ios")))
//     ))]
//     pub mod unix {
//         pub mod font;
//     }
//     #[cfg(target_os = "windows")]
//     pub mod windows {
//         pub mod font;
//     }
// }

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

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_add_ctfont(font: CTFontRef) {
    let ct_font = unsafe { CTFont::wrap_under_get_rule(font) };
    let point_size = ct_font.pt_size();
    let descriptor = ct_font.copy_descriptor();
    let font_path = descriptor.font_path();
    let mut font_metrics = WrFontMetrics::default();
    let font_tpl = FontTemplate::Native(NativeFontHandle {
        name: ct_font.postscript_name(),
        path: font_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("".to_string()),
    });
    wr_font_metrics_impl(
        font_tpl,
        point_size as ::libc::c_int,
        1.0,
        &mut font_metrics,
    );
    // println!("cf_name: {:?}, path: {:?}, size: {:?}",
    //     ct_font.postscript_name(), font_path, point_size);
}

/// cbindgen:ignore
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_font_metrics_impl(
    font_tpl: FontTemplate,
    glyph_size: ::libc::c_int,
    scale_factor: f64,
    font_metrics: &mut WrFontMetrics,
) {
    // let font_tpl = read_font_tpl(font_template, data, index);
    let mut rasterizer = WR_GLYPH_RASTERIZER.lock();
    let key = wr_font_key(font_tpl, Some(&mut rasterizer));
    let scale_factor = LayoutToDeviceScale::new(1.0 / (scale_factor as f32));
    let glyph_size: DeviceLength = DeviceLength::new(glyph_size as f32);
    let glyph_size = GlyphSize::from_layout_length(glyph_size / scale_factor);
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
                if font_metrics.min_width == 0 || font_metrics.min_width > this_width {
                    font_metrics.min_width = this_width;
                }
                if this_width > font_metrics.max_width {
                    font_metrics.max_width = this_width;
                }
                if c == 32 {
                    font_metrics.space_width = this_width;
                }
                font_metrics.average_width += this_width;
            }
            n += 1;
            // println!("ch: {ch:?}, dimensions: {dimensions:?}");
        }
    });

    font_metrics.average_width /= n;

    // if f.is_wr_initialized() {
    //     let wr = f.webrender();
    //     let font_key = wr.wr_add_font(tpl);
    //     return f.webrender().get_font_ft_face(font_key);
    // }

    // let worker = ThreadPoolBuilder::new()
    //     .thread_name(|idx|{ format!("WRWorker#{}", idx) })
    //     .build();
    // let workers = std::sync::Arc::new(worker.unwrap());
    // let mut glyph_rasterizer = GlyphRasterizer::new(workers, None, true);
    // let font_key = FontKey::default();
    // glyph_rasterizer.add_font(font_key, tpl);
    // glyph_rasterizer.get_font_ft_face(font_key);
}

// fn wr_to_emacs (d: GlyphDimensions) -> emacs_sys::bindings::font_metrics {

//     emacs_sys::bindings::font_metrics {
//         // TBD check https://freetype.org/freetype2/docs/glyphs/glyphs-3.html and wr impl details
//         // lbearing: d.left, // floor
//         // rbearing: d.width + d.left, // ceil
//         width: d.advance, // lround
//         // The subtraction of a small number is to avoid rounding up due
// 	//  to floating-point inaccuracies with some fonts, which then
// 	//  could cause unpleasant effects while scrolling (see bug
// 	//  #44284), since we then think that a glyph row's ascent is too
// 	//  small to accommodate a glyph with a higher phys_ascent.
//         // ascent: top - height, //ceil
//         // descent: top,
//     }
// }
