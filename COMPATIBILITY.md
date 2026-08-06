# Compatibility

This document is the human-readable compatibility registry for Marciana. Exact
machine-consumed revisions live under `compat/` so documentation and CI do not
maintain competing pins.

## Extraction baseline

`querygraph-memory` has been transplanted into
`crates/marciana-memory` with its behavior-preserving history. Its TypeSec and
Grust dependencies are exact public Git revisions, so Marciana itself no
longer requires sibling checkouts. This is still an extraction baseline, not a
released compatibility matrix.

| Component | Required compatibility datum | Current scaffold status |
|---|---|---|
| Marciana | API, wire schema, proposal/binding schema, composite source-scope schema, job/outcome schema, database schema range | Transplanted Git-pinned baseline; no public four-verb facade or released schema range yet |
| TypeSec | Contract version, conformance fixture version, exact release or revision | Exact reachable revision `1926f18c` is pinned by Marciana |
| Grust | Core/backend version and guarded-commit capability | Exact reachable revision `3bbd715` is pinned by Marciana |
| LakeCat | Governed-proof schema and exact release or revision | Exact reachable revision `415d131` is ready for the QueryGraph adapter pin |
| Sail | Exact current canonical-upstream revision and Arrow input/output schema | Canonical candidate recorded below; local source live gate passed, but the generic correction is not remotely reachable |
| QueryGraph | Supported route/wire baseline and exact version or revision | qg-rust has a local standalone-path cutover; clean-clone and route baseline remain pending |
| Clients | Rust/Python/JavaScript fixture versions | Not yet established |

The linked Sail pin records canonical upstream `main` revision `50567c79`,
refreshed on 2026-08-05. On 2026-08-06, the explicit binary built from the
local source checkout (including the generic Delta `MERGE` correction rebased
above that revision) passed 26 `grust-sail` tests, two live backend tests, and
two live cognition parity and evidence-secrecy tests. This is local-source
evidence only. The correction is deliberately not recorded as a compatible
dependency until it is remotely reachable, and the recorded canonical pin has
not yet passed the same gate unaided.

## Baseline procedure

Every integration baseline and release candidate must:

1. refresh the canonical upstream Sail branch and replace the recorded pin with
   its current exact revision;
2. build Sail from that source, never from an unrelated `PATH` installation;
3. run Marciana's live Sail schema and cognition tests;
4. run TypeSec store conformance, Grust persistence and recovery tests,
   LakeCat proof tests when enabled, and qg-rust route/reopen tests;
5. record exact remotely reachable dependency revisions, schema versions,
   fixture versions, database ranges, and the verified date here; and
6. prove the declared setup from a clean clone without sibling path
   dependencies.

Generic Sail changes are made in Sail upstream. Marciana updates its exact
revision after the upstream change lands; it never establishes a private
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
