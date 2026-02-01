use skia_safe::Color;

pub fn parse_color(hex: &str) -> Option<Color> {
    let (r, g, b, a) = crate::utils::parse_hex_color(hex)?;
    Some(Color::from_argb(a, r, g, b))
}

pub fn parse_percentage(s: &str) -> f32 {
    crate::utils::parse_percentage(s).unwrap_or(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============== Color Parsing Tests ==============

    #[test]
    fn test_parse_color_6digit() {
        let color = parse_color("#FF0000").expect("should parse 6-digit hex");
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
        assert_eq!(color.a(), 255); // Default alpha is 255
    }

    #[test]
    fn test_parse_color_8digit() {
        // #FF000080 = red with 50% alpha (0x80 = 128)
        let color = parse_color("#FF000080").expect("should parse 8-digit hex");
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
        assert_eq!(color.a(), 128);
    }

    #[test]
    fn test_parse_color_no_hash() {
        let color = parse_color("FF0000").expect("should parse hex without hash");
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
        assert_eq!(color.a(), 255);
    }

    #[test]
    fn test_parse_color_lowercase() {
        let color = parse_color("#ff0000").expect("should parse lowercase hex");
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
        assert_eq!(color.a(), 255);
    }

    #[test]
    fn test_parse_color_invalid_length() {
        // Too short
        assert!(parse_color("#FFF").is_none());
        // Too long
        assert!(parse_color("#FF0000FF00").is_none());
        // Odd length
        assert!(parse_color("#FF00").is_none());
    }

    #[test]
    fn test_parse_color_invalid_chars() {
        // Non-hex characters
        assert!(parse_color("#GGGGGG").is_none());
        assert!(parse_color("#ZZZZZZ").is_none());
        assert!(parse_color("#12345G").is_none());
    }

    // ============== Percentage Parsing Tests ==============

    #[test]
    fn test_parse_percentage_valid() {
        let result = parse_percentage("50%");
        assert!((result - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_percentage_zero() {
        let result = parse_percentage("0%");
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_percentage_hundred() {
        let result = parse_percentage("100%");
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_percentage_invalid() {
        // Invalid percentage should return default 0.5 (50.0 / 100.0)
        let result = parse_percentage("invalid%");
        assert!((result - 0.5).abs() < f32::EPSILON);

        // Pure gibberish
        let result = parse_percentage("abc");
        assert!((result - 0.5).abs() < f32::EPSILON);
    }
}
