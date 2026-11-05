use rfcspec_abnf::{Dialect, Error, Node, RuleSet, matches, parse};

fn at(lines: &[(usize, &str)]) -> Result<RuleSet, Error> {
    parse(Dialect::Rfc5234, lines)
}

fn syntax(result: Result<RuleSet, Error>) -> (usize, usize) {
    match result {
        Err(Error::Syntax { line, column, .. }) => (line, column),
        other => panic!("expected a syntax error, got {other:?}"),
    }
}

#[test]
fn a_semicolon_inside_a_char_val_is_not_a_comment() {
    let set = at(&[(10, "r = \";\" ; the note")]).expect("parses");
    assert_eq!(
        set.rules[0].definition,
        Node::CharVal {
            text: ";".to_string(),
            case_sensitive: false
        }
    );
    assert_eq!(set.rules[0].comments.len(), 1);
    assert_eq!(set.rules[0].comments[0].line, 10);
    assert_eq!(set.rules[0].comments[0].text, "the note");
}

#[test]
fn rule_like_text_inside_char_val_prose_or_comment_does_not_start_a_rule() {
    let set = at(&[(1, "r = \"s = t\""), (2, "    / \"w\" ; x = y")]).expect("parses");
    assert_eq!(set.rules.len(), 1);
    assert_eq!(matches(&set, "r", b"w"), Ok(1));
    assert_eq!(matches(&set, "r", b"S = T"), Ok(5));

    let prose = at(&[(1, "r = <u = v> / \"w\"")]).expect("parses");
    assert_eq!(prose.rules.len(), 1);
    assert_eq!(
        prose.rules[0].definition,
        Node::Alternation(vec![
            Node::Prose("u = v".to_string()),
            Node::CharVal {
                text: "w".to_string(),
                case_sensitive: false
            },
        ])
    );
}

#[test]
fn incremental_before_definition_names_its_line_and_column() {
    assert_eq!(
        syntax(at(&[(7, "  r =/ \"a\""), (8, "  r = \"b\"")])),
        (7, 3)
    );
}

#[test]
fn duplicate_definition_names_the_second_rule() {
    assert_eq!(syntax(at(&[(7, "r = \"a\""), (9, "R = \"b\"")])), (9, 1));
}

#[test]
fn unexpected_close_names_its_column() {
    assert_eq!(syntax(at(&[(40, "r = \"a\" )")])), (40, 9));
}

#[test]
fn trailing_slash_names_the_end_of_the_figure() {
    assert_eq!(syntax(at(&[(3, "r = \"a\" /"), (4, "")])), (3, 10));
}

#[test]
fn unclosed_group_names_its_open() {
    assert_eq!(
        syntax(at(&[(5, "r = ( \"a\" / \"b\""), (6, "s = \"c\"")])),
        (5, 5)
    );
}

#[test]
fn indented_continuation_and_mid_line_rule_start() {
    let set = at(&[(1, "r = \"a\""), (2, "      \"b\" s = \"c\"")]).expect("parses");
    assert_eq!(set.rules.len(), 2);
    assert_eq!(matches(&set, "r", b"ab"), Ok(2));
    assert_eq!(matches(&set, "s", b"c"), Ok(1));
    assert_eq!(set.rules[1].line, 2);
}
