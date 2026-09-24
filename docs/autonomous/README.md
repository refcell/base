# Autonomous Factory Docs

Durable, versioned tracking for the `experiment/base-autonomous-20260924` mission: an
experimental, non-production rework of Base toward an inlined-execution, EL-only,
stateless-consensus architecture with Commonware-based multi-node sequencing. See
`/home/refcell/base-autonomous-20260924/state/RESUME.md` (remote, non-Git, absolute path) for the
live operational checkpoint; these docs are the durable record that survives checkpoint rewrites,
and are written to stand alone for a fresh agent that has not read that file yet.

## Contents

- [`CHARTER.md`](CHARTER.md) — mission statement, measurable end-state goals, non-goals, invariants.
- [`ROADMAP.md`](ROADMAP.md) — dependency-aware milestones from current baseline (M0, still pending) onward.
- [`OWNERSHIP.md`](OWNERSHIP.md) — worktree map, one-writer-per-worktree rule, evidence-log ownership, active team/roles.
- [`adr/TEMPLATE.md`](adr/TEMPLATE.md) — ADR template for future architecture decisions.
- [`adr/DECISIONS.md`](adr/DECISIONS.md) — open decision backlog; one ADR (0001) accepted so far.
- [`adr/0001-staged-consolidation-and-state.md`](adr/0001-staged-consolidation-and-state.md) — accepted: first vertical slice (`base load-test`) and the target durable-state-ownership model.
- [`BEHAVIOR_INVENTORY.md`](BEHAVIOR_INVENTORY.md) — known vs. not-yet-inventoried behavior/verification coverage.
- [`evidence/SCHEMA.md`](evidence/SCHEMA.md) — append-only evidence log schema.
- [`evidence/evidence.jsonl`](evidence/evidence.jsonl) — the append-only log itself (seeded with one
  labeled illustrative row only; no real check has produced a pass yet).
- [`evidence/validate_evidence.py`](evidence/validate_evidence.py) — standard-library-only validator
  for `evidence.jsonl` (schema fields, identity/environment presence, pass-requires-evidence,
  example rows never counted as passes). Run with
  `python3 docs/autonomous/evidence/validate_evidence.py`; no Rust build required or invoked.
  Behavioral tests: [`evidence/test_validate_evidence.py`](evidence/test_validate_evidence.py).
- [`BLOCKERS_AND_NEXT.md`](BLOCKERS_AND_NEXT.md) — current blockers and the exact next-action queue.

## Ground rules for these docs

- Nothing here claims a check passed unless an `evidence.jsonl` row with `"example": false` (or the
  field omitted) backs it with a command, result, evidence reference, and full identity
  (`commit_sha`, `tree_sha`, `environment`).
- `evidence.jsonl` must never gain another `example: true` (or otherwise fabricated/placeholder) row.
  The one seeded illustrative row is permanent; any future illustrative example belongs only in
  `evidence/SCHEMA.md`'s own example block, never appended to the live log.
- Only the current single writer of `worktrees/factory-docs` (see `OWNERSHIP.md`) appends to
  `evidence/evidence.jsonl`. Other agents hand off their check results for that owner to append —
  they do not append directly to a worktree they do not own.
- Architecture decisions are open until their ADR is accepted (recorded in `adr/DECISIONS.md`).
  Acceptance is the supervisor's decision after considering independent oracle + reviewer critique,
  not a requirement that the advisors unanimously agree. One decision (D2) is accepted so far; see
  `adr/DECISIONS.md` for everything still open.
- This worktree (`worktrees/factory-docs`, branch `experiment/factory-bootstrap-20260924`) is docs-only.
  No source, build, or devnet changes belong here.
- Keep `AGENTS.md`/`CLAUDE.md` pointing here without restating this content.
