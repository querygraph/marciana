# Marciana memory benchmarks

The public memory products do not use one universal score. The current
benchmark set is:

| Benchmark | What it measures | Marciana gate |
| --- | --- | --- |
| [LoCoMo](https://snap-research.github.io/locomo/) | Single-hop, multi-hop, open-domain, and temporal conversational recall | Accuracy by category, temporal correctness, citations, and context tokens |
| [LongMemEval](https://github.com/xiaowu0162/longmemeval) | Single/multi-session recall, knowledge updates, temporal reasoning, abstention, and contradictions | Accuracy, update correctness, abstention, and stale-assertion rejection |
| [BEAM](https://github.com/VectorInstitute/BEAM) | 1M/10M-token long-context memory behavior | Accuracy versus tokens, formation cost, and p50/p95 latency |
| [DMR](https://arxiv.org/abs/2501.13956) | Deep-memory retrieval quality and latency | Recall@k, p50/p95 latency, and no unauthorized disclosure |
| [Letta Evals](https://docs.letta.com/guides/evals/concepts/overview) | Stateful agent evolution using dataset → target → extractor → grader → gate | State transition correctness and release gates |

Mem0 and Zep publish useful reference numbers, but their claims are not
Marciana acceptance criteria: Mem0 reports LoCoMo/LongMemEval/BEAM accuracy and
token counts, while Zep reports accuracy, retrieval latency, and context size.
Their managed stacks, models, prompts, and judge choices differ. Reproduce
their datasets with a pinned model/configuration and publish raw results before
comparing.

For orientation, Mem0’s published pages report roughly 92–94% on LoCoMo and
LongMemEval, 64.1/48.6 on BEAM at 1M/10M, and about 6.8–7.0k retrieval tokens;
Zep reports 94.7% LoCoMo and 90.2% LongMemEval with 155/162 ms retrieval
latency and 5,760/4,408-token contexts. These are vendor-reported reference
points, not targets to copy without matching their model and judge setup:
[Mem0 evaluation](https://docs.mem0.ai/core-concepts/memory-evaluation),
[Mem0 research](https://mem0.ai/research), and
[Zep research](https://www.getzep.com/research/).

Run the dependency-free harness with:

```text
python3 benchmarks/run_memory_benchmark.py --json
```

Run the harness tests with:

```text
python3 -m unittest discover -s benchmarks -p 'test_*.py'
```

The smoke suite is deliberately small. It validates current versus historical
facts, unknown-query abstention, redaction safety, deterministic ranking, and
linear versus indexed retrieval latency. The generated `benchmark-result.json`
is local verification output and is not tracked.

Before the coffee demo, the next benchmark work is to add pinned adapters for
LoCoMo, LongMemEval, and BEAM; record model/provider/revision metadata; and
reject results that omit safety, token, or latency metrics.

`corpus.py` now defines the adapter boundary: external adapters must emit
strict normalized JSONL cases and an exact HTTPS source revision. Loading is
local-only and rejects malformed, duplicate, empty, or oversized corpora.
`adapters.py` implements local LoCoMo, LongMemEval, and BEAM normalization at their
published data revisions (`3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376` and
`98d7416c24c778c2fee6e6f3006e7a073259d48f`); it never downloads data or sends question,
answer, or evidence text to a service. Its 100K/500K/1M and 10M Parquet dataset
revisions are recorded and tested in `adapters.py`. Loading those large files is an optional
PyArrow adapter unit; it keeps only question text, category, and conversation
ID, not the long chat, and is not part of the dependency-free smoke harness.

The current smoke run (1,000 repeats, 504 records) reaches 100% case accuracy
with zero redaction leaks. The indexed path measured about 5.3 µs p50 versus
421 µs for the linear path in the local run; these are harness diagnostics,
not a hosted-system claim. The JSON report emits p50/p95/p99 speedups,
environment metadata, and a machine-checkable `indexed_faster` gate.
