# svg-to-icons

A CLI + library to convert a single `icon.svg` into all common icon formats:  
**ICO** (Windows), **ICNS** (macOS), **PNG** sizes, **web/mobile**, and **social media** banners.

## Install

cargo install svg-to-icons

## CLI Usage

1. Place your `icon.svg` in your project root.
2. Run:

cargo-svgtoicons --all 

icons/
├── icon.ico
├── icon.icns
├── icon-512.png
├── icon_16x16.png
├── icon_32x32.png
├── icon_48x48.png
├── icon_64x64.png
├── icon_128x128.png
├── icon_256x256.png
├── icon_512x512.png
├── icon_1024x1024.png
├── apple-touch-icon.png
├── android-chrome-192.png
├── android-chrome-512.png
└── og-image.png

## Usage (as a library)

### Add to `Cargo.toml`

[dependencies]
svg-to-icons = "0.2.0"

---

## Example Code

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use svg_to_icons::{
    create_icns, create_ico, create_pngs, create_png_512,
    create_web_targets, create_social_media_png, svg_to_icon_data,
};

fn main() -> std::io::Result<()> {
    let mut svg_data = String::new();
    File::open("icon.svg")?.read_to_string(&mut svg_data)?;

    let output_dir = PathBuf::from("icons");
    std::fs::create_dir_all(&output_dir)?;

    let icon_sizes = [
        (16, "is32"), (32, "il32"), (48, "ih32"), (64, "ih32"),
        (128, "it32"), (256, "ic08"), (512, "ic09"), (1024, "ic10"),
    ];

    let icon_entries = svg_to_icon_data(&svg_data, &icon_sizes)?;

    // Desktop icons
    create_icns(&icon_entries, &output_dir.join("icon.icns"))?;
    create_ico(&icon_entries, &output_dir.join("icon.ico"))?;  
    create_pngs(&icon_entries, &icon_sizes, &output_dir)?;

    // 512×512 PNG
    create_png_512(&svg_data, &output_dir.join("icon-512.png"))?;

    // Web / mobile targets
    create_web_targets(&svg_data, &output_dir)?;

    // Social media banner is transparent by default, but one may choose a background color. 
    create_social_media_png(
        &svg_data,
        &output_dir.join("og-image.png"),
        1200,
        630,
        None,                        // None = transparent
        // Some([51, 65, 85, 255])   // Example: #334155
    )?;

    println!("All icons generated successfully!");
    Ok(())
}