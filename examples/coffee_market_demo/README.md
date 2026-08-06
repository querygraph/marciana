# QueryGraph coffee-market memory demo

This demo follows the AgStack/Pale Fire pattern of combining entity-aware
retrieval, temporal evidence, source citations, and feedback-driven learning,
but keeps Marciana’s authority boundary: Sail computes, Grust ranks/persists,
TypeSec authorizes, TypeDID authenticates, and memory changes are governed
verbs rather than model-owned writes.

The flow is:

1. Load Honduras coffee observations from a Dataverse native API dataset (or
   the checked-in fixture) into a Sail Spark Connect table.
2. An agronomist agent queries Sail and remembers source-backed facts.
3. A market agent remembers a price, recalls it, then improves it with a newer
   observation while retaining the old fact as historical.
4. A steward recalls the current market view, explicitly forgets one obsolete
   memory, and verifies the forgotten fact no longer appears in local recall.
5. Every agent turn is a Pydantic AI v2 structured `AgentDecision`; tools are
   the only path to memory verbs.

Reference designs: [AgStack](https://agstack.org/),
[Pale Fire](https://github.com/agstack/palefire), and the
[Pydantic AI structured-output docs](https://pydantic.dev/docs/ai/core-concepts/output/).

## Key-free run

```bash
cd /path/to/marciana
python3 -m venv /tmp/marciana-coffee-venv
/tmp/marciana-coffee-venv/bin/pip install -r examples/coffee_market_demo/requirements.txt
PYTHONPATH=. /tmp/marciana-coffee-venv/bin/python -m examples.coffee_market_demo.demo
```

This uses Pydantic AI’s deterministic `TestModel`, so the lifecycle is
reproducible without a provider key. For live services:

```bash
export DATAVERSE_DATASET_URL='https://dataverse.example/api/datasets/:persistentId/?persistentId=doi:...'
export SAIL_SPARK_CONNECT_URL='sc://127.0.0.1:15002'
export QUERYGRAPH_URL='http://127.0.0.1:8080'
PYTHONPATH=.:../querygraph/qg-python python -m examples.coffee_market_demo.demo --live
```

Start Sail from the recorded revision in `compat/sail-revision.txt`; load the
Dataverse CSV through Spark Connect, and start qg-rust with its exact-DID
memory policy as documented in `../querygraph/qg-rust/docs/memory-service.md`.
The demo deliberately does not invent a Dataverse URL: provide the dataset
URL for the coffee corpus you are authorized to use.

The live-model option accepts the Pydantic AI v2 model string supported by the
installed provider, for example `--live-model openai:gpt-5-mini`. Keep the
deterministic tool phases for repeatable integration verification.
