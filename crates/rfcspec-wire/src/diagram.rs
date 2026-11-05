//! The packet diagrams of an RFC.

use rfcspec_model::{Document, wire::CatalogueLayout};

/// The layouts parsed from a document's packet diagrams, in document order, and notes on the
/// diagrams that were not parsed. Collects none yet.
pub fn collect(_document: &Document) -> (Vec<CatalogueLayout>, Vec<String>) {
    (Vec::new(), Vec::new())
}
