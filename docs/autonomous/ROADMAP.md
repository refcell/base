# Roadmap

Dependency-aware milestones toward the charter's end-state goals. Status reflects this writing
(2026-09-24); update it, and record the supporting evidence row, whenever a milestone's status
changes. "Pending" means not started or not yet passing; it never means "assumed done."

| ID | Milestone | Depends on | Owner (per `/home/refcell/base-autonomous-20260924/state/RESUME.md`) | Status |
| --- | --- | --- | --- | --- |
| M0 | Unchanged baseline build, tests, full devnet and verify-base | — | Supervisor: sole remote executor | Build/source-built images pass. Nextest: 8620 pass, 23 fail, 83 skip. Full devnet inclusion and independent-validator receipt match pass; restart check in progress. Not universally green; see [ADR-0002](adr/0002-baseline-failure-disposition.md) and evidence ledger. |
| M1 | Architecture inventory: read-only mapping of Reth coupling, Engine API usage, derivation, P2P, conductor, Commonware surfaces | — | `architecture-inventory` scout | In progress |
| M2 | Bootstrap charter, roadmap, ownership, ADRs, behavior inventory and evidence validator | — | Supervisor / file-only authors | Corrections reviewed independently; 17 validator tests pass on gene. Candidate integration/full-devnet cycle and draft PR remain pending. |
| M3 | Council decision pass: bounded oracle + independent reviewer on the first vertical slice and state-ownership invariants (max two passes), decisions recorded as ADRs | M0, M1, M2 | oracle + reviewer (per RESUME step 2) | **D2 decided, one pass.** [ADR-0001](adr/0001-staged-consolidation-and-state.md) accepted (status: accepted experimental direction, implementation/behavioral proof pending). D3 (state-ownership) is only partially resolved by ADR-0001's target model; concrete storage/fencing atomicity (D8) is explicitly deferred. D1, D4–D7 remain open — see `adr/DECISIONS.md`. |
| M4 | Integrate real load/drain/rescue/config/output/shutdown capability as `base load-test`; remove standalone product after parity | M0 exercised and failures dispositioned per ADR-0002; M3 | File-only writer, isolated `worktrees/load-test`; supervisor executes | Implementation started. Temporary standalone wrapper permitted only for differential tests, not final acceptance. Full candidate devnet and independent review required before integration. |
| M5 | EL-only sync path with Engine API removed, L1 protocol behaviors verified intact | M4, **and D3/D8 (state-ownership + storage/fencing atomicity) resolved** | TBD | Pending. Per ADR-0001's durable-authority interpretation, the serialized execution mutation boundary (payload validation, fork rules, forkchoice, INVALID/VALID/SYNCING-equivalent outcomes, cancellation/backpressure/ordering) must be specified before Engine transport is removed — ownership invariants come before Engine changes, not after. |
| M6 | Stateless consensus with a single explicitly-owned durable execution state component | M5 | TBD | Pending. Current source does not satisfy this: `crates/consensus/service/src/service/node.rs:416-451` opens separate `SafeDB` and `CheckpointDB` stores (confirmed present, independently reviewer-verified); their restart/recovery behavior is not yet proven, and D8 (atomicity/fencing protocol) is unresolved. |
| M7 | Binary surface consolidated to only `base` and `basectl` product binaries | M4 (can proceed incrementally alongside M5/M6) | TBD | Pending. Standalone node/consensus wrappers are still referenced by `etc/docker/Dockerfile.rust-services`, `docker-bake.hcl`, `docker-compose.anvil-l1.yml`, and `.github/workflows/build-release.yml` (ADR-0001); do not claim this milestone complete after moving only one binary (`load-tester`, M4). |
| M8 | Experimental P2P merge (single stack for tx gossip + L2 payload propagation) | M5 | TBD | Pending |
| M9 | Rust conductor actor as sole sequencing-decision writer, fault-tested (leadership, fencing, failover, restart, split-brain) | M6, M8 | TBD | Pending |
| M10 | Actual Commonware multi-node sequencing sized to a declared fault model, demonstrated on a multi-node devnet with fault-tolerance evidence | M9 | TBD | Pending |

## Integration checkpoints (owned by the integrator, not this worktree)

- After M2, the integrator creates one draft PR (`gh pr create --repo refcell/base --base main --head
  refcell:experiment/base-autonomous-20260924 --draft`) once an appropriate bootstrap commit exists.
  This docs worktree does not create PRs or push; see `OWNERSHIP.md`.
- Every milestone from M4 onward repeats: small slice → tests → fresh full devnet/`verify-base` →
  independent review → commit/push → feature-map/PR/bdoc update, per
  `/home/refcell/base-autonomous-20260924/state/RESUME.md` step 6.

## Explicit unknowns affecting sequencing

- The exact scope of "minimal licensed Reth execution" (D1: which modules/crates, which license
  obligations) is not yet decided. ADR-0001 explicitly rejected "arbitrary Reth vendoring" as the
  first slice because the "closure/license inventory [is] incomplete; no proof of a bounded minimal
  slice yet" — that inventory is still needed before D1 can be decided.
- Whether P2P merge (M8) can start before Commonware sequencing (M10) design is fixed is open; the
  dependency edge above is a planning default, not a ratified decision.
- M4's `base load-test` slice does not by itself satisfy any charter goal fully (not goal 1's
  inlining, not goal 6's binary consolidation); it is a bounded first step chosen for evidenced low
  risk, per ADR-0001's council record — do not report it as closing a charter goal.
