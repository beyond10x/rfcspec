//! The tasks that propose model elements (design section 9): `machine`, `bit-widths` and
//! `requirement-link`, each with its forced tool and its mechanical check.

use std::collections::BTreeSet;

use llm_core::{ToolName, ToolSpec};
use rfcspec_model::{Edit, Layout, Machine, Status};
use serde_json::{Value, json};

use crate::{Checked, LEVELS, Subject, Task, Workspace, tool};

/// The tool the model is forced to call for `task`.
pub(crate) fn tool_for(task: Task) -> ToolSpec {
    match task {
        Task::Status | Task::RequirementFromProse => tool(),
        Task::RequirementLink => link_tool(),
        Task::BitWidths => layout_tool(),
        Task::Machine => machine_tool(),
    }
}

fn span_schema(description: &str) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["start", "end"],
        "properties": {
            "start": {"type": "integer", "minimum": 1},
            "end": {"type": "integer", "minimum": 1},
        },
        "description": description,
    })
}

fn link_tool() -> ToolSpec {
    ToolSpec {
        name: ToolName::new("propose_requirement_link").expect("a valid tool name"),
        description: "Give one overlay requirement entry satisfied by an element of the overlay."
            .into(),
        input_schema: json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["id", "source", "level", "quote", "span", "status"],
            "properties": {
                "id": {"type": "string", "description": "The item id exactly as given."},
                "source": {
                    "type": "string",
                    "enum": ["bcp14-xml", "capitals-text", "abnf-comment", "prose"],
                },
                "level": {"type": "string", "enum": LEVELS},
                "quote": {
                    "type": "string",
                    "description": "Copied verbatim from the numbered source lines, without the line numbers.",
                },
                "span": span_schema("The first and last source line of the quote, inclusive."),
                "status": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["satisfied"],
                    "properties": {
                        "satisfied": {
                            "type": "string",
                            "description": "One of the overlay elements given.",
                        },
                    },
                },
            },
        }),
    }
}

fn layout_tool() -> ToolSpec {
    let field = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["name", "bits", "signed", "order"],
        "properties": {
            "name": {"type": "string"},
            "bits": {"type": "integer", "minimum": 1, "maximum": 64},
            "signed": {"type": "boolean"},
            "order": {"type": "string", "enum": ["big-endian", "little-endian"]},
            "presence": {
                "type": "object",
                "additionalProperties": false,
                "required": ["field", "equals"],
                "properties": {"field": {"type": "string"}, "equals": {"type": "integer"}},
                "description": "Present only when an earlier field equals the value.",
            },
            "constraints": {
                "type": "array",
                "items": {
                    "oneOf": [
                        {"type": "object", "required": ["equals"], "properties": {"equals": {"type": "integer"}}},
                        {"type": "object", "required": ["range"], "properties": {"range": {"type": "array", "items": {"type": "integer"}, "minItems": 2, "maxItems": 2}}},
                        {"type": "object", "required": ["one_of"], "properties": {"one_of": {"type": "array", "items": {"type": "integer"}}}},
                    ],
                },
            },
        },
    });
    ToolSpec {
        name: ToolName::new("propose_layout").expect("a valid tool name"),
        description: "Give one overlay layout: the packet's fields with their bit widths.".into(),
        input_schema: json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["name", "quote", "span", "fields"],
            "properties": {
                "name": {"type": "string", "description": "The layout name exactly as given."},
                "quote": {
                    "type": "string",
                    "description": "Copied verbatim from the numbered source lines, without the line numbers.",
                },
                "span": span_schema("The first and last source line of the quote, inclusive."),
                "fields": {"type": "array", "minItems": 1, "items": field},
                "length": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "The fields that carry the length of what follows, most specific first.",
                },
            },
        }),
    }
}

fn machine_tool() -> ToolSpec {
    let named_fields = json!({"type": "object", "additionalProperties": {"type": "string", "enum": ["Integer", "String", "Boolean"]}});
    let transition = json!({
        "type": "object",
        "required": ["implements", "label", "from", "to", "on"],
        "properties": {
            "implements": {"type": "array", "minItems": 1, "items": {"type": "string"}},
            "label": {"type": "string"},
            "from": {"type": "string"},
            "to": {"type": "string"},
            "on": {
                "type": "object",
                "description": "{input: <name>}, {receive: <message>} or {timer: <name>}.",
            },
            "guards": {"type": "array", "items": {"type": "object"}},
            "effects": {
                "type": "array",
                "items": {"type": "object"},
                "description": "{send: {message, payload}}, {emit: <name>}, {close: true}, {arm: {timer, millis}}, {cancel: <timer>}, {flush: true} or {set: {field, value}}.",
            },
        },
    });
    ToolSpec {
        name: ToolName::new("propose_machine").expect("a valid tool name"),
        description: "Give one overlay machine of two participants over one layer of the profile."
            .into(),
        input_schema: json!({
            "type": "object",
            "required": ["name", "transport", "messages", "participants", "properties"],
            "properties": {
                "name": {"type": "string"},
                "transport": {"type": "string", "description": "One of the profile's layers given."},
                "messages": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {"name": {"type": "string"}, "fields": named_fields},
                    },
                },
                "participants": {
                    "type": "array",
                    "minItems": 2,
                    "maxItems": 2,
                    "items": {
                        "type": "object",
                        "required": ["name", "initial", "states", "transitions"],
                        "properties": {
                            "name": {"type": "string"},
                            "initial": {"type": "string"},
                            "states": {"type": "array", "items": {"type": "string"}},
                            "inputs": {"type": "array"},
                            "timers": {"type": "array", "items": {"type": "string"}},
                            "fields": {"type": "array"},
                            "transitions": {"type": "array", "items": transition},
                        },
                    },
                },
                "properties": {"type": "array"},
            },
        }),
    }
}

impl Workspace {
    /// `requirement-link`: a requirement entry whose status is `satisfied` by an element the
    /// overlay holds; a transition element lists the requirement in `implements`.
    pub(crate) fn check_link(
        &self,
        subject: &Subject,
        response: &Value,
    ) -> Result<Checked, String> {
        let entry = self.entry(subject, response)?;
        let Status::Satisfied(element) = &entry.status else {
            return Err(format!(
                "status {:?}: a link states satisfied by an element of the overlay",
                entry.status
            ));
        };
        match self.elements().get(element) {
            None => Err(format!("{element} is no element of the overlay")),
            Some(Some(transition)) if !transition.implements.contains(&entry.id) => Err(format!(
                "transition {element} does not list {} in implements",
                entry.id
            )),
            Some(_) => Ok(Checked {
                cited: vec![entry.span],
                element: Edit::Requirement(entry),
            }),
        }
    }

    /// `bit-widths`: a layout under the name asked for whose quote occurs at its span within
    /// the sections read, and which keeps the overlay layout rules (widths 1 to 64, presence on
    /// an earlier field, ranges within width).
    pub(crate) fn check_layout(
        &self,
        subject: &Subject,
        response: &Value,
    ) -> Result<Checked, String> {
        let layout: Layout = serde_json::from_value(response.clone())
            .map_err(|error| format!("the answer is not an overlay layout: {error}"))?;
        if layout.name != subject.id {
            return Err(format!(
                "the layout is named {}, asked for {}",
                layout.name, subject.id
            ));
        }
        self.quoted(&layout.quote, layout.span, subject)?;
        let problems = rfcspec_extract::layout_problems(&layout);
        if !problems.is_empty() {
            return Err(problems.join("; "));
        }
        Ok(Checked {
            cited: vec![layout.span],
            element: Edit::Layout(layout),
        })
    }

    /// `machine`: a machine whose transport is a layer of `profile.yaml`, whose every
    /// transition implements at least one catalogue requirement and only catalogue
    /// requirements, and which lowers (`rfcspec_ess::lower`) on a copy of the overlay holding
    /// it in place of a machine of its name. Cites the spans of the requirements it implements.
    pub(crate) fn check_machine(&self, response: &Value) -> Result<Checked, String> {
        let machine: Machine = serde_json::from_value(response.clone())
            .map_err(|error| format!("the answer is not an overlay machine: {error}"))?;
        let layers: Vec<&str> = self
            .profile
            .semantics
            .iter()
            .map(|layer| layer.layer.as_str())
            .collect();
        if !layers.contains(&machine.transport.as_str()) {
            return Err(format!(
                "transport {} is no layer of profile.yaml ({})",
                machine.transport,
                layers.join(", ")
            ));
        }
        let mut cited = BTreeSet::new();
        for participant in &machine.participants {
            for transition in &participant.transitions {
                let named = format!(
                    "transition {}.{}.{}",
                    machine.name, participant.name, transition.label
                );
                if transition.implements.is_empty() {
                    return Err(format!("{named} implements no requirement"));
                }
                for id in &transition.implements {
                    let Some(found) = self.catalogue.requirements.iter().find(|r| &r.id == id)
                    else {
                        return Err(format!(
                            "{named}: implements {id}, which is no catalogue requirement of RFC {}",
                            self.overlay.rfc
                        ));
                    };
                    cited.insert((found.span.start, found.span.end));
                }
            }
        }
        let mut overlay = self.overlay.clone();
        overlay.machines.retain(|m| m.name != machine.name);
        overlay.machines.push(machine.clone());
        rfcspec_ess::lower(&overlay, &self.profile.name)
            .map_err(|error| format!("the overlay with the machine does not lower: {error:#}"))?;
        Ok(Checked {
            cited: cited
                .into_iter()
                .map(|(start, end)| rfcspec_model::Span { start, end })
                .collect(),
            element: Edit::Machine(machine),
        })
    }
}
