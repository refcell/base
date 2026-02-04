## [2026-02-04] Task: Implement local try_into_op_tx_info

### Pattern: Handling Type Mismatches from Transitive Dependencies

When reth dependencies use `features = ["op"]`, they pull in `op_alloy_consensus` types transitively. Since we've forked `op-alloy` types into `base-alloy`, we get type system conflicts even though the types are structurally identical.

**Solution Pattern**: Implement local versions of helper functions that use our forked types instead of relying on reth's versions.

### Implementation Details

The `try_into_op_tx_info` function:
1. Checks if transaction is a deposit via `tx.is_deposit()`
2. If yes, fetches receipt and extracts deposit metadata
3. Constructs `OpTransactionInfo` with standard `TransactionInfo` + optional deposit metadata

Key traits used:
- `OpTransaction` - provides `is_deposit()` method
- `DepositReceipt` - provides `as_deposit_receipt()` method
- `ReceiptProvider` - provides `receipt_by_hash()` method

### File Organization

Helper functions should be placed:
- **Before** the structs that use them (better for code readability)
- At module level (not nested in impl blocks)
- With appropriate visibility (private `fn` since only used within this module)

### Verification Approach

For type compatibility fixes:
1. LSP diagnostics must be clean (no errors)
2. Compilation must succeed (`cargo check`)
3. Verify new errors are different from original errors (proves forward progress)
