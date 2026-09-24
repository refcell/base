# Evidence Log Schema

`evidence.jsonl` in this directory is an **append-only** log: one JSON object per line, newest
entries added at the end, existing lines never edited or deleted.

## Ownership: only the current docs-worktree owner appends

`worktrees/factory-docs` has exactly one writer at a time (see `../OWNERSHIP.md`). Only that owner
(in practice, the integrator during integration, or this deliverable's sole writer while it is
active) appends rows here. An agent that ran a real check but works in a *different* worktree
(`baseline-verification`, `architecture-inventory`, a code-slice worker, a reviewer, ...) does not
have write access to this worktree and must not append directly, even though it did the work.
Instead it records the fields below in its own report/handoff, and the current docs-worktree owner
appends the row. Every worker appending directly to a file it does not own would itself violate the
one-writer-per-worktree invariant; this file's ownership must therefore be a single named role, not
"any agent that runs a check."

## Fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `id` | string | yes | Stable identifier with exact syntax `evt-NNNN` (four decimal digits), unique and numerically strictly increasing within the file. |
| `timestamp_utc` | string (ISO 8601 UTC) | yes | When the check was run, as a valid UTC timestamp ending in `Z`, e.g. `2026-09-24T12:00:00Z`. |
| `actor` | string | yes | Agent/role that ran the check, e.g. `baseline-verification`, `architecture-inventory`. |
| `worktree` | string | yes | Repo-relative worktree path the check ran against, e.g. `integration`, `worktrees/factory-docs`. |
| `commit_sha` | string | yes | Exact commit hash checked (full 40-char SHA; use `0000...0` only for `example` rows). |
| `tree_sha` | string | yes for non-`example` rows | Exact working-tree identity at check time: a full 40-hex tree hash if clean, or `"<commit_sha>+dirty:<64-hex SHA-256 of git diff>"` if dirty. The commit prefix of a dirty identity must equal `commit_sha`. A dirty local override must be distinguishable from the committed tree, not silently folded into `commit_sha`. |
| `environment` | string | yes for non-`example` rows | Host/toolchain identity the check ran under, e.g. `gene / refcell, rustc 1.96.0, docker compose 5.5.0`. A check run under an unrecorded environment cannot be reproduced or trusted. |
| `runtime_overrides` | array of nonblank strings | yes for non-`example` rows | Any non-default runtime configuration used for this check (e.g. `COMPOSE_PROJECT_NAME=base-autonomous-20260924`, loopback-only port remaps, env var overrides). Empty array only if genuinely no overrides were used. |
| `category` | string | yes | One of: `build`, `test`, `devnet`, `transaction`, `fault`, `review`, `other`. |
| `command` | string | yes | The exact command executed (or a precise description if interactive). |
| `result` | string | yes | One of: `pass`, `fail`, `blocked`, `skipped`, `unknown`. `blocked`, `skipped`, and `unknown` are never equivalent to `pass`. |
| `evidence_refs` | array of nonblank strings | yes | Paths, transaction hashes, block numbers/hashes, log locations, or review IDs that back the result. Required non-empty for any `result: "pass"` row — a pass with no evidence reference is invalid. Empty array only if `result` is `blocked`/`skipped`/`unknown` with nothing to reference yet. |
| `notes` | nonblank string | yes | Freeform context; state explicitly if something is a partial/incomplete check. |
| `example` | boolean | no | The JSON boolean `true` is reserved for the historical first-line `evt-0000` seed only; omit or use boolean `false` for every real entry. Strings and other truthy values are invalid. Tooling and reviewers ignore the seed when computing real milestone status and never count it as a `pass`. |

`tree_sha`/`environment`/`runtime_overrides` are new fields added by this correction. The
already-committed seed row predates them and is grandfathered because it is `example: true` (not a
real record); every row appended from now on must include all fields above, including these three.

All required textual fields are strings containing at least one non-whitespace character. Array
elements must likewise be nonblank strings; JSON coercion or truthiness never substitutes for the declared
type. `commit_sha` is exactly 40 hexadecimal characters.
## Invariants

- Never record `result: "pass"` unless the command was actually executed this experiment and its
  output was inspected, not assumed.
- A `pass` row without at least one `evidence_refs` entry, or without `commit_sha`, `tree_sha`, or
  `environment` populated, is invalid and must be rejected — see `validate_evidence.py`, which
  enforces this mechanically.
- The historical first-line `evt-0000` seed is the only row that may set `example` to boolean `true`,
  the only row exempt from `tree_sha`/`environment`/`runtime_overrides`, and never counts as a pass.
  A later row, a different ID, or a nonboolean truthy value receives no exemption.
- Event IDs use `evt-NNNN`, are unique, and increase strictly by their four-digit numeric component.
- One line per check. Do not batch multiple checks into a single row's `evidence_refs` unless they
  are genuinely one atomic verification (e.g., a single `verify-base` run producing one receipt).
- Keep `commit_sha`/`tree_sha` exact; do not reuse a prior row's evidence for a new commit or tree.
- Do not append another example (or otherwise fabricated/placeholder) row to this file, ever. Future
  illustrative examples belong only in this schema document, never in the live log.

## Example (illustrative only — not a real record, and never to be duplicated into the log)

```json
{"id": "evt-0000", "example": true, "timestamp_utc": "1970-01-01T00:00:00Z", "actor": "docs-worker-illustrative", "worktree": "worktrees/factory-docs", "commit_sha": "0000000000000000000000000000000000000000", "tree_sha": "0000000000000000000000000000000000000000", "environment": "illustrative only - no real host", "runtime_overrides": [], "category": "other", "command": "echo example only - not executed", "result": "unknown", "evidence_refs": [], "notes": "Illustrative schema example only. Not a real check. Real entries must omit `example` or set it to false."}
```

This shows the full field set, including `tree_sha`/`environment`/`runtime_overrides` added by this
correction. The seed row already committed in `evidence.jsonl` predates those three fields (see the
note above) and is not edited retroactively, since this file is append-only and that row is clearly
`example: true`.
