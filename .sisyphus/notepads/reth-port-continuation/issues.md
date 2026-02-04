## 2026-02-04 - payload_builder.rs Import Issues

### Problem
Task 1's payload_builder.rs has compilation errors:
1. Missing `BlockHeader` trait import (needed for `.timestamp()` method on line 44)
2. Name collision: `ChainSpec` is both a generic parameter AND an imported type (line 5)

### Errors
```
error[E0599]: no method named `timestamp` found for reference `&reth_primitives_traits::SealedHeader<<ChainSpec as EthChainSpec>::Header>`
  --> crates/execution/node/src/payload_builder.rs:44:46
```

### Fix Needed
1. Add `use alloy_consensus::BlockHeader;` to imports
2. Consider renaming generic parameter to avoid confusion OR remove unused import

Note: The `base_chainspec::ChainSpec` import (line 5) appears to only be used for the constant reference (line 57), not for types.
