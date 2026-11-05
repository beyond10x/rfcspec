---
format: aep.planning-md/3
id: story:fixed-arm-bounds-check
kind: story
status: draft
title: Refuse a fixed timer arm of 0 ms or above max_time_ms
revision: 1
---
## Outcome

`rfcspec check` refuses a fixed `arm` (`millis`) of 0 or above the machine's `max_time_ms`, naming machine, participant, transition and timer, as it already does for the backoff form. `ess specify validate` refuses both values ("fixed interval must be positive and within max_time_ms"), so today the check admits an overlay that lowering then fails.

Source: review-result:ntp-timer-adversary-1, finding 3 (pre-existing, `crates/rfcspec-extract/src/check.rs:260`).

## Acceptance

- A test overlay arming a timer with `millis: 0` and one with `millis` above `max_time_ms` are each refused by `rfcspec check` with the named message.
- `rfcspec check --all profiles` exits 0.
- `task check` exits 0.

## Scope

- `crates/rfcspec-extract/src/check.rs`
