use super::model::{Style, KLyricDocumentV2};

/// Resolver for KLyric styles handling inheritance
pub struct StyleResolver<'a> {
    doc: &'a KLyricDocumentV2,
}

impl<'a> StyleResolver<'a> {
    pub fn new(doc: &'a KLyricDocumentV2) -> Self {
        Self { doc }
    }

    /// Resolve a style by name, handling inheritance
    pub fn resolve(&self, name: &str) -> Style {
        let mut resolved = Style::default();
        
        // If style exists in document
        if let Some(style) = self.doc.styles.get(name) {
            // Handle inheritance (recursive)
            if let Some(ref extends) = style.extends {
                // Prevent infinite recursion loops by simple depth check or cycle detection?
                // For MVP, just recursion is okay if we assume valid doc, but let's be safe.
                // We'll trust the recursion for now but maybe limit depth?
                // Actually, simple recursion is fine for now.
                let parent = self.resolve(extends);
                
                // Merge parent into resolved (parent is base)
                resolved = parent;
            }
            
            // Merge current style on top
            self.merge_style(&mut resolved, style);
        } else if name == "base" {
             // Fallback for "base" if not defined? Or just return default.
             // Usually "base" should be in doc or we assume defaults.
        }
        
        resolved
    }
    
    /// Merge style properties (source overrides target)
    fn merge_style(&self, target: &mut Style, source: &Style) {
        if source.font.is_some() {
            target.font = source.font.clone();
        }
        if source.colors.is_some() {
            target.colors = source.colors.clone();
        }
        if source.stroke.is_some() {
            target.stroke = source.stroke.clone();
        }
        if source.shadow.is_some() {
            target.shadow = source.shadow.clone();
        }
        if source.glow.is_some() {
            target.glow = source.glow.clone();
        }
        if source.transform.is_some() {
            target.transform = source.transform.clone();
        }
        if source.effects.is_some() {
            target.effects = source.effects.clone();
        }
        if source.layers.is_some() {
            target.layers = source.layers.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Font, StateColors, FillStroke, Stroke, Shadow, Glow, Project, Resolution};
    use std::collections::HashMap;

    /// Create a minimal document with no styles
    fn minimal_doc() -> KLyricDocumentV2 {
        KLyricDocumentV2 {
            schema: None,
            version: "2.0".to_string(),
            project: Project {
                title: "Test".to_string(),
                artist: None,
                album: None,
                duration: 10.0,
                resolution: Resolution { width: 1920, height: 1080 },
                fps: 30,
                audio: None,
                created: None,
                modified: None,
            },
            theme: None,
            styles: HashMap::new(),
            effects: HashMap::new(),
            lines: vec![],
        }
    }

    /// Create a document with given styles
    fn doc_with_styles(styles: HashMap<String, Style>) -> KLyricDocumentV2 {
        let mut doc = minimal_doc();
        doc.styles = styles;
        doc
    }

    // ========== Style Resolution Tests ==========

    #[test]
    fn test_resolve_nonexistent() {
        // Resolving a style that doesn't exist should return default Style
        let doc = minimal_doc();
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("nonexistent_style");
        
        // Default Style has all None fields
        assert!(resolved.font.is_none());
        assert!(resolved.colors.is_none());
        assert!(resolved.stroke.is_none());
        assert!(resolved.shadow.is_none());
        assert!(resolved.glow.is_none());
    }

    #[test]
    fn test_resolve_simple() {
        // Resolving a simple style returns exact match
        let mut styles = HashMap::new();
        styles.insert("main".to_string(), Style {
            extends: None,
            font: Some(Font {
                family: Some("Arial".to_string()),
                size: Some(48.0),
                weight: Some(700),
                style: None,
                letter_spacing: None,
            }),
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("main");
        
        assert!(resolved.font.is_some());
        let font = resolved.font.unwrap();
        assert_eq!(font.family, Some("Arial".to_string()));
        assert_eq!(font.size, Some(48.0));
        assert_eq!(font.weight, Some(700));
    }

    #[test]
    fn test_resolve_inheritance() {
        // Child style extends parent and inherits properties
        let mut styles = HashMap::new();
        
        // Parent style with font
        styles.insert("parent".to_string(), Style {
            extends: None,
            font: Some(Font {
                family: Some("Roboto".to_string()),
                size: Some(36.0),
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: Some(StateColors {
                inactive: Some(FillStroke { fill: Some("#888888".to_string()), stroke: None }),
                active: None,
                complete: None,
            }),
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        // Child style extends parent with additional stroke
        styles.insert("child".to_string(), Style {
            extends: Some("parent".to_string()),
            font: None, // Inherits from parent
            colors: None, // Inherits from parent
            stroke: Some(Stroke {
                width: Some(2.0),
                color: Some("#000000".to_string()),
            }),
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("child");
        
        // Should have inherited font from parent
        assert!(resolved.font.is_some());
        let font = resolved.font.unwrap();
        assert_eq!(font.family, Some("Roboto".to_string()));
        assert_eq!(font.size, Some(36.0));
        
        // Should have inherited colors from parent
        assert!(resolved.colors.is_some());
        
        // Should have own stroke
        assert!(resolved.stroke.is_some());
        let stroke = resolved.stroke.unwrap();
        assert_eq!(stroke.width, Some(2.0));
    }

    #[test]
    fn test_resolve_chain() {
        // Multi-level inheritance: A extends B extends C
        let mut styles = HashMap::new();
        
        // Base style C
        styles.insert("styleC".to_string(), Style {
            extends: None,
            font: Some(Font {
                family: Some("BaseFont".to_string()),
                size: Some(24.0),
                weight: Some(400),
                style: None,
                letter_spacing: None,
            }),
            colors: None,
            stroke: None,
            shadow: Some(Shadow {
                color: Some("#000000".to_string()),
                x: Some(2.0),
                y: Some(2.0),
                blur: Some(4.0),
            }),
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        // Middle style B extends C
        styles.insert("styleB".to_string(), Style {
            extends: Some("styleC".to_string()),
            font: Some(Font {
                family: None, // Keep parent's family
                size: Some(32.0), // Override size
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: None,
            stroke: None,
            shadow: None, // Inherits from C
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        // Top style A extends B
        styles.insert("styleA".to_string(), Style {
            extends: Some("styleB".to_string()),
            font: None, // Inherits from B
            colors: None,
            stroke: Some(Stroke {
                width: Some(1.0),
                color: Some("#FF0000".to_string()),
            }),
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("styleA");
        
        // Font should come from B (which overrode C's size)
        assert!(resolved.font.is_some());
        let font = resolved.font.unwrap();
        assert_eq!(font.size, Some(32.0)); // From B
        
        // Shadow should come from C (through B's inheritance)
        assert!(resolved.shadow.is_some());
        let shadow = resolved.shadow.unwrap();
        assert_eq!(shadow.color, Some("#000000".to_string()));
        
        // Stroke should be from A
        assert!(resolved.stroke.is_some());
        let stroke = resolved.stroke.unwrap();
        assert_eq!(stroke.color, Some("#FF0000".to_string()));
    }

    // ========== Style Merge Tests ==========

    #[test]
    fn test_merge_font() {
        // Font override behavior: source font completely replaces target font
        let mut styles = HashMap::new();
        
        styles.insert("base".to_string(), Style {
            extends: None,
            font: Some(Font {
                family: Some("OldFont".to_string()),
                size: Some(20.0),
                weight: Some(400),
                style: None,
                letter_spacing: Some(1.0),
            }),
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        styles.insert("override".to_string(), Style {
            extends: Some("base".to_string()),
            font: Some(Font {
                family: Some("NewFont".to_string()),
                size: Some(30.0),
                weight: None, // Not specified
                style: None,
                letter_spacing: None, // Not specified
            }),
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("override");
        
        let font = resolved.font.unwrap();
        // Source font entirely replaces target font
        assert_eq!(font.family, Some("NewFont".to_string()));
        assert_eq!(font.size, Some(30.0));
        // Note: weight and letter_spacing are None in source, so they become None
        assert!(font.weight.is_none());
        assert!(font.letter_spacing.is_none());
    }

    #[test]
    fn test_merge_colors() {
        // Colors merge correctly
        let mut styles = HashMap::new();
        
        styles.insert("base".to_string(), Style {
            extends: None,
            font: None,
            colors: Some(StateColors {
                inactive: Some(FillStroke { fill: Some("#AAAAAA".to_string()), stroke: None }),
                active: Some(FillStroke { fill: Some("#FFFFFF".to_string()), stroke: None }),
                complete: None,
            }),
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        styles.insert("child".to_string(), Style {
            extends: Some("base".to_string()),
            font: None,
            colors: Some(StateColors {
                inactive: None,
                active: Some(FillStroke { fill: Some("#00FF00".to_string()), stroke: Some("#000000".to_string()) }),
                complete: Some(FillStroke { fill: Some("#0000FF".to_string()), stroke: None }),
            }),
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("child");
        
        let colors = resolved.colors.unwrap();
        // Child's colors entirely replace base colors
        assert!(colors.inactive.is_none()); // Child didn't set inactive
        assert!(colors.active.is_some());
        assert_eq!(colors.active.unwrap().fill, Some("#00FF00".to_string()));
        assert!(colors.complete.is_some());
    }

    #[test]
    fn test_merge_partial() {
        // Only non-None fields override: if source.font is None, target.font is kept
        let mut styles = HashMap::new();
        
        styles.insert("base".to_string(), Style {
            extends: None,
            font: Some(Font {
                family: Some("BaseFont".to_string()),
                size: Some(24.0),
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: None,
            stroke: Some(Stroke {
                width: Some(3.0),
                color: Some("#111111".to_string()),
            }),
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });
        
        // Child only adds glow, leaves font and stroke as None
        styles.insert("child".to_string(), Style {
            extends: Some("base".to_string()),
            font: None, // Keep base font
            colors: None,
            stroke: None, // Keep base stroke
            shadow: None,
            glow: Some(Glow {
                color: Some("#FFFF00".to_string()),
                blur: Some(10.0),
                intensity: Some(0.8),
            }),
            transform: None,
            effects: None,
            layers: None,
        });
        
        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("child");
        
        // Font should be preserved from base
        assert!(resolved.font.is_some());
        assert_eq!(resolved.font.as_ref().unwrap().family, Some("BaseFont".to_string()));
        
        // Stroke should be preserved from base
        assert!(resolved.stroke.is_some());
        assert_eq!(resolved.stroke.as_ref().unwrap().width, Some(3.0));
        
        // Glow should be from child
        assert!(resolved.glow.is_some());
        assert_eq!(resolved.glow.as_ref().unwrap().blur, Some(10.0));
    }

    #[test]
    fn test_base_fallback() {
        // "base" style without definition returns default
        let doc = minimal_doc();
        let resolver = StyleResolver::new(&doc);
        
        let resolved = resolver.resolve("base");
        
        // Just returns default Style (all None)
        assert!(resolved.font.is_none());
        assert!(resolved.colors.is_none());
        assert!(resolved.stroke.is_none());
        assert!(resolved.shadow.is_none());
        assert!(resolved.glow.is_none());
    }

    #[test]
    fn test_merge_effects() {
        // Effects should be replaced by child style if present
        let mut styles = HashMap::new();
        
        styles.insert("base".to_string(), Style {
            extends: None,
            font: None,
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: Some(vec!["base_effect".to_string()]),
            layers: None,
        });
        
        // Child overrides effects
        styles.insert("override".to_string(), Style {
            extends: Some("base".to_string()),
            font: None,
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: Some(vec!["child_effect".to_string()]),
            layers: None,
        });
        
        // Child inherits effects (none in child)
        styles.insert("inherit".to_string(), Style {
            extends: Some("base".to_string()),
            font: None,
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
            effects: None,
            layers: None,
        });

        let doc = doc_with_styles(styles);
        let resolver = StyleResolver::new(&doc);
        
        // Test override
        let resolved_override = resolver.resolve("override");
        assert!(resolved_override.effects.is_some());
        let effects = resolved_override.effects.unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0], "child_effect".to_string());
        
        // Test inheritance
        let resolved_inherit = resolver.resolve("inherit");
        assert!(resolved_inherit.effects.is_some());
        let effects_inh = resolved_inherit.effects.unwrap();
        assert_eq!(effects_inh.len(), 1);
        assert_eq!(effects_inh[0], "base_effect".to_string());
    }
}
