//! EVM compatibility trait implementations for OP Stack transaction types.
//!
//! This module provides implementations of `FromRecoveredTx` and `FromTxWithEncoded` traits
//! from `alloy-evm` for `base-alloy-consensus` types, enabling seamless integration with
//! EVM execution environments like `revm`.

use alloy_eips::{Encodable2718, Typed2718};
use alloy_evm::{FromRecoveredTx, FromTxWithEncoded};
use alloy_primitives::{Address, Bytes};
use op_revm::{OpTransaction, transaction::deposit::DepositTransactionParts};
use revm::context::TxEnv;

use crate::{OpTxEnvelope, TxDeposit};

/// Implements `FromRecoveredTx<OpTxEnvelope>` for `TxEnv`
///
/// Converts an `OpTxEnvelope` into a `TxEnv` by matching on the envelope variant
/// and delegating to the appropriate transaction type's implementation.
impl FromRecoveredTx<OpTxEnvelope> for TxEnv {
    fn from_recovered_tx(tx: &OpTxEnvelope, caller: Address) -> Self {
        match tx {
            OpTxEnvelope::Legacy(tx) => Self::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip1559(tx) => Self::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip2930(tx) => Self::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip7702(tx) => Self::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Deposit(tx) => Self::from_recovered_tx(tx.inner(), caller),
        }
    }
}

/// Implements `FromRecoveredTx<TxDeposit>` for `TxEnv`
///
/// Converts a `TxDeposit` into a `TxEnv`, extracting standard transaction fields
/// and discarding deposit-specific fields (`source_hash`, mint, `is_system_transaction`).
impl FromRecoveredTx<TxDeposit> for TxEnv {
    fn from_recovered_tx(tx: &TxDeposit, caller: Address) -> Self {
        let TxDeposit {
            to,
            value,
            gas_limit,
            input,
            source_hash: _,
            from: _,
            mint: _,
            is_system_transaction: _,
        } = tx;
        Self {
            tx_type: tx.ty(),
            caller,
            gas_limit: *gas_limit,
            kind: *to,
            value: *value,
            data: input.clone(),
            ..Default::default()
        }
    }
}

/// Implements `FromTxWithEncoded<OpTxEnvelope>` for `TxEnv`
///
/// Converts an encoded `OpTxEnvelope` into a `TxEnv`. The encoded bytes are not used
/// as we can reconstruct the necessary information from the transaction structure.
impl FromTxWithEncoded<OpTxEnvelope> for TxEnv {
    fn from_encoded_tx(tx: &OpTxEnvelope, caller: Address, _encoded: Bytes) -> Self {
        Self::from_recovered_tx(tx, caller)
    }
}

/// Implements `FromRecoveredTx<OpTxEnvelope>` for `OpTransaction<TxEnv>`
///
/// Converts an `OpTxEnvelope` into an `OpTransaction<TxEnv>`, capturing the encoded
/// transaction bytes for later use.
impl FromRecoveredTx<OpTxEnvelope> for OpTransaction<TxEnv> {
    fn from_recovered_tx(tx: &OpTxEnvelope, sender: Address) -> Self {
        let encoded = tx.encoded_2718();
        Self::from_encoded_tx(tx, sender, encoded.into())
    }
}

/// Implements `FromTxWithEncoded<OpTxEnvelope>` for `OpTransaction<TxEnv>`
///
/// Converts an encoded `OpTxEnvelope` into an `OpTransaction<TxEnv>` by extracting
/// the base `TxEnv` from each variant and creating an `OpTransaction` with the encoded bytes.
impl FromTxWithEncoded<OpTxEnvelope> for OpTransaction<TxEnv> {
    fn from_encoded_tx(tx: &OpTxEnvelope, caller: Address, encoded: Bytes) -> Self {
        let base = match tx {
            OpTxEnvelope::Legacy(tx) => TxEnv::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip1559(tx) => TxEnv::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip2930(tx) => TxEnv::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Eip7702(tx) => TxEnv::from_recovered_tx(tx.tx(), caller),
            OpTxEnvelope::Deposit(tx) => TxEnv::from_recovered_tx(tx.inner(), caller),
        };
        Self { base, enveloped_tx: Some(encoded), deposit: Default::default() }
    }
}

/// Implements `FromRecoveredTx<TxDeposit>` for `OpTransaction<TxEnv>`
///
/// Converts a `TxDeposit` into an `OpTransaction<TxEnv>`, capturing both the base
/// transaction environment and deposit-specific metadata.
impl FromRecoveredTx<TxDeposit> for OpTransaction<TxEnv> {
    fn from_recovered_tx(tx: &TxDeposit, sender: Address) -> Self {
        let encoded = tx.encoded_2718();
        Self::from_encoded_tx(tx, sender, encoded.into())
    }
}

/// Implements `FromTxWithEncoded<TxDeposit>` for `OpTransaction<TxEnv>`
///
/// Converts an encoded `TxDeposit` into an `OpTransaction<TxEnv>`, preserving
/// all deposit-specific metadata (`source_hash`, mint, `is_system_transaction`).
impl FromTxWithEncoded<TxDeposit> for OpTransaction<TxEnv> {
    fn from_encoded_tx(tx: &TxDeposit, caller: Address, encoded: Bytes) -> Self {
        let base = TxEnv::from_recovered_tx(tx, caller);
        let deposit = DepositTransactionParts {
            source_hash: tx.source_hash,
            mint: Some(tx.mint),
            is_system_transaction: tx.is_system_transaction,
        };
        Self { base, enveloped_tx: Some(encoded), deposit }
    }
}
