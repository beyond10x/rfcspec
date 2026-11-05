//! The ABNF grammars of an RFC.

use rfcspec_model::{Document, wire::Grammar};

/// The grammars of a document, in document order. Collects none yet.
pub fn collect(_document: &Document) -> Vec<Grammar> {
    Vec::new()
}
