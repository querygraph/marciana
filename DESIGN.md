# Marciana Design and Ownership Decision

**Status:** accepted

**Decision date:** 2026-08-05

## Decision

Marciana is a standalone QueryGraph-stack project and the reusable memory and
cognition product layer. It is a composition layer, not another security,
storage, catalog, or compute substrate.

The native API is `remember`, `recall`, `improve`, and `forget`. Marciana owns
their orchestration, durable cognition jobs, memory-specific schemas and
adapters, receipts, recovery, and compatibility. Cognee is design inspiration,
not an API compatibility target, runtime, store, adapter, or product
completeness dependency. A Cognee-shaped edge facade is outside the baseline
and requires a separate future decision.

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

## Semantic ownership and DRY

Cross-stack evidence and state rules have one authoritative implementation:

- TypeSec canonicalizes capability-bound source manifests, cognition bindings,
  proposals, prepared commits, and security receipts.
- LakeCat canonicalizes governed scan proofs and catalog authorization evidence.
- Grust canonicalizes generic guarded-commit requests and backend receipts.
- Marciana canonicalizes its signed four-verb intents, composite governed
  source scope, job transitions, public wire values, and product-level receipt
  projection.

Adapters translate owned values and delegate validation; they do not reproduce
another layer's digest profile, state machine, retry rule, or policy check.
Shared Marciana semantics live in small domain modules and are reused by the
embedded API, workers, HTTP service, and QueryGraph integration. A convenience
API is rejected if it creates a second mutation, recovery, or authorization
path.

## Security invariant

Only the capability-gated TypeSec vault may rehydrate or mutate protected
memory. Retrieval engines return candidates or ranked identifiers. Cognition
workers receive an authorized immutable input bundle and emit inert proposals.
TypeSec reauthorizes and validates a proposal before Marciana maps the prepared
commit into one atomic Grust operation.

A LakeCat governed-scan proof identifies an authorized catalog snapshot; it
does not prove that arbitrary caller-supplied text came from that snapshot.
Marciana's trusted LakeCat adapter therefore owns scan execution,
row-to-`MemoryDraft` translation, and an exact per-scan draft allowlist. Each
governed write presents TypeSec with the proof evidence and a one-use binding
to the domain-separated digest of the exact draft. TypeSec attaches the opaque
source scope only after its host verifier consumes that binding. Public APIs
must never accept a proof plus an independently supplied draft as equivalent
evidence.

The composite governed source scope is versioned and domain separated. It
binds LakeCat's canonical source-scope digest to Marciana's exact field
mapping, ingestion profile, and row-to-`MemoryDraft` transformation version.
LakeCat remains authoritative for catalog proof, snapshot, and effective
projection semantics. TypeSec accepts the composite digest as opaque context
and attaches it only after its trusted verifier consumes the one-use draft
binding. This prevents a valid scan proof from being replayed with different
ingestion semantics without duplicating either owner's canonicalization.

Queues, outboxes, audit records, and logs contain identifiers or digests, not
memory plaintext, reusable authorization material, raw lease tokens, or raw
worker and failure data.

Public proposal and result diagnostics are deliberately redacted. A cognition
proposal is transient internal data: it is neither a public QueryGraph value
nor durable scheduler state. Recovery re-executes deterministic planning and
requires the exact durably expected digest before any apply operation.

## Cognition orchestration

Marciana exposes `improve` as one authenticated operation. It does not expose a
caller-driven plan/apply split. The worker owns this ordered state machine:

```text
authenticate + bind intent
        -> persist/recover job + acquire renewable lease
        -> TypeSec preauthorization + governed LakeCat scan
        -> trusted mapped ingestion through TypeSec
        -> fixed Sail engine execution
        -> LakeCat grant/snapshot revalidation
        -> TypeSec manifest-only reauthorization
        -> exact proposal-digest stage
        -> atomic guarded commit or typed no-change
        -> durable recovery + commit-bound TypeDID receipt
```

No proposal-derived result becomes observable before both post-engine gates.
The LakeCat observation time must remain attached to the revalidation evidence
and be checked at the TypeSec apply boundary; it is not treated as a signed
lease or an atomic catalog revision witness. The final guarded operation
revalidates source revisions, binding, labels, authorization, projections, and
proposal digest at the last available trusted boundary.

Jobs are leased, idempotent, and restartable. Lease renewal has structured
lifetime and stops when the owning operation completes or is cancelled. Lost
responses recover the backend commit identity and stored terminal outcome;
they do not replay an unchecked mutation. A valid proposal with no mutations
commits a typed no-change terminal outcome atomically with its audit evidence,
without a fabricated memory write or index-outbox entry.

Blocking TypeSec vault and storage operations execute behind a narrow blocking
adapter boundary. Async service and worker tasks await those adapters rather
than blocking the Tokio executor. Capabilities remain non-cloneable and are
moved into the exact authorized operation; convenience wrappers must not
weaken that property.

## Evidence and time semantics

Cognition audit evidence and TypeDID receipts have explicit schema versions.
They distinguish:

- the LakeCat governed grant/source-scope digest from the catalog snapshot
  digest;
- Marciana's composite ingestion-scope digest from its constituent catalog
  proof;
- the authorized input-manifest digest, expected proposal digest, prepared
  commit digest, and final committed-outcome digest; and
- request, prepared, LakeCat-revalidated, backend-committed, recovered, and
  receipt-issued times.

One timestamp is never reused as evidence for a different phase. Receipt
construction consumes a complete recovered durable outcome, so a caller cannot
construct an apparently valid receipt and fill security-relevant fields later.

## Persistence and computation

Marciana owns the logical memory ledger and memory-specific graph projection.
Grust owns the physical transaction engine. A production apply operation must
atomically check source revisions, claim idempotency, mutate the memory graph,
write an ID-only index outbox, persist audit-safe evidence, and retain a
recoverable backend commit identity.

The same transaction records the terminal cognition outcome. The no-change
variant follows the same job, idempotency, authority, audit, and recovery path,
but has zero memory mutations and zero index work.

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
