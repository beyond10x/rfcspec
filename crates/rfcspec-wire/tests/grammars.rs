use rfcspec_model::{Span, text_digest};
use rfcspec_wire::grammars;

const RULES: [&str; 2] = ["greeting = \"hi\" SP name", "name     = 1*ALPHA"];

fn synthetic() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rfc version="3" number="99999">
<front><title>A synthetic document</title></front>
<middle>
<section anchor="top" pn="section-2">
<name>Greetings</name>
<t>A paragraph before the subsection.</t>
<section anchor="intro" pn="section-2.1">
<name>Introduction</name>
<t>The rules follow.</t>
<sourcecode type="abnf">{}
{}</sourcecode>
<t>A figure without a type follows.</t>
<sourcecode>hello there
  goodbye</sourcecode>
</section>
</section>
</middle>
</rfc>
"#,
        RULES[0], RULES[1]
    )
}

#[test]
fn an_rfcxml_section_gives_one_grammar_from_its_abnf_figure_only() {
    let source = synthetic();
    let document = rfcspec_doc::xml::read(99999, "00", &source).unwrap();

    let found = grammars::collect(&document);

    assert_eq!(found.len(), 1, "{found:#?}");
    let grammar = &found[0];
    let rules = RULES.join("\n");
    assert_eq!(grammar.id, "rfc99999/grammar/2.1");
    assert_eq!(grammar.rfc, 99999);
    assert_eq!(grammar.name, "2.1");
    assert_eq!(grammar.section.as_deref(), Some("intro"));
    assert_eq!(grammar.rules, rules);
    assert_eq!(grammar.uses, Vec::new());
    assert_eq!(grammar.digest, text_digest(&rules));
    let line = |text: &str| source.lines().position(|l| l.contains(text)).unwrap() + 1;
    assert_eq!(
        grammar.span,
        Span {
            start: line(RULES[0]),
            end: line(RULES[1]),
        }
    );
}

#[test]
fn a_text_section_without_a_figure_type_gives_its_rule_figures_by_their_rule_lines() {
    let text = "\
1.  Introduction

   Some prose.

      first = %x31

   More prose that is not a rule.

      second = %x32

2.  Diagrams

      +-+-+
      |A|B|
      +-+-+
";
    let document = rfcspec_doc::read(99998, "00", text);

    let found = grammars::collect(&document);

    let ids: Vec<&str> = found.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(ids, ["rfc99998/grammar/1"]);
    let grammar = &found[0];
    assert_eq!(grammar.section.as_deref(), Some("1"));
    assert_eq!(grammar.span, Span { start: 5, end: 9 });
    assert!(grammar.rules.contains("first = %x31"));
    assert!(grammar.rules.contains("second = %x32"));
    assert!(!grammar.rules.contains("prose"), "{}", grammar.rules);
}

/// A one-section RFCXML document holding one figure of the given `type`.
fn typed(kind: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rfc version="3" number="99997">
<front><title>A synthetic document</title></front>
<middle>
<section anchor="typed" pn="section-3">
<name>Typed</name>
<sourcecode type="{kind}">{body}</sourcecode>
</section>
</middle>
</rfc>
"#
    )
}

#[test]
fn an_rfcxml_figure_typed_pseudocode_gives_no_grammar_even_with_a_rule_line() {
    let document = rfcspec_doc::xml::read(99997, "00", &typed("pseudocode", "x = 1")).unwrap();

    assert_eq!(grammars::collect(&document), Vec::new());
}

#[test]
fn an_rfcxml_figure_typed_abnf9110_gives_a_grammar() {
    let rule = "field-name = token";
    let document = rfcspec_doc::xml::read(99997, "00", &typed("abnf9110", rule)).unwrap();

    let found = grammars::collect(&document);

    let ids: Vec<&str> = found.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(ids, ["rfc99997/grammar/3"]);
    assert_eq!(found[0].rules, rule);
    assert_eq!(found[0].section.as_deref(), Some("typed"));
}
