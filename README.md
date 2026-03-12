# svg-to-icons

## Install

```bash
cargo install svg-to-icons
```

## Usage (CLI)

1. Place your `icon.svg` in your project root.
2. Run:

```bash
cargo-svgtoicons --svg icon.svg --all
```

This generates:

- `ico`
- `icns`
- `png` sizes: `16`, `32`, `48`, `64`, `128`, `256`, `512`

---

## Usage (as a library)

### Add to `Cargo.toml`

```toml
[dependencies]
svg-to-icons = "0.1.2"
```

---

## Example Code

```rust
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use svg_to_icons::{create_icns, create_ico, create_pngs, create_png_512, svg_to_icon_data};

fn main() -> std::io::Result<()> {
    let mut svg_data = String::new();
    let mut svg_file = File::open("icon.svg")?;
    svg_file.read_to_string(&mut svg_data)?;

    let icon_sizes = [
        (16, "is32"),
        (32, "il32"),
        (48, "ih32"),
        (64, "ih32"),
        (128, "it32"),
        (256, "ic08"),
    ];

    let icon_entries = svg_to_icon_data(&svg_data, &icon_sizes)?;

    let output_dir = PathBuf::from("icons");

    std::fs::create_dir_all(&output_dir)?;

    let output_icns = output_dir.join("icon.icns");
    create_icns(&icon_entries, &output_icns)?;

    let output_ico = output_dir.join("icon.ico");
    create_ico(&icon_entries, &output_ico)?;

    create_pngs(&icon_entries, &icon_sizes, &output_dir)?;

    let output_512 = output_dir.join("icon-512.png");
    create_png_512(&svg_data, &output_512)?;

    Ok(())
}
```