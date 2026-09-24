# Worktree and Ownership Scheme

## Remote layout

Remote root: `/home/refcell/base-autonomous-20260924` (host `gene`, user `refcell`).

| Path | Branch | Owner | Purpose |
| --- | --- | --- | --- |
| `integration/` | `experiment/base-autonomous-20260924` | Supervisor (integrator) | Clean baseline; sole pusher; sole owner of the draft PR and private bdoc. |
| `worktrees/factory-docs/` | `experiment/factory-bootstrap-20260924` | Docs writer (this deliverable) | Documentation only: `docs/autonomous/`, `AGENTS.md`/`CLAUDE.md` pointer. No source, build, or devnet work. |
| `logs/` | not Git | Supervisor | Command output only. |
| `state/` | not Git | Supervisor | Durable runtime handoff (`RESUME.md`, absolute path `/home/refcell/base-autonomous-20260924/state/RESUME.md`), rewritten across sessions. |

Additional `worktrees/<name>` directories are created per milestone (see `ROADMAP.md` M4+) and are
not listed here until they exist; check `git worktree list` in `integration/` for the live set
before assuming a path exists or is unowned.

## One-writer-per-worktree rule

Exactly one agent writes to a given worktree at a time. Before starting work in any worktree:

1. Read `/home/refcell/base-autonomous-20260924/state/RESUME.md` for the current owner assignment.
2. Check `git worktree list` and `git status` in that worktree for uncommitted work from another
   agent.
3. Do not duplicate an already-active owner's task; if the assignment is ambiguous, escalate rather
   than guess.

The integrator is the only agent that pushes, opens the draft PR, or publishes the bdoc. All other
agents hand off finished, committed work for the integrator to merge/push.

## Evidence log ownership (single publisher)

`docs/autonomous/evidence/evidence.jsonl` lives inside this docs worktree, which itself has exactly
one writer at a time (see above). Only the current owner of `worktrees/factory-docs` (in practice,
the integrator when integrating, or this deliverable's sole writer while it is active) appends rows
to that file. An agent working in a *different* worktree (e.g. `baseline-verification`,
`architecture-inventory`, or a future code-slice worker) does not have write access to
`worktrees/factory-docs` and must not attempt to append there directly — even if it ran the check
being recorded. Instead it records the check's fields (worktree, commit/tree identity, environment,
command, result, evidence refs — see `evidence/SCHEMA.md`) in its own report/handoff, and the
integrator appends the row as part of integration. Letting every worker append directly to a file in
a worktree it does not own would itself violate the one-writer-per-worktree rule above; that is why
the log's own ownership must be explicit rather than "any agent that runs a check."

## Active roles

| Role | Agent type | Scope |
| --- | --- | --- |
| Supervisor/integrator | local parent | Sole SSH, build/test/devnet executor, committer/pusher and publisher. Existing `gene` alias only; never inspect/print alias expansion, credentials or shell environment. |
| Implementation author | `base-factory-writer` | File-tool-only author of assigned transport payload mapped to one remote worktree. No shell, SSH, networking or credential handling. Parent applies changes and returns real verification results. |
| Independent reviewer | `base-factory-reviewer` | File-tool-only fresh-context review of supplied source/evidence snapshots; no shell, SSH or credential handling. |

Harness-native agents execute locally; no recursive spawning. Verify agent status via the durable
workflow/session receipt before assuming an agent is still active — sessions do not survive a Mac
restart automatically.

## Excluded paths (never touch from any autonomous-factory agent)

`/home/refcell/dev/base`, `/home/refcell/dev/base2`, production keys/credentials, host
auth/security configuration, unrelated tmux sessions/windows, and `creo-postgres-1`.
