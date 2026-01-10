use tiny_skia::Color;

pub fn parse_color(hex: &str) -> Option<Color> {
    // Basic hex support #RRGGBBAA or #RRGGBB
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        (r, g, b, 255)
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        (r, g, b, a)
    } else {
        return None;
    };
    
    // tiny_skia::Color::from_rgba8 returns Color directly
    Some(Color::from_rgba8(r, g, b, a))
}

pub fn parse_percentage(s: &str) -> f32 {
    s.trim_end_matches('%').parse::<f32>().unwrap_or(50.0) / 100.0
}
