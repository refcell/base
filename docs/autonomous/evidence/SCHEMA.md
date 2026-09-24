# Evidence Log Schema

`evidence.jsonl` in this directory is an **append-only** log: one JSON object per line, newest
entries added at the end, existing lines never edited or deleted. Any agent that runs a real check
(build, test, devnet, transaction, fault, review) appends a row before reporting a result.

## Fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `id` | string | yes | Stable identifier, e.g. `evt-0001`, monotonically increasing within the file. |
| `timestamp_utc` | string (ISO 8601) | yes | When the check was run. |
| `actor` | string | yes | Agent/role that ran the check, e.g. `baseline-verification`, `architecture-inventory`. |
| `worktree` | string | yes | Repo-relative worktree path the check ran against, e.g. `integration`, `worktrees/factory-docs`. |
| `commit_sha` | string | yes | Exact commit hash checked (full 40-char SHA; use `0000...0` only for `example` rows). |
| `category` | string | yes | One of: `build`, `test`, `devnet`, `transaction`, `fault`, `review`, `other`. |
| `command` | string | yes | The exact command executed (or a precise description if interactive). |
| `result` | string | yes | One of: `pass`, `fail`, `blocked`, `skipped`, `unknown`. `blocked`, `skipped`, and `unknown` are never equivalent to `pass`. |
| `evidence_refs` | array of strings | yes | Paths, transaction hashes, block numbers/hashes, log locations, or review IDs that back the result. Empty array only if `result` is `blocked`/`skipped`/`unknown` with nothing to reference yet. |
| `notes` | string | yes | Freeform context; state explicitly if something is a partial/incomplete check. |
| `example` | boolean | no | `true` only for illustrative rows seeded by documentation (like the schema example below); omit or set `false` for every real entry. Tooling and reviewers must ignore `example: true` rows when computing real milestone status. |

## Invariants

- Never record `result: "pass"` unless the command was actually executed this experiment and its
  output was inspected, not assumed.
- One line per check. Do not batch multiple checks into a single row's `evidence_refs` unless they
  are genuinely one atomic verification (e.g., a single `verify-base` run producing one receipt).
- Keep `commit_sha` exact; do not reuse a prior row's evidence for a new commit.

## Example (illustrative only — not a real record)

```json
{"id": "evt-0000", "example": true, "timestamp_utc": "1970-01-01T00:00:00Z", "actor": "docs-worker-illustrative", "worktree": "worktrees/factory-docs", "commit_sha": "0000000000000000000000000000000000000000", "category": "other", "command": "echo example only - not executed", "result": "unknown", "evidence_refs": [], "notes": "Illustrative schema example only. Not a real check. Real entries must omit `example` or set it to false."}
```

This exact object is also the seed row in `evidence.jsonl` so the file is non-empty and
self-documenting without claiming any real check has passed.
