# RFCspec

Turn an RFC into an executable specification: a requirement catalogue with a cited source for
every item, ESS documents for its machines and types, a wire-format sidecar for its bytes and bits,
and from those a generated protocol core and a conformance suite. The first worked example is
WebSocket (RFC 6455).

**Status:** design stage. Nothing of the pipeline is implemented yet.

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

## Licence

Apache-2.0. See [LICENSE](LICENSE).
