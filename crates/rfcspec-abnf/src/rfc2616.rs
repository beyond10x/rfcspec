//! The ABNF of RFC 2616 section 2.1 (dialect `rfc2616`), with `#rule` lists and implied
//! whitespace: not read yet.

use crate::{Dialect, Error, RuleSet};

/// Parse figure lines in the `rfc2616` dialect; answers [`Error::Unsupported`] until its reader
/// lands.
pub fn parse(_lines: &[(usize, &str)]) -> Result<RuleSet, Error> {
    Err(Error::Unsupported {
        dialect: Dialect::Rfc2616,
    })
}
