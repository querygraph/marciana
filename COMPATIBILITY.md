# Compatibility

This document is the human-readable compatibility registry for Marciana. Exact
machine-consumed revisions live under `compat/` so documentation and CI do not
maintain competing pins.

## Scaffold baseline

The repository currently contains governance and CI scaffolding only. It does
not yet claim a compatible Marciana API, database schema, or QueryGraph route
baseline.

| Component | Required compatibility datum | Current scaffold status |
|---|---|---|
| Marciana | API, wire schema, proposal/binding schema, composite source-scope schema, job/outcome schema, database schema range | Pending the behavior-preserving transplant |
| TypeSec | Contract version, conformance fixture version, exact release or revision | Pending owning-repository stabilization |
| Grust | Core/backend version and guarded-commit capability | Pending owning-repository stabilization |
| LakeCat | Governed-proof schema and exact release or revision | Required for catalog-backed cognition; pending owning-repository stabilization |
| Sail | Exact current canonical-upstream revision and Arrow input/output schema | Revision recorded in [`compat/sail-revision.txt`](compat/sail-revision.txt); source build baseline in progress |
| QueryGraph | Supported route/wire baseline and exact version or revision | Pending qg-rust stabilization |
| Clients | Rust/Python/JavaScript fixture versions | Not yet established |

The linked Sail pin records the exact canonical upstream `main` revision
selected on 2026-08-05. It is a scaffold/source-build candidate, not a claim
that the not-yet-transplanted Marciana live gate has passed.

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
