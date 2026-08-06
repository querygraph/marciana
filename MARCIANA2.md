# Marciana 2: A Governed Memory System Plan

**Status:** Phase 1 in progress; accepted design constraints preserved

**Reviewed:** 2026-08-05

**Scope:** comparison of Marciana with Mem0, Graphiti, Zep, Cognee, Letta,
and LangMem, followed by a prioritized product and implementation plan

## Implementation status

Phase 1 has begun with `marciana-ledger`, the adapter-independent assertion
domain. It establishes collision-resistant assertion identity, bounded source
lineage, temporal intervals, exact confidence, and a fail-closed lifecycle
whose transitions carry canonical causal assertion identifiers and evidence
digests. The next Phase 1 unit is a guarded durable projection and migration;
it must consume these types rather than reproduce their validation or state
rules.

## Executive judgment

Marciana should not become another memory extraction framework. Its defensible
design is a **governed memory system**: capability-bound spaces,
information-flow labels, provenance and quarantine, proposal-only cognition,
fresh authorization at commit time, atomic guarded mutation, and recoverable
receipts. None of the reviewed projects combines those properties as its core
contract.

The competitive systems are ahead in the layer immediately above that trust
kernel. Mem0 makes fact formation easy. Graphiti has a mature temporal
assertion graph. Zep assembles prompt-ready context and operates it as a
managed product. Cognee exposes configurable ingestion and retrieval
pipelines. Letta makes memory a durable part of agent state. LangMem separates
hot-path from background learning and treats prompts as procedural memory.

The recommended direction is therefore:

> Preserve Marciana's security and commit architecture, then add a small,
> coherent context-and-learning product on top of it.

The highest-value additions are a governed context compiler, explicit
assertion identity and temporal belief state, policy-bound memory formation,
typed memory views, and a first-class evaluation and observability plane.
Agent self-editing and prompt optimization should be supported only as
versioned cognition proposals—not as a new mutation path.

## Review method and caveats

The supplied survey was used as a comparative brief and checked against the
projects' current official documentation. Product names, APIs, and hosted
features are moving quickly, so this plan depends on architectural ideas rather
than exact competitive feature counts.

Several qualifications matter:

- Graphiti and Zep are related but not interchangeable. Graphiti is the open
  temporal-graph framework; Zep adds proprietary extraction, observations,
  context assembly, governance, and managed operation.
- Graphiti currently documents Neo4j, FalkorDB, and Amazon Neptune backends,
  not only Neo4j-compatible stores.
- Cognee's public API and documentation are changing; some `add` and
  `cognify` pages are now presented as legacy operations. Marciana should copy
  the pipeline concepts, not chase its surface API.
- Letta's current context hierarchy includes blocks, files, and other external
  context mechanisms. The classic core/archival split remains conceptually
  useful but is not a complete product description.
- Vendor benchmark and latency claims are unsuitable as acceptance criteria.
  Marciana needs reproducible, workload-specific evaluation with security and
  temporal-correctness dimensions in addition to answer accuracy.

## Comparative assessment

| System | Strongest idea to adopt | Marciana's present advantage | Gap to close | Do not copy |
|---|---|---|---|---|
| Mem0 | Simple fact formation and consolidation behind a tiny API | Capability, label, provenance, quarantine, and atomic-commit semantics | A default formation policy and low-friction SDK loop | Flat facts as the authority; model-decided deletion without a guarded lifecycle |
| Graphiti | Assertion-level bi-temporal graph, episode provenance, hybrid retrieval, typed entities | Retrieval cannot bypass the vault; derived data inherits security state | Explicit fact identity, temporal conflict lifecycle, entity schemas, retrieval recipes | Treating graph namespace as authorization or graph edges as protected-content authority |
| Zep | Token-budgeted context assembly, summaries, observations, user/thread ergonomics, operations | Stronger cryptographic and information-flow boundary; self-hostable composition | A prompt-ready governed context product and production control plane | Conflating hosted proprietary capabilities with the OSS engine; unverifiable latency claims |
| Cognee | Composable, resumable ingestion/cognition pipelines over heterogeneous sources | One guarded mutation path and clearer ownership across security, compute, and storage | Pipeline profiles, schema registry, progress, replay, and heterogeneous source adapters | A broad search enum as the primary API or separate stores with compensating consistency |
| Letta | Durable agent state, shared blocks, model-initiated memory actions | Memory tools cannot mint authority; quarantine and policy are structural | Governed working sets and agent-initiated proposals | Letting a model directly rewrite authoritative memory or always injecting unbounded blocks |
| LangMem | Hot-path/background split and procedural memory as prompt evolution | Prompt changes can be subjected to the same proposal and policy boundary | Background formation, feedback datasets, versioned prompt artifacts | Autonomous prompt mutation without evaluation, approval, rollback, or scope controls |

### What is already distinctive

Marciana's baseline should explicitly market and test these properties rather
than hiding them beneath generic “AI memory” language:

1. **Authority is separate from ranking and cognition.** Indexes return IDs;
   workers emit inert proposals; only `MemoryVault` reveals or mutates.
2. **Information flow is part of memory semantics.** Space, purpose,
   clearance, validity, quarantine, and the join of source labels apply to
   every derived artifact and recall path.
3. **Memory poisoning has a lifecycle.** Untrusted model output begins
   quarantined and cannot silently become current truth.
4. **Writes are durable protocols, not database calls.** Source
   preconditions, idempotency, graph mutation, ID-only index work, evidence,
   and recoverable commit identity form one atomic outcome.
5. **Governed data can safely feed cognition.** LakeCat evidence and TypeDID
   identity bind who asked, for what purpose, over which immutable input.
6. **Forgetting is a scoped, audited operation.** It is distinct from
   supersession and cannot degrade into an unqualified `forgetAll` endpoint.

### Where Marciana is behind

The current design is much more complete as a trust architecture than as a
developer-facing memory product. Its material gaps are:

- no prompt-ready context assembly contract;
- no complete assertion identity and conflict model in the durable graph;
- no first-class episode, fact, observation, summary, or procedural-memory
  views;
- no opinionated default extraction/consolidation profile comparable to
  Mem0's one-call experience;
- no stable schema/ontology registry for domain-specific entities and edges;
- no hot-path versus background formation policy;
- no governed agent working-set or memory-block abstraction;
- no safe prompt-learning lifecycle;
- no end-to-end evaluation corpus, relevance telemetry, or explanation model;
  and
- no hosted operational baseline for quotas, isolation, migrations,
  backup/restore, SLOs, and cost controls.

## Product model

Keep `remember`, `recall`, `improve`, and `forget` as the only authoritative
lifecycle verbs. Add typed resources and modes beneath them rather than more
top-level mutation APIs.

### Memory objects

Use six explicit, related object classes:

| Object | Role | Authority status |
|---|---|---|
| **Episode** | Immutable ingested event or source unit with event and ingestion time | Evidence; protected through the vault |
| **Assertion** | Atomic claim with stable identity, subject, predicate, object, confidence, validity interval, and lineage | Current or historical belief; never silently overwritten |
| **Entity** | Resolved identity with typed schema and aliases | Projection over assertions, not an independent truth source |
| **Summary** | Bounded synthesis of episodes/assertions/entities | Derived and label-joined; cites exact sources |
| **Observation** | Evidence-backed recurring pattern, commitment, transition, or anomaly | Derived hypothesis with support threshold and validity |
| **Procedure** | Versioned prompt, instruction, example, or tool policy used by an agent | Executable configuration; requires a stronger approval policy than ordinary facts |

This avoids forcing all memory into flat sentences while also avoiding the
claim that every useful context item is naturally a graph edge.

### Assertion lifecycle

Replace structural edge identity with a stable `assertion_id`. Model authored
and derived assertions independently and record:

- `observed_at`, `valid_from`, `valid_to`, and `ingested_at`;
- source episode and exact source-record lineage;
- extraction algorithm, model, prompt/profile, and schema versions;
- label join, quarantine/taint, confidence, and corroboration count;
- lifecycle state: `proposed`, `current`, `disputed`, `negated`, `superseded`,
  `retracted`, or `forgotten`; and
- the assertion IDs and evidence that caused each transition.

Contradiction detection must produce a proposed transition. It must not delete
or invalidate an assertion directly. The guarded commit validates temporal
preconditions and applies the transition atomically. “Unknown,” “disputed,”
“historically true,” and “currently believed” must remain distinguishable.

### Governed context compiler

Add a pure planning component that converts a `RecallIntent` into a
`ContextPlan`, followed by vault-authorized materialization into a
`ContextBundle`.

Inputs should include subject, purpose, memory spaces, clearance ceiling,
query or recent thread messages, point-in-time/as-of constraints, token budget,
allowed memory views, freshness, diversity, and citation requirements.

The planner may combine lexical, vector, entity, assertion, neighborhood,
temporal, and community candidates. It returns IDs, scores, reasons, estimated
tokens, and a deterministic plan digest. The vault then applies the common
visibility gate and materializes only permitted content. The final bundle
contains typed sections, source citations, temporal qualifiers, redaction
counts, a truncation explanation, and the plan/receipt identity.

This is Marciana's answer to Zep's context block, with three improvements:
authorization happens before content leaves the vault, the output is typed
rather than only formatted text, and every included item is explainable and
receipt-bound. An XML/text renderer can be provided as a convenience view.

### Memory formation profiles

Make the simple path genuinely simple:

```text
remember(source, formation = "profile")
recall(intent, output = "context")
improve(target, profile = "background-consolidation")
forget(scope, mode = "erase" | "retract")
```

A versioned formation profile selects chunking, extraction schema, temporal
parsing, entity resolution, deduplication, contradiction policy, summarization,
and embedding. Ship conservative profiles for conversation preferences,
episodic history, documents, structured business events, and no-inference raw
storage. Profiles compile into durable cognition jobs and proposals; they do
not create another write path.

Every inferred memory should expose a formation explanation: why it was
retained, what evidence supports it, which existing memories were considered,
which operation was proposed, and which model/profile produced the decision.

### Working sets and procedural memory

Support Letta-style always-available state as a **working set**, implemented as
a saved context policy plus bounded slots pointing to governed memory objects.
Slots may be fixed by an operator or proposed by an agent. At each turn the
context compiler resolves them under current policy; a slot is not a plaintext
authorization bypass and does not guarantee that its content remains visible.

Treat prompt optimization as cognition over `Procedure` objects:

1. collect consented trajectories and explicit outcome feedback;
2. propose a new immutable procedure version;
3. evaluate it against safety, regression, leakage, and task suites;
4. require policy-defined human or service approval;
5. activate it for a bounded agent/cohort with rollback; and
6. retain provenance and outcome telemetry.

An agent may request a working-set or procedure change, but its tool call only
starts this proposal flow.

## API evolution

Keep the four verbs stable and add versioned request types:

- `remember(RememberRequest) -> RememberOutcome` accepts text, messages, JSON,
  files, governed catalog snapshots, or references; supports synchronous raw
  persistence and optional asynchronous formation.
- `recall(RecallIntent) -> RecallOutcome` supports `items`, `graph`,
  `timeline`, and `context` views through one authorization path.
- `improve(ImproveRequest) -> JobHandle` runs named formation, consolidation,
  observation, community, re-embedding, or procedure-learning profiles.
- `forget(ForgetRequest) -> ForgetOutcome` distinguishes physical erasure,
  logical retraction, expiry, and derived-artifact cleanup while preserving
  only the minimum permitted evidence.

Add read-only job, explanation, lineage, receipt, schema, and health resources.
Do not encode every retrieval algorithm into a public `SearchType` enum.
Expose stable intent and output semantics; keep retrieval recipes versioned and
server-selectable, with an expert override for reproducible evaluation.

Agent/framework adapters for MCP, LangGraph, Letta, and common SDKs should be
thin translations into this contract. They must not reproduce authorization,
consolidation, mutation, or recovery logic.

## Delivery plan

This plan extends, and does not reorder, the accepted extraction sequence in
`DESIGN.md`. No Marciana 2 feature work begins by redesigning the crate being
transplanted.

### Phase 0 — Preserve the baseline

Complete the behavior-preserving `querygraph-memory` transplant, replace
sibling path dependencies with released or reachable exact revisions, switch
qg-rust, and prove route, wire, durable-ID, database reopen, denial, receipt,
and recovery compatibility. Record the exact Sail source build and live gate.

**Exit:** the standalone repository is a clean-clone, reproducible owner of
the existing product behavior, and foundational repositories do not depend on
it.

### Phase 1 — Make the ledger assertion-safe

Introduce durable collision-resistant IDs, explicit episode and assertion
identity, full source lineage, temporal intervals, authored/derived layers,
and the conflict lifecycle. Add versioned migrations from structural `RELATES`
edges and prove reopen, retry, collision, rollback, and mixed-version reads.
Finish mandatory audited quarantine promotion and taint propagation.

**Exit:** two assertions with the same structural triplet remain distinct;
past, current, disputed, retracted, and forgotten states have deterministic
queries and guarded transitions.

### Phase 2 — Ship durable formation

Deliver the job state machine, leases, cancellation, progress, bounded retry,
provider/profile registry, input and output schema versions, and local plus Sail
executors. Ship conservative conversation, document, JSON-event, and raw
profiles. Add persistent tenant-scoped hybrid indexing with embedding-space
identity and atomic ID-only repair work.

**Exit:** crash/restart and lost-response tests prove exactly-once authoritative
outcomes; stale inputs, revoked authority, label mismatch, provider failure,
and malformed proposals fail closed without partial state.

### Phase 3 — Ship governed context

Implement `RecallIntent`, retrieval recipes, the pure context planner,
vault-authorized materialization, token budgeting, typed context bundles,
citations, temporal qualifiers, explanations, and text/XML renderers. Add
threads and sessions only as product metadata that select spaces and recall
policy—not as authorization namespaces.

**Exit:** every returned byte is traceable to an authorized item; equivalent
recall paths pass the same visibility corpus; budgets are deterministic; and
redacted or quarantined candidates cannot leak through summaries, scores, or
explanations.

### Phase 4 — Add learning without self-corruption

Add observations, feedback datasets, working sets, and versioned procedures.
Implement background consolidation and opt-in hot-path proposals. Add offline
evaluation, approval, cohort rollout, rollback, and retention for trajectories.

**Exit:** no agent or optimizer can directly activate a prompt, clear
quarantine, widen a working set, or mutate memory; every activation is
policy-approved, evaluated, reversible, and receipt-producing.

### Phase 5 — Productize and integrate

Publish Rust, Python, and TypeScript clients from shared wire fixtures; add MCP
and framework adapters; ship tenant quotas, encryption-key boundaries,
migrations, backup/restore, audit export, dashboards, graph/lineage inspection,
cost accounting, and SLOs. Keep LakeCat optional for local memory but mandatory
for catalog-backed governed cognition.

**Exit:** a clean deployment can be restored from backup, upgraded across the
supported schema window, isolated under adversarial multi-tenant tests, and
operated from documented signals without inspecting protected plaintext.

## Evaluation program

Benchmarking is a product feature because it controls what memory formation and
retrieval changes are safe to release. Build a versioned evaluation corpus with
synthetic non-secret fixtures and user-owned private fixtures that never leave
their authorized environment.

Measure at least:

- current, historical, and as-of temporal accuracy;
- contradiction, supersession, retraction, and resurrection correctness;
- single-hop and multi-hop recall, citation precision, and source coverage;
- preference retention, episode recall, observation precision, and procedure
  regression;
- poisoned-memory rejection and quarantine propagation;
- cross-tenant, cross-space, purpose, clearance, and inference leakage;
- token utility: answer quality per injected token and context redundancy;
- write amplification, formation cost, retrieval P50/P95/P99, and recovery
  time; and
- idempotency, crash consistency, stale-proposal rejection, deletion closure,
  and index repair convergence.

Publish provider-neutral results with exact dataset, model, embedding, prompt,
profile, hardware, cache, and revision metadata. Use LoCoMo and LongMemEval only
as external smoke tests; release gates should be based on Marciana's governed
and temporal corpus.

## Priority decisions

The following choices are reviewed and recommended:

1. **Build context assembly before adding many public search modes.** It is the
   largest user-visible gap and creates one safe retrieval product.
2. **Fix assertion identity before richer cognition.** Otherwise new
   extraction modes compound an inadequate graph schema.
3. **Represent agent autonomy as proposals.** This imports the value of Letta
   and LangMem without importing their authority assumptions.
4. **Make formation profiles declarative and versioned.** This provides Mem0's
   ease and Cognee's configurability while retaining reproducibility.
5. **Keep one logical ledger and atomic mutation boundary.** Vector, lexical,
   graph, and distributed compute remain projections or proposal engines.
6. **Defer a standalone hosted service until the embedded path and failure
   semantics are stable.** The API, ledger, and compatibility fixtures come
   first; deployment topology must not define semantics.
7. **Do not implement a Cognee-compatible facade in this program.** It remains
   a separately approved edge-adapter decision under `DESIGN.md`.

## Sources reviewed

- Mem0: [add operation](https://docs.mem0.ai/core-concepts/memory-operations/add),
  [update operation](https://docs.mem0.ai/core-concepts/memory-operations/update),
  and [delete operation](https://docs.mem0.ai/core-concepts/memory-operations/delete)
- Graphiti: [welcome and concepts](https://help.getzep.com/graphiti/getting-started/welcome),
  [adding episodes](https://help.getzep.com/graphiti/core-concepts/adding-episodes),
  and [quick start/search](https://help.getzep.com/graphiti/getting-started/quick-start)
- Zep: [Zep versus Graphiti](https://help.getzep.com/zep-vs-graphiti),
  [key concepts](https://help.getzep.com/concepts), and
  [context types](https://help.getzep.com/context-types)
- Cognee: [pipelines](https://docs.cognee.ai/core-concepts/building-blocks/pipelines),
  [`cognify`](https://docs.cognee.ai/python-api/cognify), and
  [`search`](https://docs.cognee.ai/python-api/search)
- Letta: [memory blocks](https://docs.letta.com/guides/core-concepts/memory/memory-blocks)
  and [context hierarchy](https://docs.letta.com/guides/core-concepts/memory/context-hierarchy)
- LangMem: [concepts and integration patterns](https://langchain-ai.github.io/langmem/concepts/conceptual_guide/),
  [memory API](https://langchain-ai.github.io/langmem/reference/memory/), and
  [prompt optimization](https://langchain-ai.github.io/langmem/reference/prompt_optimization/)
- Marciana: [`DESIGN.md`](DESIGN.md), the TypeSec-side
  `MARCIANA-PROJECT.md`, `MARCIANA.md`, and `MEMORY.md` handoff documents, and
  the current TypeSec memory contracts and tests
