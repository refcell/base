# Decision Backlog

No ADRs have been accepted yet. This is the open backlog the bounded oracle + independent reviewer
council (per `state/RESUME.md` step 2, max two passes) must resolve. Each item becomes an ADR
(using `TEMPLATE.md`) when decided; move it from this table to a numbered file in this directory and
mark it resolved here.

| ID | Decision needed | Blocks | Status |
| --- | --- | --- | --- |
| D1 | Scope of "minimal licensed Reth execution": exact modules/crates to inline, and license compliance approach | M4 | Open |
| D2 | First vertical slice selection: which concrete execution-path change proves the inline-Reth direction with least risk | M4 | Open |
| D3 | State-ownership invariants: which single component owns durable execution state once consensus becomes stateless, and how existing state stores are retired | M6 | Open |
| D4 | Engine API removal sequencing: how EL-only sync replaces the current CL/EL split without breaking L1-derived behaviors mid-migration | M5 | Open |
| D5 | Binary consolidation plan: disposition of each non-`base`/`basectl` binary in `bin/` (merge, remove, or reclassify as dev-only) | M7 | Open |
| D6 | P2P merge design: which stack (DevP2P, libp2p, or a new one) carries both tx gossip and L2 payload propagation | M8 | Open |
| D7 | Commonware integration approach: which Commonware primitives back the conductor actor and multi-node sequencing | M9, M10 | Open |

## Ground rules

- A decision is not accepted until both the oracle and an independent, different-model-family
  reviewer agree, and the resulting ADR records alternatives, invariants affected, and either real
  validation evidence or an explicit "pending" note.
- Do not implement against an open backlog item; wait for the ADR.
