use cfg_aliases::cfg_aliases;

use anyhow::Context;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const RGB_TXT_PATH: &str = "../../etc/rgb.txt";

fn main() -> anyhow::Result<()> {
    generate_color_map()?;

    // Setup cfg aliases
    cfg_aliases! {
        android_platform: { target_os = "android" },
        wasm_platform: { target_arch = "wasm32" },
        macos_platform: { target_os = "macos" },
        ios_platform: { target_os = "ios" },
        windows_platform: { target_os = "windows" },
        apple: { any(target_os = "ios", target_os = "macos") },
        free_unix: { all(unix, not(apple), not(android_platform)) },

        x11_platform: { all(feature = "x11", free_unix, not(wasm))},
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm)) },
    }
    Ok(())
}

fn generate_color_map() -> anyhow::Result<()> {
    let file = BufReader::new(File::open(RGB_TXT_PATH)?);
    let color = file
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let result = line
                .trim()
                .split("\t\t")
                .map(|str| str.to_owned())
                .collect::<Vec<String>>();

            let color = result[0]
                .split_whitespace()
                .map(|str| str.to_owned())
                .collect::<Vec<String>>();

            let name = result[1].trim().to_lowercase();

            let red = color[0].clone();
            let green = color[1].clone();
            let blue = color[2].clone();

            (name, (red, green, blue))
        });

    let out_dir = env::var_os("OUT_DIR").context("OUT_DIR var error")?;
    let out_path = Path::new(&out_dir).join("colors.rs");

    let color_function_body = format!(
        "let mut color_map: HashMap<&'static str, (u8, u8, u8)> = HashMap::new(); {} color_map",
        color
            .map(|(name, (red, green, blue))| format!(
                "color_map.insert(\"{}\", ({}, {}, {}));\n",
                name, red, green, blue
            ))
            .collect::<Vec<String>>()
            .concat()
    );

    let color_fun_source = format!(
        "fn init_color() -> HashMap<&'static str, (u8, u8, u8)> {{ {} }}",
        color_function_body
    );

    let mut file = File::create(out_path).context("file create error")?;
    file.write_all(color_fun_source.as_bytes())
        .context("write all error")?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", RGB_TXT_PATH);
    Ok(())
}
