"""Local adapters for the released LoCoMo and LongMemEval layouts."""

from __future__ import annotations

import json
from datetime import date
from pathlib import Path

from corpus import EvaluationCase, SourcePin

LOCOMO_SOURCE = SourcePin(
    "locomo10",
    "https://github.com/snap-research/locomo.git",
    "3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376",
    "locomo10-qa-v1",
)
LONGMEMEVAL_SOURCE = SourcePin(
    "longmemeval-cleaned",
    "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned",
    "98d7416c24c778c2fee6e6f3006e7a073259d48f",
    "longmemeval-question-v1",
)


def load_locomo(path: Path, as_of: date = date.max) -> tuple[EvaluationCase, ...]:
    """Normalize LoCoMo QA annotations from an authorized local JSON file."""

    rows = json.loads(path.read_text(encoding="utf-8"))
    cases: list[EvaluationCase] = []
    for sample_index, sample in enumerate(rows):
        for qa_index, annotation in enumerate(sample.get("qa", [])):
            evidence = tuple(annotation.get("evidence") or ())
            cases.append(
                EvaluationCase(
                    case_id=f"locomo:{sample_index}:{qa_index}",
                    query=annotation["question"],
                    as_of=as_of,
                    expected_ids=evidence,
                    abstain=not bool(evidence),
                )
            )
    if not cases:
        raise ValueError("LoCoMo corpus has no QA annotations")
    return tuple(cases)


def load_longmemeval(path: Path) -> tuple[EvaluationCase, ...]:
    """Normalize LongMemEval question records from an authorized local JSON."""

    rows = json.loads(path.read_text(encoding="utf-8"))
    cases: list[EvaluationCase] = []
    for row in rows:
        question_id = row["question_id"]
        question_date = date.fromisoformat(row["question_date"][:10])
        evidence = tuple(row.get("answer_session_ids") or ())
        cases.append(
            EvaluationCase(
                case_id=f"longmemeval:{question_id}",
                query=row["question"],
                as_of=question_date,
                expected_ids=evidence,
                abstain=question_id.endswith("_abs"),
            )
        )
    if not cases:
        raise ValueError("LongMemEval corpus is empty")
    return tuple(cases)
