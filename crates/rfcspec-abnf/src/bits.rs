//! The bit-level ABNF of RFC 6455 section 5.2 (dialect `bits`): not read yet.

use crate::{Dialect, Error, RuleSet};

/// Parse figure lines in the `bits` dialect; answers [`Error::Unsupported`] until its reader
/// lands.
pub fn parse(_lines: &[(usize, &str)]) -> Result<RuleSet, Error> {
    Err(Error::Unsupported {
        dialect: Dialect::Bits,
    })
}
