use rfcspec_abnf::{Dialect, Error, MatchError, RuleImport, RuleSet, matches, parse};
use rfcspec_model::{BlockKind, Document};

fn document() -> Document {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../profiles/websocket/sources/rfc6455.txt"
    );
    let text = std::fs::read_to_string(path).expect("the locked RFC 6455 source");
    rfcspec_doc::read(6455, "", &text)
}

fn figure(document: &Document, start: usize, end: usize) -> Vec<(usize, String)> {
    let block = document
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Figure && block.span.start == start)
        .unwrap_or_else(|| panic!("a figure starting on line {start}"));
    assert_eq!(block.span.end, end, "the figure starting on line {start}");
    block
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| (start + index, line.clone()))
        .collect()
}

fn figures_of(document: &Document, section: &str, first_line: usize) -> Vec<(usize, String)> {
    document
        .blocks
        .iter()
        .filter(|block| {
            block.kind == BlockKind::Figure
                && block.section.as_deref() == Some(section)
                && block.span.start >= first_line
        })
        .flat_map(|block| {
            block
                .lines
                .iter()
                .enumerate()
                .map(|(index, line)| (block.span.start + index, line.clone()))
        })
        .collect()
}

fn borrowed(lines: &[(usize, String)]) -> Vec<(usize, &str)> {
    lines
        .iter()
        .map(|(number, text)| (*number, text.as_str()))
        .collect()
}

fn synthetic(lines: &[&str]) -> RuleSet {
    let numbered: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| (index + 1, *line))
        .collect();
    parse(Dialect::Rfc5234, &numbered).expect("the synthetic grammar parses")
}

fn import(rfc: u32, rule: &str) -> RuleImport {
    RuleImport {
        rfc,
        rule: rule.to_string(),
    }
}

fn invalid(rule: &str, offset: usize) -> Result<usize, MatchError> {
    Err(MatchError::Invalid {
        rule: rule.to_string(),
        offset,
    })
}

#[test]
fn section_3_uri_figures_parse_into_six_rules_with_four_imports() {
    let document = document();
    let mut lines = figure(&document, 741, 742);
    lines.extend(figure(&document, 744, 747));
    let set = parse(Dialect::Rfc5234, &borrowed(&lines)).expect("section 3 parses");

    let names: Vec<&str> = set.rules.iter().map(|rule| rule.name.as_str()).collect();
    assert_eq!(
        names,
        ["ws-URI", "wss-URI", "host", "port", "path", "query"]
    );
    assert_eq!(
        set.imports(),
        [
            import(3986, "host"),
            import(3986, "port"),
            import(3986, "path-abempty"),
            import(3986, "query"),
        ]
    );
    assert!(set.unresolved().is_empty(), "{:?}", set.unresolved());
    assert_eq!(
        matches(&set, "ws-URI", b"ws://example/"),
        Err(MatchError::Import(import(3986, "host")))
    );
}

#[test]
fn section_5_3_masking_figure_is_refused_naming_its_line() {
    let document = document();
    let lines = figure(&document, 1826, 1827);
    match parse(Dialect::Rfc5234, &borrowed(&lines)) {
        Err(Error::Syntax { line, .. }) => assert_eq!(line, 1826),
        other => panic!("expected a syntax error on line 1826, got {other:?}"),
    }
}

#[test]
fn frame_fin_keeps_each_comment_on_its_line() {
    let document = document();
    let lines = figure(&document, 1701, 1703);
    let set = parse(Dialect::Rfc5234, &borrowed(&lines)).expect("frame-fin parses");
    assert_eq!(set.rules.len(), 1);
    let rule = &set.rules[0];
    assert_eq!(rule.name, "frame-fin");
    assert_eq!(rule.line, 1701);
    let comments: Vec<(usize, &str)> = rule
        .comments
        .iter()
        .map(|comment| (comment.line, comment.text.as_str()))
        .collect();
    assert_eq!(
        comments,
        [
            (1701, "more frames of this message follow"),
            (1702, "final frame of this message"),
            (1703, "1 bit in length"),
        ]
    );
}

#[test]
fn bits_and_rfc2616_dispatch_to_their_unsupported_stubs() {
    let document = document();

    let frame = figures_of(&document, "5.2", 1687);
    assert!(!frame.is_empty());
    let error = parse(Dialect::Bits, &borrowed(&frame)).expect_err("bits is a stub");
    assert_eq!(
        error,
        Error::Unsupported {
            dialect: Dialect::Bits
        }
    );
    assert!(error.to_string().contains("bits"), "{error}");

    let handshake = figures_of(&document, "4.3", 0);
    assert!(!handshake.is_empty());
    let error = parse(Dialect::Rfc2616, &borrowed(&handshake)).expect_err("rfc2616 is a stub");
    assert_eq!(
        error,
        Error::Unsupported {
            dialect: Dialect::Rfc2616
        }
    );
    assert!(error.to_string().contains("rfc2616"), "{error}");
}

#[test]
fn case_sensitive_and_insensitive_char_vals() {
    let sensitive = synthetic(&["r = %s\"Ab\""]);
    assert_eq!(matches(&sensitive, "r", b"Ab"), Ok(2));
    assert_eq!(matches(&sensitive, "r", b"ab"), invalid("r", 0));

    for set in [synthetic(&["r = %i\"Ab\""]), synthetic(&["r = \"Ab\""])] {
        assert_eq!(matches(&set, "r", b"Ab"), Ok(2));
        assert_eq!(matches(&set, "r", b"ab"), Ok(2));
    }
}

#[test]
fn incremental_alternative_extends_the_rule() {
    let set = synthetic(&["r = \"a\"", "r =/ \"b\""]);
    assert_eq!(set.rules.len(), 1);
    assert_eq!(matches(&set, "r", b"a"), Ok(1));
    assert_eq!(matches(&set, "r", b"b"), Ok(1));
    assert_eq!(matches(&set, "r", b"c"), invalid("r", 0));
}

#[test]
fn num_val_ranges_and_concatenations() {
    let range = synthetic(&["r = %x30-39"]);
    assert_eq!(matches(&range, "r", b"5"), Ok(1));
    assert_eq!(matches(&range, "r", b"a"), invalid("r", 0));

    let sequence = synthetic(&["r = %d13.10"]);
    assert_eq!(matches(&sequence, "r", b"\r\n"), Ok(2));
    assert_eq!(matches(&sequence, "r", b"\r "), invalid("r", 1));
}

#[test]
fn a_prose_value_that_is_no_import_is_listed_as_unresolved() {
    let set = synthetic(&[
        "r = \"a\" / <some prose> / missing",
        "s = <t, defined in [RFC3986]>",
    ]);
    assert_eq!(set.imports(), [import(3986, "t")]);
    assert_eq!(set.unresolved(), ["<some prose>", "missing"]);
}

type CoreCase = (
    &'static str,
    &'static [u8],
    Result<usize, MatchError>,
    &'static [u8],
    Result<usize, MatchError>,
);

#[test]
fn every_core_rule_accepts_one_input_and_refuses_another() {
    let set = synthetic(&[]);
    let cases: [CoreCase; 16] = [
        ("ALPHA", b"Z", Ok(1), b"1", invalid("ALPHA", 0)),
        ("BIT", b"1", Ok(1), b"2", invalid("BIT", 0)),
        ("CHAR", b"~", Ok(1), b"\x00", invalid("CHAR", 0)),
        ("CR", b"\r", Ok(1), b"\n", invalid("CR", 0)),
        ("CRLF", b"\r\n", Ok(2), b"\n\r", invalid("CRLF", 0)),
        ("CTL", b"\x7f", Ok(1), b"a", invalid("CTL", 0)),
        ("DIGIT", b"9", Ok(1), b"a", invalid("DIGIT", 0)),
        ("DQUOTE", b"\"", Ok(1), b"'", invalid("DQUOTE", 0)),
        ("HEXDIG", b"f", Ok(1), b"g", invalid("HEXDIG", 0)),
        ("HTAB", b"\t", Ok(1), b" ", invalid("HTAB", 0)),
        ("LF", b"\n", Ok(1), b"\r", invalid("LF", 0)),
        ("LWSP", b" \r\n\t", Ok(4), b"\r\n", invalid("LWSP", 2)),
        ("OCTET", b"\xff", Ok(1), b"", invalid("OCTET", 0)),
        ("SP", b" ", Ok(1), b"\t", invalid("SP", 0)),
        ("VCHAR", b"~", Ok(1), b" ", invalid("VCHAR", 0)),
        ("WSP", b"\t", Ok(1), b"\r", invalid("WSP", 0)),
    ];
    for (rule, accepted, accepted_result, refused, refused_result) in cases {
        assert_eq!(
            matches(&set, rule, accepted),
            accepted_result,
            "{rule} on {accepted:?}"
        );
        assert_eq!(
            matches(&set, rule, refused),
            refused_result,
            "{rule} on {refused:?}"
        );
    }
}
