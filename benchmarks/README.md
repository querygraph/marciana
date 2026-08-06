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

The current smoke run (1,000 repeats, 502 records) reaches 100% case accuracy
with zero redaction leaks. The indexed path measured about 5.5 µs p50 versus
437 µs for the linear path in the local run; these are harness diagnostics,
not a hosted-system claim. The JSON report also emits p50/p95 speedup and a
machine-checkable `indexed_faster` gate.
