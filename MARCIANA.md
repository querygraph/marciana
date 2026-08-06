# Marciana Delivery Goal and Status

**Status:** active implementation goal

**Updated:** 2026-08-05

This document is the canonical execution record for extracting and completing
Marciana as a standalone QueryGraph-stack project. It records current delivery
status, work order, and acceptance gates. [DESIGN.md](DESIGN.md) remains
authoritative for ownership, dependency direction, trust boundaries, and Sail
integration. [MARCIANA2.md](MARCIANA2.md) owns the product plan that follows
the preserved baseline; it does not reorder this extraction.

## Active goal

Deliver Marciana as the reusable memory and cognition part of QueryGraph on
Sail, composed on TypeSec, Grust, and LakeCat. The standalone repository must
own the native `remember`, `recall`, `improve`, and `forget` lifecycle while
preserving the existing QueryGraph routes, wire values, durable identifiers,
database data, denial behavior, receipts, and recovery semantics.

The product boundary is:

- TypeSec and TypeDID authorize protected-content access, bind identity and
  intent, validate proposals, and issue security evidence;
- Grust owns physical stores, durable transactions, guarded commits, leases,
  recovery primitives, and generic QueryGraph execution contracts;
- LakeCat owns catalog state and governed-scan evidence;
- Sail owns generic distributed computation and Spark/Arrow behavior;
- Marciana owns the logical memory ledger, memory-specific schemas and
  adapters, cognition orchestration, four-verb behavior, and product receipts;
  and
- QueryGraph consumes Marciana through a thin integration rather than owning
  its memory-product semantics.

Only TypeSec's capability-gated vault may reveal or mutate protected memory.
Grust persists, indexes rank identifiers, LakeCat proves governed inputs, Sail
computes proposals, and Marciana orchestrates; none creates a second
authorization or mutation path.

## Current status

| Area | Status | Remaining delivery work |
|---|---|---|
| TypeSec and TypeDID | The cognition authority, bound-proposal, prepared-commit, receipt-recovery, and request-binding foundations are committed. Exact governed-source scope, manifest-only reauthorization, and receipt hardening are under final verification in their owning repository. | Finish and commit the governed boundary, version audit and receipt evidence, make proposal diagnostics non-disclosing, and rerun the owning repository's release gates against the selected revision. |
| LakeCat | Persisted governed-scan grants and proof foundations are committed. Separate snapshot and source-scope digests, projection checks, and structural proof bounds are under final verification in their owning repository. | Finish and commit those owner-side invariants, pin the selected remotely reachable revision, and include its proof tests in the clean-clone gate. |
| Grust | The durable cognition scheduler/store, leases, guarded commit and recovery, ID-only outbox, and Sail executor are under final verification in the owning repository. | Finish review and gates, commit the cohesive generic capabilities, and rebuild the owning documentation before selecting a revision. |
| QueryGraph | The TypeDID/LakeCat cognition boundary is being hardened and is under final verification in qg-rust. | Finish the boundary gates, then switch the preserved integration from the Grust-hosted crate to standalone Marciana. |
| Sail | A generic Delta `MERGE` constraint correction exists as a local commit and has live local proof. It is not remotely reachable and therefore is not a supported Marciana pin. | Land the generic fix in the canonical Sail source, refresh to the current upstream revision, build that exact source, and pass Marciana's live gate. |
| Marciana | Repository governance, ownership, compatibility scaffolding, and the reviewed product plan are established. | Transplant `querygraph-memory` with history, establish exact dependency pins, switch QueryGraph, complete the native four verbs, and pass the cross-stack gates. |

No executable Marciana compatibility baseline is claimed yet. The status above
describes work in owning repositories; it does not turn local branches or
sibling checkout paths into released dependencies.

## Required execution order

1. Finish the Grust and qg-rust boundary reviews, tests, and logical commits;
   rerun the applicable TypeSec and LakeCat conformance gates.
2. Land the generic Delta correction in canonical Sail, refresh Sail from its
   current upstream source, record the exact revision, build that checkout,
   and run the live integration proof using its explicit binary.
3. Transplant `querygraph-memory` from Grust into this repository with Git
   history and behavior intact. Preserve its crate name, routes, wire forms,
   database prefixes, durable identifiers, storage formats, and tests; do not
   combine the move with a redesign.
4. Replace sibling path dependencies with released versions or exact Git
   revisions that are reachable from configured remotes. Prove that a clean
   clone builds without the local QueryGraph workspace layout.
5. Switch qg-rust to standalone Marciana and prove route, wire, denial,
   receipt, database reopen, retry, and recovery compatibility before removing
   the former Grust copy.
6. Complete and expose the native `remember`, `recall`, `improve`, and
   `forget` lifecycle through one shared domain implementation and thin
   embedded, service, Sail, LakeCat, and QueryGraph adapters.
7. Run the full cross-stack acceptance gates, record the first executable
   compatibility matrix, and only then begin the post-baseline phases in
   [MARCIANA2.md](MARCIANA2.md).

Sail is updated throughout this sequence, not treated as a one-time vendored
dependency. Every integration baseline and release candidate starts by
refreshing the current canonical Sail upstream, contributing generic fixes
there, and recording the exact verified revision in this repository.

## Governed `improve` completion path

The public operation is one opaque Marciana command, not a public two-step
plan/apply protocol. Its durable worker executes one state machine:

1. authenticate the request and bind its TypeDID intent, tenant, subject,
   labels, engine profile, operation, and idempotency key;
2. persist or recover the job, acquire its lease, and renew the lease while
   work continues;
3. preauthorize TypeSec access and obtain LakeCat governed-scan evidence;
4. have Marciana's trusted ingestion adapter scan the authorized rows, apply
   the versioned field mapping and row transformation, and write the resulting
   governed drafts through the TypeSec vault;
5. execute the fixed cognition profile on Sail to produce an inert in-memory
   proposal;
6. before revealing any proposal-derived data, revalidate the LakeCat grant
   and snapshot and perform TypeSec manifest-only reauthorization against the
   exact current source scope;
7. validate and stage the exact proposal digest, then either atomically apply
   the prepared mutations, audit evidence, terminal job outcome, and ID-only
   index outbox or atomically record a typed no-change outcome; and
8. recover the backend commit identity when a response is lost and issue a
   versioned, commit-bound TypeDID receipt from the durable outcome.

The proposal is an internal transient value. Marciana never persists it,
returns it to QueryGraph, places it in an outbox, or exposes it in logs or
diagnostics. Restart recovery reruns deterministic planning and must match the
durably expected digest before application. Revocation, projection change,
snapshot change, scope change, lease loss, digest mismatch, or failed
reauthorization closes the job without disclosing proposal content or
partially mutating authoritative memory.

Catalog source identity is a Marciana-owned composite scope, because Marciana
alone knows the ingestion semantics. Its versioned digest binds LakeCat's
source-scope digest to the exact field mapping, ingestion profile, and
row-to-memory transformation version. The same opaque scope is attached by
TypeSec during governed ingestion and is required by planning, post-engine
reauthorization, proposal validation, audit evidence, and the final receipt.
LakeCat remains the sole owner of catalog proof and projection
canonicalization; TypeSec treats the composite scope as opaque security
context.

Prepared, revalidated, backend-committed, and receipt-issued times have
distinct meanings and are never substituted for one another. Audit and
receipt schemas explicitly version those meanings and carry separate input
snapshot, governed grant/source-scope, proposal, and committed-outcome
digests. The receipt is constructed complete from the recovered durable
outcome; callers cannot assemble it by mutating a partially initialized value.

## Acceptance gates

The active goal is complete only when all of these conditions hold:

- the dependency direction in [DESIGN.md](DESIGN.md) is enforced and no
  foundational repository depends on Marciana;
- all committed dependencies use releases or exact remotely reachable
  revisions, and a clean clone needs no sibling path dependencies;
- qg-rust consumes standalone Marciana with preserved route, wire, durable-ID,
  database reopen, denial, receipt, retry, and recovery behavior;
- all four native verbs share the same capability, policy, validation,
  mutation, and recovery authorities without alternate bypasses;
- public `improve` is one authenticated durable operation: raw proposals and
  worker state never cross the QueryGraph boundary, and retry or restart
  re-planning must match the durably expected proposal digest;
- catalog-backed cognition requires valid LakeCat governed-scan evidence, and
  Marciana's trusted adapter—not a caller—derives the exact governed drafts
  from that scan and binds each write once; a proof cannot bless independently
  supplied text, and local-only operation remains isolated behind its declared
  feature boundary;
- the governed source scope binds LakeCat's source-scope digest, the exact
  field mapping, ingestion profile, and row transformation version, and the
  identical opaque scope is enforced from ingestion through receipt;
- stale inputs, changed digests, revoked authority, label mismatches,
  idempotency collisions, malformed proposals, and provider failures fail
  closed without partial authoritative mutation;
- create/close/reopen, crash, lease expiry, retry, concurrent claim, lost
  response, and proposal-free recovery tests prove exactly-once authoritative
  outcomes and an ID-only repair outbox;
- no-change is a typed durable terminal outcome with no fabricated memory
  mutation or index work, and cancellation cannot leave lease renewal running;
- audit and receipt schemas are versioned, distinguish snapshot and grant
  identity, distinguish prepared/revalidated/committed/issued times, and bind
  the recovered backend commit identity;
- the exact recorded Sail source is built and its explicit binary passes live
  memory-schema, Arrow, Delta, and cognition tests;
- formatting, strict Clippy, all workspace tests, TypeSec conformance, Grust
  persistence/recovery, LakeCat proof, dependency-direction, and QueryGraph
  compatibility gates pass; and
- [COMPATIBILITY.md](COMPATIBILITY.md) records the verified revisions, schema
  and fixture versions, supported database range, migration path, and gate
  date without overstating a local-only result.

Production code must remain modular and DRY: files are small and
single-purpose, functions are cohesive, canonicalization and state transitions
have one implementation, adapters only translate and delegate, and substantive
tests live in separate files or integration-test targets.

## Remote authorization gate

An exact Git revision is acceptable only after it is reachable from the remote
used by a clean clone. The current local-only Sail correction cannot satisfy
that requirement. Any other selected local commits must meet the same rule.

Creating or changing remotes, pushing commits, publishing crates, and opening
external changes require explicit authorization. Until an authorized operator
lands the necessary revisions, local implementation and verification may
continue, but the remote-pin, clean-clone, and executable-baseline gates remain
open. They must not be bypassed with path dependencies, moving branches, copied
code, or an unrecorded Sail binary.

## Explicit non-goals

- Cognee is inspiration only. Its runtime, API, adapters, stores, search
  surface, and completeness model are not dependencies or compatibility
  targets. A future Cognee-shaped edge facade requires a separate decision.
- Akka and Fluree remain comparative design input, not Marciana runtime or
  storage dependencies.
- Marciana does not reproduce Grust's store portfolio, TypeSec's policy and
  vault logic, LakeCat's proof canonicalization, or Sail's generic compute
  behavior.
- The initial history-preserving transplant is not an opportunity to rename
  identifiers, alter routes or database formats, split crates, or redesign the
  public contract.
- No adapter may introduce an alternate authorization, mutation, deletion,
  idempotency, or recovery path, and no production unscoped `forgetAll`
  operation is part of the four-verb API.
