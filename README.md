# RFCspec

Turn an RFC into an executable specification: a requirement catalogue with a cited source for
every item, ESS documents for its machines and types, a wire-format sidecar for its bytes and bits,
and from those a generated protocol core and a conformance suite. The first worked example is
WebSocket (RFC 6455).

**Status:** the pipeline runs end to end for four profiles (RFC 868, RFC 6455, RFC 1035, RFC 5905); [`AGENTS.md`](AGENTS.md) records what exists.

- Documentation: https://beyond10x.github.io/rfcspec/
- Design: [`docs/design/rfcspec.md`](docs/design/rfcspec.md)
- Pitch deck: [`website/static/pitch/index.html`](website/static/pitch/index.html), published at
  https://beyond10x.github.io/rfcspec/pitch/

## Build and check

Needs Rust, [go-task](https://taskfile.dev), [`ess`](https://beyond10x.github.io/ess/) and
[`aep`](https://github.com/beyond10x/aep).

```console
task check     # plan, ESS specification, example models, generated docs, fmt, clippy, tests
task website   # build the documentation site
```

## Fetch and read

`rfcspec fetch` locks an RFC's plain text, or with `--format xml` its RFCXML v3 source, into a
profile; `rfcspec read` prints the normalized document of the locked file, from XML with each
block's section anchor and BCP 14 marks.

```console
rfcspec fetch --profile <profile> --rfc 9220 --format xml
rfcspec read --profile <profile> --rfc 9220
```

## Propose and review

A model may propose what the pipeline cannot derive mechanically, such as a requirement's
status. `rfcspec propose` records each answer as a proposal under the profile's `proposals/`,
with the model id and the spans it cites; `rfcspec propose --replay` reproduces every recording
offline, and the gate runs it. A proposal changes nothing until `rfcspec review` accepts, edits
or rejects it into the profile's overlay. `profiles/websocket/proposals/` holds 114,
one for each RFC 6455 requirement outside the profile's sections that got a well-formed answer.
The contract is
[design section 9](docs/design/rfcspec.md#9-ai-in-the-pipeline).

```console
rfcspec propose --profile profiles/websocket --task status --section 5.1 --routes <file> --route <name>
rfcspec review --profile profiles/websocket --proposal <id> accept
rfcspec propose --profile profiles/websocket --replay
```

## Licence

Apache-2.0. See [LICENSE](LICENSE).
