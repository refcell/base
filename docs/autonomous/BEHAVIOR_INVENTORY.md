# Behavior and Verification Inventory

Status at 2026-09-24. Verified observations are scoped to exact commands and source identities in `evidence/evidence.jsonl`; they never imply exhaustive protocol coverage. Static inventory is incomplete. `NOT YET INVENTORIED` means a gap in this document, not proof that code or tests do not exist.

## Baseline / already-covered behaviors

| Behavior | Documented/observed current state | Verification method | Status this experiment |
| --- | --- | --- | --- |
| Sequencer ETH-transfer inclusion | Source-built full local devnet, chain84538453; receipt status/from/to/block identity and membership asserted. | `scripts/verify-baseline-devnet.sh`; verify-base feature page | Passed on unchanged baseline; evidence evt-0004. |
| Validator propagation and restart | Independent client returns same receipt; after all long-lived services restart both old receipt and new inclusion agree. | `scripts/restart-baseline-devnet.sh` | Passed graceful restart, evt-0005; fresh sync/partitions/corruption not covered. |
| L1-derived head finality | Both nodes report finalized heads beyond included transaction; canonical L2 head hash and L1 origin hash checked, origin no later than finalized L1. | `scripts/verify-baseline-finality.sh` | Passed baseline observation2026-09-24T16:35:36Z; L1 finalized39. Not full batch-derivation or adversarial-finality proof. |
| Reth coupling in execution crates | `Cargo.lock` at commit `539605ce58aaf02fe5c0382fcd9139c1ee4207e9` contains **109** Reth packages (name starts `reth` OR `source` contains `/reth`): 104 from git `base/reth` tag `base-v2.5.2.6`, commit `5877708bbf9219c44758cd2ce28a365f738661f7` (cached git checkout HEAD confirmed to match), plus 5 from crates.io registry 0.6.0 (`reth-codecs`, `reth-codecs-derive`, `reth-primitives-traits`, `reth-rpc-traits`, `reth-zstd-compressors`, all declaring `license = "MIT OR Apache-2.0"`). This is the **entire lockfile inventory** (build/dev/runtime combined), **not** a minimal runtime closure — do not retain an earlier scout figure of "~65 crates" as a complete count; it undercounted and conflated "appears in `crates/execution/*/Cargo.toml`" with "appears in `Cargo.lock`." `LICENSE-MIT` (copyright 2022-2026 Reth Contributors) and `LICENSE-APACHE` are present at repo root; a root-depth-2 `NOTICE` file search found none, but that is not an exhaustive attribution/notice audit. | Dependency audit (`Cargo.lock` grep, `cargo tree`, git-checkout hash comparison) | **Unclassified.** Presence and exact count confirmed; which of the 109 are live runtime execution-path dependencies vs. build/dev/test-only, and whether the license/attribution retained under goal 2 is complete, are not yet determined. Owned by `architecture-inventory` scout (M1). |
| Engine API usage | `docs/guides/P2P.md` describes the standard OP-stack-style CL/EL split bridged by the Engine API over local JSON-RPC; `crates/consensus/engine/` exists in-tree (client, sync/forkchoice, task_queue, state). Not yet confirmed which RPC methods are actually exposed/dialed, or whether any in-process call path reimplements Engine-API semantics without JSON-RPC (which charter goal 4 also forbids). | RPC namespace audit | **Unconfirmed** — described in docs, not yet inventoried against running code. Owned by `architecture-inventory` scout (M1). |
| P2P stack composition | `docs/guides/P2P.md` documents two layers: DevP2P/RLPx (execution layer) and libp2p/gossipsub (consensus layer, `crates/consensus/gossip`, `crates/consensus/peers`, `crates/consensus/disc`). | Code inventory | **Unconfirmed against running behavior** — documented architecture only. |
| Commonware usage | No `commonware` dependency found anywhere in the workspace (`grep -ril commonware` on root and crate manifests returned no matches). | Dependency audit | **Confirmed absent today** — consistent with "actual Commonware multi-node sequencing" being an unbuilt end-state goal (M10), not existing behavior. |
| Conductor actor | `crates/consensus/service/src/actors/sequencer/conductor.rs` defines a `Conductor` trait (`leader()`, `active()`, `commit_unsafe_payload()`, `override_leader()`) mirroring `op-conductor`'s `CommitUnsafePayloadPath`; CLI/RPC surface at `crates/infra/basectl/src/{app/views,commands,rpc}/conductor.rs`; tests exist at `crates/consensus/service/src/actors/sequencer/tests/{actor_test.rs,admin_api_impl_test.rs}`. Whether those tests cover charter goal 8's required leadership/fencing/failover/restart/split-brain scenarios is **not established** — do not assume they do. | Code inventory | **Unclassified** — component located; behavioral coverage owned by `architecture-inventory` scout (M1) and, ultimately, goal 8's fault-test acceptance criteria. |
| Binary surface | `bin/` currently contains 21 entries (`audit-archiver`, `base`, `base-telemetry`, `basectl`, `builder`, `challenger`, `challenger-e2e`, `consensus`, `load-tester`, `node`, `proposer`, `prover`, `prover-registrar`, `roxy`, `shadow-metrics`, `sidecrush`, `snapshotter`, `snark-e2e`, `websocket-proxy`, `zk-benchmarks`, `zk-fork-dispute`), confirmed by `ls bin`. | Directory listing | **Confirmed as of this commit.** Consolidation to only `base`/`basectl` is future work (M7, D5); `challenger`/`prover`/`proposer` are real products with build/release/Docker callers (ADR-0001) and may not be relabeled internal tooling. |

## Durable state ownership (new — informs charter goal 5 / D3 / D8)

- Observed behavior: `crates/consensus/service/src/service/node.rs:416-451` opens two separate
  stores, `SafeDB` and `CheckpointDB` (independently reviewer-confirmed present); `crates/consensus/safedb/`
  exists as its own crate. Durable state is not yet unified under one owner.
- Preconditions: unknown — not yet inventoried.
- Invariants: none yet verified in code; ADR-0001 states the target (one durable owner for
  execution/chain state, head/checkpoint metadata, derivation replay positions, sequencing
  terms/fences, and signing journals) but that target is not implemented.
- Components: `crates/consensus/service/src/service/node.rs` (~lines 416-451), `crates/consensus/safedb/`;
  the module backing `CheckpointDB` was not located by path this pass — do not assume it is the same
  crate as `safedb`.
- Known verification methods: none identified this pass for restart/crash-consistency of either
  store.
- Exact commands/assertions: none known.
- Evidence/tree/env/time: none recorded; ADR-0001 review was a code-read, not a logged
  `evidence.jsonl` check (no `tree_sha`/`environment` captured for it).
- Gaps: whether `SafeDB`/`CheckpointDB` can independently diverge, and whether either persists
  before a dependent effect (e.g. signature release) is committed, are unconfirmed. ADR-0001
  explicitly defers exact atomicity/fencing design to `adr/DECISIONS.md` D8 — this row is not
  resolved by that ADR.

## Required coverage additions (not yet inventoried)

Each behavior below lists: observed behavior, preconditions, invariants, components, known
verification methods (unit/property/integration/e2e/differential/fault/recovery/compat/perf/
manual), exact commands/assertions where actually known, evidence/tree/env/time, and gaps. Fields
say "Unknown" or "None identified this pass" rather than inventing detail.

### Validator sync and propagation
- Observed: independent validator agrees with sequencer's successful transaction receipt before and after graceful full-service restart. Preconditions: isolated full devnet and chain84538453.
- Assertions: same transaction/block hash, successful status, post-restart old receipt preserved and a new higher block; evt-0004/0005 identify exact tree/environment/logs.
- Known method: end-to-end scripts linked above. Gaps: fresh state/history sync, catch-up bounds, partition behavior, corrupted state and protocol interoperability remain unverified.

### L1 finality verification
- Observed: baseline safe/finalized heads advance with submitted L1 batches. Script asserts `unsafe >= safe >= finalized >= included transaction block`, canonical finalized-L2 hash, canonical L1-origin hash and origin height no later than finalized L1.
- Components: consensus derivation finalizer, engine head state and execution RPC. Known method: `bash ../state/verify-baseline-finality.sh` after baseline inclusion and L1 finality are available.
- Evidence: baseline commit539605ce58aaf02fe5c0382fcd9139c1ee4207e9, tree1da623a8f06d439061be2c86033970cadca73b3f; gene Linux/Rust1.96.0; logs/baseline-finality1/ at2026-09-24T16:35:36Z.
- Gaps: L1 reorgs, delayed or malicious batches, finality regressions and independent differential protocol validation still require tests. These observations do not close finality acceptance.

### Deposits / batches / withdrawals / proofs
- Observed behavior: Unknown — file presence confirmed for related terms via grep, behavior not
  inventoried.
- Preconditions / Invariants: Unknown.
- Components (confirmed present via grep, behavior unverified): deposit-related —
  `crates/consensus/derive/src/attributes/stateful.rs`, `crates/consensus/derive/src/errors/{attributes.rs,pipeline.rs}`,
  `crates/consensus/derive/src/traits/attributes.rs`; withdrawal-related —
  `crates/consensus/cli/src/p2p.rs`, `crates/consensus/derive/src/pipeline/core.rs`,
  `crates/consensus/derive/src/stages/attributes_queue.rs`, `crates/consensus/derive/src/stages/batch/batch_stream.rs`,
  `crates/consensus/engine/src/{attributes.rs,query.rs,task_queue/tasks/insert/task.rs}`; proofs —
  17 subcrates under `crates/proof/` (`challenge`, `client`, `contracts`, `driver`, `executor`,
  `host`, `mpt`, `preimage`, `primitives`, `proof`, `proposer`, `prover-service`, `rpc`,
  `submission`, `tee`, `worker`, `zk`).
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: File existence does not establish correctness or test coverage. Owned by
  `architecture-inventory` scout (M1).

### Unsafe / safe / finalized head tracking and L1+L2 reorgs
- Observed behavior: Unknown — file presence confirmed via grep, behavior not inventoried.
- Preconditions / Invariants: Unknown.
- Components (confirmed present via grep, behavior unverified): `crates/consensus/derive/src/types/signals.rs`,
  `crates/consensus/derive/src/pipeline/core.rs`, `crates/consensus/derive/src/stages/batch/{batch_stream.rs,batch_validator.rs}`,
  `crates/consensus/engine/src/state/core.rs`, `crates/consensus/engine/src/sync/{checkpoint.rs,forkchoice.rs,mod.rs}`,
  `crates/consensus/engine/src/task_queue/core.rs`; see also the durable-state-ownership row above
  (`SafeDB`/`CheckpointDB`).
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: Which safe/finalized transitions and L1/L2 reorg-handling paths are actually exercised or
  correct is not established. Owned by `architecture-inventory` scout (M1).

### Fresh sync / catch-up sync / peer discovery / network partitions
- Observed behavior: Unknown — not yet inventoried.
- Preconditions / Invariants: Unknown.
- Components (confirmed present via `ls`, behavior unverified): `crates/consensus/disc/src/{builder.rs,driver.rs,error.rs,handler.rs,lib.rs,metrics.rs}`,
  `crates/consensus/gossip/`, `crates/consensus/peers/`, `crates/consensus/engine/src/sync/`.
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: Fresh-sync vs. catch-up-sync distinction, discovery behavior under partition, are not
  established. Owned by `architecture-inventory` scout (M1).

### Restart / corruption recovery
- Observed behavior: `crates/consensus/service/src/actors/sequencer/recovery.rs` defines a
  `RecoveryModeGuard` (shared atomic flag observed via direct read) gating sequencer/payload-builder
  behavior; its actual restart-recovery semantics are not inventoried. A `corrupt` keyword grep
  across `crates/` returned only unrelated hits (`crates/batcher/encoder`, `crates/common/evm2`,
  `crates/common/precompile-storage`) — a weak signal, not proof that no corruption-handling exists.
- Preconditions / Invariants: Unknown.
- Components: `crates/consensus/service/src/actors/sequencer/recovery.rs`; durable stores from the
  ownership row above.
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: Whether `SafeDB`/`CheckpointDB`/execution-DB restart is crash-consistent is unverified;
  blocked on D8. Owned by `architecture-inventory` scout (M1).

### Leadership / fencing (conductor failover, incl. split-brain)
- Observed behavior: `Conductor` trait exists with `leader()`/`active()`/`override_leader()`/
  `commit_unsafe_payload()` (see conductor-actor row above); test files exist
  (`actor_test.rs`, `admin_api_impl_test.rs`, `test_util.rs`) but their scenario coverage for
  leadership change, fencing, failover, restart, and split-brain (charter goal 8) is unconfirmed.
- Preconditions / Invariants: Unknown.
- Components: `crates/consensus/service/src/actors/sequencer/{conductor.rs,tests/}`,
  `crates/infra/basectl/src/{app/views,commands,rpc}/conductor.rs`.
- Known verification methods: Unit/integration test files exist (presence only, scenario coverage
  unconfirmed). No fault/recovery-specific test confirmed for any of the five goal-8 scenarios.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: This is the exact gap charter goal 8 requires closing before conductor acceptance; do not
  treat existing test-file presence as satisfying it. Owned by `architecture-inventory` scout (M1)
  and, ultimately, the M9 slice.

### Membership changes / adversarial sequencing
- Observed behavior: Unknown. No Commonware dependency is present (confirmed absent, see baseline
  table), so any Commonware-specific membership-change/adversarial-sequencing behavior does not yet
  exist.
- Preconditions / Invariants: Unknown.
- Components: None identified this pass.
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: Whether the current single-sequencer path has any adversarial-input handling is not
  inventoried. Owned by `architecture-inventory` scout (M1); depends on D7 (Commonware approach).

### RPC / operator / observability / resources
- Observed behavior: Unknown — file presence confirmed via `ls`, behavior not inventoried.
- Preconditions / Invariants: Unknown.
- Components (confirmed present via `ls`, behavior unverified): `crates/consensus/rpc/src/{admin.rs,base.rs,client.rs,config.rs,dev.rs,health.rs,jsonrpsee.rs,l1_watcher.rs,net.rs,output.rs,p2p.rs,response.rs,rollup.rs,sync.rs,ws.rs}`;
  `crates/infra/shadow-metrics/`, `crates/utilities/metrics/`;
  `crates/consensus/service/src/actors/sequencer/{admin_api_impl.rs,metrics.rs}`.
- Known verification methods: None identified this pass.
- Exact commands/assertions: None known.
- Evidence/tree/env/time: None recorded.
- Gaps: Resource behavior under load (backpressure, shutdown) and operator-facing RPC coverage are
  not inventoried. Owned by `architecture-inventory` scout (M1).

## Explicit unknowns register

- Which of the 109 `Cargo.lock` Reth packages are live runtime execution-path dependencies vs.
  build/dev/test-only (blocks D1, M1) — the 109/104-git/5-registry count is the full lockfile
  inventory, not the minimal closure charter goal 2 requires auditing to zero.
- Which `engine_*` RPC methods are live on which binaries today, and whether any in-process call
  path reimplements Engine-API semantics without JSON-RPC (blocks D4, M1).
- Whether any crate already partially implements EL-only sync or stateless consensus (not checked
  this pass; needs scout follow-up).
- Fault-tolerance behavior of the current P2P/consensus stack under peer loss (not exercised).
- All nine "required coverage additions" rows above (validator sync, L1 finality, deposits/batches/
  withdrawals/proofs, unsafe/safe/finalized+reorgs, fresh/catch-up/discovery/partitions, restart/
  corruption, leadership/fencing, membership/adversarial sequencing, RPC/operator/observability/
  resources) and the durable-state-ownership row — none are inventoried beyond the component paths
  listed.

Update this table (and add an `evidence.jsonl` row, appended by the docs-worktree owner per
`evidence/SCHEMA.md`) whenever a listed status changes from not-yet-inventoried/unconfirmed to
verified, or whenever a check fails.
