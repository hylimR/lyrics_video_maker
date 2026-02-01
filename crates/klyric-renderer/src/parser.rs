use super::model::KLyricDocumentV2;
use anyhow::{Context, Result};

/// Parse a KLyric v2.0 document from a JSON string
pub fn parse_document(json: &str) -> Result<KLyricDocumentV2> {
    let doc: KLyricDocumentV2 =
        serde_json::from_str(json).context("Failed to parse KLyric v2.0 JSON")?;

    // Validate version
    if doc.version != "2.0" {
        return Err(anyhow::anyhow!(
            "Unsupported KLyric version: {}",
            doc.version
        ));
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_validates_version() {
        let json = r#"{"version": "1.0", "lines": [], "project": {"title":"", "duration":0, "resolution":{"width":0,"height":0}}}"#;
        let result = parse_document(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported KLyric version"));
    }
}
