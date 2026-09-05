# contract-conformance-tests

Status: **contract-only**. This suite specifies generated client and server conformance with TypeSpec and JSON Schema contracts.

This repository is an executable acceptance-suite boundary, not evidence that the corresponding product capability is complete. The suite must target both `ores-chat` and the isolated `ores-chat-test` fixture. Promotion to `live` requires hosted execution, deterministic assertions, and redacted retained evidence.

The machine-readable plan is in `suite.json` and is validated by the organization policy action pinned to an immutable commit. Public, customer, administrator, and internal-service identities are never interchangeable.

## Independent discovery SDK consumer

`discovery/consumer.rs` contains six executable Rust integration tests against
the public API of the private `ores-chat-discovery` crate in
`ores-chat/ores-chat-lib-core`. Synthetic polynomial observations have known
closed-form slope, intercept, residual sums, and R-squared. Assertions also
cover owner/admin separation, immutable cohort replay rejection, complete-case
thresholds, family-adjusted inference, provenance, and diagnostic redaction.

This repository's workflow validates suite policy and Rust formatting only.
It does **not** claim that compilation or runtime assertions ran here. The
private product workflow checks out an immutable revision of this repository
to `tmp/discovery-acceptance` and runs:

```sh
cargo test --manifest-path discovery/Cargo.toml --locked --features external-conformance
```

The consumer source is included as a separate integration-test target and uses
only public crate APIs. Private SDK source is never copied into this public
repository, and no broad cross-organization credential is needed. The default
crate test run does not enable this optional consumer target.

These tests prove a pure computational SDK boundary, not generated wire-contract
parity, authenticated API/worker integration, database isolation, differential
privacy, vector retrieval, deployed environments, or complete platform parity.
The broad `suite.json` therefore remains `contract-only`.
