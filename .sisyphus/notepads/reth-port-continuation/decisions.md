## 2026-02-04 - Decision: Use Cargo Patch for Type Aliasing

### Context
- Task 2 revealed orphan rule prevents implementing `FromRecoveredTx`/`FromTxWithEncoded` for our forked types
- Upstream `alloy-evm` has these implementations for `op_alloy_consensus::OpTxEnvelope`
- We use `base_alloy_consensus::OpTxEnvelope` (forked)
- Compilation errors show: "`op_revm::OpTransaction<TxEnv>` implements `FromRecoveredTx<op_alloy_consensus::OpTxEnvelope>`" but NOT our type

### Viable Solutions Analyzed
1. **Cargo Patch** ✅ SELECTED
   - Add `[patch.crates-io]` to replace `op-alloy-consensus` with our `base-alloy-consensus`
   - Pros: Simple, allows upstream implementations to work, no code changes needed
   - Cons: Semantic mismatch (our types have different intentions than upstream)
   
2. **Fork alloy-evm**
   - Fork and add implementations for our types
   - Pros: Full control
   - Cons: Maintenance burden, dependency management complexity
   
3. **Wrapper Types**
   - Create newtypes in base-evm that wrap OpTransaction
   - Pros: Satisfies orphan rule
   - Cons: Adds indirection, complexity throughout codebase
   
4. **Upstream PR**
   - Submit generic implementations to alloy-evm
   - Pros: Proper long-term solution
   - Cons: Not under our control, timeline uncertainty

### Decision
Use Cargo patch to alias `op-alloy-consensus` to our `base-alloy-consensus`. This is the pragmatic solution that:
- Unblocks development immediately
- Requires minimal changes (only Cargo.toml)
- Can be revisited later if needed

### Implementation
Add to root Cargo.toml:
```toml
[patch.crates-io]
op-alloy-consensus = { path = "crates/alloy/consensus" }
```

This tells Cargo to replace all usages of `op-alloy-consensus` from crates.io with our local `base-alloy-consensus` crate.

## 2026-02-04 - Final Decision: Implement Traits in base-alloy-consensus

### Why Other Approaches Failed
1. **Cargo patch**: Doesn't work with path dependencies
2. **Re-export upstream types**: Would break custom OpTxEnvelope implementation with derive macros
3. **Rename crate**: Too invasive, breaks semantic intent

### Chosen Approach: Add trait impls to base-alloy-consensus
Even though `FromRecoveredTx` and `FromTxWithEncoded` are from `alloy_evm` (external), we CAN implement them in `base-alloy-consensus` because:
- The trait is external BUT
- At least one type parameter (`OpTxEnvelope`, `TxDeposit`) is LOCAL to base-alloy-consensus
- This satisfies the orphan rule's "local type in uncovered position" requirement

### Implementation Plan
Add to `crates/alloy/consensus/src/transaction/evm_compat.rs` (new file):
- `impl FromRecoveredTx<OpTxEnvelope> for TxEnv`
- `impl FromRecoveredTx<TxDeposit> for TxEnv`
- `impl FromTxWithEncoded<OpTxEnvelope> for TxEnv`
- `impl FromRecoveredTx<OpTxEnvelope> for OpTransaction<TxEnv>`
- `impl FromTxWithEncoded<OpTxEnvelope> for OpTransaction<TxEnv>`
- `impl FromRecoveredTx<TxDeposit> for OpTransaction<TxEnv>`
- `impl FromTxWithEncoded<TxDeposit> for OpTransaction<TxEnv>`

Gate behind `evm-compat` feature.
