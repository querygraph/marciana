"""Run a reproducible Marciana memory smoke benchmark.

This is intentionally dependency-free. It validates the harness and local
retrieval invariants before plugging in LoCoMo, LongMemEval, or BEAM datasets.
"""

from __future__ import annotations

import argparse
import json
import time
from datetime import date
from pathlib import Path

from metrics import CaseResult, summarize
from smoke_backend import IndexedBackend, LinearBackend, Record


CASES = (
    ("current-price", "honduras coffee price", date(2026, 1, 10), ("price-current",)),
    ("historical-price", "honduras coffee price", date(2025, 1, 10), ("price-old",)),
    ("soil", "coffee soil altitude", date(2026, 1, 10), ("soil",)),
    ("unknown", "cocoa futures japan", date(2026, 1, 10), ()),
    ("redaction", "private farm", date(2026, 1, 10), ()),
)

RECORDS = [
    Record("price-old", "Honduras coffee price was 3.80 USD per kg", date(2024, 1, 1), date(2026, 1, 1)),
    Record("price-current", "Honduras coffee price is 4.20 USD per kg", date(2026, 1, 1)),
    Record("soil", "Coffee farms in Honduras use volcanic soil at high altitude", date(2024, 1, 1)),
    Record("private-price", "Private farm price is 9.00 USD per kg", date(2024, 1, 1), authorized=False),
] + [
    Record(f"noise-{index}", f"unrelated agricultural note {index}", date(2024, 1, 1))
    for index in range(500)
]


def run_cases(backend: LinearBackend, repeats: int) -> dict[str, float | int]:
    results: list[CaseResult] = []
    for case_id, query, as_of, expected in CASES:
        returned: tuple[str, ...] = ()
        redacted: tuple[str, ...] = ()
        tokens = 0
        started = time.perf_counter_ns()
        for _ in range(repeats):
            ids, hidden, token_count = backend.search(query, as_of)
            returned, redacted, tokens = tuple(ids[:1]), tuple(hidden), token_count
        elapsed_us = (time.perf_counter_ns() - started) / 1_000
        results.append(
            CaseResult(
                case_id,
                tuple(expected),
                returned,
                redacted,
                elapsed_us / repeats,
                tokens * 4,
            )
        )
    return summarize(results)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repeats", type=int, default=1_000)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.repeats < 1:
        raise SystemExit("--repeats must be positive")
    linear = run_cases(LinearBackend(RECORDS), args.repeats)
    indexed = run_cases(IndexedBackend(RECORDS), args.repeats)
    report = {
        "benchmark": "marciana-memory-smoke-v1",
        "cases": len(CASES),
        "linear": linear,
        "indexed": indexed,
        "gates": {
            "accuracy": indexed["accuracy"],
            "redaction_leaks": indexed["redaction_leaks"],
        },
    }
    output = json.dumps(report, indent=2, sort_keys=True)
    print(output if args.json else output)
    Path("benchmark-result.json").write_text(output + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
