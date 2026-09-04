# Compatibility

This document is the human-readable compatibility registry for Marciana. Exact
machine-consumed revisions live under `compat/` so documentation and CI do not
maintain competing pins.

## Extraction baseline

`querygraph-memory` has been transplanted into
`crates/marciana-memory` with its behavior-preserving history. The 0.13.1
release resolves TypeSec 0.14.0, Grust 0.13.0, and LakeCat 0.4.0 from crates.io;
the lockfile records the exact registry checksums and no sibling checkout is
required.

| Component | Required compatibility datum | Current scaffold status |
|---|---|---|
| Marciana | API, wire schema, proposal/binding schema, composite source-scope schema, job/outcome schema, database schema range | Git-pinned native governed `improve` baseline plus validation-only Rust four-verb request contracts; vault-backed facade execution remains scheduled in `MARCIANA2.md` |
| TypeSec | Contract version, conformance fixture version, exact release or revision | Released 0.14.0 (Dorsoduro), including vault-authorized ranked candidate materialization and signed semantic decision receipts |
| Grust | Core/backend version and guarded-commit capability | Released 0.13.0 (Prawn) graph, Sail, Turso, Cypher, SQL-core, and memory crates |
| LakeCat | Governed-proof schema and exact release or revision | Released 0.4.0 core crate and qglake bundle |
| Sail | Exact reachable QueryGraph revision and Arrow input/output schema | QueryGraph Sail graph revision recorded below; source and refreshed production live gates pass |
| QueryGraph | Supported route/wire baseline and exact version or revision | Exact reachable revision `efd6245` consumes standalone Marciana; a fresh clone passes its active 71-test suite, doctests, and strict Clippy |
| Clients | Rust/Python/JavaScript fixture versions | Shared `compat/fixtures/api_remember_v1.json`, Python client `0.1.0` (Pydantic `>=2.7,<3`), and TypeScript client `0.1.0` (Node ESM, TypeScript `^5.7`) are independently buildable; coordinated release publication remains pending |

The linked Sail pin records reachable QueryGraph graph revision `c5309365`.
That revision merges the head `ce5dada0` of the open draft performance PR
[lakehq/sail#2400](https://github.com/lakehq/sail/pull/2400) with QueryGraph's
native Cypher graph extension. It does not represent the upstream PR as merged.
The exact production binary passed the same `grust-sail` and live cognition
gate for this refreshed baseline. It was built from `c5309365` with Rust 1.96.0,
native CPU instructions, `-O3`, thin LTO, one codegen unit, aborting panics,
no incremental state, `clang`/`lld`, and stripped symbols. The
115,921,992-byte AArch64 executable has SHA-256
`937eb113b5a03384a400cb659d12157a9fdc42fbfc19f660325708983e989199`.

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
gate was run from its selected QueryGraph Sail source. The refreshed graph
revision is recorded above; its parser/analyzer, planner, Spark Connect, and
strict Clippy source gates pass. On 2026-08-09, its exact optimized production
binary also passed 26 `grust-sail` live tests, the dedicated non-null and temp
view checks, and both governed cognition reference-parity/evidence-secrecy
tests. Marciana's two Sail cognition tests therefore represent executed live
verification against the recorded artifact, not compile-only coverage.

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
