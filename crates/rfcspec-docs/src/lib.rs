#![forbid(unsafe_code)]
//! The part of `rfcspec-docs` other crates' tests read: the protocol catalogue ([`catalog`]), the
//! RFC index rendered from the bulk summary ([`bulk`]) and what every generated page shares
//! ([`page`]).

pub mod bulk;
pub mod catalog;
pub mod page;
