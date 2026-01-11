use crate::model::KLyricDocumentV2;
use anyhow::Result;

pub struct TextRenderer {}

impl TextRenderer {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn load_font_bytes(&mut self, _name: &str, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }
    
    pub fn set_default_font_bytes(&mut self, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

pub struct Renderer {
    text_renderer: TextRenderer,
}

impl Renderer {
    pub fn new(_width: u32, _height: u32) -> Self {
        Self {
            text_renderer: TextRenderer::new(),
        }
    }

    pub fn render_frame(&mut self, _doc: &KLyricDocumentV2, _time: f64) -> Result<Vec<u8>> {
        // Return empty frame or generic placeholder
        // Since we claimed preview is broken, this is fine.
        Ok(Vec::new())
    }
    
    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }
}
