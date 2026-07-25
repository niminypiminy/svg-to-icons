# svg-to-icons

A CLI + library to convert a single `icon.svg` into all common icon formats:  
**ICO** (Windows), **ICNS** (macOS), **PNG** sizes, **web/mobile**, **Android launcher sets**, and **social media** banners.

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
    ├── og-image.png
    └── android/
        ├── ic_launcher-playstore.png
        └── res/ …

## Android launcher icons

    cargo-svgtoicons --android --android-bg '#202225'

Produces a drop-in `res/` tree:

    icons/android/
    ├── ic_launcher-playstore.png          512×512, Play Console upload
    └── res/
        ├── mipmap-anydpi-v26/
        │   ├── ic_launcher.xml            adaptive icon
        │   └── ic_launcher_round.xml
        ├── mipmap-mdpi/                   108px foreground + monochrome, 48px legacy
        ├── mipmap-hdpi/                   162 / 72
        ├── mipmap-xhdpi/                  216 / 96
        ├── mipmap-xxhdpi/                 324 / 144
        ├── mipmap-xxxhdpi/                432 / 192
        └── values/ic_launcher_background.xml

Copy `res/` over `app/src/main/res/`. The Play Store image is a Console upload, not
an app resource.

**Give it a transparent-background SVG.** A launcher draws the foreground and the
background as separate layers and masks the foreground to its own shape, so an SVG
that bakes in its own plate comes out as a small tile floating on `--android-bg`.
Pass the plate colour instead — it defaults to white, because a background layer
that isn't opaque shows through as a hole.

### Safe zone

The adaptive canvas is 108dp, of which the launcher may crop the outer 18dp per
side for its mask and parallax. Only a centred **66dp circle** survives every OEM
mask, so `--android-scale` defaults to `0.611` (66/108).

That fraction is measured against the artwork's **ink, not its viewBox** — padding
baked into the SVG won't silently shrink your icon — and the artwork is centred on
its ink, so asymmetric padding won't push it off-centre under the mask.

Scale is applied to the artwork's longest side, which is right for a round or tall
mark but optimistic for one that fills its box: a square scaled to 66dp tall
reaches 93dp corner to corner. The tool measures the furthest painted pixel and
warns with a scale that would fit:

    Warning: at scale 0.611 this artwork needs a 93dp circle, larger than the 66dp
    one a launcher mask is guaranteed to leave alone. Some masks will clip its
    edges. Pass --android-scale 0.433 to fit it, or accept the crop if those edges
    are decorative.

### Themed icons

`ic_launcher_monochrome.png` is generated as a flat silhouette, not a copy of the
colour foreground. The API 33+ themed-icon slot tints whatever it is given to one
wallpaper-derived colour, so handing it multi-colour art collapses the palette into
a solid blob — a two-tone mark loses exactly the contrast that made it a mark. Only
alpha survives, so alpha is all this file carries.

### Vector alternative

This is a raster pipeline. If your mark is already a vector and you control the
project, a hand-written `VectorDrawable` foreground beats any PNG set: it scales
cleanly, drops the density buckets entirely, and at `minSdk 26+` lets you delete
the legacy `ic_launcher.png` files too, since `mipmap-anydpi` outranks every
density bucket. Use `--android` when you're starting from an arbitrary SVG or need
to support API 25 and below.

## Usage (as a library)

### Add to `Cargo.toml`

[dependencies]
svg-to-icons = "0.3.0"

---

## Example Code

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use svg_to_icons::{
        create_android_icons, create_icns, create_ico, create_pngs, create_png_512,
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

        // Android launcher set. None = white plate, None = default 66/108 safe-zone scale.
        create_android_icons(
            &svg_data,
            &output_dir,
            Some([32, 34, 37, 255]),  // Example plate: #202225
            None,                     // Example override: Some(0.55)
        )?;

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
