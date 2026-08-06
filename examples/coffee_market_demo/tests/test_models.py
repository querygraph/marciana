from datetime import date
import asyncio
import unittest

from examples.coffee_market_demo.memory import (
    LocalGovernedMemory,
    QueryGraphMemory,
    deterministic_id,
)
from examples.coffee_market_demo.models import MemoryFact
from examples.coffee_market_demo.agents import AgentDeps, build_agent, tool_turn
from examples.coffee_market_demo.demo import run
from examples.coffee_market_demo.sail import SailWarehouse


class DemoModelTests(unittest.TestCase):
    def test_local_memory_remember_improve_and_forget(self) -> None:
        memory = LocalGovernedMemory()
        first = MemoryFact(
            memory_id=deterministic_id("old"),
            text="Honduras coffee price 3.80 USD per kg",
            source="fixture",
            observed_on=date(2025, 1, 10),
            confidence_basis_points=8000,
        )
        replacement = first.model_copy(
            update={
                "memory_id": deterministic_id("new"),
                "text": "Honduras coffee price 4.20 USD per kg",
                "observed_on": date(2026, 1, 10),
            }
        )
        memory.remember(first)
        memory.improve(first.memory_id, replacement)
        _, hits = memory.recall("Honduras coffee price")
        self.assertEqual([hit.memory_id for hit in hits], [replacement.memory_id])
        memory.forget(replacement.memory_id)
        _, hits = memory.recall("Honduras coffee price")
        self.assertEqual(hits, [])

    def test_querygraph_improve_uses_supersession_route(self) -> None:
        calls: list[tuple[str, dict]] = []

        def post(_base_url: str, path: str, payload: dict) -> dict:
            calls.append((path, payload))
            return {"result": {"id": "replacement-id"}}

        replacement = MemoryFact(
            memory_id="replacement-id",
            text="Honduras coffee price 4.20 USD per kg",
            source="fixture",
            observed_on=date(2026, 1, 10),
            confidence_basis_points=8_000,
        )
        receipt = QueryGraphMemory("https://querygraph.test", post).improve("old-id", replacement)
        self.assertEqual(receipt.memory_ids, ["old-id", "replacement-id"])
        self.assertEqual(calls[0][0], "/v1/memory/improve")
        self.assertEqual(calls[0][1]["memory_id"], "old-id")

    def test_structured_turn_action_matches_governed_tool(self) -> None:
        deps = AgentDeps(
            name="test",
            memory=LocalGovernedMemory(),
            sail=SailWarehouse(None),
            latest_rows=[],
        )
        decision = asyncio.run(tool_turn(build_agent(), deps, "forget", "forget the selected memory"))
        self.assertEqual(decision.action, "forget")

    def test_key_free_demo_runs_the_complete_governed_lifecycle(self) -> None:
        report = asyncio.run(run(False, None))
        actions = [
            report["turns"][name]["action"]
            for name in ("sail", "learned", "price", "recalled", "improved", "improved_recalled", "forgotten")
        ]
        self.assertEqual(actions, ["report", "learn", "learn", "recall", "improve", "recall", "forget"])
        self.assertEqual(report["dataverse"]["row_count"], 3)
        self.assertTrue(any('"operation":"improve"' in entry for entry in report["operation_log"]))
        self.assertTrue(any('"operation":"forget"' in entry for entry in report["operation_log"]))
