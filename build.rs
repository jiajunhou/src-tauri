use std::fs;
use std::path::PathBuf;

fn ensure_icon() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let icons_dir = PathBuf::from(manifest_dir.clone()).join("icons");
    let icon_path = icons_dir.join("icon.ico");

    if !icon_path.exists() {
        let _ = fs::create_dir_all(&icons_dir);
        let size = 32u32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let idx = ((y * size + x) * 4) as usize;
                rgba[idx] = 0x00;       // R
                rgba[idx + 1] = 0x7a;   // G
                rgba[idx + 2] = 0xcc;   // B (accent blue)
                rgba[idx + 3] = 0xff;   // A
            }
        }

        let image = ico::IconImage::from_rgba_data(size, size, rgba);
        let entry = ico::IconDirEntry::encode(&image).expect("encode icon");
        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
        icon_dir.add_entry(entry);
        let mut file = fs::File::create(&icon_path).expect("create icon file");
        icon_dir.write(&mut file).expect("write icon file");
    }
}

fn main() {
    ensure_icon();
    tauri_build::build()
}