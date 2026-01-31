use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::model::{
    document::KLyricDocumentV2,
    layout::{Anchor, Position},
    line::{Char, Line},
    project::Project,
    style::FillStroke,
    style::Shadow,
    style::Stroke,
    style::{Font, Style},
    theme::Theme,
};

/// Parse a subtitle file content into a KLyricV2 Document
pub fn import_subtitle(content: &str, filename: Option<&str>) -> Result<KLyricDocumentV2> {
    let extension = filename
        .and_then(|f| Path::new(f).extension())
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if extension == "klyric" || extension == "json" {
        // Try parsing as KLyric JSON first
        if let Ok(doc) = serde_json::from_str::<KLyricDocumentV2>(content) {
            return Ok(doc);
        }
    }

    let (lyrics, metadata) =
        if extension == "ass" || extension == "ssa" || content.contains("[Script Info]") {
            parse_ass(content)?
        } else if extension == "srt"
            || Regex::new(r"^\d+\s*\n\d{2}:\d{2}:\d{2}")
                .map(|r| r.is_match(content))
                .unwrap_or(false)
        {
            parse_srt(content)?
        } else {
            // Default to LRC
            parse_lrc(content)?
        };

    convert_to_klyric(lyrics, metadata)
}

#[derive(Debug, Clone)]
struct ParsedLyric {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub syllables: Option<Vec<ParsedSyllable>>,
    #[allow(dead_code)]
    pub raw_text: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedSyllable {
    pub text: String,
    pub start_offset: f64,
    pub duration: f64,
}

fn convert_to_klyric(
    lyrics: Vec<ParsedLyric>,
    metadata: HashMap<String, String>,
) -> Result<KLyricDocumentV2> {
    let duration = lyrics.last().map(|l| l.end_time).unwrap_or(0.0) + 2.0;

    // Default Style
    let mut styles = HashMap::new();
    let default_style = Style {
        font: Some(Font {
            family: Some("Noto Sans SC".to_string()),
            size: Some(72.0),
            weight: Some(700),
            style: Some(crate::model::style::FontStyle::Normal),
            letter_spacing: Some(0.0),
        }),
        colors: Some(crate::model::style::StateColors {
            inactive: Some(FillStroke {
                fill: Some("#888888".to_string()),
                stroke: None,
            }),
            active: Some(FillStroke {
                fill: Some("#FFFF00".to_string()),
                stroke: None,
            }),
            complete: Some(FillStroke {
                fill: Some("#FFFFFF".to_string()),
                stroke: None,
            }),
        }),
        stroke: Some(Stroke {
            width: Some(3.0),
            color: Some("#000000".to_string()),
        }),
        shadow: Some(Shadow {
            color: Some("rgba(0,0,0,0.5)".to_string()),
            x: Some(2.0),
            y: Some(2.0),
            blur: Some(4.0),
        }),
        ..Default::default()
    };
    styles.insert("base".to_string(), default_style);

    // Default Effect
    let effects = HashMap::new();
    // Simplified default effect for now
    // effects.insert("fadeIn".to_string(), Effect::default());

    let doc = KLyricDocumentV2 {
        schema: None,
        version: "2.0".to_string(),
        project: Project {
            title: metadata
                .get("title")
                .or(metadata.get("ti"))
                .cloned()
                .unwrap_or_else(|| "Untitled".to_string()),
            artist: Some(
                metadata
                    .get("artist")
                    .or(metadata.get("ar"))
                    .cloned()
                    .unwrap_or_else(|| "".to_string()),
            ),
            duration,
            resolution: crate::model::Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            audio: None,
            album: None,
            created: Some(chrono::Utc::now().to_rfc3339()),
            modified: Some(chrono::Utc::now().to_rfc3339()),
        },
        theme: Some(Theme::default()),
        styles,
        effects,
        lines: lyrics
            .into_iter()
            .enumerate()
            .map(|(idx, lyric)| convert_line_to_klyric(lyric, idx))
            .collect(),
    };

    Ok(doc)
}

fn convert_line_to_klyric(lyric: ParsedLyric, idx: usize) -> Line {
    let mut char_data = Vec::new();

    if let Some(syllables) = lyric.syllables {
        for syllable in syllables {
            let chars: Vec<char> = syllable.text.chars().collect();
            let char_count = chars.len();
            if char_count > 0 {
                let char_duration = syllable.duration / char_count as f64;
                for (i, c) in chars.iter().enumerate() {
                    let char_start =
                        lyric.start_time + syllable.start_offset + (i as f64 * char_duration);
                    let char_end = char_start + char_duration;
                    char_data.push(Char {
                        char: c.to_string(),
                        start: (char_start * 1000.0).round() / 1000.0,
                        end: (char_end * 1000.0).round() / 1000.0,
                        style: None,
                        font: None,
                        stroke: None,
                        shadow: None,
                        effects: vec![],
                        transform: None,
                    });
                }
            }
        }
    } else {
        // Even distribution
        let chars: Vec<char> = lyric.text.chars().collect();
        let total_duration = lyric.end_time - lyric.start_time;
        if !chars.is_empty() {
            let char_duration = total_duration / chars.len() as f64;
            for (i, c) in chars.iter().enumerate() {
                let char_start = lyric.start_time + (i as f64 * char_duration);
                let char_end = lyric.start_time + ((i + 1) as f64 * char_duration);
                char_data.push(Char {
                    char: c.to_string(),
                    start: (char_start * 1000.0).round() / 1000.0,
                    end: (char_end * 1000.0).round() / 1000.0,
                    style: None,
                    font: None,
                    stroke: None,
                    shadow: None,
                    effects: vec![],
                    transform: None,
                });
            }
        }
    }

    Line {
        id: Some(format!("line-{}", idx)),
        start: (lyric.start_time * 1000.0).round() / 1000.0,
        end: (lyric.end_time * 1000.0).round() / 1000.0,
        text: Some(lyric.text),
        style: Some("base".to_string()),
        effects: vec!["fadeIn".to_string()],
        position: Some(Position {
            x: Some(crate::model::layout::PositionValue::Pixels(960.0)),
            y: Some(crate::model::layout::PositionValue::Pixels(540.0)),
            anchor: Anchor::Center,
        }),
        transform: None,
        font: None,
        stroke: None,
        shadow: None,
        layout: None,
        chars: char_data,
    }
}

fn parse_lrc(content: &str) -> Result<(Vec<ParsedLyric>, HashMap<String, String>)> {
    let mut lyrics = Vec::new();
    let mut metadata = HashMap::new();

    // [mm:ss.xx] or [mm:ss:xx]
    let timestamp_regex = Regex::new(r"\[(\d{2}):(\d{2})[.:](\d{2,3})\]").unwrap();
    let metadata_regex = Regex::new(r"\[(ti|ar|al|au|length|by|offset|re|ve):([^\]]*)\]").unwrap();

    let mut offset = 0.0; // seconds

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(cap) = metadata_regex.captures(trimmed) {
            let key = cap[1].to_lowercase();
            let value = cap[2].trim().to_string();

            if key == "offset" {
                if let Ok(v) = value.parse::<f64>() {
                    offset = v / 1000.0;
                }
            }
            metadata.insert(key, value);
            continue;
        }

        let mut timestamps = Vec::new();
        // We need to verify if the line HAS timestamps
        if !timestamp_regex.is_match(trimmed) {
            continue;
        }

        for cap in timestamp_regex.captures_iter(trimmed) {
            let min: f64 = cap[1].parse().unwrap_or(0.0);
            let sec: f64 = cap[2].parse().unwrap_or(0.0);
            let raw_cs = &cap[3];
            let cs: f64 = raw_cs.parse().unwrap_or(0.0);

            // Handle 2 or 3 digit centiseconds/milliseconds
            let cs_val = if raw_cs.len() == 3 {
                cs / 1000.0
            } else {
                cs / 100.0
            };

            timestamps.push(min * 60.0 + sec + cs_val + offset);
        }

        if timestamps.is_empty() {
            continue;
        }

        let text = timestamp_regex.replace_all(trimmed, "").trim().to_string();
        if text.is_empty() {
            continue;
        }

        for start_time in timestamps {
            lyrics.push(ParsedLyric {
                text: text.clone(),
                start_time,
                end_time: 0.0, // Calculated later
                syllables: None,
                raw_text: None,
            });
        }
    }

    lyrics.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Fill end times
    let len = lyrics.len();
    for i in 0..len {
        if i < len - 1 {
            lyrics[i].end_time = lyrics[i + 1].start_time - 0.1;
        } else {
            lyrics[i].end_time = lyrics[i].start_time + 3.0;
        }

        if lyrics[i].end_time < lyrics[i].start_time {
            lyrics[i].end_time = lyrics[i].start_time + 0.5;
        }
    }

    Ok((lyrics, metadata))
}

fn parse_srt(content: &str) -> Result<(Vec<ParsedLyric>, HashMap<String, String>)> {
    let mut lyrics = Vec::new();
    let metadata = HashMap::new();

    // Split by double newline
    let blocks: Vec<&str> = Regex::new(r"\n\s*\n").unwrap().split(content).collect();

    // 00:00:20,000 --> 00:00:24,400
    let timestamp_regex = Regex::new(
        r"(\d{2}):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})[,.](\d{3})",
    )
    .unwrap();
    let html_tag_re = Regex::new(r"<[^>]+>").unwrap();
    let ass_tag_re = Regex::new(r"\{[^}]+\}").unwrap();

    for block in blocks {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 2 {
            continue;
        }

        let mut timestamp_line = "";
        let mut text_lines = Vec::new();
        let mut found_timestamp = false;

        for line in lines {
            if !found_timestamp && timestamp_regex.is_match(line) {
                timestamp_line = line;
                found_timestamp = true;
            } else if found_timestamp {
                text_lines.push(line.trim());
            }
        }

        if !found_timestamp || text_lines.is_empty() {
            continue;
        }

        let cap = timestamp_regex.captures(timestamp_line).unwrap();

        // Start
        let s_h: f64 = cap[1].parse().unwrap_or(0.0);
        let s_m: f64 = cap[2].parse().unwrap_or(0.0);
        let s_s: f64 = cap[3].parse().unwrap_or(0.0);
        let s_ms: f64 = cap[4].parse().unwrap_or(0.0);
        let start_time = s_h * 3600.0 + s_m * 60.0 + s_s + s_ms / 1000.0;

        // End
        let e_h: f64 = cap[5].parse().unwrap_or(0.0);
        let e_m: f64 = cap[6].parse().unwrap_or(0.0);
        let e_s: f64 = cap[7].parse().unwrap_or(0.0);
        let e_ms: f64 = cap[8].parse().unwrap_or(0.0);
        let end_time = e_h * 3600.0 + e_m * 60.0 + e_s + e_ms / 1000.0;

        let raw_text = text_lines.join(" ");
        // Strip HTML tags roughly
        let text = html_tag_re.replace_all(&raw_text, "").to_string();
        // Strip ASS/SSA tags roughly
        let text = ass_tag_re.replace_all(&text, "").trim().to_string();

        if !text.is_empty() {
            lyrics.push(ParsedLyric {
                text,
                start_time,
                end_time,
                syllables: None,
                raw_text: Some(raw_text), // Keep original with tags maybe?
            });
        }
    }

    Ok((lyrics, metadata))
}

fn parse_ass(content: &str) -> Result<(Vec<ParsedLyric>, HashMap<String, String>)> {
    let mut lyrics = Vec::new();
    let mut metadata = HashMap::new();

    // Script Info
    let info_key_re = Regex::new(r"^(?i)(Title|Original Script|Original Translation|Script Updated By|Update Details|Artist):(.*)$").unwrap();

    if let Some(script_info) = Regex::new(r"(?i)\[Script Info\]([\s\S]*?)(?:\[|$)")
        .unwrap()
        .captures(content)
    {
        for line in script_info[1].lines() {
            if let Some(cap) = info_key_re.captures(line) {
                metadata.insert(
                    cap[1].to_lowercase().replace(" ", ""),
                    cap[2].trim().to_string(),
                );
            }
        }
    }

    let events_match = Regex::new(r"(?i)\[Events\]([\s\S]*?)(?:\[|$)")
        .unwrap()
        .captures(content);
    if events_match.is_none() {
        return Ok((lyrics, metadata));
    }
    let events_section = &events_match.unwrap()[1];

    let mut format = Vec::new();
    let lines: Vec<&str> = events_section.lines().collect();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("format:") {
            let fmt_str = trimmed[7..].trim();
            format = fmt_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect();
            break;
        }
    }

    let idx_start = format
        .iter()
        .position(|r| r == "start")
        .unwrap_or(usize::MAX);
    let idx_end = format.iter().position(|r| r == "end").unwrap_or(usize::MAX);
    let idx_text = format
        .iter()
        .position(|r| r == "text")
        .unwrap_or(usize::MAX);

    let time_re = Regex::new(r"(\d+):(\d{2}):(\d{2})\.(\d{2})").unwrap();
    let karaoke_re = Regex::new(r"\{\\[kK]f?\s*\d+\}").unwrap();
    let strip_tag_re = Regex::new(r"\{[^}]*\}").unwrap();

    if idx_start == usize::MAX || idx_end == usize::MAX || idx_text == usize::MAX {
        return Ok((lyrics, metadata)); // Or error?
    }

    for line in &lines {
        if !line.trim().to_lowercase().starts_with("dialogue:") {
            continue;
        }

        // Simple CSV parse - WARNING: Text can contain commas!
        // We split by comma with limit.
        // Actually, dialogue format is usually precise until Text which is the last field.
        // So we can split by comma.

        let content_after_prefix = &line[9..].trim();
        // We can't just split by comma because Name/Style etc might not contain commas but Text definitely can.
        // But ASS format is fixed: "Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"
        // So we split by comma up to the max index we need.
        // Since Text is usually last, we can just split all, then rejoin from idx_text.

        let parts: Vec<&str> = content_after_prefix.split(',').collect();
        if parts.len() <= idx_text {
            continue;
        }

        let start_str = parts[idx_start].trim();
        let end_str = parts[idx_end].trim();
        let raw_text = parts[idx_text..].join(","); // Join the rest as text

        // Parse Time: h:mm:ss.cc
        let parse_time = |t: &str| -> f64 {
            if let Some(cap) = time_re.captures(t) {
                let h: f64 = cap[1].parse().unwrap_or(0.0);
                let m: f64 = cap[2].parse().unwrap_or(0.0);
                let s: f64 = cap[3].parse().unwrap_or(0.0);
                let cs: f64 = cap[4].parse().unwrap_or(0.0);
                h * 3600.0 + m * 60.0 + s + cs / 100.0
            } else {
                0.0
            }
        };

        let start_time = parse_time(start_str);
        let end_time = parse_time(end_str);

        if end_time <= start_time {
            continue;
        }

        let clean_raw = raw_text.replace(r"\N", " ").replace(r"\n", " ");
        let has_karaoke = karaoke_re.is_match(&clean_raw);

        // Remove mut, we can just assign directly
        let text: String;
        let syllables: Option<Vec<ParsedSyllable>>;

        if has_karaoke {
            // Parse karaoke
            let (clean_text, parsed_syllables) = parse_karaoke_tags(&clean_raw, start_time);
            text = clean_text;
            syllables = Some(parsed_syllables);
        } else {
            // Strip tags
            text = strip_tag_re
                .replace_all(&clean_raw, "")
                .replace(r"\N", " ")
                .replace(r"\n", " ")
                .trim()
                .to_string();
            syllables = None;
        }

        if !text.is_empty() {
            lyrics.push(ParsedLyric {
                text,
                start_time,
                end_time,
                syllables,
                raw_text: Some(raw_text),
            });
        }
    }

    lyrics.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((lyrics, metadata))
}

fn parse_karaoke_tags(raw_text: &str, _line_start: f64) -> (String, Vec<ParsedSyllable>) {
    // splits by curly braces
    // {\k10}Word {\k20}Word2
    // But text can be before tags.
    // Logic: Regex for tags vs text.

    let mut syllables = Vec::new();
    let mut clean_text = String::new();
    let mut current_offset = 0.0;

    // We iterate through segments of text and tags
    // Regex to find all tags
    let re = Regex::new(r"(\{[^}]*\})?([^\{]*)").unwrap();
    let k_re = Regex::new(r"\\([kK])f?(\d+)").unwrap();

    for cap in re.captures_iter(raw_text) {
        let tag_part = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let text_part = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        let mut duration = 0.0;

        if !tag_part.is_empty() {
            // Check for karaoke tag {\k##} or {\K##} or {\kf##}
            if let Some(k_match) = k_re.captures(tag_part) {
                let cs: f64 = k_match[2].parse().unwrap_or(0.0);
                duration = cs / 100.0;
            }
        }

        if !text_part.is_empty() {
            syllables.push(ParsedSyllable {
                text: text_part.to_string(),
                start_offset: current_offset,
                duration,
            });
            clean_text.push_str(text_part);
            current_offset += duration;
        } else if duration > 0.0 {
            // Tag without text (gap or prefix), add duration to offset?
            // Usually leading karaoke tag applies to following text...
            // Valid ASS/Karaoke is {\k10}Word
            // So if we see a tag, we store the duration.
            // If we then see text, we use that duration for that text.

            // Wait, my loop splits pairwise tag+text.
            // if tag {\k10} and text "Word", then "Word" has 10cs duration.
            // if tag {\k10} and no text, it might be a wait.
            // But we already handled it in the 'if !text_part.is_empty()' block?
            // Actually the duration extraction happens above.
            // If text_part is empty, we just increment offset?
            // Usually empty text part with duration is a pause/space logic handling.

            if text_part.is_empty() {
                current_offset += duration;
            }
        }
    }

    (clean_text, syllables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let content = "[00:12.00]Line 1\n[00:15.50]Line 2";
        let (lyrics, _) = parse_lrc(content).unwrap();
        assert_eq!(lyrics.len(), 2);
        assert_eq!(lyrics[0].text, "Line 1");
        assert_eq!(lyrics[0].start_time, 12.0);
        assert_eq!(lyrics[0].end_time, 15.4); // 15.5 - 0.1
    }

    #[test]
    fn test_parse_srt() {
        let content = "1\n00:00:20,000 --> 00:00:24,400\nHello World\n\n2\n00:00:25,000 --> 00:00:28,000\nLine 2";
        let (lyrics, _) = parse_srt(content).unwrap();
        assert_eq!(lyrics.len(), 2);
        assert_eq!(lyrics[0].text, "Hello World");
        assert_eq!(lyrics[0].start_time, 20.0);
        assert_eq!(lyrics[0].end_time, 24.4);
    }
}
