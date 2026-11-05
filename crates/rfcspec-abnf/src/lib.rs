#![forbid(unsafe_code)]
//! RFC ABNF (design section 13): the dialects a grammar is written in.

/// The ABNF dialect a grammar is written in (ESS `rfcspec.catalogue.AbnfDialect`, design
/// section 8.3): RFC 5234, RFC 2616 with its `#rule` lists and implied whitespace, or the
/// bit-level ABNF of RFC 6455 section 5.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dialect {
    Rfc5234,
    Rfc2616,
    Bits,
}

impl Dialect {
    /// Every dialect, in declaration order.
    pub const ALL: [Dialect; 3] = [Dialect::Rfc5234, Dialect::Rfc2616, Dialect::Bits];

    /// The dialect's wire name: `rfc5234`, `rfc2616` or `bits`.
    pub fn wire(self) -> &'static str {
        match self {
            Dialect::Rfc5234 => "rfc5234",
            Dialect::Rfc2616 => "rfc2616",
            Dialect::Bits => "bits",
        }
    }

    /// The dialect a wire name names; `None` for any other name.
    pub fn from_wire(name: &str) -> Option<Dialect> {
        Dialect::ALL
            .into_iter()
            .find(|dialect| dialect.wire() == name)
    }
}
