//! Adversarial cases for `LayerSemantics.port` (story:ntp-exchange-machine): YAML spellings at
//! the edge of "1 to 65535, absent stays absent".

use rfcspec_model::{Profile, from_yaml};

const LAYER: &str = "{layer: udp, unit: datagrams, delivery: {ordered: false, reliable: false, duplicates: true}, direction: half-duplex, multiplexing: single, exchange: request-response, correlation: by-id, lifetime: association, PORT roles: [client, server]}";

fn read(port: &str) -> anyhow::Result<Option<u16>> {
    let layer = LAYER.replace("PORT ", port);
    let profile: Profile = from_yaml(&format!("name: x\nrfcs: []\nsemantics:\n  - {layer}\n"))?;
    Ok(profile.semantics[0].port.map(u16::from))
}

#[test]
fn absent_and_null_are_absent() {
    assert_eq!(read("").unwrap(), None);
    assert_eq!(read("port: null,").unwrap(), None);
    assert_eq!(read("port: ~,").unwrap(), None);
}

#[test]
fn the_bounds_are_1_and_65535() {
    assert_eq!(read("port: 1,").unwrap(), Some(1));
    assert_eq!(read("port: 65535,").unwrap(), Some(65535));
    for bad in [
        "port: 0,",
        "port: 65536,",
        "port: -1,",
        "port: 123.0,",
        "port: true,",
        "port: [123],",
        "port: 4294967419,",
    ] {
        assert!(read(bad).is_err(), "{bad} was accepted as {:?}", read(bad));
    }
}
