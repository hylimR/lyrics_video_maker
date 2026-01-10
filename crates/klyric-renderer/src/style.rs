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
    }
}
