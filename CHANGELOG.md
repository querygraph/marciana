# Changelog

All notable changes to Marciana are documented in this file.

## Unreleased

### 2026-08-06

- Recorded the draft canonical Sail upstream path for the reviewed generic
  Delta `MERGE` correction; release remains pending its merge and exact-source
  validation.

- Recorded the bounded Grust Sail harness and its terminal executor, backend,
  and cognition parity/secrecy results against the local Sail binary.

- Recorded QueryGraph's completed switch to the native Marciana governed
  application and the remaining cross-stack compatibility and recovery gates.

- Added native cognition unit coverage for canonical source-selection and
  field-mapping digest boundaries, including duplicate and ambiguous mapping
  rejection.

- Added Marciana's native governed cognition composition: verified TypeDID
  intent binding, LakeCat revalidation, TypeSec authority priming and commit,
  proposal validation, receipt signing, and the opaque `improve` operation now
  live together in `marciana-cognition`.

- Updated the delivery record to distinguish completed native error-boundary
  extraction from the still-required move of governed `improve` composition.

- Added Marciana-owned, fixed public errors for governed intent, proof,
  projection, authority, and proposal binding failures. The stable categories
  make the fail-closed composition boundary available without a QueryGraph
  implementation dependency.

- Added Marciana's fixed public mapping for TypeSec protected-memory failures.
  Callers receive stable, non-disclosing categories rather than policy details,
  record identifiers, or backend-controlled text.

- Added `marciana-cognition` with the closed host-selected engine binding.
  Only fixed reference and live Sail profiles can receive protected cognition
  input; arbitrary engine implementations cannot self-assign trusted identity.
  Its explicit test-support feature supplies test doubles without exposing that
  construction path to production consumers.

- Added `marciana-catalog`, the native LakeCat proof-to-cognition-source
  adapter. It preserves LakeCat ownership of proof validation while moving the
  memory-product translation out of QueryGraph.

- Added Marciana-owned cognition proof-boundary validation, including the
  configured-catalog check and the stricter flattened table-identity budget
  required before a LakeCat proof can reach a cognition engine.

- Replaced Marciana's TypeSec and Grust sibling paths with exact reachable
  Git revisions, allowing its workspace to build and test independently of the
  local QueryGraph checkout layout.

- Recorded passing fresh-clone gates for Marciana and qg-rust, including the
  exact Git-pinned stack revisions and the remaining requirement to land the
  reviewed generic Sail correction in canonical upstream before release.

- Recorded the passing focused local Sail integration gate for the Grust
  cognition substrate while retaining the requirement for a remotely reachable
  Sail revision before Marciana claims an executable compatibility baseline.

- Replaced the obsolete scaffold-only compatibility description with the
  history-preserving extraction baseline and its explicit clean-clone and
  remotely-reachable dependency limitations.

- Recorded QueryGraph's opaque `improve` containment step: callers no longer
  receive a governed cognition proposal while the behavior-preserving
  composition is prepared for its later move into Marciana.

### 2026-08-06

- Transplanted `querygraph-memory` into standalone Marciana with preserved
  commit history, modular cognition modules, separate tests, and a compiling
  workspace over the current Grust and TypeSec checkouts.
- Matched the preserved crate's `0.12.0` compatibility line so QueryGraph can
  switch to the standalone path without changing its existing package contract.
- Switched the local qg-rust consumer to the standalone Marciana crate and
  re-ran its 100-test cognition/application suite successfully.
- Added a durable scheduler claim API that returns a lease only for pending or
  retryable work; a staged proposal now returns its digest for proposal-free
  recovery and can never be leased for re-planning.

### 2026-08-05

- Refreshed the canonical Sail source candidate to `50567c79` and recorded the
  rebased but still local Delta correction separately from the supported pin.
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
