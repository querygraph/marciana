# Marciana

Marciana is the memory and cognition product layer in the QueryGraph stack. It
composes TypeSec, Grust, Sail, and LakeCat-governed catalog integration behind
a native four-verb API:

- `remember` ingests protected memory;
- `recall` retrieves through the TypeSec vault;
- `improve` runs durable cognition and applies inert proposals through a
  guarded commit; and
- `forget` performs scoped, audited removal or retraction.

The project is currently in its repository-extraction phase. The initial
`querygraph-memory` transplant will preserve behavior, durable identifiers, and
tests before the workspace is split into smaller product and adapter crates.

## Trust boundary

TypeSec remains the authority for capability checks, labels, retention,
quarantine, protected-content access, proposal validation, and TypeDID
verification. Grust owns generic graph and transaction mechanics. Sail owns
generic distributed execution. LakeCat owns governed-scan proofs. Marciana
owns memory-product orchestration and adapters, and QueryGraph consumes it.
LakeCat is required for catalog-backed governed `improve`; local memory use may
run the native lifecycle without a catalog adapter.

Only the capability-gated TypeSec vault may reveal or mutate protected memory.
Cognition proposes, indexes rank, and stores persist; none of them is an
authorization authority.

Cognee is inspiration only. An optional Cognee-shaped facade may lower into
Marciana's native four verbs, but Marciana does not depend on Cognee's runtime,
adapters, stores, or completeness model.

## Repository contracts

- [DESIGN.md](DESIGN.md) is authoritative for project ownership, dependency
  direction, trust boundaries, and Sail integration policy.
- [COMPATIBILITY.md](COMPATIBILITY.md) records the cross-stack compatibility
  contract and points to machine-readable revision pins.
- [CHANGELOG.md](CHANGELOG.md) records every user-visible logical change.
- [AGENTS.md](AGENTS.md) defines contribution and delivery rules.

The pre-crate scaffold can be validated with:

```sh
cargo metadata --no-deps --format-version 1
```

Once the transplant lands, CI additionally requires formatting, lint,
workspace tests, conformance tests, and a live Sail gate built from the exact
recorded upstream revision.
