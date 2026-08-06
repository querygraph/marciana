# Marciana benchmark results

**Run:** `marciana-memory-smoke-v1`  
**Date:** 2026-08-06  
**Environment:** Darwin arm64, Python 3.14.6, 504 records, 1,000 repeats  
**Revision metadata:** model `reference-smoke-v1`, provider `local`, embedding
`none`, prompt `none`, profile `smoke-v1`, revision `working-tree`

## Executive result

The dependency-free smoke harness passed every release gate: all five cases
were correct and no protected value appeared in a redacted result. Indexed
retrieval was faster than the linear baseline at every measured percentile.

| Gate or measurement | Linear | Indexed |
| --- | ---: | ---: |
| Case accuracy | 100% | 100% |
| Redaction leaks | 0 | 0 |
| Mean context tokens | 4.8 | 4.8 |
| P50 latency | 572.52 µs | 7.03 µs |
| P95 latency | 580.19 µs | 9.64 µs |
| P99 latency | 580.19 µs | 9.64 µs |

Indexed speedups were 81.44× at P50, 60.21× at P95, and 60.21× at P99.
These are local engineering diagnostics, not a vendor-comparison claim.

## What is tested

The smoke corpus covers current and historical facts, unknown-query
abstention, redaction safety, deterministic ranking, token accounting, and a
linear-versus-indexed retrieval comparison. The benchmark report contains no
memory plaintext; it emits bounded metrics and release metadata only.

External adapters normalize LoCoMo, LongMemEval, and BEAM fixtures into the
same local case contract. Their source revisions are pinned, loading is
offline-only, and large Parquet datasets are optional PyArrow inputs. They are
not silently substituted for the Marciana release corpus.

## Adversarial run

**Run:** `MARCIANA-ADVERSARIAL-v1`  
**Date:** 2026-08-06  
**Environment:** Darwin arm64, Python 3.14.6, 18 cases, 100 repeats  
**Corpus digest:** `sha256:d879b8a53039d84134bf8b35f21a398c497b94605bddf1a4995854aa1cb798b9`

The deterministic reference suite passed with status `pass`. Every hard gate
held at zero: unauthorized disclosure, cross-scope leakage, residual recall
after forget, stale proposal commits, invalid provenance, replayed mutations
(including across restart), duplicate durable mutations, mishandled
adversarial input, and non-deterministic receipts. Quality reached 100%
accuracy in all eleven categories with a 0% unsupported-answer rate.
Reference-path performance measured 36.1 µs P50, 49.9 µs P95, and 57.8 µs P99
per full case run, 20.4 µs corpus formation, and 0.4 µs restart. All six
comparative systems (Mem0, Zep, Letta, Cognee, Graphiti, Akka + Fluree) were
enumerated and reported `unavailable` because no adapter command was
configured; none was silently substituted. The five public corpora were
inventoried as `unavailable` at their pinned revisions because no local
fixture path was configured. These figures are local engineering
diagnostics on the deterministic reference backend, not a hosted-system or
vendor-comparison claim.

## Reproduce

```bash
python3 -m unittest discover -s benchmarks -p 'test_*.py' -q
python3 benchmarks/run_memory_benchmark.py --json
```

The CLI accepts `--model`, `--provider`, `--embedding`, `--prompt`,
`--profile`, and `--revision`. Empty, overlong, or newline-containing values
are rejected so results remain attributable. The full benchmark design and
adapter policy are in [`benchmarks/README.md`](../benchmarks/README.md).
