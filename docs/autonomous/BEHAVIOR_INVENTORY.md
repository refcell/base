# Behavior and Verification Inventory

Status of behaviors relevant to the charter, as of 2026-09-24. "Verified" means a checkable command
was actually run this experiment and its result is recorded in `evidence/evidence.jsonl`. Nothing
below is marked verified because no such run has occurred yet (M0 is pending). Where this document
states a fact about current repository content, it is based on direct inspection (grep/`ls`) of
`worktrees/factory-docs` at commit `539605ce58aaf02fe5c0382fcd9139c1ee4207e9`, not a full audit.

| Behavior | Documented/observed current state | Verification method | Status this experiment |
| --- | --- | --- | --- |
| Sequencer ETH-transfer inclusion (local single-node devnet) | Covered by `.agents/skills/verify-base/references/features/local-devnet-transaction-inclusion.md`: `just devnet up-single`, `cast send`, `cast block`. | `verify-base` skill | **Not yet run.** Owned by `baseline-verification` worker (M0). |
| Validator sync | Explicitly out of scope of the existing feature map ("not validator sync"). | None defined | **Unknown** — no tooling exists to check this in-repo today. |
| L1 finality | Explicitly out of scope of the existing feature map ("not ... finality"). | None defined | **Unknown** — same as above. |
| Reth coupling in execution crates | `reth` appears as a dependency in `crates/execution/*/Cargo.toml` for 20 crates (chainspec, cli, consensus, eip8130-rpc, evm, exex, flashblocks, flashblocks-node, metering, node, payload, proofs, rpc, runner, shadow-indexer, shadow-indexer-db, trie, tx-forwarding, txpool-rpc, txpool-tracing), confirmed by `grep -ril reth`. | Dependency audit (`cargo tree`) | **Unclassified** — presence confirmed; whether each is a live external coupling vs. already-vendored is not yet determined. Owned by `architecture-inventory` scout (M1). |
| Engine API usage | `docs/guides/P2P.md` describes the standard OP-stack-style CL/EL split bridged by the Engine API over local JSON-RPC; `crates/consensus/engine/` exists in-tree. Not yet confirmed which RPC methods are actually exposed/dialed. | RPC namespace audit | **Unconfirmed** — described in docs, not yet inventoried against running code. Owned by `architecture-inventory` scout (M1). |
| P2P stack composition | `docs/guides/P2P.md` documents two layers: DevP2P/RLPx (execution layer) and libp2p/gossipsub (consensus layer, `crates/consensus/gossip`, `crates/consensus/peers`, `crates/consensus/disc`). | Code inventory | **Unconfirmed against running behavior** — documented architecture only. |
| Commonware usage | No `commonware` dependency found anywhere in the workspace (`grep -ril commonware` on root and crate manifests returned no matches). | Dependency audit | **Confirmed absent today** — consistent with "actual Commonware multi-node sequencing" being an unbuilt end-state goal (M10), not existing behavior. |
| Conductor actor | No dedicated "conductor" actor identified during this pass; `crates/consensus/service/src/actors/` exists but its exact responsibilities were not enumerated here. | Code inventory | **Unclassified** — owned by `architecture-inventory` scout (M1). |
| Binary surface | `bin/` currently contains 21 entries (`audit-archiver`, `base`, `base-telemetry`, `basectl`, `builder`, `challenger`, `challenger-e2e`, `consensus`, `load-tester`, `node`, `proposer`, `prover`, `prover-registrar`, `roxy`, `shadow-metrics`, `sidecrush`, `snapshotter`, `snark-e2e`, `websocket-proxy`, `zk-benchmarks`, `zk-fork-dispute`), confirmed by `ls bin`. | Directory listing | **Confirmed as of this commit.** Consolidation to only `base`/`basectl` is future work (M7, D5). |

## Explicit unknowns register

- Exact extent of Reth vendoring vs. live external coupling (blocks D1, M1).
- Which `engine_*` RPC methods are live on which binaries today (blocks D4, M1).
- Whether any crate already partially implements EL-only sync or stateless consensus (not checked
  this pass; needs scout follow-up).
- Fault-tolerance behavior of the current P2P/consensus stack under peer loss (not exercised).

Update this table (and add an `evidence.jsonl` row) whenever a listed status changes from
unconfirmed/unknown to verified, or whenever a check fails.
