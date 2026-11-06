# RFCspec

Turn an RFC into an executable specification: a requirement catalogue with a cited source for
every item, ESS documents for its machines and types, a wire-format sidecar for its bytes and bits,
and from those a generated protocol core and a conformance suite. The first worked example is
WebSocket (RFC 6455).

**Status:** v0.2.0. The pipeline runs end to end
for four profiles (RFC 868, RFC 6455, RFC 1035, RFC 5905), reads plain text and RFCXML, and
counts the requirements of every RFC in the index. The wire layer has started with an ABNF reader
and matcher. [`AGENTS.md`](AGENTS.md) records what
exists; [`CHANGELOG.md`](CHANGELOG.md) what changed.

- Documentation: https://beyond10x.github.io/rfcspec/
- RFC explorer: https://beyond10x.github.io/rfcspec/explorer/ — search the RFC index, open any
  RFC's requirements by section and its references, and see a profiled RFC's reviewed layer
- Design: [`docs/design/rfcspec.md`](docs/design/rfcspec.md)
- Pitch deck: [`website/static/pitch/index.html`](website/static/pitch/index.html), published at
  https://beyond10x.github.io/rfcspec/pitch/

## Build and check

Needs Rust, [go-task](https://taskfile.dev), [`ess`](https://beyond10x.github.io/ess/) and
[`aep`](https://github.com/beyond10x/aep); `task website` also needs Node and pnpm.

```console
task check     # plan, ESS specification, generated types, profiles, bulk summary, generated docs, fmt, clippy, tests
task website   # build the RFC explorer, then the documentation site
```

## Fetch and read

`rfcspec fetch` locks an RFC's plain text, or with `--format xml` its RFCXML v3 source, into a
profile; `rfcspec read` prints the normalized document of the locked file, from XML with each
block's section anchor and BCP 14 marks.

```console
rfcspec fetch --profile <profile> --rfc 9220 --format xml
rfcspec read --profile <profile> --rfc 9220
```

## The whole index

`bulk/rfc-summary.jsonl` holds one row per RFC the index publishes as RFCXML or plain text, with
its requirement counts by BCP 14 level, and `bulk/rfc-sections.jsonl` the counts per section and
the RFCXML references. Both are unreviewed and feed the RFC explorer. `task bulk-regenerate`
fetches into a cache outside the repository and rewrites them; `rfcspec bulk compare` lists the
RFCs whose plain-text and RFCXML readings give different requirement ids.

```console
task bulk-regenerate
rfcspec bulk compare
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
