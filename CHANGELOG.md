# Changelog

All notable changes to Marciana are documented in this file.

## Unreleased

### 2026-08-05

- Corrected restart semantics so only active pre-commit workers may re-plan;
  post-commit and lost-response handling now require the durable Grust proposal
  identity and TypeSec's proposal-free recovery path, independent of
  process-local proposal state.
- Recorded the gated TypeSec governed-source, reauthorization, recovery, and
  receipt foundation as complete while retaining remote-revision selection as
  standalone delivery work.
- Refined governed cognition into one opaque, leased `improve` state machine
  with post-engine LakeCat and TypeSec reauthorization, deterministic
  digest-only recovery, typed no-change completion, and no public or durable
  plaintext proposal.
- Defined Marciana's versioned composite source scope over LakeCat evidence,
  field mapping, ingestion profile, and row transformation, plus explicit
  audit/receipt digest and timestamp semantics.
- Corrected the delivery status to distinguish committed owner foundations
  from the TypeSec and LakeCat hardening still under final verification.
- Defined the governed-ingestion boundary: Marciana's trusted LakeCat adapter
  must derive and one-time bind exact drafts from an authorized scan, because a
  scan proof alone never authorizes independently supplied memory content.
- Established `MARCIANA.md` as the canonical active delivery record for the
  standalone extraction, current cross-stack status, execution order, and
  acceptance gates.
- Added the reviewed Marciana 2 plan, comparing leading AI-memory systems and
  prioritizing governed context assembly, assertion-safe temporal memory,
  durable formation, and proposal-based agent learning.
- Tightened the standalone design so Cognee remains inspiration rather than a
  compatibility milestone, and assigned canonical evidence and transition
  semantics to one authoritative stack layer each.
- Established Marciana's standalone repository, ownership and dependency
  contract, upstream-first Sail baseline policy, compatibility registry, and
  pre-crate CI scaffold.
