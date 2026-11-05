//! An in-process [`Model`] for tests: it answers each turn with the next scripted tool input as
//! one call to the forced tool, and counts the turns it was asked. It reaches no network and reads
//! no credential.

use std::{collections::VecDeque, sync::Mutex};

use llm_core::{
    BoxFuture, CallId, Cancel, Capabilities, Error, Id, Item, Model, Protocol, Provenance,
    StopReason, StreamSink, ToolCall, ToolChoice, TurnObservation, TurnOutcome, TurnRequest,
};
use serde_json::Value;

/// A model double answering from a script.
pub struct Double {
    provenance: Provenance,
    capabilities: Capabilities,
    answers: Mutex<VecDeque<Value>>,
    requests: Mutex<Vec<TurnRequest>>,
}

impl Double {
    /// A double named `model`, answering the given tool inputs in order.
    pub fn new(model: &str, answers: impl IntoIterator<Item = Value>) -> Self {
        let id = |value: &str| Id::new(value).expect("a double identifier");
        Self {
            provenance: Provenance {
                protocol: Protocol::Responses,
                provider: id("double"),
                account: id("double"),
                endpoint: id("double"),
                model: id(model),
                binding_revision: id("double-1"),
            },
            capabilities: Capabilities {
                tools: true,
                tool_choice: true,
                temperature: false,
                top_p: false,
                reasoning_efforts: Vec::new(),
                context_window: 32_768,
                max_output_tokens: 4_096,
                output_streaming: true,
                input_during_output: false,
            },
            answers: Mutex::new(answers.into_iter().collect()),
            requests: Mutex::new(Vec::new()),
        }
    }

    /// The turns asked so far.
    pub fn requests(&self) -> Vec<TurnRequest> {
        self.requests.lock().expect("unpoisoned").clone()
    }
}

impl Model for Double {
    fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    fn turn<'a>(
        &'a self,
        request: &'a TurnRequest,
        _sink: &'a mut dyn StreamSink,
        _cancel: &'a Cancel,
    ) -> BoxFuture<'a, Result<TurnOutcome, Error>> {
        self.requests
            .lock()
            .expect("unpoisoned")
            .push(request.clone());
        let answer = self.answers.lock().expect("unpoisoned").pop_front();
        let outcome = match (answer, &request.tool_choice) {
            (Some(arguments), ToolChoice::Named(name)) => {
                let mut observation = TurnObservation::new(self.provenance.clone());
                observation.final_usage = true;
                Ok(TurnOutcome {
                    stop_reason: StopReason::ToolCalls,
                    items: vec![Item::ToolCall(ToolCall {
                        call_id: CallId::new("call_1").expect("a call id"),
                        name: name.clone(),
                        arguments,
                    })],
                    observation,
                })
            }
            (None, _) => Err(Error::invalid("the double has no answer left")),
            (Some(_), _) => Err(Error::invalid("the double answers only a forced tool")),
        };
        Box::pin(async move { outcome })
    }
}
