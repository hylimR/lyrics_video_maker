//! Test helpers for klyric-renderer tests
//!
//! Provides utility functions for creating test documents and inspecting rendered pixels.

use klyric_renderer::{
    Char, FillStroke, Font, KLyricDocumentV2, Line, Project, Resolution, StateColors, Style,
};
use std::collections::HashMap;

/// Get pixel at (x, y) from raw RGBA buffer
///
/// # Arguments
/// * `pixels` - Raw RGBA pixel buffer
/// * `width` - Image width in pixels
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// Tuple of (red, green, blue, alpha) values
pub fn get_pixel(pixels: &[u8], width: u32, x: u32, y: u32) -> (u8, u8, u8, u8) {
    let idx = ((y * width + x) * 4) as usize;
    (
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    )
}

/// Check if pixel at (x, y) matches expected color within tolerance
///
/// # Arguments
/// * `pixels` - Raw RGBA pixel buffer
/// * `width` - Image width in pixels
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `expected_r` - Expected red value
/// * `expected_g` - Expected green value
/// * `expected_b` - Expected blue value
/// * `tolerance` - Maximum allowed difference per channel
///
/// # Returns
/// `true` if pixel matches within tolerance
#[allow(clippy::too_many_arguments)]
pub fn pixel_matches(
    pixels: &[u8],
    width: u32,
    x: u32,
    y: u32,
    expected_r: u8,
    expected_g: u8,
    expected_b: u8,
    tolerance: u8,
) -> bool {
    let (r, g, b, _a) = get_pixel(pixels, width, x, y);
    let diff_r = (r as i16 - expected_r as i16).unsigned_abs() as u8;
    let diff_g = (g as i16 - expected_g as i16).unsigned_abs() as u8;
    let diff_b = (b as i16 - expected_b as i16).unsigned_abs() as u8;
    diff_r <= tolerance && diff_g <= tolerance && diff_b <= tolerance
}

/// Check if pixel is approximately black (all channels below threshold)
pub fn pixel_is_black(pixels: &[u8], width: u32, x: u32, y: u32, threshold: u8) -> bool {
    let (r, g, b, _a) = get_pixel(pixels, width, x, y);
    r <= threshold && g <= threshold && b <= threshold
}

/// Check if pixel is approximately white (all channels above threshold)
pub fn pixel_is_white(pixels: &[u8], width: u32, x: u32, y: u32, threshold: u8) -> bool {
    let (r, g, b, _a) = get_pixel(pixels, width, x, y);
    r >= threshold && g >= threshold && b >= threshold
}

/// Count non-black pixels in the buffer
///
/// # Arguments
/// * `pixels` - Raw RGBA pixel buffer
/// * `threshold` - Minimum value for any channel to be considered non-black
///
/// # Returns
/// Number of pixels where any RGB channel exceeds threshold
pub fn count_non_black_pixels(pixels: &[u8], threshold: u8) -> usize {
    pixels
        .chunks_exact(4)
        .filter(|chunk| chunk[0] > threshold || chunk[1] > threshold || chunk[2] > threshold)
        .count()
}

/// Create a minimal valid KLyric v2.0 document with no lines
///
/// Useful for testing empty document handling and background rendering.
pub fn minimal_doc() -> KLyricDocumentV2 {
    KLyricDocumentV2 {
        schema: None,
        version: "2.0".to_string(),
        project: Project {
            title: "Test".to_string(),
            artist: None,
            album: None,
            duration: 10.0,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio: None,
            created: None,
            modified: None,
        },
        theme: None,
        styles: HashMap::new(),
        effects: HashMap::new(),
        lines: Vec::new(),
    }
}

/// Create a KLyric document with a single line of text
///
/// # Arguments
/// * `text` - The text to display
/// * `start` - Line start time in seconds
/// * `end` - Line end time in seconds
///
/// # Returns
/// A KLyricDocumentV2 with one line, each character timed evenly
pub fn doc_with_line(text: &str, start: f64, end: f64) -> KLyricDocumentV2 {
    let chars: Vec<Char> = text
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            let char_duration = (end - start) / text.len() as f64;
            let char_start = start + (i as f64 * char_duration);
            let char_end = char_start + char_duration;
            Char {
                char: ch.to_string(),
                start: char_start,
                end: char_end,
                style: None,
                font: None,
                stroke: None,
                shadow: None,
                effects: Vec::new(),
                transform: None,
            }
        })
        .collect();

    let line = Line {
        id: None,
        start,
        end,
        text: Some(text.to_string()),
        style: Some("base".to_string()),
        font: None,
        stroke: None,
        shadow: None,
        effects: Vec::new(),
        position: None,
        transform: None,
        layout: None,
        chars,
    };

    let mut styles = HashMap::new();
    styles.insert(
        "base".to_string(),
        Style {
            extends: None,
            font: Some(Font {
                family: Some("Arial".to_string()),
                size: Some(48.0),
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: Some(StateColors {
                active: Some(FillStroke {
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                }),
                inactive: Some(FillStroke {
                    fill: Some("#888888".to_string()),
                    stroke: None,
                }),
                complete: Some(FillStroke {
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                }),
            }),
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        },
    );

    KLyricDocumentV2 {
        schema: None,
        version: "2.0".to_string(),
        project: Project {
            title: "Test".to_string(),
            artist: None,
            album: None,
            duration: end + 5.0,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio: None,
            created: None,
            modified: None,
        },
        theme: None,
        styles,
        effects: HashMap::new(),
        lines: vec![line],
    }
}

/// Create a document with multiple lines for testing line switching
pub fn doc_with_multiple_lines(lines_data: &[(&str, f64, f64)]) -> KLyricDocumentV2 {
    let lines: Vec<Line> = lines_data
        .iter()
        .map(|(text, start, end)| {
            let chars: Vec<Char> = text
                .chars()
                .enumerate()
                .map(|(i, ch)| {
                    let char_duration = (end - start) / text.len() as f64;
                    let char_start = start + (i as f64 * char_duration);
                    let char_end = char_start + char_duration;
                    Char {
                        char: ch.to_string(),
                        start: char_start,
                        end: char_end,
                        style: None,
                        font: None,
                        stroke: None,
                        shadow: None,
                        effects: Vec::new(),
                        transform: None,
                    }
                })
                .collect();

            Line {
                id: None,
                start: *start,
                end: *end,
                text: Some(text.to_string()),
                style: Some("base".to_string()),
                font: None,
                stroke: None,
                shadow: None,
                effects: Vec::new(),
                position: None,
                transform: None,
                layout: None,
                chars,
            }
        })
        .collect();

    let max_end = lines_data
        .iter()
        .map(|(_, _, end)| *end)
        .fold(0.0, f64::max);

    let mut styles = HashMap::new();
    styles.insert(
        "base".to_string(),
        Style {
            extends: None,
            font: Some(Font {
                family: Some("Arial".to_string()),
                size: Some(48.0),
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: Some(StateColors {
                active: Some(FillStroke {
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                }),
                inactive: Some(FillStroke {
                    fill: Some("#888888".to_string()),
                    stroke: None,
                }),
                complete: Some(FillStroke {
                    fill: Some("#FFFFFF".to_string()),
                    stroke: None,
                }),
            }),
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        },
    );

    KLyricDocumentV2 {
        schema: None,
        version: "2.0".to_string(),
        project: Project {
            title: "Test".to_string(),
            artist: None,
            album: None,
            duration: max_end + 5.0,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio: None,
            created: None,
            modified: None,
        },
        theme: None,
        styles,
        effects: HashMap::new(),
        lines,
    }
}

/// Create a document with custom styles for testing style resolution
pub fn doc_with_styles(styles: HashMap<String, Style>) -> KLyricDocumentV2 {
    KLyricDocumentV2 {
        schema: None,
        version: "2.0".to_string(),
        project: Project {
            title: "Test".to_string(),
            artist: None,
            album: None,
            duration: 10.0,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio: None,
            created: None,
            modified: None,
        },
        theme: None,
        styles,
        effects: HashMap::new(),
        lines: Vec::new(),
    }
}

/// Assert that two f32 values are approximately equal
pub fn assert_f32_eq(actual: f32, expected: f32, epsilon: f32) {
    assert!(
        (actual - expected).abs() < epsilon,
        "Expected {} to be approximately {}, difference: {}",
        actual,
        expected,
        (actual - expected).abs()
    );
}

/// Assert that two f64 values are approximately equal
pub fn assert_f64_eq(actual: f64, expected: f64, epsilon: f64) {
    assert!(
        (actual - expected).abs() < epsilon,
        "Expected {} to be approximately {}, difference: {}",
        actual,
        expected,
        (actual - expected).abs()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pixel() {
        // Create a 2x2 image: red, green, blue, white
        let pixels = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 255, 255, // (1,1) white
        ];

        assert_eq!(get_pixel(&pixels, 2, 0, 0), (255, 0, 0, 255));
        assert_eq!(get_pixel(&pixels, 2, 1, 0), (0, 255, 0, 255));
        assert_eq!(get_pixel(&pixels, 2, 0, 1), (0, 0, 255, 255));
        assert_eq!(get_pixel(&pixels, 2, 1, 1), (255, 255, 255, 255));
    }

    #[test]
    fn test_pixel_matches() {
        let pixels = vec![255, 100, 50, 255];

        assert!(pixel_matches(&pixels, 1, 0, 0, 255, 100, 50, 0));
        assert!(pixel_matches(&pixels, 1, 0, 0, 250, 105, 45, 10));
        assert!(!pixel_matches(&pixels, 1, 0, 0, 200, 100, 50, 10));
    }

    #[test]
    fn test_pixel_is_black() {
        let black = vec![0, 0, 0, 255];
        let nearly_black = vec![10, 5, 8, 255];
        let not_black = vec![50, 0, 0, 255];

        assert!(pixel_is_black(&black, 1, 0, 0, 10));
        assert!(pixel_is_black(&nearly_black, 1, 0, 0, 10));
        assert!(!pixel_is_black(&not_black, 1, 0, 0, 10));
    }

    #[test]
    fn test_count_non_black_pixels() {
        let pixels = vec![
            0, 0, 0, 255, // black
            50, 0, 0, 255, // non-black
            0, 0, 0, 255, // black
            100, 100, 100, 255, // non-black
        ];

        assert_eq!(count_non_black_pixels(&pixels, 20), 2);
        assert_eq!(count_non_black_pixels(&pixels, 100), 0);
    }

    #[test]
    fn test_minimal_doc() {
        let doc = minimal_doc();
        assert_eq!(doc.version, "2.0");
        assert!(doc.lines.is_empty());
        assert_eq!(doc.project.title, "Test");
    }

    #[test]
    fn test_doc_with_line() {
        let doc = doc_with_line("Hi", 1.0, 3.0);

        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].chars.len(), 2);
        assert_eq!(doc.lines[0].chars[0].char, "H");
        assert_eq!(doc.lines[0].chars[1].char, "i");
        assert_eq!(doc.lines[0].start, 1.0);
        assert_eq!(doc.lines[0].end, 3.0);
    }

    #[test]
    fn test_doc_with_multiple_lines() {
        let doc = doc_with_multiple_lines(&[("Hello", 1.0, 3.0), ("World", 4.0, 6.0)]);

        assert_eq!(doc.lines.len(), 2);
        assert_eq!(doc.lines[0].text, Some("Hello".to_string()));
        assert_eq!(doc.lines[1].text, Some("World".to_string()));
    }

    #[test]
    fn test_assert_f32_eq() {
        assert_f32_eq(1.0, 1.0, 0.001);
        assert_f32_eq(1.0, 1.0001, 0.001);
    }

    #[test]
    fn test_assert_f64_eq() {
        assert_f64_eq(1.0, 1.0, 0.0001);
        assert_f64_eq(1.0, 1.00001, 0.0001);
    }
}
