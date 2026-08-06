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

**Status:** implemented; the deterministic reference suite passes with every
hard gate at zero. See `docs/BENCHMARK-RESULTS.md` for the recorded run.

### Implemented

The deterministic policy-aware backend is implemented in
`benchmarks/adversarial_backend.py`. It models the benchmark’s security and
durability boundary without pretending to replace the Rust TypeSec vault. It
covers:

- scoped tenant and memory-space authorization;
- purpose and clearance checks;
- valid-time filtering;
- nonce replay rejection, durable across restart;
- source-digest validation for improvement proposals;
- stale proposal rejection;
- idempotent improvement retry with identical receipts;
- scoped forgetting with derived-memory invalidation;
- oversized query and memory rejection; and
- state-preserving restart behavior.

The scenario corpus in `benchmarks/adversarial_cases.py` contains eighteen
cases across retrieval, temporal, abstention, authorization, provenance,
mutation, replay, recovery, forget, reproducibility, and robustness —
including replay-across-restart, empty and oversized input, Unicode
confusables, and prompt-injection containment. Expectations are explicit per
case: an expected ranked prefix, a mandatory-abstention flag, and forbidden
IDs that must never appear.

The remaining units from the original plan are now delivered:

- `benchmarks/run_adversarial_benchmark.py` — runner, versioned corpus
  manifest verification (`--pin-corpus` to regenerate), receipt-determinism
  double-run, formation/restart timing, and the machine-readable report;
- `benchmarks/adversarial_report.py` — hard-gate evaluation, category
  metrics, percentile performance, bounded-report enforcement;
- `benchmarks/adversarial_adapters.py` — comparative adapter protocol,
  the Marciana reference adapter, and explicit command-configured external
  adapters with an all-system inventory;
- `benchmarks/adversarial_corpora.py` — pinned, offline-only public-corpus
  inventory (LoCoMo, LongMemEval, BEAM, DMR, Letta-Evals);
- separate test files for the backend, corpus, adapters, and report; and
- the corpus manifest fixture in
  `benchmarks/fixtures/marciana-adversarial-v1/manifest.json`.

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
| Akka + Fluree | Execute only when its adapter command is explicitly configured; Akka and Fluree remain external comparative systems, never Marciana runtime dependencies |

Unavailable systems must produce a structured result with system name,
adapter version, missing configuration, and status `unavailable`. A failed
adapter must be reported as `error`, not converted into a passing result.

## Runner contract

The implemented command is:

```sh
python3 benchmarks/run_adversarial_benchmark.py \
  --systems all \
  --corpus benchmarks/fixtures/marciana-adversarial-v1 \
  --model reference-smoke-v1 \
  --provider local \
  --profile adversarial-v1 \
  --json reports/marciana-adversarial-v1.json
```

External systems execute only when explicitly configured through
`MARCIANA_ADVERSARIAL_<SYSTEM>_CMD` (`MEM0`, `ZEP`, `LETTA`, `COGNEE`,
`GRAPHITI`, `AKKA_FLUREE`). The configured command receives the case corpus
as JSON on stdin and must print one outcome per case; endpoint and credential
configuration belong to the adapter command's own environment. Public-corpus
fixtures are configured through `MARCIANA_ADVERSARIAL_<CORPUS>_PATH`
(`LOCOMO`, `LONGMEMEVAL`, `BEAM`, `DMR`, `LETTA_EVALS`) and are normalized
offline against their pinned source revisions.

The report should include:

```json
{
  "benchmark": "MARCIANA-ADVERSARIAL-v1",
  "corpus_digest": "sha256:…",
  "status": "pass|fail|incomplete",
  "metadata": {},
  "hard_gates": {},
  "systems": {
    "marciana": {"status": "executed"},
    "mem0": {"status": "unavailable"}
  },
  "cases": [],
  "quality": {},
  "performance": {},
  "public_corpora": {}
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
