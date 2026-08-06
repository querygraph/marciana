#!/usr/bin/env python3
"""End-to-end Dataverse → Sail → TypeDID/TypeSec/Grust memory demonstration.

Provider-free by default (Pydantic AI TestModel); set --live-model to use the
configured Pydantic AI provider. Set DATAVERSE_DATASET_URL and
SAIL_SPARK_CONNECT_URL to exercise live services.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
from pathlib import Path

from .agents import AgentDeps, build_agent, tool_turn
from .dataverse import DataverseClient
from .memory import LocalGovernedMemory, QueryGraphMemory
from .models import DataverseLoadReport
from .sail import SailWarehouse


ROOT = Path(__file__).parent


def live_memory(base_url: str):
    from querygraph.api_auth import governed_post
    from querygraph.typedid import TypeDidAgent

    credential = TypeDidAgent.new("CoffeeResearchAgent", seed="demo:coffee:agent")
    def signed_post(url: str, path: str, payload: dict):
        return governed_post(url, path, payload, credential)

    return QueryGraphMemory(base_url, signed_post), credential


async def run(live: bool, live_model: str | None) -> dict:
    dataset_url = os.getenv("DATAVERSE_DATASET_URL")
    sail = SailWarehouse(os.getenv("SAIL_SPARK_CONNECT_URL") if live else None)
    rows = DataverseClient(dataset_url, ROOT / "data" / "coffee_honduras.csv").rows()
    loaded = sail.load(rows)
    report = DataverseLoadReport(dataset_url=dataset_url or "urn:fixture:coffee-honduras", row_count=len(rows), source_file="coffee_honduras.csv", table_name=sail.table_name, sail_loaded=loaded)
    memory = LocalGovernedMemory()
    credential = None
    if live:
        memory, credential = live_memory(os.environ["QUERYGRAPH_URL"])
    deps = AgentDeps(name="agronomist", memory=memory, sail=sail, latest_rows=[row.model_dump() for row in rows])
    agent = build_agent(live_model)

    sail_turn = await tool_turn(agent, deps, "query_sail", "Inspect the loaded Honduras coffee market table.")
    learned = await tool_turn(agent, deps, "remember", "Remember the agronomic coffee facts from the loaded source.")
    price_memory = await tool_turn(agent, deps, "remember", "Remember the current Honduras coffee price.")
    first_memory_id = deps.last_memory_id
    recalled = await tool_turn(agent, deps, "recall", "Recall Honduras coffee price.")
    improved = await tool_turn(agent, deps, "improve", "Improve the remembered price with the newest market observation.")
    improved_recalled = await tool_turn(agent, deps, "recall", "Recall Honduras coffee price after the improvement.")
    forgotten = await tool_turn(agent, deps, "forget", "Forget the selected obsolete price memory.") if first_memory_id else None
    return {
        "framework": "Pydantic AI v2",
        "stack": ["Dataverse", "Sail Spark Connect", "LakeCat boundary", "Grust", "TypeSec", "TypeDID", "Marciana"],
        "palefire_pattern": ["entity-aware retrieval", "temporal evidence", "source citations", "feedback-driven improvement"],
        "dataverse": report.model_dump(),
        "credentialed": credential is not None,
        "operation_log": deps.operation_log,
        "turns": {"sail": sail_turn.model_dump(), "learned": learned.model_dump(), "price": price_memory.model_dump(), "recalled": recalled.model_dump(), "improved": improved.model_dump(), "improved_recalled": improved_recalled.model_dump(), "forgotten": forgotten.model_dump() if forgotten else None},
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--live", action="store_true", help="connect to Sail and QueryGraph services")
    parser.add_argument("--live-model", help="Pydantic AI v2 model string, e.g. openai:gpt-5-mini")
    args = parser.parse_args()
    if args.live and not os.getenv("QUERYGRAPH_URL"):
        raise SystemExit("--live requires QUERYGRAPH_URL")
    print(json.dumps(asyncio.run(run(args.live, args.live_model)), indent=2, sort_keys=True, default=str))


if __name__ == "__main__":
    main()
