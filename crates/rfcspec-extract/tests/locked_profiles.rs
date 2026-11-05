//! The text-path catalogues of the RFCs the profiles lock (story:text-reader-structure). Their
//! overlays key statuses by catalogue id and their generated code carries source line numbers,
//! so a change to the text reader or the extraction must not move one of their requirements.
//! Both digests were pinned against the reader before that story changed it.

use std::{fs, path::PathBuf};

use rfcspec_extract::extract;
use rfcspec_model::{Catalogue, sha256_hex, to_json};

/// The text-path catalogue of `profiles/<profile>`'s locked `rfc<N>.txt`.
fn catalogue(profile: &str, rfc: u32) -> Catalogue {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../profiles")
        .join(profile)
        .join(format!("sources/rfc{rfc}.txt"));
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let text = String::from_utf8(bytes.clone()).unwrap();
    extract(&rfcspec_doc::read(rfc, &sha256_hex(&bytes), &text))
}

/// What is pinned of the text-path catalogue of `profiles/<profile>`'s locked `rfc<N>.txt`: how
/// many requirements it lists, the SHA-256 of its requirement list (one `<id> <level>
/// <start>-<end>` line per requirement) and the SHA-256 of the whole catalogue as JSON, which
/// also pins every requirement's text and the ids, texts and spans of the prose candidates,
/// dialogue steps and examples (RFC 868 and 1035 have no BCP 14 requirement; their overlays key
/// prose candidates). The grammars (story:catalogue-grammars) are left out of the catalogue
/// digest: they are collected from figures, and the digest pins what the overlays key.
/// [`grammars_pinned`] pins them instead.
fn pinned(profile: &str, rfc: u32) -> (usize, String, String) {
    let mut catalogue = catalogue(profile, rfc);
    catalogue.grammars.clear();
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

/// What is pinned of the grammars of a locked text-path catalogue (story:catalogue-grammars):
/// each grammar's id and span, in document order, and the SHA-256 of the grammars as JSON, which
/// also pins their names, sections, rules and digests.
fn grammars_pinned(profile: &str, rfc: u32) -> (Vec<(String, usize, usize)>, String) {
    let catalogue = catalogue(profile, rfc);
    (
        catalogue
            .grammars
            .iter()
            .map(|g| (g.id.clone(), g.span.start, g.span.end))
            .collect(),
        sha256_hex(to_json(&catalogue.grammars).unwrap().as_bytes()),
    )
}

/// Profile, RFC, the grammars' ids and spans, the grammars' digest.
type GrammarPin = (
    &'static str,
    u32,
    &'static [(&'static str, usize, usize)],
    &'static str,
);

const GRAMMARS: &[GrammarPin] = &[
    (
        "time",
        868,
        &[],
        "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570",
    ),
    (
        "dns",
        1035,
        &[],
        "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570",
    ),
    (
        "ntp",
        5905,
        &[
            ("rfc5905/grammar/6", 825, 830),
            ("rfc5905/grammar/8", 1600, 1604),
            ("rfc5905/grammar/10", 2093, 2143),
            ("rfc5905/grammar/11.2.3", 2508, 2583),
            ("rfc5905/grammar/A.5.1", 4435, 4546),
            ("rfc5905/grammar/A.5.5.1", 5129, 5178),
            ("rfc5905/grammar/A.5.5.6", 5513, 5626),
        ],
        "0caddb94d3e79029d729164cf458b64445085311f2d64a84112d2e8e9dd22e29",
    ),
    (
        "websocket",
        6455,
        &[
            ("rfc6455/grammar/3", 741, 747),
            ("rfc6455/grammar/4.2.2", 1321, 1327),
            ("rfc6455/grammar/4.3", 1384, 1423),
            ("rfc6455/grammar/5.2", 1687, 1783),
            ("rfc6455/grammar/5.3", 1826, 1827),
            ("rfc6455/grammar/9.1", 2671, 2679),
        ],
        "26a4ff782273d15b13cf2b4cac9c89644c9bc6a5ae3ea8dfc8d2521a94d329cd",
    ),
];

#[test]
fn the_text_path_catalogues_of_the_locked_rfcs_keep_every_grammar_id_span_and_text() {
    let actual: Vec<_> = GRAMMARS
        .iter()
        .map(|&(profile, rfc, ..)| (profile, rfc, grammars_pinned(profile, rfc)))
        .collect();
    let expected: Vec<_> = GRAMMARS
        .iter()
        .map(|&(profile, rfc, spans, digest)| {
            let spans = spans
                .iter()
                .map(|&(id, start, end)| (id.to_owned(), start, end))
                .collect();
            (profile, rfc, (spans, digest.to_owned()))
        })
        .collect();
    assert_eq!(actual, expected);
}
