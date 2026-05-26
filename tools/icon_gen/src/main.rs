mod design;
mod png;

use std::fs;

/// Android mipmap densities: (folder_suffix, scale_factor)
const ANDROID_DENSITIES: &[(&str, u32)] = &[
    ("mdpi",    2),
    ("hdpi",    3),
    ("xhdpi",   4),
    ("xxhdpi",  6),
    ("xxxhdpi", 8),
];

fn main() {
    let out_dir = std::env::args().nth(1)
        .unwrap_or_else(|| "android/demo/src/main/res".to_string());

    let base = design::brick_canvas();

    for (density, scale) in ANDROID_DENSITIES {
        let dir = format!("{}/mipmap-{}", out_dir, density);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {dir}: {e}"));

        let canvas = base.scale(*scale);
        let png = canvas.encode_png();

        let path = format!("{}/ic_launcher.png", dir);
        fs::write(&path, &png).unwrap_or_else(|e| panic!("write {path}: {e}"));

        println!("  {}×{}  →  {}", canvas.width, canvas.height, path);
    }

    println!("Done. Add android:icon=\"@mipmap/ic_launcher\" to AndroidManifest.xml.");
}
