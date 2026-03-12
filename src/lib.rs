use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use image::ImageFormat;
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use resvg::render;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

pub fn svg_to_icon_data(
    svg_data: &str,
    sizes: &[(u32, &'static str)],
) -> io::Result<Vec<(Vec<u8>, &'static str)>> {
    let opt = Options::default();
    let tree = Tree::from_str(svg_data, &opt).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut icon_entries = Vec::new();

    for &(size, icon_type) in sizes {
        let mut pixmap = Pixmap::new(size, size)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create pixmap"))?;

        let scale = size as f32 / tree.size().width().max(tree.size().height());
        let transform = Transform::from_scale(scale, scale);

        render(&tree, transform, &mut pixmap.as_mut());

        let png_data = pixmap.encode_png().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        icon_entries.push((png_data, icon_type));
    }

    Ok(icon_entries)
}

pub fn create_icns(icon_entries: &[(Vec<u8>, &'static str)], output_path: &PathBuf) -> io::Result<()> {
    let mut icns_data = Vec::new();
    icns_data.extend(b"icns");

    let mut total_size = 8;
    for (data, _) in icon_entries {
        total_size += data.len() + 8;
    }

    icns_data.extend((total_size as u32).to_be_bytes());

    for (data, icon_type) in icon_entries {
        icns_data.extend(icon_type.as_bytes());
        icns_data.extend(((data.len() + 8) as u32).to_be_bytes());
        icns_data.extend(data);
    }

    let mut output = File::create(output_path)?;
    output.write_all(&icns_data)?;
    println!("Created {:?}", output_path);
    Ok(())
}

pub fn create_ico(icon_entries: &[(Vec<u8>, &'static str)], output_path: &PathBuf) -> io::Result<()> {
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    for (png_data, _) in icon_entries {
        let img = image::load_from_memory_with_format(png_data, ImageFormat::Png)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        if width > 256 || height > 256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Icon size {}x{} exceeds ICO maximum of 256x256", width, height),
            ));
        }

        let icon_img = IconImage::from_rgba_data(width, height, rgba.into_raw());
        let entry = IconDirEntry::encode(&icon_img)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        icon_dir.add_entry(entry);
    }

    let mut file = File::create(output_path)?;
    icon_dir.write(&mut file)?;
    println!("Created {:?}", output_path);
    Ok(())
}

pub fn create_pngs(
    icon_entries: &[(Vec<u8>, &'static str)],
    sizes: &[(u32, &'static str)],
    output_dir: &PathBuf,
) -> io::Result<()> {
    for ((png_data, _), &(size, _)) in icon_entries.iter().zip(sizes.iter()) {
        let output_png = output_dir.join(format!("icon_{}x{}.png", size, size));
        let mut file = File::create(&output_png)?;
        file.write_all(png_data)?;
        println!("Created {:?}", output_png);
    }
    Ok(())
}

pub fn create_png_512(svg_data: &str, output_path: &PathBuf) -> io::Result<()> {
    let opt = Options::default();
    let tree = Tree::from_str(svg_data, &opt).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut pixmap = Pixmap::new(512, 512)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create 512 pixmap"))?;

    let scale = 512.0 / tree.size().width().max(tree.size().height());
    let transform = Transform::from_scale(scale, scale);
    render(&tree, transform, &mut pixmap.as_mut());

    let png_data = pixmap.encode_png().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut file = File::create(output_path)?;
    file.write_all(&png_data)?;
    println!("Created {:?}", output_path);
    Ok(())
}