use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use image::ImageFormat;
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use resvg::render;
use resvg::tiny_skia::{Color, Paint, Pixmap, PixmapPaint, Rect, Transform};
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

pub fn create_web_targets(svg_data: &str, output_dir: &PathBuf) -> io::Result<()> {
    let web_targets = [
        (180, "apple-touch-icon.png"),
        (192, "android-chrome-192.png"),
        (512, "android-chrome-512.png"),
    ];

    for (size, filename) in web_targets {
        // Reuse svg_to_icon_data to generate the raw png data
        let entry = svg_to_icon_data(svg_data, &[(size, "")])?;
        let output_path = output_dir.join(filename);
        let mut file = File::create(&output_path)?;
        file.write_all(&entry[0].0)?;
        println!("Created {:?}", output_path);
    }
    
    Ok(())
}

pub fn create_social_media_png(
    svg_data: &str, 
    output_path: &PathBuf, 
    canvas_width: u32, 
    canvas_height: u32,
    bg_color: [u8; 4] // [R, G, B, A] parameter
) -> io::Result<()> {
    let opt = Options::default();
    let tree = Tree::from_str(svg_data, &opt).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let logo_size = 512;
    let mut logo_pixmap = Pixmap::new(logo_size, logo_size).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Failed to create logo pixmap")
    })?;

    let scale = logo_size as f32 / tree.size().width().max(tree.size().height());
    let transform = Transform::from_scale(scale, scale);
    render(&tree, transform, &mut logo_pixmap.as_mut());

    let mut canvas = Pixmap::new(canvas_width, canvas_height).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Failed to create canvas pixmap")
    })?;

    // NEW: Apply the custom color here
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(bg_color[0], bg_color[1], bg_color[2], bg_color[3]));
    
    canvas.fill_rect(
        Rect::from_xywh(0.0, 0.0, canvas_width as f32, canvas_height as f32).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );

    let x_offset = ((canvas_width - logo_size) / 2) as i32;
    let y_offset = ((canvas_height - logo_size) / 2) as i32;

    canvas.draw_pixmap(
        x_offset,
        y_offset,
        logo_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    let png_data = canvas.encode_png().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut file = File::create(output_path)?;
    file.write_all(&png_data)?;
    println!("Created {:?}", output_path);
    
    Ok(())
}
