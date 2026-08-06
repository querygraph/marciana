# Compatibility

This document is the human-readable compatibility registry for Marciana. Exact
machine-consumed revisions live under `compat/` so documentation and CI do not
maintain competing pins.

## Extraction baseline

`querygraph-memory` has been transplanted into
`crates/marciana-memory` with its behavior-preserving history. Its TypeSec and
Grust dependencies are exact public Git revisions, so Marciana itself no
longer requires sibling checkouts. This is the first verified executable
baseline, not yet a versioned crate release.

| Component | Required compatibility datum | Current scaffold status |
|---|---|---|
| Marciana | API, wire schema, proposal/binding schema, composite source-scope schema, job/outcome schema, database schema range | Git-pinned native governed `improve` baseline plus validation-only Rust four-verb request contracts; vault-backed facade execution remains scheduled in `MARCIANA2.md` |
| TypeSec | Contract version, conformance fixture version, exact release or revision | Exact reachable revision `14bd5427` is pinned by Marciana, including vault-authorized ranked candidate materialization |
| Grust | Core/backend version and guarded-commit capability | Exact reachable revision `3bbd715` is pinned by Marciana |
| LakeCat | Governed-proof schema and exact release or revision | Exact reachable revision `415d131` is ready for the QueryGraph adapter pin |
| Sail | Exact reachable QueryGraph revision and Arrow input/output schema | QueryGraph Sail revision recorded below; it passed the live gate |
| QueryGraph | Supported route/wire baseline and exact version or revision | Exact reachable revision `efd6245` consumes standalone Marciana; a fresh clone passes its active 71-test suite, doctests, and strict Clippy |
| Clients | Rust/Python/JavaScript fixture versions | Shared `compat/fixtures/api_remember_v1.json`, Python client `0.1.0` (Pydantic `>=2.7,<3`), and TypeScript client `0.1.0` (Node ESM, TypeScript `^5.7`) are independently buildable; coordinated release publication remains pending |

The linked Sail pin records upstream merge `d8280bf4`. PR
[lakehq/sail#2374](https://github.com/lakehq/sail/pull/2374) is merged; the
explicit binary remains subject to the same `grust-sail` and live cognition
gate, which must be rerun for this refreshed baseline.

## Baseline procedure

Every integration baseline and release candidate must:

1. refresh the selected QueryGraph Sail branch and replace the recorded pin
   with its current exact revision;
2. build Sail from that source, never from an unrelated `PATH` installation;
3. run Marciana's live Sail schema and cognition tests;
4. run TypeSec store conformance, Grust persistence and recovery tests,
   LakeCat proof tests when enabled, and qg-rust route/reopen tests;
5. record exact remotely reachable dependency revisions, schema versions,
   fixture versions, database ranges, and the verified date here; and
6. prove the declared setup from a clean clone without sibling path
   dependencies.

The previous verified baseline passed the clean-clone gate on 2026-08-06: Marciana's
full workspace (including 72 `querygraph-memory` unit tests, integration
recovery/commit/outbox suites, doctests, and strict Clippy) and qg-rust's
active 71-test suite plus doctests and strict Clippy were built from fresh
clones using only the exact Git revisions named above. The recorded Sail live
gate was run from the selected QueryGraph Sail source. The refreshed upstream
revision is recorded above and its live gate is pending rerun.
Marciana's Sail-feature integration target compiles successfully against the
recorded Grust/Sail bindings; the two ignored cognition tests still require a
running Spark Connect endpoint and are not represented as live verification.

Generic Sail changes are contributed upstream. Marciana consumes exact
remotely reachable QueryGraph Sail revisions and never establishes a private
generic-Sail fork.

## Required release matrix fields

The first executable baseline must replace the scaffold table with a versioned
row containing:

- Marciana release, four-verb API version, and wire-fixture version;
- TypeSec release/revision, memory contract, and conformance-fixture version;
- cognition proposal, binding, canonical-digest, and receipt schema versions;
- composite governed-source, field-mapping, ingestion-profile, row-transform,
  audit-evidence, durable-job, and terminal-outcome schema versions;
- Grust release/revision, backend versions, and guarded-commit capability;
- LakeCat release/revision and governed-proof schema for catalog-backed builds;
- Sail exact upstream revision plus Arrow input/output schema versions;
- readable database schema range and migration path;
- QueryGraph release/revision and preserved route baseline; and
- compatible Rust, Python, and JavaScript client versions.

Each row is immutable after release. A later baseline refreshes Sail again and
adds a new row rather than rewriting historical compatibility.
