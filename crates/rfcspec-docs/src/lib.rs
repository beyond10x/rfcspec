#![forbid(unsafe_code)]
//! The part of `rfcspec-docs` other crates' tests read: the protocol catalogue ([`catalog`]), the
//! RFC index rendered from the bulk summary ([`bulk`]), the RFC explorer's shards ([`explorer`]) and what every generated page shares
//! ([`page`]).

pub mod bulk;
pub mod catalog;
pub mod explorer;
pub mod page;
