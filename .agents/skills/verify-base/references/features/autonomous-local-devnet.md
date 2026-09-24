# Isolated factory devnet and restart

## Observable behavior and prerequisites

On the disposable local chain (84538453), a funded transfer submitted to the sequencer returns a
successful receipt. The receipt identifies the expected sender/recipient and matches both its block
and an independent validator's receipt. Restarting all long-lived services without regenerating
genesis or deleting data preserves that receipt; a new transfer is included at a higher block.

Requires gene's Docker/Compose, Rust 1.96.0, Go, Foundry `forge`/`cast`, `just`, `jq`, and this
experiment's dedicated checkout. Only public test accounts from `etc/docker/devnet-env` are used.
Never print their keys or source unrelated credentials. No production endpoint or funds are allowed.

## Isolation and commands

The versioned scripts are in `docs/autonomous/scripts/`. They intentionally target the dedicated
`/home/refcell/base-autonomous-20260924` experiment and pin its unchanged baseline revision. They
are not generic production tooling. Copy them into that root's `state/` directory before running.

Do not invoke stock `just devnet up-single` on a shared host without examining its project name and
cleanup behavior: the default `docker` project collided with an unrelated checkout here. The
runtime override uses project `base-autonomous-20260924`, unique container/image names, loopback
ports and experiment-only writable data. Client metrics uses18090 rather than occupied8090.

From the experiment integration checkout on gene (run long commands in dedicated tmux windows):

```bash
bash ../state/run-baseline-images.sh
bash ../state/start-baseline-devnet.sh
# In a second window after startup begins; readiness is bounded:
bash ../state/verify-baseline-devnet.sh baseline-verify2
bash ../state/restart-baseline-devnet.sh
```

The image script requires the prior build/test completion log; it does not silently skip them.
For a new candidate, make a reviewed runtime copy with that exact checkout/ref and fresh scoped
data/attempt identifiers. Never substitute a different tree while keeping baseline evidence labels.
Existing run directories cause verification to fail rather than silently rebroadcast. Investigate a
submission timeout before retrying. Startup stays attached in tmux. Restart excludes the one-shot
`setup-devnet` genesis generator. No broad cleanup or deletion is part of this procedure.

## Machine assertions and evidence

`verify-baseline-devnet.sh` checks all four L2 RPC chain IDs/heights, all fourteen Compose service
states (the successful one-shot setup may be exited), receipt status/from/to/hash, receipt/block
hash agreement, block transaction membership and the independent validator receipt. It exits
nonzero on failure. `restart-baseline-devnet.sh` repeats inclusion and additionally checks both
nodes still return the old receipt/block hash and the new block number is higher.

Baseline source: `539605ce58aaf02fe5c0382fcd9139c1ee4207e9`; tree:
`1da623a8f06d439061be2c86033970cadca73b3f`. Linux x86_64, Rust1.96.0, Compose5.5.0.
Inclusion passed2026-09-24T16:19:47Z; restart passed2026-09-24T16:24:10Z.
Initial transaction: `0x49a7f1a4aa93da00e7250fb8d314f755476d33510d47f37ae31b8595405a0bcf`.
Post-restart transaction: `0x4112f5197b8c221af1f7798822e191002d5919496837a357f5ef74711d934a90`.
Full evidence is under the experiment root's `logs/baseline-verify2/` and
`logs/baseline-restart1/`; the append-only index is `docs/autonomous/evidence/evidence.jsonl`.

## Limits and further verification

This is end-to-end inclusion, propagation and graceful-restart evidence only. It does not prove
L1 deposit/batch correctness, L1/L2 reorg behavior, L1 finality, fresh state sync, corruption recovery,
Byzantine tolerance, membership changes, performance or all RPC compatibility. The larger feature
inventory retains these gaps. Baseline nextest has23 failures and83 skips; do not call it green.
Real zk ELF/proving was excluded by the disclosed build stub. Unit/property/action/system/fuzz,
differential and fault-injection methods must be added or linked as each behavior is investigated;
this page does not claim to enumerate every existing test.
