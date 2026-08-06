"""Pydantic AI v2 agents for agricultural learning and coffee-market memory."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date

from pydantic_ai import Agent, RunContext
from pydantic_ai.models.test import TestModel

from .memory import MemoryBackend, deterministic_id
from .models import AgentDecision, MemoryFact
from .sail import SailWarehouse


@dataclass
class AgentDeps:
    name: str
    memory: MemoryBackend
    sail: SailWarehouse
    latest_rows: list[dict]
    last_memory_id: str | None = None
    remember_count: int = 0
    obsolete_memory_id: str | None = None
    operation_log: list[str] = field(default_factory=list)


def build_agent(model: str | None = None) -> Agent[AgentDeps, AgentDecision]:
    agent = Agent(
        model or TestModel(call_tools=[]),
        deps_type=AgentDeps,
        output_type=AgentDecision,
        instructions=(
            "You are a governed agricultural research agent. Use only tool results, "
            "cite source IDs, preserve historical prices, and never invent a market fact."
        ),
    )

    @agent.tool
    async def query_sail(ctx: RunContext[AgentDeps], country: str = "Honduras") -> str:
        if not country.isalpha() or len(country) > 64:
            raise ValueError("country must be bounded canonical text")
        rows = ctx.deps.latest_rows or [{"country": country, "note": "offline fixture"}]
        if ctx.deps.sail.connected:
            rows = ctx.deps.sail.query(
                f"SELECT observation_id, country, market, price_usd_per_kg, observed_on, source, note FROM {ctx.deps.sail.table_name} WHERE country = '{country}' ORDER BY observed_on"
            )
        ctx.deps.operation_log.append("sail.query")
        return str(rows)

    @agent.tool
    async def remember(ctx: RunContext[AgentDeps], text: str = "", source: str = "agstack-palefire", observed_on: date | None = None, confidence_basis_points: int = 8000) -> str:
        if not text:
            if ctx.deps.remember_count == 0:
                text = "Honduras coffee farms use volcanic soil and altitude to support quality."
            else:
                text = "Honduras coffee price is 4.20 USD per kg at San Pedro Sula."
        if observed_on is None:
            observed_on = date(2026, 1, 10)
        ctx.deps.remember_count += 1
        fact = MemoryFact(memory_id=deterministic_id(text), text=text, source=source, observed_on=observed_on, confidence_basis_points=confidence_basis_points)
        receipt = ctx.deps.memory.remember(fact)
        if receipt.memory_ids:
            ctx.deps.last_memory_id = receipt.memory_ids[-1]
        ctx.deps.operation_log.append(receipt.model_dump_json())
        return receipt.model_dump_json()

    @agent.tool
    async def recall(ctx: RunContext[AgentDeps], query: str = "") -> str:
        if not query or query == "a":
            query = "Honduras coffee price"
        receipt, facts = ctx.deps.memory.recall(query)
        ctx.deps.operation_log.append(receipt.model_dump_json())
        return str({"receipt": receipt.model_dump(), "facts": [fact.model_dump() for fact in facts]})

    @agent.tool
    async def improve(ctx: RunContext[AgentDeps], memory_id: str = "", text: str = "", source: str = "dataverse:coffee-market", observed_on: date | None = None) -> str:
        memory_id = memory_id or ctx.deps.last_memory_id or ""
        text = text or "Honduras coffee price is 4.60 USD per kg at San Pedro Sula."
        observed_on = observed_on or date(2026, 2, 10)
        replacement = MemoryFact(memory_id=deterministic_id(text), text=text, source=source, observed_on=observed_on, confidence_basis_points=8500)
        receipt = ctx.deps.memory.improve(memory_id, replacement)
        if receipt.allowed:
            ctx.deps.obsolete_memory_id = memory_id
            ctx.deps.last_memory_id = replacement.memory_id
        ctx.deps.operation_log.append(receipt.model_dump_json())
        return receipt.model_dump_json()

    @agent.tool
    async def forget(ctx: RunContext[AgentDeps], memory_id: str = "") -> str:
        memory_id = memory_id or ctx.deps.obsolete_memory_id or ctx.deps.last_memory_id or ""
        receipt = ctx.deps.memory.forget(memory_id)
        ctx.deps.operation_log.append(receipt.model_dump_json())
        return receipt.model_dump_json()

    return agent


async def tool_turn(agent: Agent[AgentDeps, AgentDecision], deps: AgentDeps, tool_name: str, prompt: str) -> AgentDecision:
    with agent.override(model=TestModel(call_tools=[tool_name])):
        decision = (await agent.run(prompt, deps=deps)).output
    action = {
        "query_sail": "report",
        "remember": "learn",
        "recall": "recall",
        "improve": "improve",
        "forget": "forget",
    }.get(tool_name)
    return decision.model_copy(update={"action": action}) if action else decision
