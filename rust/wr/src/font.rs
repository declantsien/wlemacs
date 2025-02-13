pub mod platform {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::platform::macos::font;
    #[cfg(any(
        target_os = "android",
        all(unix, not(any(target_os = "ios", target_os = "macos")))
    ))]
    pub use super::platform::unix::font;
    #[cfg(target_os = "windows")]
    pub use super::platform::windows::font;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub mod macos {
        pub mod font;
    }
    #[cfg(any(
        target_os = "android",
        all(unix, not(any(target_os = "macos", target_os = "ios")))
    ))]
    pub mod unix {
        pub mod font;
    }
    #[cfg(target_os = "windows")]
    pub mod windows {
        pub mod font;
    }
}

use crate::frame::FrameExtWrCommon;
use emacs_sys::bindings::font_info;
use emacs_sys::font::FontRef;
use emacs_sys::lisp::ExternalPtr;
use webrender::api::FontInstanceKey;
use webrender::api::GlyphDimensions;
use webrender::api::GlyphIndex;

use emacs_sys::frame::FrameRef;

pub type FontInfoRef = ExternalPtr<font_info>;

use emacs_sys::bindings::font;
use emacs_sys::bindings::font_property_index;
use emacs_sys::bindings::frame;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::LazyLock;
use webrender::api::units::*;
use webrender::api::*;
use wr_glyph_rasterizer::BaseFontInstance;
use wr_glyph_rasterizer::FontInstance;
use wr_glyph_rasterizer::GlyphRasterizer;

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
                FontSize,
                Option<FontInstanceOptions>,
                Option<FontInstancePlatformOptions>,
                Vec<FontVariation>,
            ),
            FontInstance,
        >,
    >,
> = LazyLock::new(Default::default);

const namespace: IdNamespace = IdNamespace(1);

pub fn wr_font_tpl(font: *mut font) -> FontTemplate {
    let mut font = FontRef::new(font);
    let font_info = FontInfoRef::new(font.as_mut() as *mut font_info);
    let filename = font.props[font_property_index::FONT_FILE_INDEX as usize];

    let path = std::path::PathBuf::from(String::from(filename));
    let index = font_info.index as u32;
    FontTemplate::Native(NativeFontHandle { path, index })
}

fn wr_font_key(font: *mut font, rasterizer: Option<&mut GlyphRasterizer>) -> FontKey {
    let font_tpl = wr_font_tpl(font);
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
    glyph_size: FontSize,
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
            glyph_size.to_f32_px(),
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
pub extern "C" fn wr_prepara_font(f: *mut frame, font: *mut font) {
    let font_tpl = wr_font_tpl(font);
    // println!("font template: {:?}", font_tpl);
    let f = FrameRef::new(f);
    let mut font = FontRef::new(font);
    let font_info = FontInfoRef::new(font.as_mut() as *mut font_info);

    let mut rasterizer = WR_GLYPH_RASTERIZER.lock();
    let key = wr_font_key(font.as_mut(), Some(&mut rasterizer));
    let scale = f.scale_factor() as f32;
    let glyph_size = FontSize::from_f64_px(font.pixel_size as f64 * scale as f64);
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
            println!("ch: {ch:?}, dimensions: {dimensions:?}");
        }
    });
    font.average_width /= n;

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
