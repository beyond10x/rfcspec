//! Communication semantics (design section 6): one value per axis for each layer of a profile,
//! `rfcspec.profile.LayerSemantics`. Each transport is its own layer. The values are the wire
//! spellings of `ess/domains/profile.yaml`; an unknown value or field is a parse error.

use serde::{Deserialize, Serialize};

/// The semantics of one layer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerSemantics {
    /// The layer's name, such as `tcp` or `udp`; distinct within a profile.
    pub layer: String,
    pub unit: Unit,
    pub delivery: Delivery,
    pub direction: Direction,
    pub multiplexing: Multiplexing,
    pub exchange: Exchange,
    pub correlation: Correlation,
    pub lifetime: Lifetime,
    /// The lower layer that supplies the channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub over: Option<Layering>,
    pub roles: Vec<String>,
}

/// What the carrier hands the protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unit {
    Bytes,
    Messages,
    Datagrams,
}

/// Ordering, loss and duplication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub ordered: bool,
    pub reliable: bool,
    /// Duplicates are possible.
    pub duplicates: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    Simplex,
    HalfDuplex,
    FullDuplex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Multiplexing {
    Single,
    Streams,
    Sessions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Exchange {
    RequestResponse,
    RequestStream,
    Session,
    Publish,
    OneShot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Correlation {
    None,
    ByOrder,
    ById,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lifetime {
    Connectionless,
    Connection,
    Association,
}

/// `over <profile>.<exchange or stream>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layering {
    pub profile: String,
    pub endpoint: Endpoint,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Endpoint {
    Exchange,
    Stream,
}

/// The wire spelling of a unit-like enum value, as it is written in `profile.yaml`.
pub fn wire<T: Serialize>(value: &T) -> String {
    serde_yaml::to_string(value)
        .map(|text| text.trim().to_owned())
        .unwrap_or_default()
}

impl Delivery {
    /// `ordered, reliable, no duplicates` and the like.
    pub fn describe(&self) -> String {
        format!(
            "{}, {}, {}",
            if self.ordered { "ordered" } else { "unordered" },
            if self.reliable { "reliable" } else { "lossy" },
            if self.duplicates {
                "duplicates"
            } else {
                "no duplicates"
            }
        )
    }
}

impl Layering {
    /// `over <profile>.<endpoint> <name>`, as design section 6.1 writes it.
    pub fn describe(&self) -> String {
        format!("{}.{} {}", self.profile, wire(&self.endpoint), self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile, from_yaml};

    const LAYER: &str = "{layer: udp, unit: datagrams, delivery: {ordered: false, reliable: false, duplicates: true}, direction: half-duplex, multiplexing: single, exchange: request-response, correlation: none, lifetime: connectionless, roles: [client, server]}";

    #[test]
    fn semantics_read_the_wire_spellings_and_default_to_none() {
        let profile: Profile = from_yaml("name: x\nrfcs: []\n").unwrap();
        assert!(profile.semantics.is_empty());
        let profile: Profile =
            from_yaml(&format!("name: x\nrfcs: []\nsemantics:\n  - {LAYER}\n")).unwrap();
        let layer = &profile.semantics[0];
        assert_eq!(layer.layer, "udp");
        assert_eq!(layer.direction, Direction::HalfDuplex);
        assert_eq!(layer.exchange, Exchange::RequestResponse);
        assert_eq!(wire(&layer.exchange), "request-response");
        assert_eq!(layer.delivery.describe(), "unordered, lossy, duplicates");
        assert_eq!(layer.over, None);
    }

    #[test]
    fn an_unknown_value_or_field_is_a_parse_error() {
        for bad in [
            LAYER.replace("half-duplex", "duplex"),
            LAYER.replace("datagrams", "Datagrams"),
            LAYER.replace("roles:", "colour: red, roles:"),
            LAYER.replace("duplicates: true", "duplicates: true, late: true"),
        ] {
            assert!(
                from_yaml::<Profile>(&format!("name: x\nrfcs: []\nsemantics:\n  - {bad}\n"))
                    .is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn layering_reads_and_describes() {
        let over = LAYER.replace(
            "roles:",
            "over: {profile: tcp, endpoint: stream, name: bytes}, roles:",
        );
        let profile: Profile =
            from_yaml(&format!("name: x\nrfcs: []\nsemantics:\n  - {over}\n")).unwrap();
        assert_eq!(
            profile.semantics[0].over.as_ref().unwrap().describe(),
            "tcp.stream bytes"
        );
    }
}
