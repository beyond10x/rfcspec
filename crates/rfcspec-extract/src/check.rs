//! The overlay check: every quote occurs verbatim in the source at the span it names, and every id
//! an overlay item refers to exists.

use std::collections::BTreeSet;

use rfcspec_model::{
    Catalogue, Effect, OVERLAY_FORMAT, Overlay, Span, Status, Trigger, collapse_whitespace,
};

/// Check an overlay against its catalogue and source. Returns one problem per line, each naming
/// the item; an empty list means the overlay holds.
pub fn check(overlay: &Overlay, catalogue: &Catalogue, source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut problems = Vec::new();
    if overlay.format != OVERLAY_FORMAT {
        problems.push(format!(
            "overlay: format {:?}, expected {OVERLAY_FORMAT}",
            overlay.format
        ));
    }
    if overlay.rfc != catalogue.rfc {
        problems.push(format!(
            "overlay: RFC {}, catalogue RFC {}",
            overlay.rfc, catalogue.rfc
        ));
    }

    let mut quote = |item: String, quote: &str, span: Span| {
        if let Some(problem) = quote_problem(&lines, quote, span) {
            problems.push(format!("{item}: {problem}"));
        }
    };
    for requirement in &overlay.requirements {
        quote(
            format!("requirement {}", requirement.id),
            &requirement.quote,
            requirement.span,
        );
    }
    for layout in &overlay.layouts {
        quote(
            format!("layout {}", layout.name),
            &layout.quote,
            layout.span,
        );
    }
    for finding in &overlay.findings {
        quote(
            format!("finding {}", finding.id),
            &finding.quote,
            finding.span,
        );
    }

    let mut seen = BTreeSet::new();
    for requirement in &overlay.requirements {
        if !seen.insert(requirement.id.as_str()) {
            problems.push(format!("requirement {}: duplicate id", requirement.id));
        }
    }

    let mut elements = BTreeSet::new();
    for layout in &overlay.layouts {
        elements.insert(format!("layout.{}", layout.name));
    }
    let requirement_ids: BTreeSet<&str> =
        overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    let step_ids: BTreeSet<&str> = catalogue
        .dialogue_steps
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    for machine in &overlay.machines {
        let messages: BTreeSet<&str> = machine.messages.iter().map(|m| m.name.as_str()).collect();
        for participant in &machine.participants {
            let states: BTreeSet<&str> = participant.states.iter().map(String::as_str).collect();
            for transition in &participant.transitions {
                let name = transition.name();
                let item = format!("transition {}.{}.{name}", machine.name, participant.name);
                elements.insert(format!("{}.{}.{name}", machine.name, participant.name));
                for id in &transition.implements {
                    if !requirement_ids.contains(id.as_str()) && !step_ids.contains(id.as_str()) {
                        problems.push(format!(
                            "{item}: implements {id}, which is neither a requirement nor a dialogue step"
                        ));
                    }
                }
                for state in [&transition.from, &transition.to] {
                    if !states.contains(state.as_str()) {
                        problems.push(format!("{item}: unknown state {state}"));
                    }
                }
                let mut used = Vec::new();
                match &transition.on {
                    Trigger::Receive(message) => used.push(message),
                    Trigger::Input(input) => {
                        if !participant.inputs.contains(input) {
                            problems.push(format!("{item}: unknown input {input}"));
                        }
                    }
                }
                for effect in &transition.effects {
                    if let Effect::Send(send) = effect {
                        used.push(&send.message);
                    }
                }
                for message in used {
                    if !messages.contains(message.as_str()) {
                        problems.push(format!("{item}: unknown message {message}"));
                    }
                }
            }
        }
    }
    for requirement in &overlay.requirements {
        if let Status::Satisfied(element) = &requirement.status
            && !elements.contains(element)
        {
            problems.push(format!(
                "requirement {}: satisfied by {element}, which is no layout or transition",
                requirement.id
            ));
        }
    }

    let items: BTreeSet<&str> = catalogue
        .examples
        .iter()
        .map(|e| e.id.as_str())
        .chain(catalogue.prose_candidates.iter().map(|c| c.id.as_str()))
        .chain(requirement_ids.iter().copied())
        .chain(step_ids.iter().copied())
        .collect();
    for finding in &overlay.findings {
        if !items.contains(finding.item.as_str()) {
            problems.push(format!(
                "finding {}: item {} is not in the catalogue",
                finding.id, finding.item
            ));
        }
    }
    problems
}

fn quote_problem(lines: &[&str], quote: &str, span: Span) -> Option<String> {
    if span.start == 0 || span.start > span.end || span.end > lines.len() {
        return Some(format!(
            "{span} is outside the source ({} lines)",
            lines.len()
        ));
    }
    let quote = collapse_whitespace(quote);
    if quote.is_empty() {
        return Some("empty quote".into());
    }
    let text = collapse_whitespace(&lines[span.start - 1..span.end].join("\n"));
    (!text.contains(&quote)).then(|| format!("quote {quote:?} does not occur at {span}"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rfcspec_model::read_yaml;

    use super::*;
    use crate::tests::{catalogue, source};

    fn overlay() -> Overlay {
        read_yaml(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time/overlay.yaml"),
        )
        .unwrap()
    }

    #[test]
    fn the_committed_overlay_holds() {
        assert_eq!(
            check(&overlay(), &catalogue(), &source()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_quote_changed_by_one_character_is_named() {
        let mut overlay = overlay();
        let requirement = overlay
            .requirements
            .iter_mut()
            .find(|r| r.id == "R-TCP-3")
            .unwrap();
        requirement.quote = requirement.quote.replacen("refuse", "refuze", 1);
        let problems = check(&overlay, &catalogue(), &source());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0].starts_with("requirement R-TCP-3: quote"),
            "{problems:#?}"
        );
    }

    #[test]
    fn a_quote_at_the_wrong_span_is_named() {
        let mut overlay = overlay();
        overlay.findings[0].span = Span { start: 95, end: 95 };
        let problems = check(&overlay, &catalogue(), &source());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(problems[0].starts_with("finding F-1:"), "{problems:#?}");
    }

    #[test]
    fn the_overlay_holds_the_reviewed_parts() {
        let overlay = overlay();
        assert!(overlay.requirements.iter().all(|r| r.source == "prose"));
        assert_eq!(overlay.requirements.len(), 6);
        assert_eq!(overlay.layouts.len(), 1);
        let machines: Vec<&str> = overlay.machines.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(machines, ["tcp", "udp"]);
        let finding = &overlay.findings[0];
        assert_eq!(finding.item, "E5");
        let example = catalogue()
            .examples
            .into_iter()
            .find(|e| e.id == "E5")
            .unwrap();
        assert_eq!(example.seconds, -1297728000);
        assert!(u32::try_from(example.seconds).is_err());
    }
}
