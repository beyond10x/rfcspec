---
format: aep.planning-md/3
id: story:payload-field-names
kind: story
status: draft
title: Refuse a send payload whose field names differ from its message's declared fields
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-extract/tests/adversary_send_payload.rs
revision: 2
---
## Story

`rfcspec check` refuses a `send` payload whose field names are not exactly its message's declared fields. Today a payload key the message does not declare is skipped (`crates/rfcspec-extract/src/check.rs:262`), a misspelt key escapes the type check, and a declared field missing from the payload is not noticed. `ess protocol explore` 0.86.0 refuses each of these after lowering with `[PayloadFieldsMismatch]: send payload must name exactly the message's declared fields`.

Found by adversary pass 1 on story:send-payload-type-check (`review-result:send-payload-adversary-1`) as pre-existing. Its three cases are kept, ignored and naming this story, in `crates/rfcspec-extract/tests/adversary_send_payload.rs`.

## Acceptance

- The three ignored cases in `adversary_send_payload.rs` are un-ignored and pass:
  - `a_payload_field_the_message_does_not_declare_is_named`;
  - `a_misspelt_payload_field_of_the_wrong_type_is_named`;
  - `a_declared_message_field_the_payload_leaves_out_is_named`.
- `rfcspec check --all profiles` exits 0.

## Scope

- `crates/rfcspec-extract/src/check.rs`
- `crates/rfcspec-extract/tests/adversary_send_payload.rs`

## Order

It edits `check.rs`, so it waits behind the check.rs chain of the current plan.
