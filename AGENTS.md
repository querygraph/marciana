# Repository Guidance

## Architectural boundaries

- Read and preserve `DESIGN.md` before changing ownership, dependency
  direction, trust boundaries, persistence semantics, or Sail integration.
- Marciana is the product and composition layer. TypeSec, Grust, Sail, and
  LakeCat must not depend on Marciana.
- Only TypeSec's capability-gated `MemoryVault` may reveal or mutate protected
  memory. Stores persist, indexes rank, and cognition proposes.
- Cognee is inspiration only. Any optional compatibility facade lowers into
  Marciana's native four verbs and must not import Cognee runtime, storage, or
  adapter behavior.
- Preserve route, wire, durable identifier, and database compatibility during
  extraction. Change them only with an explicit versioned migration.

## Sail integration

- Refresh from the current canonical Sail upstream source for every integration
  baseline and release candidate.
- Record the exact verified revision in `compat/sail-revision.txt` and the
  result in `COMPATIBILITY.md`.
- Build that source revision and run live tests against its explicit binary;
  never accept an arbitrary binary from `PATH` as proof.
- Contribute generic fixes and capabilities upstream to Sail, then update the
  Marciana revision. Never fork, copy, or privately patch generic Sail
  behavior in Marciana.
- Keep only memory-specific schemas, proposal computation, and adapter code in
  Marciana.

## Code quality

- Keep production files small and single-purpose. Split a module when it gains
  a second responsibility; do not use large files as informal namespaces.
- Keep functions cohesive, make invalid states difficult to construct, and
  keep orchestration shallow by extracting named domain operations.
- Apply DRY to semantics, not just syntax. Canonical digest construction,
  validation, state transitions, retry rules, and shared adapter mapping each
  have one authoritative implementation.
- Keep adapters thin: translate types and delegate to shared domain logic.
- Put tests in separate files, sibling `tests/` modules, or integration-test
  targets. Do not grow production modules with large inline test sections.
- Prefer deterministic fixtures and table-driven conformance tests. Test
  persistence with create, close, reopen, retry, collision, and recovery paths.
- Do not weaken domain types with public convenience constructors solely to
  make tests easier; build test values through the real authority boundary.

## Changelog

- Maintain `CHANGELOG.md` for every logical user-visible behavior,
  documentation, compatibility, API, schema, packaging, and release change.
- Add the entry in the same change that introduces the outcome. Keep unreleased
  work under `Unreleased`, grouped by the date it landed.
- Keep entries concise and outcome-focused. Move entries to a version section
  only when preparing that release.

## Prompt-boundary commits

- Before starting a new user prompt, finish any completed unit already in the
  working tree: update the changelog, run the relevant checks, and commit it.
- Keep commits separated by logical unit. Do not mix an earlier completed unit
  with new work simply because both are present locally.
- If work is incomplete, bring it to a clean, verified stopping point before
  beginning the next prompt.

## Compatibility and verification

- Keep `COMPATIBILITY.md` and machine-readable pins current in the same change
  as a dependency, schema, route, or fixture change.
- Committed dependencies must use released versions or remotely reachable exact
  revisions. Clean builds must not depend on sibling path layout.
- Before delivery run formatting, strict Clippy, all workspace tests, TypeSec
  conformance, persistence/recovery tests, dependency-direction checks, and the
  live Sail gate applicable to the change.
- Ignore local render, database, log, and inspection artifacts; do not commit
  them unless they are intentional fixtures.
