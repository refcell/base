# Unified load testing

## Observable behavior and invariants

`base load-test CONFIG` performs the existing bounded or continuous load workload. It supports
skip-drain, drain-only, real-token recovery and seed/mnemonic rescue through the shared
`base-load-tests-cli` library. Normal load still requires CONFIG; rescue must not require CONFIG.
Root `base --chain` must not silently override YAML chain selection. The unified entrypoint reuses
Base tracing; the standalone migration wrapper initializes its own subscriber.

The old wrapper is temporary, not final product-consolidation acceptance. Remove it only after the
remaining real-token, mnemonic and interactive-TTY checks, or an explicit compatible design change.
No performance improvement is claimed by this extraction.

## Preconditions

Use only the isolated local devnet (chain84538453), public test accounts from
`etc/docker/devnet-env`, and fresh evidence directories. Never use production endpoints/funds or
print key values. The fixture runs four senders at210,000gas/s for5s; rescue uses0.01ETH funding per
sender, exceeding the existing0.001ETH L1-fee reserve. Repeated runs must be serialized when reusing
a seed so sender nonces do not collide.

## Commands and assertions

Focused parser/config checks on gene:

```bash
BASE_SUCCINCT_ELF_STUB=1 cargo nextest run --workspace --all-features \
  --exclude base-system-tests --locked --test-threads 1 \
  -E 'package(base-load-tests-cli) | test(unified_parser_) | test(rejects_top_level_chain_for_load_test) | test(terminal_support)'
```

Experiment runtime scripts are versioned in `docs/autonomous/scripts/`; copy them to the dedicated
root's `state/` directory before use. They deliberately target that experiment, not arbitrary hosts.

- `run-load-parity.sh FLAVOR MODE [SUFFIX]`: FLAVOR is baseline/unified/wrapper; MODE is load,
  skip-drain, drain or continuous. Requires a new output directory; never overwrites a run.
- `load-parity-matrix.sh` records the initial differential matrix; `load-parity-after-fix.sh`
  repeats candidate cases after the rescue parser fix.
- `load-rescue-parity.sh` funds seeded local accounts without draining, invokes rescue, asserts
  funder balance increased and repeats rescue to assert no further balance change for this fixture.
- Continuous mode sends SIGINT after8s, permits30s for cleanup, requires successful exit and JSON
  assertions. A timeout/error is not automatically retried because transactions may have landed.

For measured load, require no fatal error, positive submitted count, confirmed==submitted,
zero failures/reverts and complete receipt coverage. `LOAD_TEST_OUTPUT` must contain the expected
JSON schema. Drain-only must report successful draining; positive rescue checks actual balances,
not just an exit code. Full node verification remains required separately via the devnet feature.

## Latest evidence and limitations

Code commit450b4d7b60a74194c4196283b85d500e1392f946, tree280f6307e2c67939eee2dabc3a105b7d22337024.
Seventeen selected tests passed on gene at2026-09-24T18:27:59Z. Initial12-case differential matrix
passed17:06:20Z; corrected candidate8-case matrix passed17:27:22Z; positive rescue on both corrected
entrypoints passed17:22:27Z. Full source-built candidate node startup and restart passed separately,
including load and SIGINT on resumed candidate nodes (18:04:15Z). Exact logs are indexed under the
experiment root (`load-parity-matrix1.log`, `load-parity-after-fix1.log`, `load-rescue-parity3.log`,
`load-focused-committed1.log`, `candidate-verify1.log`, `candidate-restart1.log`).

The original baseline rescue parser also failed on missing CONFIG; this was a pre-existing bug,
not an extraction regression. The first positive rescue fixture was below the fee reserve and
was corrected without lowering the production reserve or assertions.

Unverified: positive real-token and mnemonic recovery, interactive footer/log synchronization,
full external Depot workflow, representative performance and resource bounds. The benchmark shim
requires its sibling `base` executable. Full candidate nextest remains8,635pass/25fail/83skip;
isolated passes do not erase recorded failures. This page is evolving coverage, not an exhaustive
claim about all possible verification methods.
