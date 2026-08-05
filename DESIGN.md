# Marciana Design and Ownership Decision

**Status:** accepted

**Decision date:** 2026-08-05

## Decision

Marciana is a standalone QueryGraph-stack project and the reusable memory and
cognition product layer. It is a composition layer, not another security,
storage, catalog, or compute substrate.

The native API is `remember`, `recall`, `improve`, and `forget`. Marciana owns
their orchestration, durable cognition jobs, memory-specific schemas and
adapters, receipts, recovery, and compatibility. Cognee may inspire an
optional edge facade, but it is never a runtime, store, adapter, or product
completeness dependency.

## Dependency direction

```text
QueryGraph applications
        |
        v
    Marciana
   /   |   |  \
  v    v   v   v
TypeSec Grust Sail LakeCat (governed catalog paths)
```

TypeSec, Grust, Sail, and LakeCat must never depend on Marciana. A capability
needed by multiple consumers belongs in the relevant foundational repository;
Marciana contributes it upstream and then consumes the released or exact
remotely reachable revision.

| Owner | Authoritative responsibility | Marciana responsibility |
|---|---|---|
| TypeSec | Capabilities, policy, protected content, labels, retention, quarantine, proposal validation, TypeDID | Use its contracts; never bypass `MemoryVault` |
| Grust | Generic graph/query types, transactions, guarded commits, durable backends | Memory graph projection and adapters |
| Sail | Generic Arrow/Spark execution and distributed compute | Memory-specific schemas and cognition executors |
| LakeCat | Iceberg catalog state and governed-scan proof | Required for catalog-backed governed cognition; feature-isolated for local-only use |
| QueryGraph | Navigator, QGLake, semantic models, governed answers | Consume Marciana through a thin integration |
| Marciana | Four verbs, jobs, memory ledger, cognition, adapters, recovery, compatibility | Own and version product behavior |

## Security invariant

Only the capability-gated TypeSec vault may rehydrate or mutate protected
memory. Retrieval engines return candidates or ranked identifiers. Cognition
workers receive an authorized immutable input bundle and emit inert proposals.
TypeSec reauthorizes and validates a proposal before Marciana maps the prepared
commit into one atomic Grust operation.

Queues, outboxes, audit records, and logs contain identifiers or digests, not
memory plaintext, reusable authorization material, raw lease tokens, or raw
worker and failure data.

## Persistence and computation

Marciana owns the logical memory ledger and memory-specific graph projection.
Grust owns the physical transaction engine. A production apply operation must
atomically check source revisions, claim idempotency, mutate the memory graph,
write an ID-only index outbox, persist audit-safe evidence, and retain a
recoverable backend commit identity.

Sail workers calculate proposals; they never receive an authoritative mutation
handle. Memory-specific Sail schemas, queries, and executors live in Marciana.
Generic execution, Arrow staging, Spark Connect behavior, and engine fixes live
in Sail or the generic Grust-to-Sail integration.

## Sail upstream policy

Sail is refreshed from its canonical upstream source for every new Marciana
integration baseline and release candidate. The baseline process is:

1. fetch the current canonical upstream branch and select that current commit;
2. record the exact commit in `compat/sail-revision.txt`;
3. build Sail from that source checkout and invoke Marciana's live integration
   gate with the resulting explicit binary path; and
4. record the verified schema and compatibility result in
   [COMPATIBILITY.md](COMPATIBILITY.md).

A released baseline remains reproducible because it names an exact revision,
but the next baseline starts by refreshing upstream instead of silently
carrying the old pin. A generic defect or missing generic capability is fixed
upstream in Sail and consumed by updating the recorded revision. Marciana must
not fork, copy, or privately patch generic Sail behavior. Marciana-local code
is limited to memory-specific schemas, proposal-producing computation, and the
adapter boundary.

CI verifies that the recorded revision is the fetched current upstream commit,
builds it from source, and, once crates exist, requires the live Marciana gate.
Using an unrelated `sail` binary from `PATH` is not an integration proof.

## Extraction rules

The first code move transplants `querygraph-memory` with history and without a
simultaneous redesign. It retains the crate name, route behavior, database
prefixes, record identifiers, storage format, and tests. qg-rust switches to
the relocated crate and proves route and reopen compatibility before the Grust
copy is removed.

Only after that baseline is green may the code split into API, cognition,
Grust, Sail, LakeCat, service, and facade crates. LakeCat integration may be a
feature boundary for local-only deployments, but it is mandatory for every
catalog-backed governed `improve` path. Committed dependencies must be released
versions or exact Git revisions reachable from a remote; a sibling checkout
layout may be supported locally but is never required to build or test a clean
clone.

## Code structure

Production modules are small and single-purpose. Canonicalization, validation,
state transitions, and adapter-independent policy are implemented once in
central helpers. Adapters translate at boundaries rather than reproduce domain
logic. Tests live in separate files or integration-test targets so production
modules remain readable and reviewable.
