pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8, u8)> {
    let bytes = hex.as_bytes();
    let mut i = 0;

    // Skip optional '#'
    if i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }

    let len = bytes.len() - i;

    // Helper to parse 2 hex chars
    let parse_byte = |idx: usize| -> Option<u8> {
        if idx + 1 >= bytes.len() {
            return None;
        }
        let h = match bytes[idx] {
             b'0'..=b'9' => bytes[idx] - b'0',
             b'a'..=b'f' => bytes[idx] - b'a' + 10,
             b'A'..=b'F' => bytes[idx] - b'A' + 10,
             _ => return None,
        };
        let l = match bytes[idx + 1] {
             b'0'..=b'9' => bytes[idx + 1] - b'0',
             b'a'..=b'f' => bytes[idx + 1] - b'a' + 10,
             b'A'..=b'F' => bytes[idx + 1] - b'A' + 10,
             _ => return None,
        };
        Some((h << 4) | l)
    };

    if len == 6 {
        let r = parse_byte(i)?;
        let g = parse_byte(i + 2)?;
        let b = parse_byte(i + 4)?;
        Some((r, g, b, 255))
    } else if len == 8 {
        let r = parse_byte(i)?;
        let g = parse_byte(i + 2)?;
        let b = parse_byte(i + 4)?;
        let a = parse_byte(i + 6)?;
        Some((r, g, b, a))
    } else {
        None
    }
}

pub fn parse_percentage(s: &str) -> Option<f32> {
    s.trim_end_matches('%').parse::<f32>().ok().map(|p| p / 100.0)
}
