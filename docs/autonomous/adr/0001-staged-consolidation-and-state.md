# ADR-0001: staged product consolidation and durable state authority

Status: accepted experimental direction; implementation and behavioral proof pending.
Date: 2026-09-24.
Source inspected: 539605ce58aaf02fe5c0382fcd9139c1ee4207e9 / tree 1da623a8f06d439061be2c86033970cadca73b3f.
Decision owner: local supervisor/integrator. No production adoption authorized.
Deciders: oracle (fork, anthropic/claude-opus-4-8) + reviewer (fresh, cbhq-openai/gpt-5.6-sol);
final decision by the supervisor/integrator, not by advisor unanimity (see "Council record and
disposition" below and `adr/DECISIONS.md`'s ground rules).
Provenance: authored by the supervisor after the council pass described below; transported
verbatim into this docs worktree from `factory-control/ADR-0001.md` by the docs-correction worker.
Resolves: `adr/DECISIONS.md` D2 in full; informs D3 (target model only — concrete storage/fencing
atomicity is explicitly deferred to D8, not resolved here).

## Decision and alternatives

After exercising the unchanged baseline and resolving required node/devnet prerequisites, first integrate the real load-testing CLI into `base load-test`. Reuse the existing load-tests library and preserve load, drain, rescue/recovery, configuration, output and shutdown behavior. Retire its standalone product entry point only after equivalence checks; migrate all callers. Preserve unrelated test orchestration temporarily, but audit `base-bench` separately rather than claiming that moving one binary completes binary consolidation.

Rejected as FIRST slice, not rejected forever:
- Retire standalone node/consensus wrappers: wrappers are small, but `etc/docker/Dockerfile.rust-services:87-92,121-122,225,245-246`, docker-bake.hcl:81,114-117, docker-compose.anvil-l1.yml:111 and `.github/workflows/build-release.yml` still reference these products. Requires wider image/release/devnet migration first.
- Direct Engine adapter: valuable next, but BaseEngineApi validation, task ordering/error/reset semantics and durable authority must be mapped first.
- Arbitrary Reth vendoring: closure/license inventory incomplete; no proof of a bounded minimal slice yet.
- Conductor metric-only change: not substantive progress toward the requested architecture.

The first slice is not full goal-5 acceptance. Product binaries such as challenger/prover or infrastructure services may not be declared tooling just because they sit outside the main node. Every executable needs purpose, users, artifact/release/build callers and a remove/merge/legitimate-test-tool disposition.

## Durable authority interpretation

Target: one node-owned durable state service is authoritative for execution/canonical chain state, head/checkpoint metadata, derivation replay positions, sequencing terms/fences and consensus signing journals. It may use appropriately separated storage domains; one owner does NOT mean pretending MDBX and a future journal are one atomic database without a recovery protocol.

Consensus/derivation logic evaluates inputs and proposes effects; it owns no independently authoritative durable store. Volatile actor caches are allowed only with specified reconstruction. Necessary Commonware vote/lock journals MUST persist before vote/signature release through the durable owner, so stateless consensus does not mean unsafe memory-only BFT. No bypass/reconstruction from untrusted peer claims may authorize repeat signing. The actor implementations may maintain volatile protocol state, but durable authority is outside them.

Current source does NOT satisfy this: `crates/consensus/service/src/service/node.rs:416-451` opens separate SafeDB and CheckpointDB. Do not assert existing execution DB persists every safe/finalized/derivation invariant. Inventory these stores and prove restart behavior before migration.

Before removing Engine transport, specify and implement a serialized execution mutation boundary preserving payload validation, fork rules, forkchoice constraints, INVALID/VALID/SYNCING-equivalent outcomes, cancellation, backpressure and ordered task/reset behavior. Multi-store failure atomicity, fencing token format and commit/recovery protocol require subsequent concrete ADRs and crash-point tests. This ADR does not falsely solve those implementation details.

EL-only sync means execution network history/state acquisition is the only supported sync path. It does not permit peers to declare canonical safety: deposits, calldata/blobs/channels/batches, system configuration, sequencing windows, origin validation, L1 reorgs and finality remain deterministic protocol checks. Replace the old architecture, not these behaviors. Unsafe L2 ordering/availability, sequencing quorum commitment, L1-derived safety and L1 finality remain distinct.

## First slice invariants and validation

- `base load-test` accepts equivalent supported config/flags and produces equivalent observable exit/output behavior. Avoid double tracing/runtime initialization when embedded.
- No default traffic to non-local targets in experiments; public devnet accounts only. Recovery/rescue tests never touch real funds.
- Audit and migrate `.depot/workflows/benchmark.yml:65-66`, `crates/infra/load-tests/Justfile`, docs, package/default-member references. Keep genuine `etc/systems` orchestration distinct and explicitly pending audit.
- Test parser/config precedence, `LOAD_TEST_OUTPUT`, invalid config failure, output artifact shape, signals/cancellation/drain/recovery. Preserve work/task cleanup on early failures.
- Run full fresh exact-candidate devnet plus verify-base receipt/block assertions; exercise load against that devnet and verify inclusion/receipts; test interruption and restart/recovery paths. Recheck exact source tree and runtime override identity.
- Independent review before integration/push. Baseline failures and proving-stub limitations remain separate evidence, never an invented green gate.

## Council record and disposition

Pass 1 workflow 31e08b7a-d71b-4c6f-96d3-9fafafc0aea8; fallback oracle forked/context-aware (anthropic/claude-opus-4-8, run afe64664-4894-4b78-bc65-3765176293bf), reviewer fresh (cbhq-openai/gpt-5.6-sol, run cefce020-54c5-4503-97f4-db9418304a2b). No council-* profiles available. Two independent reports completed, three major agreements (baseline first, ownership before Engine deletion, preserve L1-derived semantics), one first-slice dispute.

Parent settled first-slice dispute using direct Docker/release caller evidence above; no second pass needed. Accepted reviewer load-test recommendation, oracle warning about broader binary surface, both ownership/order critiques. Rejected oracle's unverified current-EL-durability premise and classification of load-test consolidation as inherently evasive; explicit remaining-binary inventory prevents claiming premature completion. Concrete state-ownership choices remain the parent's responsibility, not advisor unanimity.

Confidence: medium, static evidence only. Change direction if baseline shows load tooling cannot exercise local devnet, equivalence requires unbounded changes, or a smaller real execution slice has better evidenced safety. Upstream adoption blocked pending full compatibility, security/performance, license/dependency, fault/recovery and interoperability evidence; this experiment changes no live protocol.
