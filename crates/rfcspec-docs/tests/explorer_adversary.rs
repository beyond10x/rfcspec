//! Adversarial cases over the committed explorer shards: the app links a number when its shard
//! does not list it in `not_in_index`, so every such number must have a row in the shard of its
//! hundred, and every listed one must have none; the shard file of a number is the one the app
//! computes (`explorer/src/lib/shard.ts`, `shardFile`) at the hundred boundaries.

use std::{collections::BTreeSet, fs, path::PathBuf};

use rfcspec_docs::explorer;
use serde_json::Value;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `shardFile` of `explorer/src/lib/shard.ts`, transcribed.
fn app_shard_file(rfc: u64) -> String {
    format!("{:05}.json", rfc / 100 * 100)
}

#[test]
fn the_writer_and_the_app_name_the_same_shard_at_the_hundred_boundaries() {
    for (rfc, file) in [
        (1u32, "00000.json"),
        (99, "00000.json"),
        (100, "00100.json"),
        (9999, "09900.json"),
        (10000, "10000.json"),
        (10099, "10000.json"),
        (10100, "10100.json"),
    ] {
        assert_eq!(app_shard_file(u64::from(rfc)), file);
        assert_eq!(
            explorer::path(rfc / 100 * 100),
            format!("{}/{file}", explorer::DIR),
            "RFC {rfc}"
        );
    }
}

#[test]
fn every_number_a_view_links_has_a_row_and_every_unlinked_one_has_none() {
    let dir = repository().join(explorer::DIR);
    let mut rows: BTreeSet<u64> = BTreeSet::new();
    let mut shards = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let shard: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        for rfc in shard["rfcs"].as_array().unwrap() {
            let n = rfc["rfc"].as_u64().unwrap();
            assert_eq!(app_shard_file(n), name, "RFC {n} is in {name}");
            rows.insert(n);
        }
        shards.push((name, shard));
    }
    let mut problems = Vec::new();
    for (name, shard) in &shards {
        let absent: BTreeSet<u64> = shard["not_in_index"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap())
            .collect();
        for n in &absent {
            if rows.contains(n) {
                problems.push(format!("{name}: {n} listed as not in the index has a row"));
            }
        }
        for rfc in shard["rfcs"].as_array().unwrap() {
            let named = ["obsoletes", "obsoleted_by", "updates", "updated_by"]
                .iter()
                .flat_map(|key| rfc[*key].as_array().unwrap().iter())
                .chain(
                    rfc["references"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|r| &r["target_rfc"]),
                )
                .map(|n| n.as_u64().unwrap());
            for n in named {
                if !absent.contains(&n) && !rows.contains(&n) {
                    problems.push(format!(
                        "{name}: RFC {} links {n}, which has no row",
                        rfc["rfc"]
                    ));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
