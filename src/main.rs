use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use svg_to_icons::{create_icns, create_ico, create_pngs, create_png_512, svg_to_icon_data};

#[derive(Parser, Debug)]
#[command(version, about = "Convert SVG to icons (ICO, ICNS, PNGs).")]
struct Args {
    /// Path to the input SVG file (default: ./icon.svg)
    #[arg(short, long, default_value = "icon.svg")]
    svg: PathBuf,

    /// Output directory for generated files (default: ./icons)
    #[arg(short, long, default_value = "icons")]
    output_dir: PathBuf,

    /// Generate ICNS (for macOS)
    #[arg(long)]
    icns: bool,

    /// Generate ICO (for Windows)
    #[arg(long)]
    ico: bool,

    /// Generate individual PNGs
    #[arg(long)]
    pngs: bool,

    /// Generate 512x512 clean PNG
    #[arg(long)]
    png_512: bool,

    /// Generate all formats (overrides individual flags)
    #[arg(short, long)]
    all: bool,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    std::fs::create_dir_all(&args.output_dir)?;

    let mut svg_data = String::new();
    let mut svg_file = std::fs::File::open(&args.svg)?;
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

    let generate_all = args.all;
    let generate_icns = generate_all || args.icns;
    let generate_ico = generate_all || args.ico;
    let generate_pngs = generate_all || args.pngs;
    let generate_png_512 = generate_all || args.png_512;

    if !(generate_icns || generate_ico || generate_pngs || generate_png_512) {
        println!("No output formats specified. Use --all or specific flags (e.g., --icns).");
        return Ok(());
    }

    if generate_icns {
        let output_icns = args.output_dir.join("icon.icns");
        create_icns(&icon_entries, &output_icns)?;
    }

    if generate_ico {
        let output_ico = args.output_dir.join("icon.ico");
        create_ico(&icon_entries, &output_ico)?;
    }

    if generate_pngs {
        create_pngs(&icon_entries, &icon_sizes, &args.output_dir)?;
    }

    if generate_png_512 {
        let output_512 = args.output_dir.join("icon-512.png");
        create_png_512(&svg_data, &output_512)?;
    }

    Ok(())
}