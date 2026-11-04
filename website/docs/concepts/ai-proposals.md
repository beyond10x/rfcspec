---
title: AI proposals
sidebar_position: 5
description: Where RFCspec uses a model, what a proposal must carry, how it is checked and reviewed, and why no gate ever calls a model.
lede: AI turns RFC prose into structure; every proposal quotes the text it rests on, is checked mechanically, recorded, replayed, and changes nothing until a reviewer accepts it.
source: design section 9 and decisions D5, D6
source_url: https://github.com/beyond10x/rfcspec/blob/main/docs/design/rfcspec.md
status: decided
---

:::caution[Decided, not built]
The proposal contract is a design decision. No proposal stage exists yet.
:::

## Where a model helps

Mechanical extraction stops at prose. A model is used for exactly these tasks:

| Task | Output | Checked before a human sees it |
|---|---|---|
| classify an RFC's kind | kind and confidence | schema only |
| the party of a requirement | client, server, sender, recipient | the party is one of the profile's roles |
| communication semantics | one value per axis, each with a quoted sentence | every quote occurs verbatim in the locked source |
| machines | participants and transitions, each citing a span and requirement ids | `ess` validates the model; every cited span exists; every MUST in scope is linked or listed as unlinked |
| constraints from prose, such as "MUST be 0 unless negotiated otherwise" | a guard on a field | the field exists in the layout |
| bit widths from ABNF comments | widths | the sum matches the packet diagram |
| rule mappings across RFC revisions | a mapping table | both rules exist |

## The contract

A proposal is a typed document with the model id, a digest of the prompt and inputs, the response
and the spans it cites. Proposals are stored and replayed: the propose stage with the same inputs
and a recorded response returns the recording, so continuous integration and every later stage
run offline and give the same result. A proposal changes nothing until a review decision is
written to the profile's overlay.

## Prior work

Research has extracted protocol state machines from RFC text before: RFCNLP from the TCP and DCCP
RFCs, SAGE for ICMP, and later LLM-based work (PROSPER, RFSeek, FlowFSM). The lesson those papers
report is that extraction is partial and needs a cited trail. RFCspec adds the trail and the use:
the extracted machine becomes an ESS document that is validated, explored and conformance-tested.
