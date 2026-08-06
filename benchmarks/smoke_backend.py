"""Deterministic indexed and linear backends for benchmark-harness validation."""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import date
from typing import Iterable

TOKEN = re.compile(r"[a-z0-9]+")


@dataclass(frozen=True)
class Record:
    record_id: str
    text: str
    valid_from: date
    invalid_at: date | None = None
    authorized: bool = True

    def visible_at(self, as_of: date) -> bool:
        return self.valid_from <= as_of and (
            self.invalid_at is None or as_of < self.invalid_at
        )


def tokens(value: str) -> frozenset[str]:
    return frozenset(TOKEN.findall(value.lower()))


def rank(
    records: Iterable[Record], query_tokens: frozenset[str], as_of: date
) -> tuple[list[str], list[str], int]:
    """Shared deterministic scoring for every backend candidate set."""

    scored: list[tuple[int, str]] = []
    redacted: list[str] = []
    for record in records:
        if not record.visible_at(as_of):
            continue
        score = len(query_tokens & tokens(record.text))
        if score == 0:
            continue
        if not record.authorized:
            redacted.append(record.record_id)
        else:
            scored.append((score, record.record_id))
    scored.sort(key=lambda item: (-item[0], item[1]))
    return [record_id for _, record_id in scored], sorted(redacted), len(scored)


class LinearBackend:
    def __init__(self, records: list[Record]) -> None:
        self.records = tuple(records)

    def search(self, query: str, as_of: date) -> tuple[list[str], list[str], int]:
        return rank(self.records, tokens(query), as_of)


class IndexedBackend(LinearBackend):
    def __init__(self, records: list[Record]) -> None:
        super().__init__(records)
        self._by_token: dict[str, set[int]] = {}
        for index, record in enumerate(self.records):
            for token in tokens(record.text):
                self._by_token.setdefault(token, set()).add(index)

    def search(self, query: str, as_of: date) -> tuple[list[str], list[str], int]:
        query_tokens = tokens(query)
        candidate_indexes = set().union(
            *(self._by_token.get(token, set()) for token in query_tokens)
        )
        return rank(
            (self.records[index] for index in candidate_indexes), query_tokens, as_of
        )
