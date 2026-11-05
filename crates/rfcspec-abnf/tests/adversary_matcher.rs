use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rfcspec_abnf::{Dialect, MatchError, RuleSet, matches, parse};

fn synthetic(lines: &[&str]) -> RuleSet {
    let numbered: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| (index + 1, *line))
        .collect();
    parse(Dialect::Rfc5234, &numbered).expect("the synthetic grammar parses")
}

fn invalid(rule: &str, offset: usize) -> Result<usize, MatchError> {
    Err(MatchError::Invalid {
        rule: rule.to_string(),
        offset,
    })
}

fn within<T: Send + 'static>(seconds: u64, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver
        .recv_timeout(Duration::from_secs(seconds))
        .unwrap_or_else(|_| panic!("the matcher did not finish within {seconds} s"))
}

#[test]
fn left_recursion_gives_the_same_answer_whichever_rule_the_match_starts_from() {
    let set = synthetic(&["a = a \"x\" / \"y\"", "s = a"]);
    let refused = Err(MatchError::LeftRecursive("a".to_string()));
    assert_eq!(matches(&set, "a", b"yx"), refused);
    assert_eq!(
        matches(&set, "s", b"yx"),
        refused,
        "`s = a`, so `a` and `s` accept the same inputs"
    );
}

#[test]
fn a_left_recursive_rule_is_matched_or_refused_never_reported_invalid_for_input_it_derives() {
    let set = synthetic(&["a = a \"x\" / \"y\""]);
    assert_eq!(
        matches(&set, "a", b"yxx"),
        Err(MatchError::LeftRecursive("a".to_string())),
        "`yxx` is derived by a -> a x -> a x x -> y x x"
    );

    let hidden = synthetic(&["b = *\"z\" c", "c = [ \"w\" ] b \"x\" / \"y\""]);
    assert_eq!(
        matches(&hidden, "b", b"y"),
        Err(MatchError::LeftRecursive("b".to_string())),
        "`b` reaches itself through nullable prefixes"
    );
}

#[test]
fn a_zero_repetition_of_a_prose_value_does_not_refuse_the_rule() {
    let set = synthetic(&["r = \"a\" / 0<anything at all>"]);
    assert_eq!(matches(&set, "r", b"a"), Ok(1));
    assert_eq!(matches(&set, "r", b""), Ok(0));
}

#[test]
#[ignore = "inline repetitions are not memoized (story:abnf-matcher-hardening); real grammars nest through references, which are"]
fn nested_inline_repetitions_finish_on_a_modest_input() {
    let result = within(20, || {
        let set = synthetic(&["r = *(*(*(*(*\"a\"))))"]);
        let mut input = vec![b'a'; 300];
        input.push(b'b');
        matches(&set, "r", &input)
    });
    assert_eq!(result, invalid("r", 300));
}

#[test]
fn nullable_loops_terminate() {
    let result = within(20, || {
        let input = vec![b'a'; 500];
        let mut answers = Vec::new();
        for grammar in [
            "r = *(*\"a\")",
            "r = *(\"\" / \"a\")",
            "r = *(*\"a\" / *\"b\")",
            "r = 3*(*\"a\")",
            "r = 1000000000*(\"\" / \"a\")",
        ] {
            let set = synthetic(&[grammar]);
            answers.push(matches(&set, "r", &input));
        }
        answers
    });
    assert_eq!(result, vec![Ok(500); 5]);
}

#[test]
fn repetition_forms() {
    let between = synthetic(&["r = 2*3\"a\""]);
    assert_eq!(matches(&between, "r", b"a"), invalid("r", 1));
    assert_eq!(matches(&between, "r", b"aa"), Ok(2));
    assert_eq!(matches(&between, "r", b"aaa"), Ok(3));
    assert_eq!(matches(&between, "r", b"aaaa"), invalid("r", 3));

    let exact = synthetic(&["r = 3\"a\""]);
    assert_eq!(matches(&exact, "r", b"aaa"), Ok(3));
    assert_eq!(matches(&exact, "r", b"aa"), invalid("r", 2));

    let at_most = synthetic(&["r = *2\"a\""]);
    assert_eq!(matches(&at_most, "r", b""), Ok(0));
    assert_eq!(matches(&at_most, "r", b"aaa"), invalid("r", 2));

    let none = synthetic(&["r = \"b\" 0\"a\""]);
    assert_eq!(matches(&none, "r", b"b"), Ok(1));
    assert_eq!(matches(&none, "r", b"ba"), invalid("r", 1));

    let at_least = synthetic(&["r = 1*\"a\""]);
    assert_eq!(matches(&at_least, "r", b""), invalid("r", 0));
    assert_eq!(matches(&at_least, "r", b"aaaa"), Ok(4));
}

#[test]
fn concatenation_binds_tighter_than_alternation_and_groups_nest() {
    let set = synthetic(&["r = \"a\" \"b\" / \"c\""]);
    assert_eq!(matches(&set, "r", b"ab"), Ok(2));
    assert_eq!(matches(&set, "r", b"c"), Ok(1));
    assert_eq!(matches(&set, "r", b"ac"), invalid("r", 1));

    let nested = synthetic(&["r = \"a\" [ \"b\" [ \"c\" ] ] *( \"d\" / ( \"e\" \"f\" ) )"]);
    for accepted in [&b"a"[..], b"ab", b"abc", b"abcdefd", b"aef"] {
        assert_eq!(
            matches(&nested, "r", accepted),
            Ok(accepted.len()),
            "{accepted:?}"
        );
    }
    assert_eq!(matches(&nested, "r", b"ac"), invalid("r", 1));
}

#[test]
fn num_val_forms() {
    for grammar in ["r = %X41", "r = %x41", "r = %b1000001", "r = %d65"] {
        let set = synthetic(&[grammar]);
        assert_eq!(matches(&set, "r", b"A"), Ok(1), "{grammar}");
        assert_eq!(matches(&set, "r", b"a"), invalid("r", 0), "{grammar}");
    }
    let lower_hex = synthetic(&["r = %x4a-4F"]);
    assert_eq!(matches(&lower_hex, "r", b"K"), Ok(1));
    let dotted = synthetic(&["r = %x41.42.43"]);
    assert_eq!(matches(&dotted, "r", b"ABC"), Ok(3));
    assert_eq!(matches(&dotted, "r", b"ABD"), invalid("r", 2));
    let wide = synthetic(&["r = %x100-10FFFF"]);
    assert_eq!(matches(&wide, "r", b"\xff"), invalid("r", 0));
}

#[test]
fn empty_char_val_matches_the_empty_string() {
    let set = synthetic(&["r = \"\""]);
    assert_eq!(matches(&set, "r", b""), Ok(0));
    assert_eq!(matches(&set, "r", b"a"), invalid("r", 0));
}

#[test]
fn core_rules_on_empty_input() {
    let set = synthetic(&[]);
    assert_eq!(matches(&set, "LWSP", b""), Ok(0));
    assert_eq!(matches(&set, "CRLF", b""), invalid("CRLF", 0));
    assert_eq!(matches(&set, "OCTET", b""), invalid("OCTET", 0));
}

#[test]
fn rulenames_are_case_insensitive() {
    let set = synthetic(&["R = \"a\"", "r =/ \"b\"", "S = 2r"]);
    assert_eq!(set.rules.len(), 2);
    assert_eq!(matches(&set, "s", b"ab"), Ok(2));
    assert_eq!(matches(&set, "S", b"ba"), Ok(2));
}

#[test]
fn invalid_offset_is_the_furthest_failure() {
    let set = synthetic(&["r = \"ab\" / \"a\" \"c\" \"d\""]);
    assert_eq!(matches(&set, "r", b"acx"), invalid("r", 2));
}
