---
format: aep.planning-md/3
id: story:send-payload-type-check
kind: story
status: draft
title: Refuse a send payload typed differently from its message field
relations:
- decomposes: epic:hardening-follow-ups
revision: 1
---
## Story

`rfcspec check` types guard operands and `set` values (story:overlay-type-checks) but not the payload a `send` effect gives a message field. A payload value typed differently from its message field passes `rfcspec check`; whether `ess protocol explore` refuses it is not yet known.

## Acceptance

A unit test in `crates/rfcspec-extract/src/check.rs` builds an overlay whose `send` gives a Boolean to an Integer message field and asserts one problem naming the transition, the field and both types; the committed profiles still check clean; `task check` exits 0.

Found by story:overlay-type-checks (implementation report, 2026-11-05).
