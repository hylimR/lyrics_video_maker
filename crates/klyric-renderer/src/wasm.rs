use wasm_bindgen::prelude::*;
use super::renderer::Renderer;
use super::parser::parse_document;
use super::model::KLyricDocumentV2;

#[wasm_bindgen]
pub struct KLyricWasmRenderer {
    renderer: Renderer,
    current_doc: Option<KLyricDocumentV2>,
}

#[wasm_bindgen]
impl KLyricWasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        // Set panic hook for better error messages in browser console
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Debug);
        
        Self {
            renderer: Renderer::new(width, height),
            current_doc: None,
        }
    }
    
    #[wasm_bindgen]
    pub fn load_document(&mut self, json: &str) -> Result<(), JsValue> {
        let doc = parse_document(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.current_doc = Some(doc);
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn render_frame(&mut self, time: f64) -> Result<Vec<u8>, JsValue> {
        if let Some(doc) = &self.current_doc {
             let pixmap = self.renderer.render_frame(doc, time)
                 .map_err(|e| JsValue::from_str(&e.to_string()))?;
             Ok(pixmap)
        } else {
             Ok(Vec::new())
        }
    }
    
    #[wasm_bindgen]
    pub fn load_font(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        self.renderer.text_renderer_mut().load_font_bytes(name, data.to_vec())
             .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
}
