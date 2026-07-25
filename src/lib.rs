use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use image::ImageFormat;
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use resvg::render;
use resvg::tiny_skia::{Color, Paint, Pixmap, PixmapPaint, PremultipliedColorU8, Rect, Transform};
use resvg::usvg::{Options, Tree};

/// Android's density buckets and their scale factor against mdpi (160 dpi), which
/// is the 1× baseline where one dp is one pixel.
const ANDROID_DENSITIES: [(&str, f32); 5] = [
    ("mdpi", 1.0),
    ("hdpi", 1.5),
    ("xhdpi", 2.0),
    ("xxhdpi", 3.0),
    ("xxxhdpi", 4.0),
];

/// Edge of the adaptive icon canvas, in dp. Fixed by the platform.
const ANDROID_ADAPTIVE_DP: f32 = 108.0;

/// Edge of the legacy (pre-API-26) square launcher icon, in dp.
const ANDROID_LEGACY_DP: f32 = 48.0;

/// Fraction of the adaptive canvas the artwork should fill by default.
///
/// The launcher may crop the outer 18dp on each side of the 108dp canvas for its
/// mask and for parallax, leaving 72dp visible; only a centred 66dp circle is
/// guaranteed to survive every OEM mask. 66/108 keeps the artwork inside it.
const ANDROID_SAFE_ZONE: f32 = 66.0 / 108.0;

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

    let mut valid_count = 0;

    for (png_data, _) in icon_entries {
        let img = image::load_from_memory_with_format(png_data, ImageFormat::Png)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // ICO format maximum size is 256x256
        if width > 256 || height > 256 {
            continue; // Skip large sizes as they're for ICNS and high-res PNGs)
        }

        let icon_img = IconImage::from_rgba_data(width, height, rgba.into_raw());
        let entry = IconDirEntry::encode(&icon_img)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        icon_dir.add_entry(entry);
        valid_count += 1;
    }

    if valid_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No valid icon sizes (≤ 256x256) were found for ICO generation",
        ));
    }

    let mut file = File::create(output_path)?;
    icon_dir.write(&mut file)?;
    println!("Created {:?} ({} sizes: 16–256 px)", output_path, valid_count);
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
    bg_color: Option<[u8; 4]>,  // None = transparent (new default)
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

    // Only fill a background if the user actually requested a color
    if let Some(color) = bg_color {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));
        
        canvas.fill_rect(
            Rect::from_xywh(0.0, 0.0, canvas_width as f32, canvas_height as f32).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    }

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

/// What an SVG actually paints, in the tree's own coordinate space.
struct Ink {
    /// Tightest rect containing every painted pixel.
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// Distance from the centre of that rect to the furthest painted pixel.
    ///
    /// This is what a circular mask actually tests, and it is not the rect's
    /// half-diagonal: a ring, a lens, a monogram all leave their bounding corners
    /// empty, so measuring the box would condemn artwork that fits comfortably.
    radius: f32,
}

/// Measure `tree`'s ink by rendering a probe and scanning its alpha.
///
/// Done by rendering rather than by walking the node tree: a box taken from path
/// geometry alone misses half a stroke's width on every edge, and misses filter
/// bleed and clipping entirely. Probe error is one part in `probe`, far below a
/// pixel at any density we emit.
fn ink_bounds(tree: &Tree, probe: u32) -> io::Result<Ink> {
    let mut pixmap = Pixmap::new(probe, probe)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create probe pixmap"))?;

    let longest = tree.size().width().max(tree.size().height());
    let scale = probe as f32 / longest;
    render(tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    let (mut min_x, mut min_y) = (u32::MAX, u32::MAX);
    let (mut max_x, mut max_y) = (0u32, 0u32);
    for (i, px) in pixmap.pixels().iter().enumerate() {
        if px.alpha() == 0 {
            continue;
        }
        let (x, y) = (i as u32 % probe, i as u32 / probe);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if min_x == u32::MAX {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SVG paints nothing — cannot fit empty artwork to the icon canvas",
        ));
    }

    // Second pass, now that the centre is known.
    let centre_x = (min_x + max_x) as f32 / 2.0;
    let centre_y = (min_y + max_y) as f32 / 2.0;
    let mut radius = 0.0f32;
    for (i, px) in pixmap.pixels().iter().enumerate() {
        if px.alpha() == 0 {
            continue;
        }
        let (x, y) = (i as u32 % probe, i as u32 / probe);
        radius = radius.max((x as f32 - centre_x).hypot(y as f32 - centre_y));
    }

    Ok(Ink {
        x: min_x as f32 / scale,
        y: min_y as f32 / scale,
        width: (max_x - min_x + 1) as f32 / scale,
        height: (max_y - min_y + 1) as f32 / scale,
        radius: radius / scale,
    })
}

/// Render `tree` onto a transparent `canvas`×`canvas` pixmap.
///
/// `fill` picks between the two genuinely different intents an icon canvas has:
///
/// - `None` draws the SVG as authored, viewBox fitted edge to edge. Whatever
///   padding the author chose is the icon's padding. This is what an unmasked
///   target — a legacy launcher icon, a store listing — should get.
/// - `Some(f)` scales and centres the *artwork* so its longest side covers `f` of
///   the canvas, ignoring viewBox padding. This is what a masked target needs,
///   because the safe zone is a promise about ink, not about coordinate space.
fn render_fitted(tree: &Tree, canvas: u32, fill: Option<f32>) -> io::Result<Pixmap> {
    let mut pixmap = Pixmap::new(canvas, canvas)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create pixmap"))?;

    let transform = match fill {
        None => {
            let longest = tree.size().width().max(tree.size().height());
            let scale = canvas as f32 / longest;
            let dx = (canvas as f32 - tree.size().width() * scale) / 2.0;
            let dy = (canvas as f32 - tree.size().height() * scale) / 2.0;
            Transform::from_translate(dx, dy).pre_scale(scale, scale)
        }
        Some(f) => {
            let ink = ink_bounds(tree, 512)?;
            let scale = (canvas as f32 * f) / ink.width.max(ink.height);
            // Centre the ink, not the viewBox — asymmetric padding would otherwise
            // push the artwork off-centre under the mask.
            let dx = canvas as f32 / 2.0 - (ink.x + ink.width / 2.0) * scale;
            let dy = canvas as f32 / 2.0 - (ink.y + ink.height / 2.0) * scale;
            Transform::from_translate(dx, dy).pre_scale(scale, scale)
        }
    };

    render(tree, transform, &mut pixmap.as_mut());

    Ok(pixmap)
}

/// Flatten a rendered pixmap to a white silhouette, keeping its alpha.
///
/// The themed-icon slot tints whatever it is given to a single wallpaper-derived
/// colour, so only the alpha channel carries any information. Collapsing the
/// colours here rather than shipping the full-colour art means what you see in
/// the generated file is what the launcher will draw.
fn to_silhouette(pixmap: &mut Pixmap) {
    for px in pixmap.pixels_mut() {
        // Pixels are premultiplied, so opaque white at alpha `a` is (a, a, a, a).
        let a = px.alpha();
        if let Some(white) = PremultipliedColorU8::from_rgba(a, a, a, a) {
            *px = white;
        }
    }
}

fn write_pixmap(pixmap: &Pixmap, path: &PathBuf) -> io::Result<()> {
    let data = pixmap
        .encode_png()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    File::create(path)?.write_all(&data)?;
    println!("Created {:?}", path);
    Ok(())
}

fn write_text(contents: &str, path: &PathBuf) -> io::Result<()> {
    File::create(path)?.write_all(contents.as_bytes())?;
    println!("Created {:?}", path);
    Ok(())
}

/// Format an RGBA quad the way an Android colour resource wants it: `#RRGGBB`
/// when fully opaque, `#AARRGGBB` otherwise (alpha leads, unlike CSS).
fn android_hex(color: [u8; 4]) -> String {
    let [r, g, b, a] = color;
    if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b)
    }
}

/// Generate a complete Android launcher icon set.
///
/// Writes a `res/` tree under `<output_dir>/android/`, laid out so it can be
/// copied straight over `app/src/main/res/`:
///
/// ```text
/// android/res/
/// ├── mipmap-anydpi-v26/ic_launcher.xml, ic_launcher_round.xml
/// ├── mipmap-{m,h,x,xx,xxx}dpi/
/// │     ic_launcher.png             legacy square icon, 48dp
/// │     ic_launcher_foreground.png  adaptive foreground, 108dp
/// │     ic_launcher_monochrome.png  themed-icon silhouette, 108dp
/// ├── values/ic_launcher_background.xml
/// └── ic_launcher-playstore.png     512×512, for the Play Console listing
/// ```
///
/// Sizing: the adaptive layers are drawn on a 108dp canvas of which the launcher
/// may crop the outer 18dp per side, so `foreground_scale` — the fraction of the
/// canvas the artwork fills — defaults to 66/108, keeping it inside the centred
/// 66dp circle that every OEM mask leaves alone. That fraction is measured against
/// the artwork's ink and not its viewBox, so padding baked into the SVG does not
/// silently shrink the icon; the artwork is centred on its ink for the same
/// reason. The legacy PNGs are the whole SVG at 48dp, viewBox and all, because
/// pre-API-26 launchers draw them unmasked and as authored.
///
/// Supply a **transparent-background** SVG. The foreground is masked to the
/// launcher's shape and `bg_color` is painted behind it, so an SVG that bakes in
/// its own plate renders as a small tile floating on that colour. Pass the plate
/// as `bg_color` instead; it defaults to white, since an adaptive background that
/// is not opaque shows through as a hole.
///
/// The Play Store image is deliberately the full SVG at 512×512 rather than the
/// safe-zone crop: Play renders it unmasked, and Console rejects transparency, so
/// it is composited onto `bg_color`.
pub fn create_android_icons(
    svg_data: &str,
    output_dir: &PathBuf,
    bg_color: Option<[u8; 4]>,
    foreground_scale: Option<f32>,
) -> io::Result<()> {
    let opt = Options::default();
    let tree =
        Tree::from_str(svg_data, &opt).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let background = bg_color.unwrap_or([255, 255, 255, 255]);
    let fill = foreground_scale.unwrap_or(ANDROID_SAFE_ZONE);
    if !(fill > 0.0 && fill <= 1.0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "foreground_scale must be greater than 0 and at most 1",
        ));
    }

    // `fill` sizes the artwork by its longest side, which is right for a round or
    // tall mark but optimistic for one that fills its box: scaled so its height is
    // 66dp, a square's corners sit 93dp apart and the mask eats them. Check the
    // furthest ink rather than shipping a quietly clipped icon.
    let ink = ink_bounds(&tree, 512)?;
    let spread = 2.0 * ink.radius * fill * ANDROID_ADAPTIVE_DP / ink.width.max(ink.height);
    let safe_dp = ANDROID_SAFE_ZONE * ANDROID_ADAPTIVE_DP;
    if spread > safe_dp {
        eprintln!(
            "Warning: at scale {:.3} this artwork needs a {:.0}dp circle, larger than the {:.0}dp \
             one a launcher mask is guaranteed to leave alone. Some masks will clip its edges. \
             Pass --android-scale {:.3} to fit it, or accept the crop if those edges are decorative.",
            fill,
            spread,
            safe_dp,
            fill * safe_dp / spread,
        );
    }

    let res_dir = output_dir.join("android").join("res");

    for (bucket, density) in ANDROID_DENSITIES {
        let bucket_dir = res_dir.join(format!("mipmap-{}", bucket));
        std::fs::create_dir_all(&bucket_dir)?;

        let adaptive_px = (ANDROID_ADAPTIVE_DP * density).round() as u32;
        let legacy_px = (ANDROID_LEGACY_DP * density).round() as u32;

        let foreground = render_fitted(&tree, adaptive_px, Some(fill))?;
        write_pixmap(&foreground, &bucket_dir.join("ic_launcher_foreground.png"))?;

        let mut monochrome = foreground;
        to_silhouette(&mut monochrome);
        write_pixmap(&monochrome, &bucket_dir.join("ic_launcher_monochrome.png"))?;

        // Legacy icons are drawn unmasked, so they get the SVG as authored.
        let legacy = render_fitted(&tree, legacy_px, None)?;
        write_pixmap(&legacy, &bucket_dir.join("ic_launcher.png"))?;
    }

    let anydpi_dir = res_dir.join("mipmap-anydpi-v26");
    std::fs::create_dir_all(&anydpi_dir)?;

    // Both entries carry the same layers. `ic_launcher_round` is vestigial under
    // adaptive icons — the launcher masks the same art either way — but manifests
    // still reference it via android:roundIcon, so emitting it avoids a dangling
    // resource on projects that have not dropped the attribute.
    let adaptive_xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">
    <background android:drawable=\"@color/ic_launcher_background\" />
    <foreground android:drawable=\"@mipmap/ic_launcher_foreground\" />
    <monochrome android:drawable=\"@mipmap/ic_launcher_monochrome\" />
</adaptive-icon>
";
    write_text(adaptive_xml, &anydpi_dir.join("ic_launcher.xml"))?;
    write_text(adaptive_xml, &anydpi_dir.join("ic_launcher_round.xml"))?;

    let values_dir = res_dir.join("values");
    std::fs::create_dir_all(&values_dir)?;
    write_text(
        &format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <resources>\n    \
             <color name=\"ic_launcher_background\">{}</color>\n\
             </resources>\n",
            android_hex(background)
        ),
        &values_dir.join("ic_launcher_background.xml"),
    )?;

    // Play Console rejects an icon with an alpha channel, so this one is flattened
    // onto the background colour instead of being left transparent.
    let mut playstore = Pixmap::new(512, 512)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create 512 pixmap"))?;
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(
        background[0],
        background[1],
        background[2],
        255,
    ));
    playstore.fill_rect(
        Rect::from_xywh(0.0, 0.0, 512.0, 512.0).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
    let art = render_fitted(&tree, 512, None)?;
    playstore.draw_pixmap(
        0,
        0,
        art.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    write_pixmap(
        &playstore,
        &output_dir.join("android").join("ic_launcher-playstore.png"),
    )?;

    println!(
        "Android set ready: copy {:?} over app/src/main/res/ (ic_launcher-playstore.png is a Console upload, not an app resource)",
        res_dir
    );

    Ok(())
}