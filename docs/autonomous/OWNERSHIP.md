# Worktree and Ownership Scheme

## Remote layout

Remote root: `/home/refcell/base-autonomous-20260924` (host `gene`, user `refcell`).

| Path | Branch | Owner | Purpose |
| --- | --- | --- | --- |
| `integration/` | `experiment/base-autonomous-20260924` | Supervisor (integrator) | Clean baseline; sole pusher; sole owner of the draft PR and private bdoc. |
| `worktrees/factory-docs/` | `experiment/factory-bootstrap-20260924` | Docs writer (this deliverable) | Documentation only: `docs/autonomous/`, `AGENTS.md`/`CLAUDE.md` pointer. No source, build, or devnet work. |
| `logs/` | not Git | Supervisor | Command output only. |
| `state/` | not Git | Supervisor | Durable runtime handoff (`RESUME.md`), rewritten across sessions. |

Additional `worktrees/<name>` directories are created per milestone (see `ROADMAP.md` M4+) and are
not listed here until they exist; check `git worktree list` in `integration/` for the live set
before assuming a path exists or is unowned.

## One-writer-per-worktree rule

Exactly one agent writes to a given worktree at a time. Before starting work in any worktree:

1. Read `state/RESUME.md` for the current owner assignment.
2. Check `git worktree list` and `git status` in that worktree for uncommitted work from another
   agent.
3. Do not duplicate an already-active owner's task; if the assignment is ambiguous, escalate rather
   than guess.

The integrator is the only agent that pushes, opens the draft PR, or publishes the bdoc. All other
agents hand off finished, committed work for the integrator to merge/push.

## Active team (per `state/RESUME.md`, workflow `1cf18246-c84b-43dc-be4f-c6f37c4ee4bb`)

| Role | Agent type | Scope |
| --- | --- | --- |
| `baseline-verification` | worker | Unchanged-baseline build/tests/devnet; sole build/devnet owner; capped at 8 build jobs, one heavy build at a time. |
| `architecture-inventory` | scout | Read-only inventory of Reth/binary/Engine API/derivation/P2P/conductor/Commonware surfaces. |
| docs writer (this deliverable) | worker | `docs/autonomous/` bootstrap docs, `worktrees/factory-docs` only. |

Harness-native agents execute locally; no recursive spawning. Verify agent status via the durable
workflow/session receipt before assuming an agent is still active — sessions do not survive a Mac
restart automatically.

## Excluded paths (never touch from any autonomous-factory agent)

`/home/refcell/dev/base`, `/home/refcell/dev/base2`, production keys/credentials, host
auth/security configuration, unrelated tmux sessions/windows, and `creo-postgres-1`.
