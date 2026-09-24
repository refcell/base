# Roadmap

Dependency-aware milestones toward the charter's end-state goals. Status reflects this writing
(2026-09-24); update it, and record the supporting evidence row, whenever a milestone's status
changes. "Pending" means not started or not yet passing; it never means "assumed done."

| ID | Milestone | Depends on | Owner (per `state/RESUME.md`) | Status |
| --- | --- | --- | --- | --- |
| M0 | Baseline verification: unchanged-tree build, tests, and single-node devnet transaction inclusion (`verify-base`) | — | `baseline-verification` worker | **Pending** — not yet run this experiment |
| M1 | Architecture inventory: read-only mapping of Reth coupling, Engine API usage, derivation, P2P, conductor, Commonware surfaces | — | `architecture-inventory` scout | In progress |
| M2 | Bootstrap docs: charter, roadmap, ownership, ADR scaffold, behavior inventory, evidence schema (this deliverable) | — | docs writer (`worktrees/factory-docs`) | In progress / this commit |
| M3 | Council decision pass: bounded oracle + independent reviewer agree on the first vertical slice and state-ownership invariants (max two passes), decisions recorded as ADRs | M0, M1, M2 | oracle + reviewer (per RESUME step 2) | Pending |
| M4 | First vertical slice: small inlined-Reth execution change in a new worker worktree, with focused tests, full fresh exact-tree devnet + `verify-base`, and independent different-family review | M3 | new code worker (not yet assigned) | Pending |
| M5 | EL-only sync path with Engine API removed, L1 protocol behaviors verified intact | M4 | TBD | Pending |
| M6 | Stateless consensus with a single explicitly-owned durable execution state component | M5 | TBD | Pending |
| M7 | Binary surface consolidated to only `base` and `basectl` product binaries | M4 (can proceed incrementally alongside M5/M6) | TBD | Pending |
| M8 | Experimental P2P merge (single stack for tx gossip + L2 payload propagation) | M5 | TBD | Pending |
| M9 | Rust conductor actor as sole sequencing-decision writer | M6, M8 | TBD | Pending |
| M10 | Actual Commonware multi-node sequencing, demonstrated on a multi-node devnet with fault-tolerance evidence | M9 | TBD | Pending |

## Integration checkpoints (owned by the integrator, not this worktree)

- After M2, the integrator creates one draft PR (`gh pr create --repo refcell/base --base main --head
  refcell:experiment/base-autonomous-20260924 --draft`) once an appropriate bootstrap commit exists.
  This docs worktree does not create PRs or push; see `OWNERSHIP.md`.
- Every milestone from M4 onward repeats: small slice → tests → fresh full devnet/`verify-base` →
  independent review → commit/push → feature-map/PR/bdoc update, per `state/RESUME.md` step 6.

## Explicit unknowns affecting sequencing

- The exact scope of "minimal licensed Reth execution" (which modules/crates, which license
  obligations) is not yet decided; it is an open item in `adr/DECISIONS.md`, expected to be resolved
  in M3.
- Whether P2P merge (M8) can start before Commonware sequencing (M10) design is fixed is open; the
  dependency edge above is a planning default, not a ratified decision.
