//! The text-path catalogues of the RFCs the profiles lock (story:text-reader-structure). Their
//! overlays key statuses by catalogue id and their generated code carries source line numbers,
//! so a change to the text reader or the extraction must not move one of their requirements.
//! Both digests were pinned against the reader before that story changed it.

use std::{fs, path::PathBuf};

use rfcspec_extract::extract;
use rfcspec_model::{sha256_hex, to_json};

/// What is pinned of the text-path catalogue of `profiles/<profile>`'s locked `rfc<N>.txt`: how
/// many requirements it lists, the SHA-256 of its requirement list (one `<id> <level>
/// <start>-<end>` line per requirement) and the SHA-256 of the whole catalogue as JSON, which
/// also pins every requirement's text and the ids, texts and spans of the prose candidates,
/// dialogue steps and examples (RFC 868 and 1035 have no BCP 14 requirement; their overlays key
/// prose candidates).
fn pinned(profile: &str, rfc: u32) -> (usize, String, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../profiles")
        .join(profile)
        .join(format!("sources/rfc{rfc}.txt"));
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let text = String::from_utf8(bytes.clone()).unwrap();
    let catalogue = extract(&rfcspec_doc::read(rfc, &sha256_hex(&bytes), &text));
    let list: String = catalogue
        .requirements
        .iter()
        .map(|r| format!("{} {} {}-{}\n", r.id, r.keyword, r.span.start, r.span.end))
        .collect();
    (
        catalogue.requirements.len(),
        sha256_hex(list.as_bytes()),
        sha256_hex(to_json(&catalogue).unwrap().as_bytes()),
    )
}

/// Profile, RFC, requirement count, requirement-list digest, catalogue digest.
const LOCKED: &[(&str, u32, usize, &str, &str)] = &[
    (
        "time",
        868,
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "1edabd925f010ee1a7e55f6f7cd7c7213fac6cbfdb973dce7eb212ea66caf28e",
    ),
    (
        "dns",
        1035,
        0,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "b8cf391e8d1705e06d9e2a267cbad8416125876fd41fc912dd0996cf46ce96eb",
    ),
    (
        "ntp",
        5905,
        13,
        "1b90a6e93d435413b5928fc50126104641bb239a5eb61735fccd14a2ee2c6d2c",
        "b4082a73cc0358a5c780a0bbb7acd23a9ba4dca503e46c605edaf3897e94da8d",
    ),
    (
        "websocket",
        6455,
        196,
        "17951ea0ab2c11444ff3445508566f05688dcaa639244399eb94b398e4982101",
        "dcb3f3cd7c8bdd11705e59064c11dbe459b45449c016c6780262b340fcaffef5",
    ),
];

#[test]
fn the_text_path_catalogues_of_the_locked_rfcs_keep_every_requirement_id_level_and_span() {
    let actual: Vec<(&str, u32, usize, String, String)> = LOCKED
        .iter()
        .map(|&(profile, rfc, ..)| {
            let (count, list, catalogue) = pinned(profile, rfc);
            (profile, rfc, count, list, catalogue)
        })
        .collect();
    let expected: Vec<(&str, u32, usize, String, String)> = LOCKED
        .iter()
        .map(|&(profile, rfc, count, list, catalogue)| {
            (profile, rfc, count, list.to_owned(), catalogue.to_owned())
        })
        .collect();
    assert_eq!(actual, expected);
}
