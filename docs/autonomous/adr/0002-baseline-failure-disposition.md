# ADR-0002: preserve baseline failures while developing an isolated CLI slice

Status: accepted for M4 development only; no blanket test waiver or completion claim.
Date: 2026-09-24. Decision owner: supervisor/integrator.

## Evidence

At unchanged commit `539605ce58aaf02fe5c0382fcd9139c1ee4207e9`, tree
`1da623a8f06d439061be2c86033970cadca73b3f`:

- Workspace/all-target build passes after required contract generation and Go PATH setup.
- Workspace/all-feature nextest (excluding the separate system-test package) runs 8,643 tests:
  8,620 pass, 23 fail, 83 skip. Nineteen failures cannot pull the pinned public MinIO image (401);
  two crash-backtrace tests fail reproducibly, one metering test aborts during teardown, and one RPC
  test binds port 8080 occupied by unrelated Java. Leave that process untouched. Do not remove,
  disable, or soften these tests. Logs are indexed in `../evidence/evidence.jsonl`.
- Source-built full Compose devnet runs with all long-lived services present. Sequencer transaction
  `0x49a7f1a4aa93da00e7250fb8d314f755476d33510d47f37ae31b8595405a0bcf` has successful receipt,
  expected sender/recipient, matching block hash and block transaction membership. Independent
  validator returns the same transaction/block receipt. This proves inclusion and propagation,
  not L1 finality or complete sync correctness.
- Real zk proving is not covered: build/test use `BASE_SUCCINCT_ELF_STUB=1`. Safe/finalized progression,
  all L1 protocol behaviors and crash/corruption recovery require additional evidence.

## Decision

Proceed with M4 **isolated implementation**, which changes CLI organization rather than consensus,
execution, derivation or storage semantics. The user requires an exercised unchanged baseline and
honest failure classification, not deletion of failing checks or a fabricated universally green
baseline. Earlier bootstrap wording requiring all M0 tests to pass before any development was an
agent-added constraint, not a user requirement; replace that wording with this explicit disposition.

Keep the old standalone CLI as a temporary differential wrapper until parity is established. Do
not remove its product entry point before equivalent load/drain/recovery/shutdown tests pass on the
local devnet. Final binary acceptance still forbids retaining it.

Integration requires focused tests, full exact-candidate devnet/verify-base, restart and relevant
CLI interruption/recovery evidence, independent review, and comparison against the baseline failure
set. Any additional failure or changed failure mechanism must be investigated. Preexisting failures
remain failed/blocked in every report, never relabeled pass. The unrelated failures remain backlog
items; real-proof verification remains a separately disclosed gap.

## Alternatives and limits

Blocking all isolated work on a public registry outage does not improve evidence for the CLI slice.
Ignoring/removing checks would weaken safety and is rejected. Changing host services or registry
credentials is not authorized by this decision. This exception does not license high-risk
Engine/state/derivation/Commonware changes before their own invariants and behavioral baselines exist.

## Evidence correction and follow-up
The initial classification counted 20 image failures and missed a SIGABRT row. Event0007 records
the corrected breakdown above; the total and failed status did not change. Candidate450b4d has
25 failures, including the preexisting pacing fixture mismatch and differing metering teardown
abort cases. Original baseline pacing also reproduced intermittently; affected source/features
are unchanged. Isolated reruns and independent triage support this bounded disposition, not a
green-suite or statistical-equivalence claim. Fix the pacing fixture separately without changing
its deadline/assertion; broader metering lifetime recovery remains open.
