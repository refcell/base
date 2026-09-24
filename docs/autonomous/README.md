# Autonomous Factory Docs

Durable, versioned tracking for the `experiment/base-autonomous-20260924` mission: an
experimental, non-production rework of Base toward an inlined-execution, EL-only,
stateless-consensus architecture with Commonware-based multi-node sequencing. See
`state/RESUME.md` (remote, non-Git) for the live operational checkpoint; these docs are the
durable record that survives checkpoint rewrites.

## Contents

- [`CHARTER.md`](CHARTER.md) — mission statement, measurable end-state goals, non-goals, invariants.
- [`ROADMAP.md`](ROADMAP.md) — dependency-aware milestones from current baseline (M0, still pending) onward.
- [`OWNERSHIP.md`](OWNERSHIP.md) — worktree map, one-writer-per-worktree rule, active team/roles.
- [`adr/TEMPLATE.md`](adr/TEMPLATE.md) — ADR template for future architecture decisions.
- [`adr/DECISIONS.md`](adr/DECISIONS.md) — open decision backlog; no decisions have been accepted yet.
- [`BEHAVIOR_INVENTORY.md`](BEHAVIOR_INVENTORY.md) — known vs. unknown behavior/verification coverage.
- [`evidence/SCHEMA.md`](evidence/SCHEMA.md) — append-only evidence log schema.
- [`evidence/evidence.jsonl`](evidence/evidence.jsonl) — the append-only log itself (seeded with one
  labeled illustrative row only; no real check has produced a pass yet).
- [`BLOCKERS_AND_NEXT.md`](BLOCKERS_AND_NEXT.md) — current blockers and the exact next-action queue.

## Ground rules for these docs

- Nothing here claims a check passed unless an `evidence.jsonl` row with `"example": false` (or the
  field omitted) backs it with a command, result, and evidence reference.
- No architecture decision here is final. `adr/DECISIONS.md` items are open until an ADR is
  accepted by the bounded oracle + independent reviewer council described in `state/RESUME.md`.
- This worktree (`worktrees/factory-docs`, branch `experiment/factory-bootstrap-20260924`) is docs-only.
  No source, build, or devnet changes belong here.
- Keep `AGENTS.md`/`CLAUDE.md` pointing here without restating this content.
