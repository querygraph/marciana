"""Memory verbs with a deterministic local backend and QueryGraph HTTP seam."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from datetime import date
from typing import Any, Protocol

from .models import MemoryFact, MemoryReceipt

SPACE_ID = "memory/agstack/coffee"
PURPOSE = "coffee-market-research"


class MemoryBackend(Protocol):
    def remember(self, fact: MemoryFact) -> MemoryReceipt: ...
    def recall(self, query: str) -> tuple[MemoryReceipt, list[MemoryFact]]: ...
    def improve(self, memory_id: str, replacement: MemoryFact) -> MemoryReceipt: ...
    def forget(self, memory_id: str) -> MemoryReceipt: ...


@dataclass
class LocalGovernedMemory:
    facts: dict[str, MemoryFact] = field(default_factory=dict)

    def remember(self, fact: MemoryFact) -> MemoryReceipt:
        self.facts[fact.memory_id] = fact
        return MemoryReceipt(operation="remember", allowed=True, memory_ids=[fact.memory_id])

    def recall(self, query: str) -> tuple[MemoryReceipt, list[MemoryFact]]:
        terms = set(query.lower().split())
        hits = [fact for fact in self.facts.values() if fact.state == "current" and terms & set(fact.text.lower().split())]
        return MemoryReceipt(operation="recall", allowed=True, memory_ids=[fact.memory_id for fact in hits]), hits

    def improve(self, memory_id: str, replacement: MemoryFact) -> MemoryReceipt:
        if memory_id not in self.facts:
            return MemoryReceipt(operation="improve", allowed=False, detail="memory not found")
        self.facts[memory_id] = self.facts[memory_id].model_copy(update={"state": "superseded"})
        self.facts[replacement.memory_id] = replacement
        return MemoryReceipt(operation="improve", allowed=True, memory_ids=[memory_id, replacement.memory_id])

    def forget(self, memory_id: str) -> MemoryReceipt:
        fact = self.facts.get(memory_id)
        if not fact:
            return MemoryReceipt(operation="forget", allowed=False, detail="memory not found")
        self.facts[memory_id] = fact.model_copy(update={"state": "forgotten"})
        return MemoryReceipt(operation="forget", allowed=True, memory_ids=[memory_id])


@dataclass
class QueryGraphMemory:
    """Signed QueryGraph route adapter; credential signing is injected by caller.

    Payloads match the pinned wire in ``crates/marciana-memory/src/api.rs``
    (`space_id`, `memory_ids`, no client-only fields).
    """

    base_url: str
    post_signed: Any
    as_of: date = field(default_factory=date.today)

    def _post(self, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        result = self.post_signed(self.base_url, path, payload)
        if hasattr(result, "body"):
            if not result.allowed:
                raise PermissionError(json.dumps(result.body, sort_keys=True))
            return result.body
        return result

    def remember(self, fact: MemoryFact) -> MemoryReceipt:
        body = self._post("/v1/memory/remember", {"space_id": SPACE_ID, "text": fact.text, "purpose": PURPOSE})
        memory_id = str(body.get("result", body).get("id", fact.memory_id))
        return MemoryReceipt(operation="remember", allowed=bool(body.get("allowed", True)), memory_ids=[memory_id])

    def recall(self, query: str) -> tuple[MemoryReceipt, list[MemoryFact]]:
        body = self._post("/v1/memory/recall", {"space_id": SPACE_ID, "query": query, "purpose": PURPOSE})
        hits = body.get("result", body).get("hits", [])
        facts = [MemoryFact(memory_id=str(hit["id"]), text=str(hit.get("content", hit.get("text", ""))), source="querygraph", observed_on=self.as_of) for hit in hits]
        return MemoryReceipt(operation="recall", allowed=bool(body.get("allowed", True)), memory_ids=[fact.memory_id for fact in facts]), facts

    def improve(self, memory_id: str, replacement: MemoryFact) -> MemoryReceipt:
        body = self._post(
            "/v1/memory/improve",
            {
                "space_id": SPACE_ID,
                "memory_id": memory_id,
                "replacement": {
                    "space_id": SPACE_ID,
                    "text": replacement.text,
                    "purpose": PURPOSE,
                },
            },
        )
        result = body.get("result", body)
        replacement_id = str(result.get("id", replacement.memory_id))
        return MemoryReceipt(
            operation="improve",
            allowed=bool(body.get("allowed", True)),
            memory_ids=[memory_id, replacement_id],
            detail="new assertion supersedes the old assertion; history is retained",
        )

    def forget(self, memory_id: str) -> MemoryReceipt:
        body = self._post("/v1/memory/forget", {"space_id": SPACE_ID, "memory_ids": [memory_id], "purpose": PURPOSE})
        return MemoryReceipt(operation="forget", allowed=bool(body.get("allowed", True)), memory_ids=[memory_id])


def deterministic_id(text: str) -> str:
    return "coffee-" + hashlib.sha256(text.encode()).hexdigest()[:16]
