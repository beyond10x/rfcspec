//! Adversarial cases for signed fields in the multi-field generator: the generated crate for
//! signed widths other than 8 must compile, pass its own tests and be rustfmt-clean.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use rfcspec_model::{Catalogue, Field, Overlay, read_yaml};

fn ntp() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/ntp")
}

fn inputs() -> (Overlay, Catalogue) {
    let source = fs::read_to_string(ntp().join("sources/rfc5905.txt")).unwrap();
    let catalogue = rfcspec_extract::extract(&rfcspec_doc::read(5905, "digest", &source));
    let overlay: Overlay = read_yaml(&ntp().join("overlay.yaml")).unwrap();
    (overlay, catalogue)
}

fn field(name: &str, bits: u32, signed: bool) -> Field {
    Field {
        name: name.to_owned(),
        bits,
        signed,
        order: "big-endian".to_owned(),
        presence: None,
        constraints: Vec::new(),
    }
}

fn library(fields: Vec<Field>) -> String {
    let (mut overlay, catalogue) = inputs();
    overlay.layouts[0].fields = fields;
    let files = rfcspec_gen::rust(&overlay, &catalogue, "ntp").unwrap();
    files
        .into_iter()
        .find(|g| g.file == "src/lib.rs")
        .unwrap()
        .text
}

/// What rustfmt (edition 2024, default 100 columns) makes of `text`.
fn rustfmt(text: &str) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

fn first_difference(a: &str, b: &str) -> String {
    a.lines()
        .zip(b.lines())
        .find(|(x, y)| x != y)
        .map(|(x, y)| format!("generated: {x}\nrustfmt:   {y}"))
        .unwrap_or_default()
}

/// Compile the generated library's tests with rustc and run them.
fn run_generated_tests(dir: &Path, text: &str) -> (bool, String) {
    fs::create_dir_all(dir).unwrap();
    let lib = dir.join("lib.rs");
    fs::write(&lib, text).unwrap();
    let bin = dir.join("generated-tests");
    let compiled = Command::new("rustc")
        .args(["--edition", "2024", "--test", "-o"])
        .arg(&bin)
        .arg(&lib)
        .output()
        .unwrap();
    if !compiled.status.success() {
        return (
            false,
            String::from_utf8_lossy(&compiled.stderr).into_owned(),
        );
    }
    let ran = Command::new(&bin).output().unwrap();
    (
        ran.status.success(),
        String::from_utf8_lossy(&ran.stdout).into_owned(),
    )
}

#[test]
fn signed_fields_of_widths_2_12_16_17_33_63_and_64_generate_a_crate_whose_tests_pass() {
    let text = library(vec![
        field("a", 2, true),
        field("pad", 6, false),
        field("b", 12, true),
        field("c", 4, false),
        field("d", 16, true),
        field("e", 17, true),
        field("p2", 15, false),
        field("f", 33, true),
        field("p3", 31, false),
        field("g", 64, true),
        field("h", 63, true),
        field("p4", 1, false),
    ]);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary-signed-widths");
    let (ok, output) = run_generated_tests(&dir, &text);
    assert!(ok, "{output}");
    for name in [
        "vector_a_minus_2_accepted",
        "vector_b_minus_2048_accepted",
        "vector_d_minus_32768_accepted",
        "vector_e_minus_65536_accepted",
        "vector_f_minus_4294967296_accepted",
        "vector_g_minus_9223372036854775808_accepted",
        "vector_g_9223372036854775807_accepted",
        "vector_h_minus_4611686018427387904_accepted",
    ] {
        assert!(
            output.contains(&format!("{name} ... ok")),
            "{name}\n{output}"
        );
    }
    assert_eq!(
        text,
        rustfmt(&text),
        "{}",
        first_difference(&text, &rustfmt(&text))
    );
}

#[test]
fn a_signed_field_with_a_long_name_generates_rustfmt_clean_code() {
    let text = library(vec![
        field("leap", 2, false),
        field("version", 3, false),
        field("mode", 3, false),
        field("stratum", 8, false),
        field("poll-interval-in-log2-seconds", 8, true),
        field("precision", 8, true),
    ]);
    let formatted = rustfmt(&text);
    assert!(text == formatted, "{}", first_difference(&text, &formatted));
}

/// The generation error for the ntp overlay with its fields replaced and `length` set.
fn refusal(fields: Vec<Field>, length: &[&str]) -> String {
    let (mut overlay, catalogue) = inputs();
    overlay.layouts[0].fields = fields;
    overlay.layouts[0].length = length.iter().map(|s| (*s).to_owned()).collect();
    rfcspec_gen::rust(&overlay, &catalogue, "ntp")
        .err()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "generated without error".to_owned())
}

const SIGNED_REFUSAL: &str = "layout header: field s is signed; a signed field takes no constraint, presence or length and selects no field";

#[test]
fn a_signed_field_with_a_range_constraint_is_refused() {
    let mut s = field("s", 8, true);
    s.constraints = vec![rfcspec_model::Constraint::Range([-5, 5])];
    assert_eq!(refusal(vec![s], &[]), SIGNED_REFUSAL);
}

#[test]
fn a_signed_field_with_presence_is_refused() {
    let mut s = field("s", 8, true);
    s.presence = Some(rfcspec_model::Presence {
        field: "k".to_owned(),
        equals: 1,
    });
    assert_eq!(refusal(vec![field("k", 8, false), s], &[]), SIGNED_REFUSAL);
}

#[test]
fn a_signed_field_that_selects_a_field_is_refused() {
    let mut x = field("x", 8, false);
    x.presence = Some(rfcspec_model::Presence {
        field: "s".to_owned(),
        equals: 1,
    });
    assert_eq!(refusal(vec![field("s", 8, true), x], &[]), SIGNED_REFUSAL);
}

#[test]
fn a_settled_base_whose_selector_refuses_0_generates_a_crate_whose_tests_pass() {
    use rfcspec_model::{Constraint, Presence};
    let mut k = field("k", 3, false);
    k.constraints = vec![Constraint::Range([1, 3])];
    let mut x = field("x", 5, false);
    x.presence = Some(Presence {
        field: "k".to_owned(),
        equals: 1,
    });
    x.constraints = vec![Constraint::Range([2, 5])];
    let mut y = field("y", 5, false);
    y.presence = Some(Presence {
        field: "k".to_owned(),
        equals: 0,
    });
    let mut e = field("e", 8, false);
    e.constraints = vec![Constraint::Equals(3)];
    let mut o = field("o", 8, false);
    o.constraints = vec![Constraint::OneOf(vec![2, 7])];
    let text = library(vec![k, x, y, field("s", 3, true), e, o]);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary-settled-base");
    let (ok, output) = run_generated_tests(&dir, &text);
    assert!(ok, "{output}");
    assert!(output.contains("test result: ok. 16 passed"), "{output}");
}

#[test]
fn a_signed_length_field_is_refused() {
    assert_eq!(refusal(vec![field("s", 8, true)], &["s"]), SIGNED_REFUSAL);
}
