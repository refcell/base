## [2026-02-04] Task: Implement local try_into_op_tx_info

### Decision: Local Function vs Trait Implementation

**Chosen**: Implement as a module-level free function

**Alternatives Considered**:
1. Trait implementation on `OpTxInfoMapper`
2. Extension trait
3. Free function (chosen)

**Rationale**:
- Matches reth's pattern (they use a free function too)
- Simpler than trait machinery
- Only used in one place (the `TxInfoMapper` impl)
- No need for wider reusability

### Decision: Import Structure

**Chosen**: Import both `OpDepositInfo` and `OpTransactionInfo` together in one use statement

```rust
use base_alloy_consensus::{
    transaction::{OpDepositInfo, OpTransactionInfo},
    OpTransaction,
};
```

**Rationale**:
- Groups related types together
- Clearer that both come from the same submodule
- Follows Rust conventions for multiple imports from same path
