# Decision Backlog

One ADR has been accepted: [`0001-staged-consolidation-and-state.md`](0001-staged-consolidation-and-state.md)
(resolves D2 in full; informs D3's target model only). Every other item below remains open for the
bounded oracle + independent reviewer council (per
`/home/refcell/base-autonomous-20260924/state/RESUME.md` step 2, max two passes) to resolve. Each
resolved item becomes a numbered ADR file in this directory (using `TEMPLATE.md`) and is marked
resolved here with a link.

| ID | Decision needed | Blocks | Status |
| --- | --- | --- | --- |
| D1 | Scope of "minimal licensed Reth execution": exact modules/crates to inline, and license compliance approach | later Reth-inlining slice(s) | Open. ADR-0001 rejected "arbitrary Reth vendoring" as the *first* slice because the closure/license inventory is incomplete; that inventory still has to happen before D1 can be decided. |
| D2 | First vertical slice selection | M4 | **Resolved by [ADR-0001](0001-staged-consolidation-and-state.md).** Integrate the existing load-testing CLI as `base load-test`, after baseline (M0) passes and required node/devnet prerequisites are met. Retire the standalone entry point only after equivalence checks; this does not by itself satisfy goal 1 (inlining) or goal 6 (binary consolidation). |
| D3 | State-ownership invariants: which single component owns durable execution state once consensus becomes stateless, and how existing state stores are retired | M6; must be settled before M5 (Engine API removal) starts | **Partially resolved by ADR-0001.** Target model: one node-owned durable state service authoritative for execution/canonical chain state, head/checkpoint metadata, derivation replay positions, sequencing terms/fences, and consensus signing journals; consensus/derivation logic owns no independently authoritative durable store. Confirmed not yet satisfied: `crates/consensus/service/src/service/node.rs:416-451` currently opens separate `SafeDB` and `CheckpointDB` stores (independently reviewer-confirmed). Concrete multi-store failure-atomicity, fencing-token format, and commit/recovery protocol are **not** resolved by ADR-0001 — see D8. |
| D4 | Engine API removal sequencing: how EL-only sync replaces the current CL/EL split without breaking L1-derived behaviors mid-migration | M5 | Open — blocked on D3/D8 (ownership invariants) being settled first. Per ADR-0001, the serialized execution mutation boundary (payload validation, fork rules, forkchoice, INVALID/VALID/SYNCING-equivalent outcomes, cancellation/backpressure/ordering) must be specified before Engine transport is removed. |
| D5 | Binary consolidation plan: disposition of each non-`base`/`basectl` binary in `bin/` (merge, remove, or reclassify as dev-only) | M7 | Open. ADR-0001 confirms standalone node/consensus wrappers are still referenced by `etc/docker/Dockerfile.rust-services`, `docker-bake.hcl`, `docker-compose.anvil-l1.yml`, and `.github/workflows/build-release.yml`, so removing them needs a wider image/release/devnet migration. Product binaries (e.g. `challenger`, `prover`) may not be reclassified as tooling without a documented purpose/users/caller disposition per binary. |
| D6 | P2P merge design: which stack (DevP2P, libp2p, or a new one) carries both tx gossip and L2 payload propagation | M8 | Open |
| D7 | Commonware integration approach: choose the primitives and declare the devnet topology, exact voter identities, membership source, admission/removal and transition policy, quorum calculation tied to the fault model, and expected safety/liveness behavior for failures and transitions. Static membership is permitted only if explicitly chosen and tested, never left unspecified. | M9, M10 | Open |
| D8 | Concrete durable-state storage/fencing atomicity protocol: multi-store failure atomicity across the durable owner's storage domains, fencing-token format, and commit/recovery protocol | M6; blocks D4/M5 | Open. Explicitly deferred by ADR-0001 ("this ADR does not falsely solve those implementation details"); needs its own ADR plus crash-point tests before Engine API removal (D4) proceeds. |

## Ground rules

- The supervisor (parent), not the advisors, decides. The oracle and an independent,
  different-model-family reviewer each produce an independent report with a recommendation,
  evidence, assumptions, risks, and confidence; they are not required to unanimously agree, and the
  supervisor may decide against either or both after weighing their critiques. This is how D2 was
  resolved: the reviewer's load-test recommendation was accepted, the oracle's broader-binary-surface
  warning was folded in as an explicit remaining-binary inventory requirement (D5), and the oracle's
  unverified current-EL-durability premise and "load-test consolidation is inherently evasive" claim
  were rejected. The resulting ADR records the decision, alternatives considered, invariants
  affected, and either real validation evidence or an explicit "pending" note.
- Do not implement against an open backlog item; wait for the accepted ADR. D2 is accepted; D1, the
  remainder of D3 (tracked as D8), D4, D5, D6, and D7 remain open — do not begin their
  implementation.
