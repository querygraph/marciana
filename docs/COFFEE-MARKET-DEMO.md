# Honduras coffee-market demo

The demo in [`examples/coffee_market_demo`](../examples/coffee_market_demo)
exercises the QueryGraph memory stack as a small agricultural research
workflow. It loads a Dataverse-shaped Honduras fixture, optionally writes and
queries Sail over Spark Connect, and runs typed Pydantic AI v2 tools against
Marciana's governed memory boundary.

The default path is deterministic and key-free. A live run can provide a
Dataverse URL, Sail Spark Connect URL, QueryGraph URL, and a Pydantic AI model;
credentials and authorization remain outside the example.

## Lifecycle

```text
Dataverse fixture → Sail table → report
                              ↓
                    remember agronomic fact
                    remember price observation
                              ↓
                            recall
                              ↓
                     improve newer price
                              ↓
                            recall
                              ↓
                    forget obsolete observation
```

The lifecycle preserves historical observations: `improve` creates a governed
replacement and `forget` targets the obsolete memory. Each operation returns a
typed receipt, while the agent emits source IDs and citations rather than
inventing market facts.

## Typed domain contracts

```python
class MemoryFact(BaseModel):
    model_config = ConfigDict(extra="forbid")

    memory_id: str
    text: str
    source: str
    observed_on: date
    state: Literal["current", "superseded", "forgotten"] = "current"
    confidence_basis_points: int = Field(ge=0, le=10_000)


class AgentDecision(BaseModel):
    agent: str
    action: Literal["learn", "recall", "improve", "forget", "report"]
    claim: str
    confidence_basis_points: int = Field(ge=0, le=10_000)
    memory_ids: list[str] = Field(default_factory=list)
    citations: list[str] = Field(default_factory=list)
```

The models are defined in `models.py`; strict fields keep the example's wire
shape explicit and testable.

## Governed tools

```python
@agent.tool
async def improve(
    ctx: RunContext[AgentDeps],
    memory_id: str = "",
    text: str = "",
    source: str = "dataverse:coffee-market",
    observed_on: date | None = None,
) -> str:
    memory_id = memory_id or ctx.deps.last_memory_id or ""
    replacement = MemoryFact(
        memory_id=deterministic_id(text or "Honduras coffee price is 4.60 USD per kg at San Pedro Sula."),
        text=text or "Honduras coffee price is 4.60 USD per kg at San Pedro Sula.",
        source=source,
        observed_on=observed_on or date(2026, 2, 10),
        confidence_basis_points=8500,
    )
    receipt = ctx.deps.memory.improve(memory_id, replacement)
    ctx.deps.operation_log.append(receipt.model_dump_json())
    return receipt.model_dump_json()
```

The other tools are `query_sail`, `remember`, `recall`, and `forget`. Their
implementation delegates storage and authorization to the injected memory
backend; the agent is only a typed proposal/tool caller.

## Deterministic Pydantic AI v2 execution

```python
async def tool_turn(agent, deps, tool_name: str, prompt: str) -> AgentDecision:
    with agent.override(model=TestModel(call_tools=[tool_name])):
        decision = (await agent.run(prompt, deps=deps)).output
    action = {
        "query_sail": "report",
        "remember": "learn",
        "recall": "recall",
        "improve": "improve",
        "forget": "forget",
    }.get(tool_name)
    return decision.model_copy(update={"action": action})
```

The explicit action mapping prevents a model's generic structured output from
misrepresenting which governed operation actually ran. The regression suite
asserts the complete action sequence:

```python
["report", "learn", "learn", "recall", "improve", "recall", "forget"]
```

## Run it

First install the required Python dependencies (pydantic and Pydantic AI v2)
from [`../examples/coffee_market_demo/requirements.txt`](../examples/coffee_market_demo/requirements.txt);
the tests and demo fail with `ModuleNotFoundError` without them:

```bash
pip install -r examples/coffee_market_demo/requirements.txt
python3 -m unittest discover \
  -s examples/coffee_market_demo/tests -q
python3 -m examples.coffee_market_demo.demo
```

Use `--live --live-model <provider:model>` only when the external services and
credentials are configured.
