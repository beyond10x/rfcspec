//! The wire layer's catalogue and overlay types (design section 8; ESS `rfcspec.catalogue`
//! `Grammar`, `RuleImport`, `Layout`, `LayoutField` and `AbnfDialect`).

use serde::{Deserialize, Serialize};

use crate::{Constraint, Presence, Span};

/// The ABNF rules of one figure of an RFC, kept verbatim (ESS `rfcspec.catalogue.Grammar`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grammar {
    pub id: String,
    pub rfc: u32,
    pub name: String,
    /// The section the figure lies in: its anchor when read from RFCXML, its number (`5.2`) when
    /// read from text; absent outside every numbered section, as for [`crate::Block::section`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// The rules verbatim.
    pub rules: String,
    /// The rules of other RFCs the grammar uses.
    pub uses: Vec<RuleImport>,
    pub span: Span,
    /// The SHA-256 of the spanned text.
    pub digest: String,
}

/// A grammar's use of another RFC's ABNF rule (ESS `rfcspec.catalogue.RuleImport`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleImport {
    pub rfc: u32,
    pub rule: String,
}

/// A packet diagram of the catalogue parsed to fields (ESS `rfcspec.catalogue.Layout`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueLayout {
    pub id: String,
    pub rfc: u32,
    pub name: String,
    /// The section the diagram lies in: its anchor when read from RFCXML, its number (`5.2`) when
    /// read from text; absent outside every numbered section, as for [`crate::Block::section`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    pub fields: Vec<CatalogueField>,
    pub span: Span,
    /// The SHA-256 of the spanned text.
    pub digest: String,
}

/// One field of a parsed packet diagram (ESS `rfcspec.catalogue.LayoutField`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueField {
    pub name: String,
    /// The bit width, absent where the length depends on another field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bits: Option<u32>,
    /// The length read from another field of the layout, where the width is not fixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<DependentLength>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
}

/// A field's length read from another field of the layout (ESS
/// `rfcspec.catalogue.DependentLength`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependentLength {
    pub field: String,
    pub unit: LengthUnit,
}

/// The unit a dependent length counts in (ESS `rfcspec.catalogue.LengthUnit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LengthUnit {
    Bits,
    Octets,
}

/// The ABNF dialect a grammar is written in (ESS `rfcspec.catalogue.AbnfDialect`, design
/// section 8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbnfDialect {
    Rfc5234,
    Rfc2616,
    Bits,
}

/// An overlay's reviewed dialect for one catalogue grammar, keyed by the grammar's id and the
/// SHA-256 of the text it was reviewed against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayGrammar {
    pub id: String,
    pub dialect: AbnfDialect,
    pub text_digest: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abnf_dialect_writes_the_ess_wire_names() {
        for (dialect, wire) in [
            (AbnfDialect::Rfc5234, "rfc5234"),
            (AbnfDialect::Rfc2616, "rfc2616"),
            (AbnfDialect::Bits, "bits"),
        ] {
            assert_eq!(crate::wire(&dialect), wire);
            let read: AbnfDialect = serde_json::from_str(&format!("\"{wire}\"")).unwrap();
            assert_eq!(read, dialect);
        }
        assert!(serde_json::from_str::<AbnfDialect>("\"abnf\"").is_err());
    }
}
