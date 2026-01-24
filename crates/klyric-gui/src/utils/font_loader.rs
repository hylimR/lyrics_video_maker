use std::path::PathBuf;
use walkdir::WalkDir;
use ttf_parser::Face;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct FontInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_chinese: bool,
}

/// Check if a font supports Chinese characters by looking for a common character '我' (U+6211)
fn supports_chinese(face: &Face) -> bool {
    face.glyph_index('我').is_some()
}

pub fn scan_system_fonts() -> Vec<FontInfo> {
    let mut fonts = Vec::new();
    let mut dirs = Vec::new();

    // Windows system fonts
    dirs.push(PathBuf::from("C:\\Windows\\Fonts"));

    // User local fonts (AppData)
    if let Some(user_font_dir) = dirs::font_dir() {
        dirs.push(user_font_dir);
    }
    
    // Also check standard local app data location just in case dirs crate misses it on some setups
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
         let local_fonts = PathBuf::from(local_app_data).join("Microsoft\\Windows\\Fonts");
         if local_fonts.exists() {
             dirs.push(local_fonts);
         }
    }

    log::info!("Scanning fonts in directories: {:?}", dirs);

    for dir in dirs {
        if !dir.exists() { continue; }
        
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_lowercase();
                if ext == "ttf" || ext == "otf" || ext == "ttc" {
                    if let Ok(data) = std::fs::read(path) {
                        // Parse the font (or first matching face in collection)
                        // Note: For TTC, we might want to iterate faces, but typically taking index 0 is enough for UI lists
                        // or we could iterate all faces. For now, let's keep it simple and take the first one or iterate standard logic.
                        
                        // We use index 0. If it's a collection, we might miss others, but usually main face is 0.
                        // Ideally we should check process_face for index 0..count.
                        
                        // Let's iterate up to a reasonable number of faces for TTC
                        let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
                        
                        for i in 0..count {
                            if let Ok(face) = Face::parse(&data, i) {
                                // Get family name
                                // Name ID 1 is Font Family, 4 is Full Name. 
                                // We prefer Family name for grouping, but for selection we need a unique name usually.
                                // For UI picking, Family Name is best.
                                
                                let mut name = None;
                                
                                // Try getting English name first (Language ID 1033)
                                for name_record in face.names() {
                                    if name_record.name_id == 1 && name_record.language_id == 1033 {
                                        name = name_record.to_string();
                                        break;
                                    }
                                }
                                
                                // Fallback to any name ID 1
                                if name.is_none() {
                                     for name_record in face.names() {
                                        if name_record.name_id == 1 {
                                            name = name_record.to_string();
                                            break;
                                        }
                                    }
                                }
                                
                                if let Some(n) = name {
                                    if n.starts_with(".") { continue; } // Skip system internal fonts
                                    
                                    let is_chinese = supports_chinese(&face);
                                    
                                    fonts.push(FontInfo {
                                        name: n,
                                        path: path.to_path_buf(),
                                        is_chinese,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Deduplicate by name
    fonts.sort_by(|a, b| a.name.cmp(&b.name));
    fonts.dedup_by(|a, b| a.name == b.name);
    
    log::info!("Found {} unique fonts", fonts.len());
    
    fonts
}
