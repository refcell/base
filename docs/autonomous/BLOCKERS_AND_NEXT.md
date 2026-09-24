# Blockers and Next-Action Queue

Canonical live checkpoint is `state/RESUME.md` (non-Git, rewritten across sessions); this file is
the durable, Git-tracked summary kept in sync with it. If the two disagree, `state/RESUME.md` wins
and this file should be updated to match.

## Current blockers

- None blocking this docs deliverable. `worktrees/factory-docs` is unblocked and docs-only.
- M0 (baseline verification) has not run this experiment: no build, test, devnet, transaction, or
  fault-test evidence exists yet. Nothing downstream of M0 may claim a pass.
- M1 (architecture inventory) is in progress but not yet complete; several rows in
  `BEHAVIOR_INVENTORY.md` are marked unclassified/unconfirmed pending its output.
- No ADR has been accepted; all `adr/DECISIONS.md` items are open and block their respective
  milestones (see `ROADMAP.md`).

## Exact next-action queue (per `state/RESUME.md` gate status)

1. Consume the baseline-verification and architecture-inventory workflow outputs; resolve any
   needed local-only devnet port/volume overrides without weakening behavior or touching unrelated
   host settings.
2. Convene the bounded oracle (fork) + independent, different-model-family reviewer on the first
   vertical slice and state-ownership invariants (max two passes); record decisions as ADRs in
   `adr/`, resolving items in `adr/DECISIONS.md`.
3. This docs deliverable (charter, roadmap, ownership, ADR scaffold, behavior inventory, evidence
   schema) — in progress via this commit; independent review still pending.
4. Baseline build/tests/full devnet must actually run and pass before any architectural source
   change. A prebuilt binary is not a baseline. Record the run in `evidence/evidence.jsonl`.
5. Integrator creates one draft PR once an appropriate bootstrap commit exists:
   `gh pr create --repo refcell/base --base main --head refcell:experiment/base-autonomous-20260924
   --draft`, conventional title, Toshi-generated attribution, no upstream PR.
6. First code slice in a new remote worker worktree: focused tests, candidate integration, full
   fresh exact-tree devnet/`verify-base`, restart and relevant fault checks, independent
   different-family review, validated commit/push, feature-map/PR/bdoc updates — then repeat per
   milestone.

## Recovery notes

- Before resuming any step, read `state/RESUME.md`, remote Git/worktree status
  (`git worktree list`, `git status` in each), tmux windows/logs, and active-agent status; do not
  assume prior evidence still applies after source changes.
- The Mac keep-awake task (`base-factory-keep-awake`) is not durable supervision; a restarted
  supervisor must re-establish it if actively running, and must not claim indefinite execution after
  the local session ends.
- Stop only for external/authorization/safety blockers; checkpoint the exact next action and
  disclose all unverified goals rather than assuming completion.
