use rfcspec_abnf::{Dialect, MATCH_DEPTH_LIMIT, MatchError, matches, parse};

#[test]
fn nested_groups_two_thousand_deep_do_not_overflow_the_stack() {
    assert_eq!(
        nested(2000),
        Err(MatchError::TooDeep {
            rule: "nest".to_string(),
            limit: MATCH_DEPTH_LIMIT,
        })
    );
}

#[test]
fn nested_groups_two_hundred_deep_do_not_overflow_the_stack() {
    assert_eq!(nested(200), Ok(400));
}

fn nested(depth: usize) -> Result<usize, MatchError> {
    let set = parse(
        Dialect::Rfc5234,
        &[(1, "nest = \"(\" *( \"x\" / nest ) \")\"")],
    )
    .expect("the synthetic grammar parses");
    let mut input = vec![b'('; depth];
    input.extend(std::iter::repeat_n(b')', depth));
    matches(&set, "nest", &input)
}
