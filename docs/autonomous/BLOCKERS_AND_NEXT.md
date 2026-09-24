# Blockers and next actions

Canonical live checkpoint: `/home/refcell/base-autonomous-20260924/state/RESUME.md`, mirrored in the
Mac supervisor's `factory-control/RESUME.md`. Recheck Git/worktree status, actual tmux commands and
exclusive ownership before resuming; the remote process state and source identity outrank stale
status prose. The Mac runs agents; gene runs source/build/test/devnet commands.

## Current state

- Code450b4d7b60a74194c4196283b85d500e1392f946 is integrated on the experiment branch and fork-only
  [draft PR #1](https://github.com/refcell/base/pull/1). `base load-test` is real functionality; the
  thin standalone wrapper remains temporary, with removal criteria in its README and feature map.
- Fresh source-built candidate devnet, runtime image identity, receipt/block/validator agreement,
  graceful stop/start persistence, load/SIGINT checks and17focused tests pass. Independent code
  review found no correctness blocker; independent failure triage found no extraction-caused defect.
- Full suites are NOT green. Baseline:8620pass/23fail/83skip. Candidate:8635pass/25fail/83skip.
  Corrected baseline breakdown is19image401 failures,2SIGSEGV tests,1RPC port collision,1metering
  teardown abort. The earlier20image classification is corrected in append-only event0007.
- Candidate adds an intermittently failing preexisting pacing fixture and two different metering
  teardown abort cases while the original metering abort case passes. A baseline artifact reproduced
  the pacing failure; affected source/features/dependency edges are unchanged. Isolated passes do
  not erase full-run failures or prove statistical equivalence. Preserve all checks and evidence.
- Fork review CI cannot start because `BaseRunnerGroup` is unavailable. Do not provision paid
  infrastructure, copy credentials or weaken checks to manufacture green. Local review continues.
- Real zk proving is not covered by `BASE_SUCCINCT_ELF_STUB=1`. Real-token/mnemonic/TTY migration
  checks, broad sync/reorg/corruption/finality/adversarial/performance coverage remain incomplete.

## Next queue

1. Fix the verified pacing fixture mismatch in its isolated worktree: both block-time values should
   be200ms as documented, not2s. Keep the210ms deadline and `safety_cycles >= 2` assertion. Validate
   and independently review; this is not permission to weaken a check.
2. Finish positive real-token/mnemonic and interactive-TTY checks for M4-B; document shared logging
   behavior and benchmark shim sibling dependency. Then remove the standalone migration product.
3. Continue the code/dependency/license inventory and accepted ADR roadmap. Resolve concrete
   durable ownership, fencing and recovery before Engine/derivation changes; no hidden CL-sync or
   transport fallback may remain at final acceptance.
4. Every code milestone needs focused tests, exact-candidate full devnet, relevant fault/recovery
   checks, independent challenge and traceable signed commits. Supervisor alone integrates/pushes,
   updates the draft PR, feature map, evidence ledger and private bdoc. Never merge automatically.

All end-state goals remain open; neither the bootstrap nor one CLI slice closes the mission.
