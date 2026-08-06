# MARCIANA-ADVERSARIAL-v1

## Goal

Build and run a deterministic adversarial cognition benchmark for Marciana and
configured comparative memory systems. The benchmark must test not only whether
a system retrieves a fact, but whether it remains correct, secure, auditable,
reproducible, and responsive when the memory and request path are actively
trying to mislead it.

The benchmark must cover:

- retrieval quality and ranking;
- temporal correctness and variable memory horizons;
- contradictions, updates, negation, retraction, and resurrection;
- unknown-query abstention and unsupported-answer refusal;
- tenant, space, purpose, clearance, and TypeDID authorization;
- replay, stale proposals, forged source bindings, and mutation safety;
- provenance, source digests, citations, and catalog-scope binding;
- forgetting across lexical, vector, graph, cache, and derived-memory paths;
- malformed input, Unicode confusables, prompt injection, and oversized input;
- formation/index cost and P50/P95/P99 latency; and
- restart, retry, idempotency, recovery, and deterministic receipts.

The benchmark name is `MARCIANA-ADVERSARIAL-v1`. It must use pinned corpus,
model, provider, embedding, prompt, hardware, and source-revision metadata.
Reports must contain bounded IDs, digests, counts, and timings—not memory
plaintext.

## Release policy

Safety failures are hard release blockers and must not be averaged away by a
quality score. The following gates must all be zero:

- unauthorized plaintext disclosure;
- cross-tenant or cross-space leakage;
- residual recall after forget;
- stale or forged proposal commits;
- invalid provenance accepted;
- replayed mutation accepted;
- duplicate durable mutation; and
- non-deterministic receipts for identical runs.

Quality and performance are reported separately:

- answer accuracy, Recall@k, MRR, and ranking stability;
- temporal and update correctness;
- contradiction resolution;
- abstention precision and unsupported-answer rate;
- citation precision and completeness;
- context-token usage;
- formation/index cost;
- P50/P95/P99 latency;
- restart/recovery latency; and
- memory growth under adversarial load.

## Current status

**Status:** implementation in progress; benchmark not yet complete or
published.

### Implemented checkpoint

The deterministic policy-aware backend is implemented in
`benchmarks/adversarial_backend.py`. It models the benchmark’s security and
durability boundary without pretending to replace the Rust TypeSec vault. It
currently covers:

- scoped tenant and memory-space authorization;
- purpose and clearance checks;
- valid-time filtering;
- nonce replay rejection;
- source-digest validation for improvement proposals;
- stale proposal rejection;
- idempotent improvement retry;
- scoped forgetting with derived-memory invalidation; and
- state-preserving restart behavior.

The first adversarial scenario set is implemented in
`benchmarks/adversarial_cases.py`. It currently contains cases for:

1. current retrieval;
2. historical retrieval;
3. unknown-query abstention;
4. tenant isolation;
5. clearance isolation;
6. purpose denial;
7. forged source binding;
8. stale proposal;
9. replayed mutation;
10. idempotent retry;
11. forget plus derived-memory invalidation;
12. restart reproducibility; and
13. query-order invariance.

These files are intentionally still uncommitted while the previous benchmark
implementation unit is incomplete. This document records their status; it
does not claim that they have passed a release run.

### Not yet implemented

- adversarial benchmark runner and machine-readable report schema;
- category-level metrics and hard-gate evaluation;
- performance repetition and percentile measurement;
- comparative adapter protocol;
- executable adapters for Marciana, Mem0, Zep, Letta, Cognee, and Graphiti;
- explicit command/HTTP configuration for external systems;
- all-system inventory showing executed, failed, or unavailable status;
- full tests for the backend and scenario corpus;
- normalized LoCoMo, LongMemEval, BEAM, DMR, and Letta-Evals execution;
- benchmark report and release documentation; and
- final changelog, commit, push, and verification.

## Comparative systems

The runner must enumerate every configured system, never silently substitute
one backend for another:

| System | Required status handling |
|---|---|
| Marciana | Execute locally against the deterministic/reference path and, when available, the native service path |
| Mem0 | Execute only when its package/service and credentials are explicitly configured |
| Zep | Execute only when its endpoint/API configuration is explicitly configured |
| Letta | Execute only when its package/service configuration is explicitly configured |
| Cognee | Execute only when explicitly configured; no Cognee dependency may be added to Marciana |
| Graphiti | Execute only when its package/service configuration is explicitly configured |

Unavailable systems must produce a structured result with system name,
adapter version, missing configuration, and status `unavailable`. A failed
adapter must be reported as `error`, not converted into a passing result.

## Recommended runner contract

The eventual command should look like:

```sh
python3 benchmarks/run_adversarial_benchmark.py \
  --systems all \
  --corpus benchmarks/fixtures/marciana-adversarial-v1 \
  --model reference-smoke-v1 \
  --provider local \
  --profile adversarial-v1 \
  --json reports/marciana-adversarial-v1.json
```

The report should include:

```json
{
  "benchmark": "MARCIANA-ADVERSARIAL-v1",
  "status": "pass|fail|incomplete",
  "metadata": {},
  "hard_gates": {},
  "systems": {
    "marciana": {"status": "executed"},
    "mem0": {"status": "unavailable"}
  },
  "cases": [],
  "quality": {},
  "performance": {}
}
```

The benchmark is not complete until the report distinguishes executed systems
from unavailable systems and all hard-gate outcomes are explicit.

## Existing related baseline

The current dependency-free smoke benchmark remains separate and must continue
to pass:

```sh
python3 -m unittest discover -s benchmarks -p 'test_*.py' -q
python3 benchmarks/run_memory_benchmark.py --json
```

Its latest documented result is 100% accuracy and zero redaction leaks on 504
records, with indexed retrieval faster than the linear baseline. That result is
not a cross-vendor comparison and must not be presented as one.

## Acceptance criteria

The goal is complete only when:

1. the adversarial corpus and adapter protocol are versioned;
2. separate tests cover every hard gate and metamorphic invariant;
3. Marciana executes the full deterministic suite;
4. every named comparative system is attempted and explicitly classified;
5. optional public-corpus adapters are pinned and offline-only;
6. the report contains quality, safety, provenance, and performance metrics;
7. reports contain no memory plaintext;
8. full Python and Rust verification passes;
9. the changelog records the delivered benchmark; and
10. implementation, report, and documentation are committed and pushed.

## Constraints

The benchmark must preserve Marciana’s architectural boundaries:

- TypeSec’s capability-gated `MemoryVault` remains the only protected-memory
  reveal/mutation authority;
- Grust persists and commits graph state;
- indexes rank but do not authorize;
- cognition proposes inert data;
- Sail executes memory-specific proposal computation;
- LakeCat remains the governed catalog-proof authority; and
- Cognee, Akka, Fluree, and comparative systems remain inspiration or external
  adapters, never hidden runtime dependencies.
