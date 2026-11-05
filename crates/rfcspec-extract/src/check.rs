//! The overlay check: every quote occurs verbatim in the source at the span it names, every id
//! an overlay item refers to exists, and a requirement read mechanically (`capitals-text`,
//! `abnf-comment`) keeps its catalogue id and lies within its catalogue span.

use std::collections::{BTreeMap, BTreeSet};

use rfcspec_model::{
    ABNF_COMMENT, Bcp14Requirement, CAPITALS_TEXT, Catalogue, Constraint, Effect, OVERLAY_FORMAT,
    Overlay, Property, Span, Status, Trigger, Value, collapse_whitespace,
};

/// Check an overlay against its catalogue and source. Returns one problem per line, each naming
/// the item; an empty list means the overlay holds.
pub fn check(overlay: &Overlay, catalogue: &Catalogue, source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let furniture = rfcspec_doc::furniture_lines(source);
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
        if let Some(problem) = quote_problem(&lines, &furniture, quote, span) {
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
        let mut earlier = BTreeSet::new();
        for field in &layout.fields {
            let item = format!("layout {} field {}", layout.name, field.name);
            if let Some(presence) = &field.presence
                && !earlier.contains(presence.field.as_str())
            {
                problems.push(format!(
                    "{item}: present when {}, which is no earlier field of the layout",
                    presence.field
                ));
            }
            if !(1..=64).contains(&field.bits) {
                problems.push(format!("{item}: {} bits, not 1 to 64", field.bits));
            } else {
                let (low, high) = field.bounds();
                for constraint in &field.constraints {
                    if let Constraint::Range([from, to]) = constraint
                        && !(low <= i128::from(*from) && from <= to && i128::from(*to) <= high)
                    {
                        problems.push(format!(
                            "{item}: range {from} to {to} does not fit {} bits",
                            field.bits
                        ));
                    }
                }
            }
            earlier.insert(field.name.as_str());
        }
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
                let mut timers = Vec::new();
                let trigger_fields = match &transition.on {
                    Trigger::Receive(message) => {
                        used.push(message);
                        machine
                            .messages
                            .iter()
                            .find(|m| &m.name == message)
                            .map(|m| m.fields.clone())
                            .unwrap_or_default()
                    }
                    Trigger::Input(input) => match participant.input(input) {
                        Some(input) => input.fields(),
                        None => {
                            problems.push(format!("{item}: unknown input {input}"));
                            BTreeMap::new()
                        }
                    },
                    Trigger::Timer(timer) => {
                        timers.push(timer);
                        BTreeMap::new()
                    }
                };
                for effect in &transition.effects {
                    match effect {
                        Effect::Send(send) => {
                            used.push(&send.message);
                            for (field, value) in &send.payload {
                                if let Value::Field(source) = value
                                    && !trigger_fields.contains_key(&source.input)
                                {
                                    problems.push(format!(
                                        "{item}: payload {field} reads {}, which is no field of its trigger",
                                        source.input
                                    ));
                                }
                            }
                        }
                        Effect::Arm(arm) => timers.push(&arm.timer),
                        Effect::Cancel(timer) => timers.push(timer),
                        Effect::Emit(_) | Effect::Close(_) | Effect::Flush(_) => {}
                    }
                }
                for message in used {
                    if !messages.contains(message.as_str()) {
                        problems.push(format!("{item}: unknown message {message}"));
                    }
                }
                for timer in timers {
                    if !participant.timers.contains(timer) {
                        problems.push(format!("{item}: unknown timer {timer}"));
                    }
                }
            }
        }
        let participants: BTreeSet<&str> = machine
            .participants
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        for property in &machine.properties {
            let participant = match property {
                Property::EventCount(count) => &count.participant,
                Property::FlushBeforeClose(flush) => &flush.flush_before_close,
            };
            if !participants.contains(participant.as_str()) {
                problems.push(format!(
                    "property {}.{}: unknown participant {participant}",
                    machine.name,
                    property.name()
                ));
            }
        }
    }

    let catalogued: BTreeMap<&str, &Bcp14Requirement> = catalogue
        .requirements
        .iter()
        .map(|r| (r.id.as_str(), r))
        .collect();
    for requirement in overlay
        .requirements
        .iter()
        .filter(|r| r.source == CAPITALS_TEXT || r.source == ABNF_COMMENT)
    {
        match catalogued.get(requirement.id.as_str()) {
            None => problems.push(format!(
                "requirement {}: source {}, but {} is no catalogue requirement",
                requirement.id, requirement.source, requirement.id
            )),
            Some(found)
                if found.span.start > requirement.span.start
                    || requirement.span.end > found.span.end =>
            {
                problems.push(format!(
                    "requirement {}: {} does not lie within the catalogue requirement's {}",
                    requirement.id, requirement.span, found.span
                ));
            }
            Some(_) => {}
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

/// Whether a quote occurs at its span; page-furniture lines inside the span are skipped, so a
/// sentence continued across a page break is quoted as one.
fn quote_problem(lines: &[&str], furniture: &[usize], quote: &str, span: Span) -> Option<String> {
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
    let kept: Vec<&str> = (span.start..=span.end)
        .filter(|line| !furniture.contains(line))
        .map(|line| lines[line - 1])
        .collect();
    let text = collapse_whitespace(&kept.join("\n"));
    (!text.contains(&quote)).then(|| format!("quote {quote:?} does not occur at {span}"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rfcspec_model::{Requirement, read_yaml};

    use super::*;
    use crate::tests::{catalogue, catalogue_6455, source, source_6455};

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

    fn websocket_overlay(requirements: Vec<Requirement>) -> Overlay {
        Overlay {
            format: OVERLAY_FORMAT.into(),
            rfc: 6455,
            kind: None,
            requirements,
            layouts: vec![],
            machines: vec![],
            findings: vec![],
        }
    }

    fn reviewed(id: &str, quote: &str, start: usize, end: usize) -> Requirement {
        Requirement {
            id: id.into(),
            source: "capitals-text".into(),
            level: "SHOULD".into(),
            transports: vec![],
            quote: quote.into(),
            span: Span { start, end },
            status: Status::NotApplicable("test".into()),
            probe: None,
        }
    }

    fn committed_websocket() -> Overlay {
        read_yaml(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/websocket/overlay.yaml"),
        )
        .unwrap()
    }

    #[test]
    fn the_committed_websocket_overlay_holds() {
        assert_eq!(
            check(&committed_websocket(), &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_unknown_timer_and_a_payload_field_its_trigger_lacks_are_named() {
        let mut overlay = committed_websocket();
        let client = &mut overlay.machines[0].participants[0];
        client.timers.clear();
        let Effect::Send(send) = &mut client.transitions[0].effects[0] else {
            panic!("the first effect sends");
        };
        send.payload.insert(
            "reason".into(),
            Value::Field(rfcspec_model::FieldRef {
                input: "reason".into(),
            }),
        );
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        let start = "transition close.client.R7.1.2-1-start-closing-handshake";
        assert!(
            problems.contains(&format!(
                "{start}: payload reason reads reason, which is no field of its trigger"
            )),
            "{problems:#?}"
        );
        assert!(
            problems.contains(&format!("{start}: unknown timer close-timeout")),
            "{problems:#?}"
        );
    }

    #[test]
    fn a_quote_across_a_page_break_skips_the_furniture_inside_its_span() {
        let quote = "The list of extensions that are needed SHOULD appear in the /reason/ part of the Close frame.";
        let overlay = websocket_overlay(vec![reviewed("R7.4.1-5", quote, 2572, 2583)]);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
        let glued = websocket_overlay(vec![reviewed(
            "R7.4.1-5",
            "The list of extensions that Fette & Melnikov",
            2572,
            2583,
        )]);
        assert_eq!(check(&glued, &catalogue_6455(), &source_6455()).len(), 1);
    }
}
