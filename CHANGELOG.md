# Changelog

All notable changes to Marciana are documented in this file.

## Unreleased

- Pin the refreshed Sail candidate to the exact reachable QueryGraph graph
  revision carrying upstream performance PR #2400's head, while preserving the
  explicit production-binary live gate and distinguishing an open PR from a
  merged upstream release. The optimized pinned artifact passes all 26 Sail
  adapter tests, the dedicated non-null and temp-view checks, and Marciana's
  two live cognition parity/secrecy tests.
- Reuse an outbox claim's already-loaded durable outcome during full commit
  recovery instead of loading and decoding the same graph node twice.
- Add end-to-end production benchmarks for authoritative cognition mutation,
  no-change, recovery, and outbox claim paths.
- Reuse the exact validated scheduler job state when a high-level claim
  acquires its lease, while retaining fresh-load recovery for CAS races.
- Add production benchmarks for durable cognition submission, loading, lease
  claims, and progress transitions.
- Canonicalize total-ordered observation evidence in place without stable-sort
  scratch allocation, with a deterministically scrambled benchmark corpus.
- Format parsed assertion UUIDs once for both canonicality checking and owned
  identity construction.
- Validate native cognition snapshots without hashing and discarding their
  JSON, reuse the validated label join during proposal construction, compare
  canonical TypeSec manifest IDs directly, and sort only borrowed metadata.
- Canonicalize total-ordered assertion transition evidence with in-place
  unstable sorting, avoiding stable-sort scratch allocation, and cover both
  descending and deterministically scrambled evidence in benchmarks.
- Represent closed assertion-query state membership as a compact mask instead
  of a heap-allocated tree, preserving duplicate-insensitive query semantics.
- Canonicalize cognition source selections through sorted borrowed IDs before
  cloning only binding-owned output, share projection validation across the
  composition boundary, and validate three-field mapping uniqueness in place.
- Totally order feedback records by trajectory, time, and outcome so dataset
  identity is input-order independent, using in-place unstable canonical sort.
- Recompute vector-manifest digests without cloning membership and apply
  repair batches through a prevalidated per-ID delta instead of a full set copy.
- Partition contradiction candidates by normalized assertion prefix before
  pair generation, and normalize text in one allocation without a word vector.
- Evaluate validated context-plan candidates through borrowed case membership
  checks and average corpus metrics in one pass without temporary collections.
- Return owned session- and thread-bound recall intents directly instead of
  cloning their bounded working-set IDs after validation and digest binding.
- Size governed snapshot JSON buffers from already-validated projection bytes
  before canonical serialization and digesting.
- Pre-size observation and feedback canonical identity buffers to avoid
  geometric growth copies while preserving their exact durable digests.
- Digest canonically ordered cognition audit IDs in place while retaining an
  order-independent fallback for defensive export of manually built records.
- Generate audit-export benchmark evidence with the canonical affected-ID
  ordering required by validated TypeSec cognition outcomes.
- Validate canonical schema-window versions directly as ASCII decimal text
  instead of allocating a formatted copy of every parsed version.
- Detect duplicate cognition projection fields without tree-node allocation,
  using an allocation-free small-input path and a compact sorted borrowed index
  for larger governed projections.
- Lower validated Ossie documents by consuming their owned strings, reserving
  exact output capacities, and delegating duplicate detection to ontology
  canonicalization instead of building a redundant tree index.
- Reuse validated working-set slot ordering during canonical digesting instead
  of allocating and sorting the same identities twice.
- Validate Ossie selections through its canonical sorted indexes, using binary
  search for metrics and a single merge-style pass for dimension subsets.
- Resolve point-in-time assertion state by binary partitioning the validated
  monotonic transition log instead of scanning its full history.
- Cache indexed-vector norms, compute each query norm once, avoid entity locks
  for plain retrieval, and partially select bounded vector results before
  cloning and deterministically sorting the winners.
- Make large context plans use one validated candidate and pinned-ID
  membership index instead of repeated linear scans during ordering.
- Add production-profile Criterion coverage for ledger history queries,
  vector and hybrid retrieval, context planning, Grust-backed memory-store
  operations, analytics planning, evaluation, sessions, vector manifests,
  assertion projection, cognition canonicalization, accounting, operational
  metadata, ontology, formation, learning artifacts, working-set recall,
  audit export, lineage, rollout, governed identity, assertion identity parsing,
  the full native cognition engine boundary, and the LakeCat proof adapter.
- Replace stale Letta benchmark claims across results, blog, and book sources
  with the retained current App Server/Agent SDK result, and explicitly leave
  adapter-mediated isolation unsupported.
- Update the adversarial-benchmark blog post with the completed six-system
  comparison, a dedicated section on why the boundary is unforgeable rather
  than merely careful, links to the new adversari.al site and the
  *Adversarial Cognition* FirstPair book, and an invitation for
  vendor-authored adapters and other contributions. Replace the headboard
  with the Plato-Diogenes rug-dispute illustration and keep the Venetian
  galleys headboard as `headboard-rovinj.png`.
- Factor the adversarial cognition benchmark out to the standalone project
  querygraph/adversarial-cognition and point Marciana's benchmark document,
  book, blog post, and results docs at it; Marciana keeps the vendored suite
  as its release gate.
- Record the first cross-system comparative runs (Akka + Fluree, Letta) from
  the standalone repository's executable OSS adapters in the docs and blog.

- Prepare the `0.12.1` registry release for `querygraph-memory`,
  `marciana-catalog`, `marciana-cognition`, and `marciana-ledger`, replacing
  development Git dependencies with released TypeSec, Grust, and LakeCat
  crates.
- Track TypeSec main for active QueryGraph integration so qg-rust and Marciana
  share one TypeSec dependency universe during development.
- Stamp the adversarial-benchmark blog textpack with its versioned link and
  version marker through the FirstPair delivery workflow.
- Opt marciana-memory into the workspace pedantic lint set and resolve every
  surfaced finding, pin MSRV and workspace dependency specs across all
  crates, split the memory crate root into bridge, graph-codec, and
  legacy-migration modules with one node-id authority and an unchanged
  public API, and make graph traversal dedupe O(1) while preserving order.
- Add the adversarial-benchmark blog headboard — Venetian galleys racing
  toward Rovinj led by the Bucintoro under the banner of St Mark, with
  Marciana on the flagship's side — rendered reproducibly from a committed
  SVG-composition script.
- Cover MARCIANA-ADVERSARIAL-v1 in the book: the delivered adversarial
  release corpus replaces the aspirational one, a new appendix section pairs
  the adversarial gate suite with the context-evaluation protocol, and the
  release checklist requires a passing run with every hard gate at zero.
- Preserve vector manifest membership when re-indexing an existing record
  fails; rollback now removes only newly inserted members.
- Bind Ossie metric expressions and labeled, count-prefixed sections into
  binding digests under a v2 domain tag, and reject control characters in
  Ossie text fields.
- Validate query and reason digests with the single authoritative hex-checking
  validator.

## 0.12.1 — 2026-08-06

- Published `querygraph-memory`, `marciana-catalog`, `marciana-cognition`, and
  `marciana-ledger` against released TypeSec 0.13.1, Grust 0.12.1, and LakeCat
  0.3.0 crates.
- Validate cognition progress leases against trusted time instead of
  worker-supplied timestamps.
- Align the Python and TypeScript clients and the coffee demo to the pinned
  v1 wire contract: `space_id` field naming, exact four-verb shapes without
  client-only kind/clearance/mode fields, per-item forget ID validation, and
  a shared-fixture round-trip test in both clients.
- Reject mismatched receipt operations in the Python client and validate
  nested improve replacements and forget ID items in the TypeScript client.
- Send governed forget from the coffee demo with the shared `memory_ids` and
  `purpose` fields, report honest receipt outcomes, and make the demo as-of
  date injectable for deterministic tests.
- Print a human smoke-benchmark summary without `--json`, share one scoring
  helper across smoke backends with sorted redactions, and cap corpus query
  length in the strict loader.
- Correct stale documentation: present-tense extracted-baseline framing in
  the README, the TypeSec pin row, the archived QueryGraph memory-goal
  status, book excerpts aligned to the real client API and wire, chapter
  references by name, consistent Ossie naming and book-title citations,
  canonical smoke-benchmark numbers, demo prerequisite ordering, and merged
  duplicate unreleased changelog date headings.
- Publish the comprehensive MARCIANA-ADVERSARIAL-v1 benchmark document —
  threat model, corpus, gate mapping, report schema, recorded results, and a
  fairness policy answering anticipated comparative-system objections — as
  tracked Markdown and PDF, with a companion blog post linking the code.
- Let external adversarial adapters report their own recorded version and
  declare individual cases unsupported; unsupported cases are counted
  separately and never scored as passes, failures, or gate violations.
- Deliver the MARCIANA-ADVERSARIAL-v1 benchmark: an eighteen-case adversarial
  corpus with a versioned manifest, a deterministic policy-aware reference
  backend with durable replay protection and idempotent receipts, hard-gate
  and category-metric evaluation, percentile performance measurement, and a
  bounded machine-readable report.
- Enumerate comparative systems (Mem0, Zep, Letta, Cognee, Graphiti, and
  Akka + Fluree) through an explicit command-configured adapter protocol that
  reports executed, error, or unavailable status without substitution.
- Inventory pinned offline-only public corpora (LoCoMo, LongMemEval, BEAM,
  DMR, Letta-Evals) with exact source revisions in the adversarial report.
- Record the passing adversarial reference run in the benchmark results.
- Run the dependency-free Python benchmark suite in CI alongside the Rust
  workspace checks.
- Document the MARCIANA-ADVERSARIAL-v1 goal, release gates, implementation
  checkpoint, comparative-system status, and acceptance criteria.
- Add the Marciana 2 release post with Ossie, benchmark, cognition, and
  end-to-end coffee-market coverage, including a generated library headboard.
- Add the source-owned Marciana book manuscript, FirstPair build contract,
  diagrams, benchmark chapter, enterprise semantic-layer discussion, and
  publisher-mark-matched cover assets.
- Include the book root in HTML resource resolution so cover assets embed in
  single-file and chapter readers.
- Expand the manuscript with repository boundary maps, enterprise deployment
  patterns, evaluation protocol, and Ossie/Croissant conformance guidance.
- Keep the evaluation formulas portable across PDF, EPUB, and HTML renderers.
- Let Pandoc provide the single canonical heading-numbering scheme in every
  edition.
- Add the tracked QueryGraph blog headboard to the FirstPair book configuration.
- Register the Marciana slug and QueryGraph shelf in the source FirstPair contract.
- Add a tested Apache Ossie JSON importer and deterministic ontology/query-plan adapter.
- Clarify Fluree as the semantic-ledger/query role in the comparative Akka port,
  with TypeDID and TypeSec preceding Marciana projections.
- Advance the Sail compatibility pin to the merged upstream baseline.
- Add scope-bound vector manifests and atomic ID-only repair batches.
- Persist and recover vector manifests through guarded Grust commits.
- Revalidate decoded vector scope identities before manifest recovery.
- Add deterministic content-free context evaluation for quality, token utility,
  and forbidden-ID leakage.
- Add bounded ordered evaluation corpora and stable aggregate summaries.
- Add evaluator-bound receipts for reproducible context-evaluation releases.
- Add a checked-in content-free context evaluation fixture.
- Reject release receipts for failed or leaking evaluation summaries.
- Expose materialization as-of cutoffs in bundles, citations, and renderers.
- Add thread metadata and a shared thread/session context facade path.
- Synchronize tenant vector membership manifests with index operations.
- Include materialization cutoffs in context explanations.
- Correct benchmark documentation to reflect the implemented optional BEAM adapter.
- Require provider-neutral model, embedding, prompt, profile, hardware, and revision metadata in benchmark reports.
- Make coffee demo decisions report the explicitly executed governed tool action.
- Add a full key-free coffee demo lifecycle regression.
- Record benchmark release gates and deterministic coffee-demo lifecycle status.
- Align compatibility documentation with the final Sail PR 2374 merge pin.
- Add standalone benchmark-results and coffee-market demo guides with verified code excerpts.
- Add bounded content-free P50/P95/P99 latency percentile snapshots.
- Add a capability-bound facade path for session-scoped context planning.
- Add bounded session metadata that binds session, space, and recall-policy
  identity into recall planning without changing authorization semantics.

### 2026-08-06

- Restored the executable Sail baseline gate: CI now starts the exact pinned
  upstream Sail binary (or uses an explicit `SAIL_ENDPOINT`) before running
  Marciana's ignored live cognition tests.
- Kept the durable progress validator clean under strict all-feature Clippy.
- Made learning APIs pass strict all-feature Clippy with explicit error
  contracts and borrowed evidence input.
- Extended the memory benchmark report with p99 latency, speedup, and
  reproducibility metadata required by the evaluation plan.
- Added a strict, local-only pinned corpus contract for future LoCoMo,
  LongMemEval, and BEAM adapters.
- Added closed conversation, document, JSON-event, and raw formation profiles;
  each is bound to an existing native cognition operation.
- Added explicit bounded embedding-space identity to vector indexes so model
  and preprocessing changes cannot silently reuse incompatible vectors.
- Added local LoCoMo and LongMemEval adapters pinned to exact repository
  revisions; BEAM remains explicitly unconfigured pending a verified source.
- Recorded exact BEAM and BEAM-10M dataset revisions, with a tested source-pin
  seam for the forthcoming optional Parquet normalizer.
- Added the lazy optional-PyArrow BEAM normalizer with bounded conversation-ID
  targets and explicit abstention handling.
- Added a transport-neutral typed Python client boundary for remember, recall,
  improve, and forget, with strict Pydantic v2 wire models and tests.
- Added independent wheel build metadata for the Python client.
- Added a thin MCP tool registry/dispatcher over the typed Python client,
  preserving host-owned transport and authorization.
- Verified compilation of the Sail-feature live cognition target against the
  recorded Grust bindings; live endpoint execution remains explicitly pending.
- Added a content-free, bounded health snapshot for operational integrations;
  readiness exposes component revisions without protected memory data.
- Added content-free four-verb operation metrics with denial, total-latency,
  and max-latency counters for SLO/dashboard integrations.
- Updated the compatibility registry with the independently buildable Python
  and TypeScript client baselines.
- Added validation-only Rust request contracts for the four memory verbs;
  authorization and mutation remain behind the existing TypeSec boundary.
- Added safe lowering from remember/recall requests into existing TypeSec draft
  and query types, still before capability authorization.
- Completed safe lowering for improve replacement drafts and scoped forget ID
  selectors without acquiring capabilities or mutating storage.
- Added `MemoryFacade`, which executes all four validated requests only through
  TypeSec capability-bound vault operations.
- Added an integration test proving facade remember/recall/forget execution
  against the Grust graph store and TypeSec RBAC capabilities.
- Corrected facade `improve` to use TypeSec's atomic supersession path and
  extended integration coverage to prove old history is replaced safely.
- Aligned the coffee demo's QueryGraph adapter with the supersession endpoint;
  it no longer reduces `improve` to an unrelated `remember` call.
- Aligned TypeScript wire fields with the shared Rust/Python snake_case
  contract for memory IDs and receipts.
- Added strict serde derives and a checked-in Rust four-verb wire fixture with
  unknown-field rejection.
- Centralized the remember wire fixture under `compat/fixtures` and added
  Python/TypeScript consumers to prevent cross-client schema drift.
- Aligned Python and TypeScript forget requests on the shared `memory_ids`
  wire field.
- Added runtime Python/TypeScript tests that reject regression to the old
  `ids` forget field.
- Added a closed formation registry with explicit provider capabilities and
  source/output record budgets for Reference and Sail bindings.
- Enforced the selected formation provider's source and proposal ceilings at
  the governed cognition application boundary.
- Added fail-closed context-plan verification for candidate identity, token
  accounting, ordering metadata, and plan-digest integrity before vault reads.
- Added a Graph/Sail-backed `MemoryFacade::materialize_context` seam so typed
  context bundles use the same capability-bound vault path as the four verbs.
- Added deterministic typed context sections grouped by TypeSec memory kind,
  while preserving redacted results as metadata-only entries.
- Added a content-free materialization receipt digest bound to the plan and
  visible/redacted IDs; text and XML renderers now carry both identities.
- Bound materialization receipts to the target space, clearance ceiling, and
  request purpose so equal result sets cannot be confused across policies.
- Added a bounded, digest-only working-set policy with proposal, approval,
  activation, revocation, and capability-independent recall-intent stages.
- Bound working-set identities and pinned slots into context planning and
  deterministic plan digests; pinned candidates must fit the token budget.
- Added proposal-only procedure cohort rollouts with evaluated-procedure
  binding, bounded traffic and trajectory retention, approval, activation, and
  rollback.
- Added content-free fixed-array tenant quotas with context-aware operation
  accounting, exhaustion, clock, and window-reset checks.
- Added a versioned redacted audit-export projection with hashed identities,
  lineage digests, deterministic affected-ID summaries, and schema bounds.
- Added tenant-scoped, content-free resource accounting with deployment-owned
  microcredit rates, operation separation, and saturating usage snapshots.
- Added deterministic operation-scoped SLO evaluation over content-free metrics,
  with conservative latency and denial-rate checks.
- Added a digest-only tenant/encryption-key boundary contract with exact scope
  matching and monotonic key-revision rotation; key material remains host-owned.
- Added a fixed-order digest-only lineage graph projection over audit exports,
  including bounded stage edges and affected-ID summaries.
- Added a bounded inclusive schema-family version window and backup-restore
  validation seam for supported upgrade ranges.
- Added an immutable bounded schema/ontology registry for typed fields and
  edges, with canonical ordering, exact resolution, and stable digests.
- Added an explicit tenant-scoped vector-index seam with persistable scope
  digests, embedding-space identity, and fail-closed cross-tenant operations.
- Added a closed formation run-mode contract: background remains the default,
  while eligible profiles may opt into hot-path proposal generation without
  gaining direct mutation authority.
- Added content-free formation explanations bound to the resolved profile,
  provider, run mode, evidence/proposal digests, and bounded record counts.
- Added a versioned content-free backup manifest and restore compatibility
  validator for deployment-owned backup/restore workflows.
- Added an independently buildable TypeScript client with strict four-verb
  request validation, injected transport, and Node tests.
- Ignored generated TypeScript build and dependency directories as local
  verification artifacts.

- Added lease-bound, digest-only cognition progress to the durable scheduler.
  Progress phases and counters are bounded, timestamp-monotonic, persisted
  under job schema v3, and never carry worker or model text.

- Added a closed formation provider binding contract with explicit input/output
  schema versions and fixed source/output record ceilings.

- Added a pure content-free context planner that deterministically applies a
  token budget to ranked IDs and emits a receipt-bound plan digest.

- Added typed context-bundle materialization through TypeSec's authorized
  candidate recall, preserving redacted metadata and plan identity without a
  parallel content path.

- Added deterministic citations plus bounded text and XML renderers for
  authorized context bundles; redacted candidates remain metadata-only.

- Added a closed retrieval-recipe identity to context intent and plan
  digests, keeping ranking policy deployment-owned and reproducible.

- Bound rendered context views to their plan digest so downstream consumers
  retain the exact selection identity alongside visible content.

- Added a content-free context explanation reporting selected, redacted, and
  budgeted candidate counts under the same plan identity.

- Added governed learning artifacts: evidence-backed observation lifecycles,
  bounded order-stable feedback datasets, and procedure activation gated by
  evaluation and approval.

- Bound procedure evaluation to a specific dataset and procedure digest with
  a fixed passing threshold before approval or activation is possible.

- Added a dependency-free memory benchmark harness covering temporal recall,
  abstention, redaction safety, token accounting, deterministic ranking, and
  indexed-vs-linear latency, with standalone Python tests and benchmark notes.

- Refreshed the recorded Sail compatibility baseline to upstream merge
  `2c1b2e45` from `lakehq/sail#2374`.

- Added an end-to-end Pydantic AI v2 Honduras coffee-market demo under
  `examples/coffee_market_demo`, with Dataverse loading, optional Sail Spark
  Connect execution, TypeDID/QueryGraph memory seams, and remember/recall/
  improve/forget agent turns.

- Added the closed MARCIANA2 formation-profile registry. Versioned background
  deduplication and reconciliation profiles select exactly one native
  cognition operation and reject unrecognized profile identities.

- Bound formation profiles into TypeDID cognition intent v3 and the durable
  verified request identity. Profile/config and profile/operation mismatches
  now fail before governed source materialization.

- Completed MARCIANA2 Phase 1's assertion-safe baseline: deterministic
  assertion candidates now materialize only through TypeSec's capability-gated
  ranked-ID recall, preserving purpose, validity, retention, quarantine, and
  clearance checks. Legacy-import evidence also round-trips through strict
  durable decoding.

- Updated the TypeSec compatibility pin to `14bd5427`, which supplies the
  vault-owned ranked candidate materialization primitive used by assertion
  recall.

- Added `marciana-ledger`, the canonical assertion domain for MARCIANA2 Phase
  1. Assertions now have collision-resistant identity independent of their
  structural graph triplet, bounded lineage and temporal validity, and a
  fail-closed, evidence-carrying belief lifecycle ready for guarded durable
  projection.

- Made persisted assertion values fail closed: deserialization now reuses the
  canonical identity, interval, lineage, evidence, and lifecycle validators
  rather than allowing stored JSON to construct an invalid ledger state.

- Added an inert, atomic-commit-ready Grust projection for validated
  assertions. Distinct assertion and relationship identities preserve equal
  structural triplets without creating a second memory mutation path.

- Added deterministic as-of assertion queries derived from the immutable
  transition history, preserving the distinction between historical current,
  disputed, and currently active beliefs.

- Rejected assertion transitions dated before the assertion was ingested,
  closing a temporal-history construction gap in persisted ledger recovery.

- Aligned assertion validity with TypeSec's existing half-open invalidation
  semantics, preventing an assertion from appearing current at its exact
  invalidation instant during migration or as-of recall.

- Added retry-stable conversion from legacy structural relation inputs to
  explicit assertions, retaining import evidence and historical validity
  without generating a new unguarded mutation path.

- Distinguished source-only legacy import evidence from causal assertion
  transitions, preventing migration from fabricating a self-causal belief
  change while keeping ordinary lifecycle transitions fully evidenced.

- Added a fail-closed adapter from actual legacy `RELATES` edges and trusted
  source records to retry-stable assertion projections, preserving the source
  record's half-open historical validity without exposing its content.

- Added an idempotent one-batch storage maintenance migration for legacy
  relation projections. It reports only a migration count and uses fixed
  diagnostics, keeping protected source values out of migration results.

- Verified that assertion migration preserves the established legacy graph
  neighborhood read behavior, allowing mixed-version deployment before the
  assertion-aware recall API is introduced.

- Moved assertion migration onto Grust's durable guarded-commit protocol and
  verified idempotent migration against persistent Turso storage.

- Added deterministic, content-free assertion candidate queries for as-of
  lifecycle inspection and currently valid beliefs, preserving the vault as
  the only protected-content materialization boundary.

- Completed the first verified executable Marciana baseline: clean Marciana
  and qg-rust clones pass their full declared test, doctest, and strict Clippy
  gates using exact remote dependencies; the recorded QueryGraph Sail source
  passes the bounded live executor, backend, and cognition gates.

- Adopted the verified remotely reachable QueryGraph Sail revision as the
  production baseline. Generic upstream Sail contribution remains active but
  no longer blocks Marciana delivery.

- Recorded the upstream Sail contribution path for the reviewed generic Delta
  `MERGE` correction; upstream acceptance is not a Marciana release blocker.

- Recorded the bounded Grust Sail harness and its terminal executor, backend,
  and cognition parity/secrecy results against the local Sail binary.

- Recorded QueryGraph's completed switch to the native Marciana governed
  application and its verified compatibility and recovery gates.

- Added native cognition unit coverage for canonical source-selection and
  field-mapping digest boundaries, including duplicate and ambiguous mapping
  rejection.

- Added Marciana's native governed cognition composition: verified TypeDID
  intent binding, LakeCat revalidation, TypeSec authority priming and commit,
  proposal validation, receipt signing, and the opaque `improve` operation now
  live together in `marciana-cognition`.

- Updated the delivery record to distinguish completed native error-boundary
  extraction from the still-required move of governed `improve` composition.

- Added Marciana-owned, fixed public errors for governed intent, proof,
  projection, authority, and proposal binding failures. The stable categories
  make the fail-closed composition boundary available without a QueryGraph
  implementation dependency.

- Added Marciana's fixed public mapping for TypeSec protected-memory failures.
  Callers receive stable, non-disclosing categories rather than policy details,
  record identifiers, or backend-controlled text.

- Added `marciana-cognition` with the closed host-selected engine binding.
  Only fixed reference and live Sail profiles can receive protected cognition
  input; arbitrary engine implementations cannot self-assign trusted identity.
  Its explicit test-support feature supplies test doubles without exposing that
  construction path to production consumers.

- Added `marciana-catalog`, the native LakeCat proof-to-cognition-source
  adapter. It preserves LakeCat ownership of proof validation while moving the
  memory-product translation out of QueryGraph.

- Added Marciana-owned cognition proof-boundary validation, including the
  configured-catalog check and the stricter flattened table-identity budget
  required before a LakeCat proof can reach a cognition engine.

- Replaced Marciana's TypeSec and Grust sibling paths with exact reachable
  Git revisions, allowing its workspace to build and test independently of the
  local QueryGraph checkout layout.

- Recorded passing fresh-clone gates for Marciana and qg-rust, including the
  exact Git-pinned stack revisions and the non-blocking generic Sail upstream
  contribution.

- Recorded the passing focused local Sail integration gate for the Grust
  cognition substrate while retaining the requirement for a remotely reachable
  Sail revision before Marciana claims an executable compatibility baseline.

- Replaced the obsolete scaffold-only compatibility description with the
  history-preserving extraction baseline and its explicit clean-clone and
  remotely-reachable dependency limitations.

- Recorded QueryGraph's opaque `improve` containment step: callers no longer
  receive a governed cognition proposal while the behavior-preserving
  composition is prepared for its later move into Marciana.

- Transplanted `querygraph-memory` into standalone Marciana with preserved
  commit history, modular cognition modules, separate tests, and a compiling
  workspace over the current Grust and TypeSec checkouts.
- Matched the preserved crate's `0.12.0` compatibility line so QueryGraph can
  switch to the standalone path without changing its existing package contract.
- Switched the local qg-rust consumer to the standalone Marciana crate and
  re-ran its 100-test cognition/application suite successfully.
- Added a durable scheduler claim API that returns a lease only for pending or
  retryable work; a staged proposal now returns its digest for proposal-free
  recovery and can never be leased for re-planning.

### 2026-08-05

- Refreshed the canonical Sail source candidate to `50567c79` and recorded the
  rebased but still local Delta correction separately from the supported pin.
- Corrected restart semantics so only active pre-commit workers may re-plan;
  post-commit and lost-response handling now require the durable Grust proposal
  identity and TypeSec's proposal-free recovery path, independent of
  process-local proposal state.
- Recorded the gated TypeSec governed-source, reauthorization, recovery, and
  receipt foundation as complete while retaining remote-revision selection as
  standalone delivery work.
- Refined governed cognition into one opaque, leased `improve` state machine
  with post-engine LakeCat and TypeSec reauthorization, deterministic
  digest-only recovery, typed no-change completion, and no public or durable
  plaintext proposal.
- Defined Marciana's versioned composite source scope over LakeCat evidence,
  field mapping, ingestion profile, and row transformation, plus explicit
  audit/receipt digest and timestamp semantics.
- Corrected the delivery status to distinguish committed owner foundations
  from the TypeSec and LakeCat hardening still under final verification.
- Defined the governed-ingestion boundary: Marciana's trusted LakeCat adapter
  must derive and one-time bind exact drafts from an authorized scan, because a
  scan proof alone never authorizes independently supplied memory content.
- Established `MARCIANA.md` as the canonical active delivery record for the
  standalone extraction, current cross-stack status, execution order, and
  acceptance gates.
- Added the reviewed Marciana 2 plan, comparing leading AI-memory systems and
  prioritizing governed context assembly, assertion-safe temporal memory,
  durable formation, and proposal-based agent learning.
- Tightened the standalone design so Cognee remains inspiration rather than a
  compatibility milestone, and assigned canonical evidence and transition
  semantics to one authoritative stack layer each.
- Established Marciana's standalone repository, ownership and dependency
  contract, upstream-first Sail baseline policy, compatibility registry, and
  pre-crate CI scaffold.
