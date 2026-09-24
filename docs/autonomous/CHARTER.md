# Charter

## Mission

Experimentally rework this fork of Base into a unified node that inlines a minimal, licensed Reth
execution engine, removing the external Reth process/coupling, while preserving all existing L1
protocol behaviors. This is a non-production experiment on the `gene` host only: no real funds,
keys, production traffic, or paid infrastructure, and no changes to upstream `base/base`.

## Measurable end-state goals

Each goal below must have an objective, checkable acceptance signal recorded in
`evidence/evidence.jsonl` before it is considered met. None are met as of this writing.

1. **Inline minimal licensed Reth execution.** The execution path runs from an in-tree, licensed,
   minimally-scoped copy of the needed Reth logic under `base-`-prefixed crates, not a separate
   external Reth binary/process. Acceptance: architecture inventory shows no external Reth process
   in the running node's process tree during a devnet run, and `verify-base` transaction inclusion
   still passes against the inlined path.
2. **Eliminate external Reth coupling.** Acceptance: `cargo tree` / dependency audit shows the
   workspace no longer pulls a full external `reth-*` node crate set for runtime execution (vendored
   or inlined source is distinguished from a live coupling); documented in the architecture
   inventory with the exact crates affected.
3. **EL-only sync with L1 protocol behaviors intact.** The node syncs and serves execution-layer
   state without a separate consensus-layer sync process, while batch derivation, deposits, and
   other existing L1-derived behaviors continue to work. Acceptance: `verify-base` and any added
   derivation-specific checks pass with only the EL-only sync path running.
4. **No Engine API.** No `engine_*` JSON-RPC namespace is exposed or dialed between processes.
   Acceptance: RPC namespace audit of the running binaries shows zero `engine_*` methods.
5. **Stateless consensus with explicitly owned durable execution state.** The consensus actor(s)
   hold no independent long-lived execution state; exactly one component owns durable execution
   state, documented by name. Acceptance: code inventory shows a single state-owning component and
   no duplicate state stores.
6. **Only `base`/`basectl` product binaries.** Acceptance: released binaries are limited to `base`
   and `basectl`; every other current entry under `bin/` is removed, merged into those two, or
   explicitly reclassified as an internal dev-only tool excluded from release artifacts.
7. **Experimentally merge P2P.** A single P2P stack carries both transaction gossip and L2 payload
   propagation (replacing the current DevP2P + libp2p split described in
   `docs/guides/P2P.md`). Acceptance: demonstrated on a devnet run with evidence of both message
   classes traversing the merged stack.
8. **Rust conductor actor.** A single Rust actor owns sequencing/conductor responsibilities with a
   documented interface and colocated tests. Acceptance: the actor exists, is the sole writer of
   sequencing decisions, and has passing behavioral tests.
9. **Actual Commonware multi-node sequencing.** A multi-node devnet (two or more sequencer-capable
   nodes) reaches agreement on a block sequence using Commonware primitives — not a single-node
   simulation. Acceptance: independent-reviewer-witnessed multi-node run with fault-tolerance
   evidence recorded.

## Non-goals / explicit exclusions

- No production or live-protocol changes; no real funds or keys.
- No paid infrastructure or model-runtime credentials on `gene`.
- No mutation of upstream `base/base` (issues, PRs, comments) under any circumstance.
- No claim of validator sync or L1 finality verification; the existing feature map only covers
  sequencer transaction inclusion, and that scope is unchanged by this charter.
- No recursive agent spawning; no builds/devnets from the docs worktree.

## Invariants (carried from `state/RESUME.md`, do not weaken)

- One writer per remote worktree; the integrator alone owns integration, pushes, draft PR, and the
  private bdoc.
- Every code milestone requires a full exact-tree devnet + `verify-base` run and independent review
  before it counts as done. Unknown, skipped, or blocked results are never recorded as a pass.
- File-editing tools author content; shell/Python patch scripts never author repository content.
- No force pushes or resets; explicit fork remotes only.
