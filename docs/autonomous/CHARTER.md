# Charter

## Mission

Experimentally rework this fork of Base into a unified node that inlines a minimal, licensed Reth
execution engine, eliminating the external Reth dependency (not merely a separate Reth process),
while preserving all existing L1 protocol behaviors. This is a non-production experiment on the
`gene` host only: no real funds, keys, production traffic, or paid infrastructure, and no changes
to upstream `base/base`.

## Measurable end-state goals

Each goal below must have an objective, checkable acceptance signal recorded in
`evidence/evidence.jsonl` before it is considered met. None are met as of this writing.

1. **Inline minimal licensed Reth execution — not merely process removal.** The execution path runs
   from in-tree, licensed, minimally-scoped Reth-derived logic under `base-`-prefixed crates.
   Running that logic in the same OS process as everything else is necessary but not sufficient:
   the external Reth *dependency* itself, not just a separate Reth *process*, must be gone (see
   goal 2). Acceptance: architecture inventory shows no external Reth process in the running node's
   process tree during a devnet run, `verify-base` transaction inclusion passes against the inlined
   path, and goal 2's zero-dependency audit passes.
2. **Zero external Reth dependency, at every layer.** Acceptance: a full audit shows zero
   `reth-*`/`reth` entries as (a) `Cargo.toml`/`Cargo.lock` crates.io or git dependencies in any
   manifest — build, dev, or runtime feature — (b) build-script (`build.rs`) fetch/build steps,
   (c) CI/dev tooling that builds or runs an external Reth binary, and (d) docs/guides that instruct
   running against an external Reth process. "Vendored" or "distinguished from a live coupling" is
   not an acceptable substitute for zero: inlined source must physically live in-tree under
   `base-`-prefixed crates. License/attribution notices and copyright headers for the inlined source
   are retained and are not themselves a dependency. Documented in the architecture inventory with
   the exact crates affected and the exact commands run (`cargo tree`, manifest/lockfile grep,
   build-script inspection).
3. **EL-only sync with L1 protocol behaviors intact; CL sync code deleted, not just decoupled.** The
   node syncs and serves execution-layer state without a separate consensus-layer sync *process*,
   and the CL-sync code path and its dependency footprint are removed from the tree — not left
   in-tree and merely not invoked as a separate process. Batch derivation, deposits, and other
   existing L1-derived behaviors continue to work. Acceptance: `verify-base` and any added
   derivation-specific checks pass with only the EL-only sync path present in the binary and
   running; a code/dependency audit confirms the removed CL-sync code path is absent, not dormant.
4. **No Engine API, including in-process.** No `engine_*` JSON-RPC namespace is exposed or dialed
   between processes, and the Engine API request/response types and client/server code are deleted
   from the tree. An in-process function call that reimplements Engine-API semantics (the same
   payload/forkchoice request/response shapes, just without the JSON-RPC transport) does not satisfy
   this goal. Acceptance: RPC namespace audit of the running binaries shows zero `engine_*` methods,
   and a code audit confirms the `engine_*` types and the CL/EL Engine-API client/server code (e.g.
   `crates/consensus/engine/`) are removed or fully replaced by a direct, non-Engine-API call path —
   not retained internally with the same protocol shape.
5. **Stateless consensus with a single explicitly-owned durable state.** The consensus/derivation
   actor(s) hold no independently authoritative durable state of their own — no consensus-side store
   that could diverge from, or be recovered independently of, the execution-layer state. Exactly one
   component, documented by name, owns durable state including recovery, forkchoice, and
   safe/finalized checkpoint tracking; the consensus actor(s) evaluate inputs and propose effects
   against that owner rather than keeping a parallel authoritative copy (volatile, reconstructible
   caches are fine). Existing safety checks (reorg/depth limits, forkchoice validation,
   equivocation/duplicate-payload rejection, and any Commonware vote/lock durability required before
   a vote or signature is released) must be preserved, not dropped as a side effect of removing
   state. See [`adr/0001-staged-consolidation-and-state.md`](adr/0001-staged-consolidation-and-state.md)
   for the accepted target model and confirmed current gap (separate `SafeDB`/`CheckpointDB` stores).
   Acceptance: code inventory shows a single state-owning component for recovery/forkchoice/
   checkpoints, no duplicate authoritative stores, and existing safety-check test coverage still
   passes unchanged in behavior.
6. **Only `base`/`basectl` product binaries — no relabeling to evade this.** Acceptance: released
   binaries are limited to `base` and `basectl`; every other current entry under `bin/` is removed or
   merged into those two. "Reclassified as an internal dev-only tool" is available only for
   genuinely internal development utilities (e.g. benchmarking/fixture-generation scripts with no
   external users) — it may not be used to relabel an actual shipped product or externally-relied-on
   service (for example `challenger`, `prover`, `proposer`, or other infra components in the live
   proof/challenge/withdrawal path) as "internal tooling" to avoid consolidating or removing it. Each
   binary needs a documented purpose, users, and artifact/release/build callers before it is
   assigned a remove/merge/legitimate-dev-tool disposition (`adr/DECISIONS.md` D5).
7. **Experimentally merge P2P.** A single P2P stack carries both transaction gossip and L2 payload
   propagation (replacing the current DevP2P + libp2p split described in
   `docs/guides/P2P.md`). Acceptance: demonstrated on a devnet run with evidence of both message
   classes traversing the merged stack.
8. **Rust conductor actor, fault-tested.** A single Rust actor owns sequencing/conductor
   responsibilities with a documented interface and colocated tests. Acceptance: the actor exists,
   is the sole writer of sequencing decisions, has passing behavioral tests, **and** acceptance
   explicitly includes fault tests for: leadership change (handoff to a new leader), fencing (a
   demoted/stale leader cannot commit), failover (a follower takes over after leader failure),
   restart (the actor recovers correct state after process restart), and split-brain (two nodes each
   believing they are leader do not both commit conflicting sequences). Passing behavioral tests
   that do not cover all five scenarios do not satisfy this goal.
9. **Actual Commonware multi-node sequencing, sized to a declared fault model.** A multi-node devnet
   reaches agreement on a block sequence using Commonware primitives — not a single-node simulation.
   Before implementation or a claimed run, its topology, exact voter identities, membership source,
   admission/removal rules, membership-transition policy, quorum calculation, and expected behavior
   under each tested failure must be explicit. Static membership is allowed only when deliberately
   declared and covered by tests; an unspecified or accidental static set is not acceptable.
   The node count and quorum follow the declared fault model, not an arbitrary "two or more." A
   Byzantine fault-tolerant model for `f=1` needs at least `3f+1 = 4` voting nodes; a crash-fault-only
   model needs `2f+1 = 3` for `f=1` and must be stated as such explicitly. Two nodes do not demonstrate
   `f=1` BFT and must not be claimed as such. Acceptance: those membership and quorum details and the
   declared fault model are recorded before the run; an independent-reviewer-witnessed multi-node run
   demonstrates the expected quorum/liveness and safety behavior while injecting node failure,
   partition, and any declared membership transition against the declared `f`.

## Non-goals / explicit exclusions

- No production or live-protocol changes; no real funds or keys.
- No paid infrastructure or model-runtime credentials on `gene`.
- No mutation of upstream `base/base` (issues, PRs, comments) under any circumstance.
- The existing feature map covers only sequencer transaction inclusion today; validator sync and L1
  finality verification are not yet implemented or verified in this experiment. This is a **current
  coverage gap**, not a permanent exclusion: EL-only sync (goal 3) does not license dropping
  canonical L1 deposits/batches/reorg/finality verification from the required end state.
  `BEHAVIOR_INVENTORY.md` tracks this as unresolved coverage, not an accepted non-goal.
- No recursive agent spawning; no builds/devnets from the docs worktree.

## Invariants (carried from `/home/refcell/base-autonomous-20260924/state/RESUME.md`, do not weaken)

- One writer per remote worktree; the integrator alone owns integration, pushes, draft PR, and the
  private bdoc.
- Every code milestone requires a full exact-tree devnet + `verify-base` run and independent review
  before it counts as done. Unknown, skipped, or blocked results are never recorded as a pass.
- File-editing tools author content; shell/Python patch scripts never author repository content.
- No force pushes or resets; explicit fork remotes only.
- Recovery/resume state lives at the absolute remote path
  `/home/refcell/base-autonomous-20260924/state/RESUME.md` (host `gene`, user `refcell`) — never a
  repo-relative `state/RESUME.md`, which does not resolve from every worktree/cwd. These versioned
  docs must stand alone and be usable by a fresh agent that has not yet read that file.
