//! The overlay check: every quote occurs verbatim in the source at the span it names, every id
//! an overlay item refers to exists, and a requirement read mechanically (`capitals-text`,
//! `abnf-comment`) keeps its catalogue id and lies within its catalogue span; a participant's
//! local fields have distinct names, a guard compares operands of one type and a `set` writes a
//! value of its field's type (a literal has its own type, a local operand its field's, an input
//! operand its trigger field's); a machine's declared channel capacity is a positive integer; a
//! timer armed with `millis` has a positive interval within the machine's `max_time_ms`; a
//! timer armed with a backoff is declared by its participant, armed by it with `millis` on some
//! transition (else it has no prior interval to multiply), and has a positive factor and a
//! positive ceiling within the machine's `max_time_ms` (declared or the lowering default); and
//! every review decision judges the current catalogue text of its item.

use std::collections::{BTreeMap, BTreeSet};

use rfcspec_model::{
    ABNF_COMMENT, Bcp14Requirement, CAPITALS_TEXT, Catalogue, Constraint, DecisionKind, Edit,
    Effect, Interval, Layout, OVERLAY_FORMAT, Operand, Overlay, Property, Span, Status, Trigger,
    Value, collapse_whitespace, text_digest,
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
        problems.extend(layout_problems(layout));
    }
    let requirement_ids: BTreeSet<&str> =
        overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    let implementable: BTreeSet<&str> = requirement_ids
        .iter()
        .copied()
        .chain(catalogue.requirements.iter().map(|r| r.id.as_str()))
        .collect();
    let step_ids: BTreeSet<&str> = catalogue
        .dialogue_steps
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    for machine in &overlay.machines {
        if machine.capacity == Some(0) {
            problems.push(format!(
                "machine {}: capacity 0 is not a positive integer",
                machine.name
            ));
        }
        let messages: BTreeSet<&str> = machine.messages.iter().map(|m| m.name.as_str()).collect();
        for participant in &machine.participants {
            let states: BTreeSet<&str> = participant.states.iter().map(String::as_str).collect();
            let mut names = BTreeSet::new();
            for field in &participant.fields {
                let item = format!(
                    "local field {}.{}.{}",
                    machine.name, participant.name, field.name
                );
                if !names.insert(field.name.as_str()) {
                    problems.push(format!("{item}: a second local field of this name"));
                }
                let article = match field.kind.as_str() {
                    "Integer" => "an",
                    "Boolean" => "a",
                    other => {
                        problems.push(format!("{item}: type {other}, not Integer or Boolean"));
                        continue;
                    }
                };
                if field.initial.kind() != field.kind {
                    problems.push(format!(
                        "{item}: initial {} is not {article} {}",
                        field.initial, field.kind
                    ));
                }
            }
            let armed_with_millis: BTreeSet<&str> = participant
                .transitions
                .iter()
                .flat_map(|transition| &transition.effects)
                .filter_map(|effect| match effect {
                    Effect::Arm(arm) if matches!(arm.interval, Interval::Fixed { .. }) => {
                        Some(arm.timer.as_str())
                    }
                    _ => None,
                })
                .collect();
            for transition in &participant.transitions {
                let name = transition.name();
                let item = format!("transition {}.{}.{name}", machine.name, participant.name);
                elements.insert(format!("{}.{}.{name}", machine.name, participant.name));
                for id in &transition.implements {
                    if !implementable.contains(id.as_str()) && !step_ids.contains(id.as_str()) {
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
                for operand in transition
                    .guards
                    .iter()
                    .flat_map(|guard| [&guard.left, &guard.right])
                {
                    match operand {
                        Operand::Local(local) if participant.field(&local.local).is_none() => {
                            problems.push(format!(
                                "{item}: guard reads {}, which is no local field of the participant",
                                local.local
                            ));
                        }
                        Operand::Input(source) if !trigger_fields.contains_key(&source.input) => {
                            problems.push(format!(
                                "{item}: guard reads {}, which is no field of its trigger",
                                source.input
                            ));
                        }
                        _ => {}
                    }
                }
                let typed = |operand: &Operand| -> Option<(String, String)> {
                    match operand {
                        Operand::Integer(n) => Some((n.to_string(), "Integer".into())),
                        Operand::Boolean(b) => Some((b.to_string(), "Boolean".into())),
                        Operand::Local(local) => participant
                            .field(&local.local)
                            .map(|f| (format!("local {}", local.local), f.kind.clone())),
                        Operand::Input(source) => trigger_fields
                            .get(&source.input)
                            .map(|kind| (format!("input {}", source.input), kind.clone())),
                    }
                };
                for guard in &transition.guards {
                    if let (Some((left, left_kind)), Some((right, right_kind))) =
                        (typed(&guard.left), typed(&guard.right))
                        && left_kind != right_kind
                    {
                        problems.push(format!(
                            "{item}: guard compares {left} ({left_kind}) with {right} ({right_kind})"
                        ));
                    }
                }
                let typed_value = |value: &Value| -> Option<(String, String)> {
                    match value {
                        Value::Integer(n) => Some((n.to_string(), "Integer".to_owned())),
                        Value::Boolean(b) => Some((b.to_string(), "Boolean".to_owned())),
                        Value::Field(source) => trigger_fields
                            .get(&source.input)
                            .map(|kind| (format!("input {}", source.input), kind.clone())),
                    }
                };
                for effect in &transition.effects {
                    match effect {
                        Effect::Set(set) => {
                            match participant.field(&set.field) {
                                None => problems.push(format!(
                                    "{item}: set {}, which is no local field of the participant",
                                    set.field
                                )),
                                Some(field) => {
                                    if let Some((value, kind)) = typed_value(&set.value)
                                        && kind != field.kind
                                    {
                                        problems.push(format!(
                                            "{item}: set {} ({}) to {value} ({kind})",
                                            set.field, field.kind
                                        ));
                                    }
                                }
                            }
                            if let Value::Field(source) = &set.value
                                && !trigger_fields.contains_key(&source.input)
                            {
                                problems.push(format!(
                                    "{item}: set {} reads {}, which is no field of its trigger",
                                    set.field, source.input
                                ));
                            }
                        }
                        Effect::Send(send) => {
                            used.push(&send.message);
                            let message_fields = machine
                                .messages
                                .iter()
                                .find(|m| m.name == send.message)
                                .map(|m| &m.fields);
                            for (field, value) in &send.payload {
                                if let Some(field_kind) =
                                    message_fields.and_then(|fields| fields.get(field))
                                    && let Some((value, kind)) = typed_value(value)
                                    && &kind != field_kind
                                {
                                    problems.push(format!(
                                        "{item}: send {} payload {field} ({field_kind}) given {value} ({kind})",
                                        send.message
                                    ));
                                }
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
                        Effect::Arm(arm) => {
                            timers.push(&arm.timer);
                            if let Interval::Fixed { millis } = arm.interval {
                                let head = format!("{item}: arm {} with millis", arm.timer);
                                let max_time_ms = machine.bounds().max_time_ms;
                                if millis == 0 {
                                    problems.push(format!("{head} 0, not a positive integer"));
                                } else if millis > u64::from(max_time_ms) {
                                    problems.push(format!(
                                        "{head} {millis} above the machine's max_time_ms {max_time_ms}"
                                    ));
                                }
                            }
                            if let Interval::Backoff(backoff) = &arm.interval {
                                let head = format!("{item}: arm {} with backoff", arm.timer);
                                if participant.timers.contains(&arm.timer)
                                    && !armed_with_millis.contains(arm.timer.as_str())
                                {
                                    problems.push(format!(
                                        "{head}, but the participant never arms {} with millis, so it has no prior interval",
                                        arm.timer
                                    ));
                                }
                                if backoff.factor == 0 {
                                    problems
                                        .push(format!("{head} factor 0, not a positive integer"));
                                }
                                let max_time_ms = machine.bounds().max_time_ms;
                                if backoff.ceiling_ms == 0 {
                                    problems.push(format!(
                                        "{head} ceiling_ms 0, not a positive integer"
                                    ));
                                } else if backoff.ceiling_ms > u64::from(max_time_ms) {
                                    problems.push(format!(
                                        "{head} ceiling_ms {} above the machine's max_time_ms {max_time_ms}",
                                        backoff.ceiling_ms
                                    ));
                                }
                            }
                        }
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
            let item = format!("property {}.{}", machine.name, property.name());
            let named: Vec<(&String, Option<&String>)> = match property {
                Property::EventCount(count) => vec![(&count.participant, None)],
                Property::FlushBeforeClose(flush) => vec![(&flush.flush_before_close, None)],
                Property::NeverState(never) => vec![(&never.participant, Some(&never.state))],
                Property::StateImplication(implication) => vec![
                    (&implication.participant, Some(&implication.state)),
                    (&implication.other, Some(&implication.other_state)),
                ],
            };
            for (participant, state) in named {
                if !participants.contains(participant.as_str()) {
                    problems.push(format!("{item}: unknown participant {participant}"));
                } else if let Some(state) = state
                    && !machine
                        .participants
                        .iter()
                        .any(|p| &p.name == participant && p.states.contains(state))
                {
                    problems.push(format!(
                        "{item}: unknown state {state} of participant {participant}"
                    ));
                }
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
    problems.extend(decisions(overlay, catalogue));
    problems
}

/// The layout rules on one layout, each problem naming the layout and the field: every width is
/// 1 to 64 bits, a presence names an earlier field and a value that field's width holds, and
/// every value of every constraint (`equals`, `one_of`, both ends of a `range`) fits the field's
/// width, as `rfcspec generate` requires.
pub fn layout_problems(layout: &Layout) -> Vec<String> {
    let mut problems = Vec::new();
    let mut earlier: BTreeMap<&str, &rfcspec_model::Field> = BTreeMap::new();
    for field in &layout.fields {
        let item = format!("layout {} field {}", layout.name, field.name);
        if let Some(presence) = &field.presence {
            match earlier.get(presence.field.as_str()) {
                None => problems.push(format!(
                    "{item}: present when {}, which is no earlier field of the layout",
                    presence.field
                )),
                Some(named)
                    if (1..=64).contains(&named.bits) && !fits(named.bounds(), presence.equals) =>
                {
                    problems.push(format!(
                        "{item}: present when {} equals {}, which does not fit its {} bits",
                        presence.field, presence.equals, named.bits
                    ));
                }
                Some(_) => {}
            }
        }
        if !(1..=64).contains(&field.bits) {
            problems.push(format!("{item}: {} bits, not 1 to 64", field.bits));
        } else {
            let bounds = field.bounds();
            let bits = field.bits;
            for constraint in &field.constraints {
                match constraint {
                    Constraint::Range([from, to]) => {
                        if !(fits(bounds, *from) && from <= to && fits(bounds, *to)) {
                            problems.push(format!(
                                "{item}: range {from} to {to} does not fit {bits} bits"
                            ));
                        }
                    }
                    Constraint::Equals(value) => {
                        if !fits(bounds, *value) {
                            problems
                                .push(format!("{item}: equals {value} does not fit {bits} bits"));
                        }
                    }
                    Constraint::OneOf(values) => {
                        for value in values.iter().filter(|v| !fits(bounds, **v)) {
                            problems.push(format!(
                                "{item}: one_of value {value} does not fit {bits} bits"
                            ));
                        }
                    }
                }
            }
        }
        earlier.insert(field.name.as_str(), field);
    }
    problems
}

/// Whether a value lies within a field's inclusive bounds.
fn fits((low, high): (i128, i128), value: i64) -> bool {
    (low..=high).contains(&i128::from(value))
}

/// What a decision's item names: a catalogue item of the overlay's RFC (`rfc<N>/<id>`) and its
/// text, a layout (`rfc<N>/layout/<name>`), or the whole profile (`<profile>`, the item of a
/// machine proposal).
enum Decided<'a> {
    Catalogue(&'a str, &'a str),
    Layout(&'a str),
    Profile,
    Unknown,
}

/// The decision rules (ESS `rfcspec.review.Decision`): each decision's item is a catalogue item
/// of the overlay's RFC, a layout or the whole profile; a catalogue item's `text_digest` is the
/// digest of its current catalogue text (a layout's or machine's judges the sections its proposal
/// was asked about, which `rfcspec check` compares through the proposal); one item holds at most
/// one decision per digest, one proposal at most one decision; an `edit` carries an entry of its
/// item's kind; a `reject`, and only a `reject`, carries a non-blank reason; an `accept` or `edit`
/// of a catalogue item has its requirement in `requirements`, of a layout the layout in
/// `layouts`, and an `edit` the entry it carries (a machine `edit` an equal machine in
/// `machines`). Each problem names the decision's item.
fn decisions(overlay: &Overlay, catalogue: &Catalogue) -> Vec<String> {
    let texts: BTreeMap<&str, &str> = catalogue
        .requirements
        .iter()
        .map(|r| (r.id.as_str(), r.text.as_str()))
        .chain(
            catalogue
                .prose_candidates
                .iter()
                .map(|c| (c.id.as_str(), c.text.as_str())),
        )
        .collect();
    let prefix = format!("rfc{}/", overlay.rfc);
    let mut problems = Vec::new();
    let mut keys = BTreeSet::new();
    let mut proposals = BTreeSet::new();
    for decision in &overlay.decisions {
        let item = format!("decision {}", decision.item);
        let decided = match decision.item.strip_prefix(&prefix) {
            Some(id) => match (id.strip_prefix("layout/"), texts.get_key_value(id)) {
                (Some(name), _) if !name.is_empty() => Decided::Layout(name),
                (_, Some((id, text))) => Decided::Catalogue(id, text),
                _ => Decided::Unknown,
            },
            None if !decision.item.is_empty() && !decision.item.contains('/') => Decided::Profile,
            None => Decided::Unknown,
        };
        match decided {
            Decided::Unknown => problems.push(format!(
                "{item}: no catalogue item of RFC {} has this id",
                overlay.rfc
            )),
            Decided::Catalogue(_, text) => {
                let digest = text_digest(text);
                if decision.text_digest != digest {
                    problems.push(format!(
                        "{item}: text_digest {} is not {digest}, the digest of the item's catalogue text",
                        decision.text_digest
                    ));
                }
            }
            Decided::Layout(_) | Decided::Profile => {}
        }
        if !keys.insert((decision.item.as_str(), decision.text_digest.as_str())) {
            problems.push(format!(
                "{item}: a second decision under text_digest {}",
                decision.text_digest
            ));
        }
        if let Some(proposal) = &decision.proposal_id
            && !proposals.insert(proposal.as_str())
        {
            problems.push(format!(
                "{item}: proposal {proposal} already has a decision"
            ));
        }
        let kind = decision.kind;
        match (kind, decision.reason.as_deref()) {
            (DecisionKind::Reject, None) => {
                problems.push(format!("{item}: reject decision carries no reason"));
            }
            (DecisionKind::Reject, Some(reason)) if reason.trim().is_empty() => {
                problems.push(format!("{item}: reject decision carries an empty reason"));
            }
            (DecisionKind::Reject, Some(_)) | (_, None) => {}
            (_, Some(_)) => problems.push(format!(
                "{item}: {kind} decision carries a reason; only a reject does"
            )),
        }
        let settles = matches!(kind, DecisionKind::Accept | DecisionKind::Edit);
        match (&decided, &decision.edit) {
            (_, None) if kind == DecisionKind::Edit => {
                problems.push(format!("{item}: edit decision carries no entry"));
            }
            (Decided::Catalogue(id, _), edit) if settles => {
                match (overlay.requirements.iter().find(|r| r.id == *id), edit) {
                    (None, _) => problems.push(format!(
                        "{item}: {kind} decision, but requirements holds no {id}"
                    )),
                    (Some(_), Some(edit)) if edit.requirement().is_none() => {
                        problems.push(format!(
                            "{item}: edit decision carries a {}, not a requirement entry",
                            edit.kind()
                        ));
                    }
                    (Some(requirement), Some(Edit::Requirement(edit))) if requirement != edit => {
                        problems.push(format!(
                            "{item}: edit decision, but requirement {} differs from the entry it carries",
                            requirement.id
                        ));
                    }
                    _ => {}
                }
            }
            (Decided::Layout(name), edit) if settles => {
                match (overlay.layouts.iter().find(|l| l.name == *name), edit) {
                    (_, Some(edit)) if !matches!(edit, Edit::Layout(_)) => {
                        problems.push(format!(
                            "{item}: edit decision carries a {}, not a layout",
                            edit.kind()
                        ));
                    }
                    (None, _) => problems.push(format!(
                        "{item}: {kind} decision, but layouts holds no {name}"
                    )),
                    (Some(layout), Some(Edit::Layout(edit))) if layout != edit => {
                        problems.push(format!(
                            "{item}: edit decision, but layout {name} differs from the one it carries"
                        ));
                    }
                    _ => {}
                }
            }
            (Decided::Profile, Some(edit)) if kind == DecisionKind::Edit => match edit {
                Edit::Machine(machine) if !overlay.machines.contains(machine) => {
                    problems.push(format!(
                        "{item}: edit decision, but machines holds no machine equal to the one it carries"
                    ));
                }
                Edit::Machine(_) => {}
                other => problems.push(format!(
                    "{item}: edit decision carries a {}, not a machine",
                    other.kind()
                )),
            },
            (Decided::Unknown, None) if settles => problems.push(format!(
                "{item}: {kind} decision, but requirements holds no {}",
                decision
                    .item
                    .strip_prefix(&prefix)
                    .unwrap_or(&decision.item)
            )),
            _ => {}
        }
    }
    problems
}

/// Whether a quote occurs at its span of `lines`, the source split into lines; the 1-based
/// page-furniture lines inside the span are skipped, so a sentence continued across a page break
/// is quoted as one. `None` when it does; otherwise the problem, as `rfcspec check` reports it.
pub fn quote_problem(
    lines: &[&str],
    furniture: &[usize],
    quote: &str,
    span: Span,
) -> Option<String> {
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

    use rfcspec_model::{
        Backoff, Decision, DecisionKind, FieldRef, Guard, Interval, Literal, LocalField, LocalRef,
        NeverState, Operand, Requirement, Set, StateImplication, read_yaml, text_digest,
    };

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
            decisions: vec![],
        }
    }

    const R5_1_1: &str = "a client MUST mask all frames that it sends to the server";
    const R5_1_2: &str =
        "The server MUST close the connection upon receiving a frame that is not masked.";

    fn catalogue_text(id: &str) -> String {
        catalogue_6455()
            .requirements
            .into_iter()
            .find(|r| r.id == id)
            .unwrap()
            .text
    }

    fn decision(id: &str, kind: DecisionKind, proposal: &str) -> Decision {
        Decision {
            item: format!("rfc6455/{id}"),
            text_digest: text_digest(&catalogue_text(id)),
            kind,
            proposal_id: Some(proposal.into()),
            edit: None,
            reason: (kind == DecisionKind::Reject).then(|| "the level is wrong".into()),
        }
    }

    /// `R5.1-1` accepted, `R5.1-2` edited and `R5.1-5` rejected.
    fn decided() -> Overlay {
        let edited = reviewed("R5.1-2", R5_1_2, 1493, 1496);
        let mut overlay =
            websocket_overlay(vec![reviewed("R5.1-1", R5_1_1, 1490, 1493), edited.clone()]);
        let mut edit = decision("R5.1-2", DecisionKind::Edit, "p2");
        edit.edit = Some(Edit::Requirement(edited));
        overlay.decisions = vec![
            decision("R5.1-1", DecisionKind::Accept, "p1"),
            edit,
            decision("R5.1-5", DecisionKind::Reject, "p3"),
        ];
        overlay
    }

    #[test]
    fn accept_edit_and_reject_decisions_hold() {
        assert_eq!(
            check(&decided(), &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_text_digest_that_is_not_the_catalogue_texts_is_named() {
        let mut overlay = decided();
        overlay.decisions[2].text_digest = text_digest("another text");
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0].starts_with("decision rfc6455/R5.1-5: text_digest"),
            "{problems:#?}"
        );
    }

    #[test]
    fn a_reject_without_a_reason_and_a_reason_on_another_kind_are_named() {
        for (reason, says) in [(None, "no reason"), (Some(" \n"), "an empty reason")] {
            let mut overlay = decided();
            overlay.decisions[2].reason = reason.map(str::to_owned);
            let problems = check(&overlay, &catalogue_6455(), &source_6455());
            assert_eq!(problems.len(), 1, "{problems:#?}");
            assert!(
                problems[0].starts_with("decision rfc6455/R5.1-5: reject decision carries")
                    && problems[0].contains(says),
                "{problems:#?}"
            );
        }
        let mut overlay = decided();
        overlay.decisions[0].reason = Some("fine".into());
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0].starts_with("decision rfc6455/R5.1-1: accept decision carries a reason"),
            "{problems:#?}"
        );
    }

    #[test]
    fn two_decisions_on_one_proposal_are_named() {
        let mut overlay = decided();
        overlay.decisions[2].proposal_id = Some("p1".into());
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0].starts_with("decision rfc6455/R5.1-5: proposal p1"),
            "{problems:#?}"
        );
    }

    #[test]
    fn an_accept_or_edit_without_its_requirement_is_named() {
        let mut overlay = decided();
        overlay.requirements.clear();
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(
            problems[0].starts_with("decision rfc6455/R5.1-1: accept"),
            "{problems:#?}"
        );
        assert!(
            problems[1].starts_with("decision rfc6455/R5.1-2: edit"),
            "{problems:#?}"
        );
    }

    #[test]
    fn an_edit_without_its_entry_and_an_item_outside_the_catalogue_are_named() {
        let mut overlay = decided();
        overlay.decisions[1].edit = None;
        overlay.decisions[2].item = "rfc6455/R99-1".into();
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(
            problems[0].starts_with("decision rfc6455/R5.1-2: edit"),
            "{problems:#?}"
        );
        assert!(
            problems[1].starts_with("decision rfc6455/R99-1:"),
            "{problems:#?}"
        );
    }

    /// The `frame-header` layout of the proposal fixture profile `no-machines`.
    fn fixture_layout() -> rfcspec_model::Layout {
        let overlay: Overlay = read_yaml(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../rfcspec-propose/tests/fixtures/no-machines/overlay.yaml"),
        )
        .unwrap();
        overlay.layouts[0].clone()
    }

    /// A Ping/Pong machine implementing requirements of sections 5.5.2 and 5.5.3 that have no
    /// overlay status.
    fn ping_pong() -> rfcspec_model::Machine {
        rfcspec_model::from_yaml(
            "name: ping-pong\ntransport: tcp\nmessages:\n- {name: ping, fields: {data: Integer}}\n- {name: pong, fields: {data: Integer}}\nparticipants:\n- name: client\n  initial: open\n  states: [open, pinging]\n  inputs: [{name: ping, fields: {data: Integer}}]\n  transitions:\n  - {implements: [R5.5.2-4], label: send-ping, from: open, to: pinging, on: {input: ping}, effects: [{send: {message: ping, payload: {data: {input: data}}}}]}\n  - {implements: [R5.5.2-2], label: pong-received, from: pinging, to: open, on: {receive: pong}}\n- name: server\n  initial: open\n  states: [open]\n  transitions:\n  - {implements: [R5.5.2-2, R5.5.3-1], label: pong, from: open, to: open, on: {receive: ping}, effects: [{send: {message: pong, payload: {data: {input: data}}}}]}\nproperties: []\n",
        )
        .unwrap()
    }

    /// An overlay without requirements, holding a layout decided by
    /// `layout_kind` and the `ping` machine decided by `machine_kind`.
    fn element_decisions(layout_kind: DecisionKind, machine_kind: DecisionKind) -> Overlay {
        let mut overlay = websocket_overlay(vec![]);
        let mut layout = fixture_layout();
        layout.name = "frame-header-copy".into();
        overlay.layouts = vec![layout.clone()];
        let ping = ping_pong();
        overlay.machines = vec![ping.clone()];
        overlay.decisions = vec![
            Decision {
                item: "rfc6455/layout/frame-header-copy".into(),
                text_digest: text_digest("the layout's sections"),
                kind: layout_kind,
                proposal_id: Some("p1".into()),
                edit: (layout_kind == DecisionKind::Edit).then_some(Edit::Layout(layout)),
                reason: (layout_kind == DecisionKind::Reject).then(|| "a duplicate".into()),
            },
            Decision {
                item: "websocket".into(),
                text_digest: text_digest("the machine's sections"),
                kind: machine_kind,
                proposal_id: Some("p2".into()),
                edit: (machine_kind == DecisionKind::Edit).then_some(Edit::Machine(ping)),
                reason: (machine_kind == DecisionKind::Reject)
                    .then(|| "one machine is enough".into()),
            },
        ];
        overlay
    }

    #[test]
    fn layout_and_machine_decisions_hold_and_a_machine_implements_unreviewed_requirements() {
        for (layout, machine) in [
            (DecisionKind::Accept, DecisionKind::Accept),
            (DecisionKind::Edit, DecisionKind::Edit),
            (DecisionKind::Reject, DecisionKind::Reject),
        ] {
            assert_eq!(
                check(
                    &element_decisions(layout, machine),
                    &catalogue_6455(),
                    &source_6455()
                ),
                Vec::<String>::new(),
                "{layout} {machine}"
            );
        }
    }

    #[test]
    fn an_accepted_layout_missing_and_an_edit_of_another_kind_or_text_are_named() {
        let mut overlay = element_decisions(DecisionKind::Accept, DecisionKind::Edit);
        overlay.layouts.clear();
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0]
                .starts_with("decision rfc6455/layout/frame-header-copy: accept decision, but layouts holds no frame-header-copy"),
            "{problems:#?}"
        );

        let mut overlay = element_decisions(DecisionKind::Edit, DecisionKind::Edit);
        overlay.decisions[0].edit = overlay.decisions[1].edit.clone();
        overlay.machines[0].transport = "udp".into();
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(
            problems[0].starts_with(
                "decision rfc6455/layout/frame-header-copy: edit decision carries a machine"
            ),
            "{problems:#?}"
        );
        assert!(
            problems[1].starts_with(
                "decision websocket: edit decision, but machines holds no machine equal to the one it carries"
            ),
            "{problems:#?}"
        );
    }

    #[test]
    fn a_transition_implementing_an_id_outside_the_catalogue_is_named() {
        let mut overlay = element_decisions(DecisionKind::Reject, DecisionKind::Reject);
        overlay.machines[0].participants[0].transitions[0].implements = vec!["R99-1".into()];
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0]
                .ends_with("implements R99-1, which is neither a requirement nor a dialogue step"),
            "{problems:#?}"
        );
    }

    #[test]
    fn the_layout_rules_hold_on_one_layout() {
        let mut layout = fixture_layout();
        assert_eq!(layout_problems(&layout), Vec::<String>::new());
        layout.fields[6].bits = 65;
        assert_eq!(
            layout_problems(&layout),
            ["layout frame-header field payload-len: 65 bits, not 1 to 64"]
        );
    }

    #[test]
    fn every_constraint_and_presence_value_outside_its_field_width_is_named() {
        let mut layout = fixture_layout();
        layout.fields[7].constraints = vec![
            Constraint::Equals(65536),
            Constraint::OneOf(vec![126, 70000]),
            Constraint::Range([126, 65536]),
            Constraint::Equals(-1),
        ];
        layout.fields[8].presence.as_mut().unwrap().equals = 128;
        assert_eq!(
            layout_problems(&layout),
            [
                "layout frame-header field extended-payload-length-16: equals 65536 does not fit 16 bits",
                "layout frame-header field extended-payload-length-16: one_of value 70000 does not fit 16 bits",
                "layout frame-header field extended-payload-length-16: range 126 to 65536 does not fit 16 bits",
                "layout frame-header field extended-payload-length-16: equals -1 does not fit 16 bits",
                "layout frame-header field extended-payload-length-64: present when payload-len equals 128, which does not fit its 7 bits",
            ]
        );
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

    const START: &str = "transition close.client.R7.1.2-1-start-closing-handshake";
    const ECHOED: &str = "transition close.client.R5.5.1-9-R7.1.2-2-close-echoed";

    /// The committed websocket overlay with a local field `token` on the client, and a `set` on
    /// its first transition and a guard on its second built from the operands given.
    fn guarded(set_field: &str, set_value: Value, guard: Operand) -> Overlay {
        let mut overlay = committed_websocket();
        let client = &mut overlay.machines[0].participants[0];
        client.fields.push(LocalField {
            name: "token".into(),
            kind: "Integer".into(),
            initial: Literal::Integer(0),
        });
        client.transitions[0].effects.push(Effect::Set(Set {
            field: set_field.into(),
            value: set_value,
        }));
        client.transitions[1].guards.push(Guard {
            left: guard,
            right: Operand::Local(LocalRef {
                local: "token".into(),
            }),
        });
        overlay
    }

    fn input(field: &str) -> FieldRef {
        FieldRef {
            input: field.into(),
        }
    }

    #[test]
    fn declared_local_fields_set_and_guards_hold() {
        let overlay = guarded(
            "token",
            Value::Field(input("code")),
            Operand::Input(input("code")),
        );
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_guard_or_set_naming_an_undeclared_local_field_is_named() {
        let overlay = guarded(
            "missing",
            Value::Integer(1),
            Operand::Local(LocalRef {
                local: "absent".into(),
            }),
        );
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(
            problems,
            [
                format!("{START}: set missing, which is no local field of the participant"),
                format!("{ECHOED}: guard reads absent, which is no local field of the participant"),
            ]
        );
    }

    #[test]
    fn a_guard_whose_right_operand_names_an_undeclared_local_field_is_named() {
        let mut overlay = guarded(
            "token",
            Value::Integer(1),
            Operand::Local(LocalRef {
                local: "token".into(),
            }),
        );
        overlay.machines[0].participants[0].transitions[1].guards[0].right =
            Operand::Local(LocalRef {
                local: "absent".into(),
            });
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(
            problems,
            [format!(
                "{ECHOED}: guard reads absent, which is no local field of the participant"
            )]
        );
    }

    #[test]
    fn a_guard_or_set_reading_a_field_its_trigger_lacks_is_named() {
        let overlay = guarded(
            "token",
            Value::Field(input("reason")),
            Operand::Input(input("reason")),
        );
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(
            problems,
            [
                format!("{START}: set token reads reason, which is no field of its trigger"),
                format!("{ECHOED}: guard reads reason, which is no field of its trigger"),
            ]
        );
    }

    #[test]
    fn a_local_field_whose_initial_literal_does_not_match_its_type_is_named() {
        let mut overlay = committed_websocket();
        let client = &mut overlay.machines[0].participants[0];
        client.fields = vec![
            LocalField {
                name: "token".into(),
                kind: "Integer".into(),
                initial: Literal::Boolean(true),
            },
            LocalField {
                name: "seen".into(),
                kind: "Boolean".into(),
                initial: Literal::Integer(0),
            },
            LocalField {
                name: "text".into(),
                kind: "String".into(),
                initial: Literal::Integer(0),
            },
        ];
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(
            problems,
            [
                "local field close.client.token: initial true is not an Integer",
                "local field close.client.seen: initial 0 is not a Boolean",
                "local field close.client.text: type String, not Integer or Boolean",
            ]
        );
    }

    fn machine<'a>(overlay: &'a mut Overlay, name: &str) -> &'a mut rfcspec_model::Machine {
        overlay
            .machines
            .iter_mut()
            .find(|m| m.name == name)
            .unwrap()
    }

    #[test]
    fn a_guard_comparing_operands_of_different_types_is_named() {
        let mut overlay = committed_websocket();
        let server = &mut machine(&mut overlay, "fragments").participants[1];
        assert_eq!(server.name, "server");
        let guard = &mut server.transitions[0].guards[1];
        assert_eq!(guard.left, Operand::Input(input("fin")));
        guard.right = Operand::Integer(1);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [
                "transition fragments.server.R5.2-1-text-message: guard compares input fin (Boolean) with 1 (Integer)"
            ]
        );
    }

    #[test]
    fn a_guard_operand_takes_the_type_of_its_trigger_field_literal_or_local_field() {
        let mut overlay = committed_websocket();
        let client = &mut machine(&mut overlay, "ping").participants[0];
        client.fields.push(LocalField {
            name: "seen".into(),
            kind: "Boolean".into(),
            initial: Literal::Boolean(false),
        });
        client.transitions[1].guards = vec![
            Guard {
                left: Operand::Input(input("data")),
                right: Operand::Integer(7),
            },
            Guard {
                left: Operand::Local(LocalRef {
                    local: "seen".into(),
                }),
                right: Operand::Boolean(true),
            },
            Guard {
                left: Operand::Local(LocalRef {
                    local: "seen".into(),
                }),
                right: Operand::Input(input("data")),
            },
        ];
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [
                "transition ping.client.R5.5.2-2-pong-echoed: guard compares local seen (Boolean) with input data (Integer)"
            ]
        );
    }

    #[test]
    fn a_set_whose_value_type_differs_from_its_field_is_named() {
        let mut overlay = committed_websocket();
        let client = &mut machine(&mut overlay, "ping").participants[0];
        let Effect::Set(set) = &mut client.transitions[0].effects[0] else {
            panic!("the first effect sets");
        };
        set.value = Value::Boolean(true);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [
                "transition ping.client.R5.5.2-1-R5.5.2-4-send-ping: set ping-data (Integer) to true (Boolean)"
            ]
        );
    }

    #[test]
    fn a_send_payload_typed_differently_from_its_message_field_is_named() {
        let mut overlay = committed_websocket();
        let client = &mut machine(&mut overlay, "ping").participants[0];
        let Effect::Send(send) = &mut client.transitions[0].effects[1] else {
            panic!("the second effect sends");
        };
        assert_eq!(send.message, "ping");
        send.payload.insert("data".into(), Value::Boolean(true));
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [
                "transition ping.client.R5.5.2-1-R5.5.2-4-send-ping: send ping payload data (Integer) given true (Boolean)"
            ]
        );
    }

    #[test]
    fn a_capacity_of_zero_is_named_and_a_positive_one_holds() {
        let mut overlay = committed_websocket();
        machine(&mut overlay, "close").capacity = Some(1);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
        machine(&mut overlay, "close").capacity = Some(0);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            ["machine close: capacity 0 is not a positive integer"]
        );
    }

    /// The committed WebSocket overlay whose close client, after its fixed arm of
    /// `close-timeout`, arms `timer` with a backoff.
    fn close_backoff(timer: &str, factor: u32, ceiling_ms: u64) -> Overlay {
        let mut overlay = committed_websocket();
        let client = &mut machine(&mut overlay, "close").participants[0];
        let effects = &mut client.transitions[0].effects;
        assert!(
            matches!(&effects[1], Effect::Arm(arm) if arm.timer == "close-timeout" && matches!(arm.interval, Interval::Fixed { .. })),
            "the second effect arms close-timeout with millis"
        );
        effects.insert(
            2,
            Effect::Arm(rfcspec_model::Arm {
                timer: timer.into(),
                interval: Interval::Backoff(Backoff { factor, ceiling_ms }),
            }),
        );
        overlay
    }

    #[test]
    fn a_backoff_of_a_timer_the_participant_never_arms_with_millis_is_named() {
        let mut overlay = close_backoff("close-timeout", 2, 8000);
        machine(&mut overlay, "close").participants[0].transitions[0]
            .effects
            .remove(1);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [format!(
                "{CLOSE_START}: arm close-timeout with backoff, but the participant never arms close-timeout with millis, so it has no prior interval"
            )]
        );
    }

    const CLOSE_START: &str = "transition close.client.R7.1.2-1-start-closing-handshake";

    #[test]
    fn a_backoff_within_the_machine_bounds_holds() {
        assert_eq!(
            check(
                &close_backoff("close-timeout", 2, 30000),
                &catalogue_6455(),
                &source_6455()
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_backoff_factor_of_zero_is_named() {
        assert_eq!(
            check(
                &close_backoff("close-timeout", 0, 8000),
                &catalogue_6455(),
                &source_6455()
            ),
            [format!(
                "{CLOSE_START}: arm close-timeout with backoff factor 0, not a positive integer"
            )]
        );
    }

    #[test]
    fn a_backoff_ceiling_of_zero_is_named() {
        assert_eq!(
            check(
                &close_backoff("close-timeout", 2, 0),
                &catalogue_6455(),
                &source_6455()
            ),
            [format!(
                "{CLOSE_START}: arm close-timeout with backoff ceiling_ms 0, not a positive integer"
            )]
        );
    }

    #[test]
    fn a_backoff_ceiling_above_the_declared_or_default_max_time_is_named() {
        let mut overlay = close_backoff("close-timeout", 2, 30001);
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [format!(
                "{CLOSE_START}: arm close-timeout with backoff ceiling_ms 30001 above the machine's max_time_ms 30000"
            )]
        );
        let close = machine(&mut overlay, "close");
        close.bounds = None;
        let Effect::Arm(fixed) = &mut close.participants[0].transitions[0].effects[1] else {
            panic!("the second effect arms");
        };
        fixed.interval = Interval::Fixed { millis: 1000 };
        let Effect::Arm(arm) = &mut close.participants[0].transitions[0].effects[2] else {
            panic!("the third effect arms");
        };
        arm.interval = Interval::Backoff(Backoff {
            factor: 2,
            ceiling_ms: 1001,
        });
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            [format!(
                "{CLOSE_START}: arm close-timeout with backoff ceiling_ms 1001 above the machine's max_time_ms 1000"
            )]
        );
    }

    /// The committed WebSocket overlay whose close client arms `close-timeout` with `millis`.
    fn close_fixed(millis: u64) -> Overlay {
        let mut overlay = committed_websocket();
        let close = machine(&mut overlay, "close");
        assert_eq!(close.bounds().max_time_ms, 30000);
        let Effect::Arm(arm) = &mut close.participants[0].transitions[0].effects[1] else {
            panic!("the second effect arms");
        };
        assert!(
            arm.timer == "close-timeout" && matches!(arm.interval, Interval::Fixed { .. }),
            "the second effect arms close-timeout with millis"
        );
        arm.interval = Interval::Fixed { millis };
        overlay
    }

    #[test]
    fn a_fixed_arm_of_zero_millis_is_named() {
        assert_eq!(
            check(&close_fixed(0), &catalogue_6455(), &source_6455()),
            [format!(
                "{CLOSE_START}: arm close-timeout with millis 0, not a positive integer"
            )]
        );
    }

    #[test]
    fn a_fixed_arm_above_the_machine_max_time_is_named() {
        assert_eq!(
            check(&close_fixed(30001), &catalogue_6455(), &source_6455()),
            [format!(
                "{CLOSE_START}: arm close-timeout with millis 30001 above the machine's max_time_ms 30000"
            )]
        );
    }

    #[test]
    fn a_fixed_arm_at_the_machine_max_time_holds() {
        assert_eq!(
            check(&close_fixed(30000), &catalogue_6455(), &source_6455()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_backoff_of_a_timer_the_participant_does_not_declare_is_named() {
        assert_eq!(
            check(
                &close_backoff("poll", 2, 8000),
                &catalogue_6455(),
                &source_6455()
            ),
            [format!("{CLOSE_START}: unknown timer poll")]
        );
    }

    #[test]
    fn two_local_fields_of_one_name_are_named() {
        let mut overlay = committed_websocket();
        let client = &mut machine(&mut overlay, "ping").participants[0];
        client.fields.push(LocalField {
            name: "ping-data".into(),
            kind: "Integer".into(),
            initial: Literal::Integer(1),
        });
        assert_eq!(
            check(&overlay, &catalogue_6455(), &source_6455()),
            ["local field ping.client.ping-data: a second local field of this name"]
        );
    }

    #[test]
    fn a_never_state_naming_an_undeclared_participant_or_state_is_named() {
        let mut overlay = committed_websocket();
        let never = |name: &str, participant: &str, state: &str| {
            Property::NeverState(NeverState {
                name: name.into(),
                participant: participant.into(),
                state: state.into(),
            })
        };
        overlay.machines[0].properties.extend([
            never("no-ghost", "ghost", "closed"),
            never("no-limbo", "client", "limbo"),
            never("no-closing", "client", "closing"),
        ]);
        let problems = check(&overlay, &catalogue_6455(), &source_6455());
        assert_eq!(
            problems,
            [
                "property close.no-ghost: unknown participant ghost",
                "property close.no-limbo: unknown state limbo of participant client",
            ]
        );
    }

    fn implication_problems(
        participant: &str,
        state: &str,
        other: &str,
        other_state: &str,
    ) -> Vec<String> {
        let mut overlay = committed_websocket();
        overlay.machines[0]
            .properties
            .push(Property::StateImplication(StateImplication {
                name: "implied".into(),
                participant: participant.into(),
                state: state.into(),
                other: other.into(),
                other_state: other_state.into(),
            }));
        check(&overlay, &catalogue_6455(), &source_6455())
    }

    #[test]
    fn a_state_implication_between_declared_participants_and_states_holds() {
        assert_eq!(
            implication_problems("client", "closed", "server", "closed"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_state_implication_whose_participant_is_undeclared_is_named() {
        assert_eq!(
            implication_problems("ghost", "closed", "server", "closed"),
            ["property close.implied: unknown participant ghost"]
        );
    }

    #[test]
    fn a_state_implication_whose_other_participant_is_undeclared_is_named() {
        assert_eq!(
            implication_problems("client", "closed", "ghost", "closed"),
            ["property close.implied: unknown participant ghost"]
        );
    }

    #[test]
    fn a_state_implication_whose_state_is_undeclared_is_named() {
        assert_eq!(
            implication_problems("client", "limbo", "server", "closed"),
            ["property close.implied: unknown state limbo of participant client"]
        );
    }

    #[test]
    fn a_state_implication_whose_other_state_is_undeclared_is_named() {
        assert_eq!(
            implication_problems("client", "closed", "server", "limbo"),
            ["property close.implied: unknown state limbo of participant server"]
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
