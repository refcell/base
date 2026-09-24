# Blockers and Next-Action Queue

Canonical live checkpoint is the absolute remote path
`/home/refcell/base-autonomous-20260924/state/RESUME.md` (non-Git, rewritten across sessions), not
a repo-relative `state/RESUME.md` — that path does not resolve consistently from every worktree.
This file is the durable, Git-tracked summary kept in sync with it. If the two disagree, the
absolute `RESUME.md` wins and this file should be updated to match.

## Current blockers

- None blocking this docs deliverable itself. `worktrees/factory-docs` is unblocked and docs-only.
  The prior docs commit (`b024152ff684f0b3e3220917df68bd0e235b6b8b`) was rejected pending correction
  (see `/home/refcell/base-autonomous-20260924/state/DECISIONS-AND-EVIDENCE.md`, 2026-09-24
  "bootstrap documentation rejected pending corrections"); this commit is that correction.
- Baseline build and source-built images pass. Nextest: 8,620 passed, 23 failed, 83 skipped; not green. Twenty failures are MinIO image pull401, two crash-backtrace failures reproduce, one fixed8080 RPC test conflicts with unrelated Java. Full devnet inclusion, validator receipt agreement, graceful restart/persistence and bounded canonical-head/finality observations pass. Adversarial finality, reorgs and real zk proving remain unverified; see ledger and ADR-0002.
- M1 (architecture inventory) is in progress but not yet complete; several rows in
  `BEHAVIOR_INVENTORY.md` are marked not-yet-inventoried pending its output.
- Accepted directions: [ADR-0001](adr/0001-staged-consolidation-and-state.md) and [ADR-0002 baseline disposition](adr/0002-baseline-failure-disposition.md).
  (D2, and the target durable-authority model informing D3). D1, the remainder of D3 (concrete
  storage/fencing atomicity, tracked as D8), and D4–D7 remain open in `adr/DECISIONS.md` and block
  their respective milestones (see `ROADMAP.md`). Do not implement against an open item.

## Exact next-action queue

1. Finish baseline restart/persistence observations and append exact-tree evidence. Preserve and investigate all 23 baseline failures; do not weaken checks. Supervisor owns all remote execution.
2. Council pass 1 is complete for D2: [ADR-0001](adr/0001-staged-consolidation-and-state.md) is
   accepted; no second pass was needed for it. D1, D3's remaining atomicity/fencing design (D8), and
   D4–D7 remain open — do not begin their implementation ahead of an accepted ADR.
3. Evidence validator corrections passed 17 tests on gene and independent review. Commit the reviewed bootstrap candidate, run its exact-tree full devnet cycle, and open the draft PR only on the fork.
4. Proceed with isolated M4 development under [ADR-0002](adr/0002-baseline-failure-disposition.md), which records the baseline failures without calling them passes. Keep the standalone load-test wrapper only until differential validation.
5. Before M4 integration require focused tests, full exact-candidate devnet/verify-base, restart/interruption/recovery evidence and independent review. Investigate every new failure.
6. Integrator creates one draft PR once an appropriate bootstrap commit exists:
   `gh pr create --repo refcell/base --base main --head refcell:experiment/base-autonomous-20260924
   --draft`, conventional title, Toshi-generated attribution, no upstream PR.
7. Repeat the milestone loop (tests → fresh full devnet/`verify-base` → restart/fault checks →
   independent different-family review → validated commit/push → feature-map/PR/bdoc update) for
   each milestone from M4 onward. Resolve D3/D8 (ownership invariants) before starting D4/M5
   (Engine API removal) — ownership comes before Engine changes, not after.

## Recovery notes

- Before resuming any step, read `/home/refcell/base-autonomous-20260924/state/RESUME.md`, remote
  Git/worktree status (`git worktree list`, `git status` in each), tmux windows/logs, and
  active-agent status; do not assume prior evidence still applies after source changes.
- The Mac keep-awake task (`base-factory-keep-awake`) is not durable supervision; a restarted
  supervisor must re-establish it if actively running, and must not claim indefinite execution after
  the local session ends.
- Stop only for external/authorization/safety blockers; checkpoint the exact next action and
  disclose all unverified goals rather than assuming completion.
