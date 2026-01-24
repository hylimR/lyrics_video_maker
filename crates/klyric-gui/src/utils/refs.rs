use std::sync::Arc;
use std::hash::{Hash, Hasher};
use klyric_renderer::model::document::KLyricDocumentV2;

/// A wrapper around Arc<KLyricDocumentV2> that implements Hash and PartialEq
/// based on the Arc pointer. This is used for Iced lazy widgets to detect
/// when the document version has changed (since AppState uses copy-on-write).
#[derive(Clone, Debug)]
pub struct DocumentRef(pub Arc<KLyricDocumentV2>);

impl PartialEq for DocumentRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DocumentRef {}

impl Hash for DocumentRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
