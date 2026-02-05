//! Reth trait implementations for base-alloy-consensus types.
//!
//! This module provides implementations of reth traits required for database storage
//! and node operation:
//! - `InMemorySize` for memory accounting
//! - `SignedTransaction` for transaction signing
//! - `Compact` for database serialization (reth-codecs)
//! - `SerdeBincodeCompat` for bincode serialization

use alloy_consensus::{Sealed, Signed, TxEip1559, TxEip2930, TxEip7702, TxLegacy};
use alloy_primitives::{Address, B256, Bytes, Signature, TxKind, U256};
use bytes::{Buf, BufMut};
use reth_codecs::Compact;
use reth_db_api::{
    DatabaseError,
    table::{Compress, Decompress},
};
use reth_primitives_traits::{InMemorySize, SignedTransaction};

use crate::{
    DEPOSIT_TX_TYPE_ID, OpDepositReceipt, OpPooledTransaction, OpReceipt, OpTxEnvelope, OpTxType,
    OpTypedTransaction, TxDeposit,
};

// ============================================================================
// InMemorySize implementations
// ============================================================================

impl InMemorySize for OpTxType {
    #[inline]
    fn size(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl InMemorySize for OpDepositReceipt {
    fn size(&self) -> usize {
        self.inner.size() + core::mem::size_of::<Option<u64>>() * 2
    }
}

impl InMemorySize for OpReceipt {
    fn size(&self) -> usize {
        match self {
            Self::Legacy(receipt)
            | Self::Eip2930(receipt)
            | Self::Eip1559(receipt)
            | Self::Eip7702(receipt) => receipt.size(),
            Self::Deposit(receipt) => receipt.size(),
        }
    }
}

impl InMemorySize for OpTypedTransaction {
    fn size(&self) -> usize {
        match self {
            Self::Legacy(tx) => tx.size(),
            Self::Eip2930(tx) => tx.size(),
            Self::Eip1559(tx) => tx.size(),
            Self::Eip7702(tx) => tx.size(),
            Self::Deposit(tx) => tx.size(),
        }
    }
}

impl InMemorySize for OpPooledTransaction {
    fn size(&self) -> usize {
        match self {
            Self::Legacy(tx) => tx.size(),
            Self::Eip2930(tx) => tx.size(),
            Self::Eip1559(tx) => tx.size(),
            Self::Eip7702(tx) => tx.size(),
        }
    }
}

impl InMemorySize for OpTxEnvelope {
    fn size(&self) -> usize {
        match self {
            Self::Legacy(tx) => tx.size(),
            Self::Eip2930(tx) => tx.size(),
            Self::Eip1559(tx) => tx.size(),
            Self::Eip7702(tx) => tx.size(),
            Self::Deposit(tx) => tx.size(),
        }
    }
}

impl SignedTransaction for OpPooledTransaction {}

impl SignedTransaction for OpTxEnvelope {}

// ============================================================================
// Compact trait implementations for database serialization
// ============================================================================

/// Compact identifier constants for transaction types.
/// For backwards compatibility purposes only 2 bits of the type are encoded in the identifier
/// parameter.
mod compact_ids {
    /// Identifier parameter for legacy transaction
    pub(super) const COMPACT_IDENTIFIER_LEGACY: usize = 0;
    /// Identifier parameter for EIP-2930 transaction
    pub(super) const COMPACT_IDENTIFIER_EIP2930: usize = 1;
    /// Identifier parameter for EIP-1559 transaction
    pub(super) const COMPACT_IDENTIFIER_EIP1559: usize = 2;
    /// Extended identifier flag - full transaction type is read from buffer
    pub(super) const COMPACT_EXTENDED_IDENTIFIER_FLAG: usize = 3;
}

use compact_ids::*;

/// Helper struct for deriving Compact on `TxDeposit` fields.
/// This mirrors the structure of [`TxDeposit`] for compact encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct TxDepositCompact {
    source_hash: B256,
    from: Address,
    to: TxKind,
    mint: Option<u128>,
    value: U256,
    gas_limit: u64,
    is_system_transaction: bool,
    input: Bytes,
}

impl Compact for TxDepositCompact {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        // Encode fields in order
        self.source_hash.to_compact(buf);
        self.from.to_compact(buf);
        self.to.to_compact(buf);

        // Encode mint as optional
        let mint_flag = if self.mint.is_some() { 1usize } else { 0usize };
        if let Some(mint) = self.mint {
            mint.to_compact(buf);
        }

        self.value.to_compact(buf);
        self.gas_limit.to_compact(buf);

        // Encode is_system_transaction as u8
        buf.put_u8(self.is_system_transaction as u8);

        self.input.to_compact(buf);

        // Return flags: mint present flag
        mint_flag
    }

    fn from_compact(mut buf: &[u8], len: usize) -> (Self, &[u8]) {
        let (source_hash, rest) = B256::from_compact(buf, buf.len());
        buf = rest;

        let (from, rest) = Address::from_compact(buf, buf.len());
        buf = rest;

        let (to, rest) = TxKind::from_compact(buf, buf.len());
        buf = rest;

        // Decode mint based on flag
        let mint = if len > 0 {
            let (mint_val, rest) = u128::from_compact(buf, buf.len());
            buf = rest;
            Some(mint_val)
        } else {
            None
        };

        let (value, rest) = U256::from_compact(buf, buf.len());
        buf = rest;

        let (gas_limit, rest) = u64::from_compact(buf, buf.len());
        buf = rest;

        let is_system_transaction = buf.get_u8() != 0;

        let (input, rest) = Bytes::from_compact(buf, buf.len());

        (Self { source_hash, from, to, mint, value, gas_limit, is_system_transaction, input }, rest)
    }
}

impl Compact for TxDeposit {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        let compact = TxDepositCompact {
            source_hash: self.source_hash,
            from: self.from,
            to: self.to,
            mint: if self.mint == 0 { None } else { Some(self.mint) },
            value: self.value,
            gas_limit: self.gas_limit,
            is_system_transaction: self.is_system_transaction,
            input: self.input.clone(),
        };
        compact.to_compact(buf)
    }

    fn from_compact(buf: &[u8], len: usize) -> (Self, &[u8]) {
        let (compact, remaining) = TxDepositCompact::from_compact(buf, len);
        let tx = Self {
            source_hash: compact.source_hash,
            from: compact.from,
            to: compact.to,
            mint: compact.mint.unwrap_or_default(),
            value: compact.value,
            gas_limit: compact.gas_limit,
            is_system_transaction: compact.is_system_transaction,
            input: compact.input,
        };
        (tx, remaining)
    }
}

impl Compact for OpTxType {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        match self {
            Self::Legacy => COMPACT_IDENTIFIER_LEGACY,
            Self::Eip2930 => COMPACT_IDENTIFIER_EIP2930,
            Self::Eip1559 => COMPACT_IDENTIFIER_EIP1559,
            Self::Eip7702 => {
                buf.put_u8(alloy_consensus::constants::EIP7702_TX_TYPE_ID);
                COMPACT_EXTENDED_IDENTIFIER_FLAG
            }
            Self::Deposit => {
                buf.put_u8(DEPOSIT_TX_TYPE_ID);
                COMPACT_EXTENDED_IDENTIFIER_FLAG
            }
        }
    }

    fn from_compact(mut buf: &[u8], identifier: usize) -> (Self, &[u8]) {
        (
            match identifier {
                COMPACT_IDENTIFIER_LEGACY => Self::Legacy,
                COMPACT_IDENTIFIER_EIP2930 => Self::Eip2930,
                COMPACT_IDENTIFIER_EIP1559 => Self::Eip1559,
                COMPACT_EXTENDED_IDENTIFIER_FLAG => {
                    let extended_identifier = buf.get_u8();
                    match extended_identifier {
                        alloy_consensus::constants::EIP7702_TX_TYPE_ID => Self::Eip7702,
                        DEPOSIT_TX_TYPE_ID => Self::Deposit,
                        _ => panic!("Unsupported OpTxType identifier: {extended_identifier}"),
                    }
                }
                _ => panic!("Unknown identifier for OpTxType: {identifier}"),
            },
            buf,
        )
    }
}

impl Compact for OpTypedTransaction {
    fn to_compact<B>(&self, out: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        let identifier = self.tx_type().to_compact(out);
        match self {
            Self::Legacy(tx) => tx.to_compact(out),
            Self::Eip2930(tx) => tx.to_compact(out),
            Self::Eip1559(tx) => tx.to_compact(out),
            Self::Eip7702(tx) => tx.to_compact(out),
            Self::Deposit(tx) => tx.to_compact(out),
        };
        identifier
    }

    fn from_compact(buf: &[u8], identifier: usize) -> (Self, &[u8]) {
        let (tx_type, buf) = OpTxType::from_compact(buf, identifier);
        match tx_type {
            OpTxType::Legacy => {
                let (tx, buf) = Compact::from_compact(buf, buf.len());
                (Self::Legacy(tx), buf)
            }
            OpTxType::Eip2930 => {
                let (tx, buf) = Compact::from_compact(buf, buf.len());
                (Self::Eip2930(tx), buf)
            }
            OpTxType::Eip1559 => {
                let (tx, buf) = Compact::from_compact(buf, buf.len());
                (Self::Eip1559(tx), buf)
            }
            OpTxType::Eip7702 => {
                let (tx, buf) = Compact::from_compact(buf, buf.len());
                (Self::Eip7702(tx), buf)
            }
            OpTxType::Deposit => {
                let (tx, buf) = Compact::from_compact(buf, buf.len());
                (Self::Deposit(tx), buf)
            }
        }
    }
}

// ============================================================================
// Envelope Compact traits for OpTxEnvelope
// ============================================================================

/// A trait for extracting transaction without type and signature and serializing it using
/// [`Compact`] encoding.
trait ToTxCompact {
    /// Serializes inner transaction using [`Compact`] encoding.
    fn to_tx_compact(&self, buf: &mut (impl BufMut + AsMut<[u8]>));
}

/// A trait for deserializing transaction without type and signature using [`Compact`] encoding.
trait FromTxCompact {
    /// The transaction type that represents the set of transactions.
    type TxType;

    /// Deserializes inner transaction using [`Compact`] encoding.
    fn from_tx_compact(buf: &[u8], tx_type: Self::TxType, signature: Signature) -> (Self, &[u8])
    where
        Self: Sized;
}

impl ToTxCompact for OpTxEnvelope {
    fn to_tx_compact(&self, buf: &mut (impl BufMut + AsMut<[u8]>)) {
        match self {
            Self::Legacy(tx) => tx.tx().to_compact(buf),
            Self::Eip2930(tx) => tx.tx().to_compact(buf),
            Self::Eip1559(tx) => tx.tx().to_compact(buf),
            Self::Eip7702(tx) => tx.tx().to_compact(buf),
            Self::Deposit(tx) => tx.inner().to_compact(buf),
        };
    }
}

impl FromTxCompact for OpTxEnvelope {
    type TxType = OpTxType;

    fn from_tx_compact(buf: &[u8], tx_type: OpTxType, signature: Signature) -> (Self, &[u8]) {
        match tx_type {
            OpTxType::Legacy => {
                let (tx, buf) = TxLegacy::from_compact(buf, buf.len());
                let tx = Signed::new_unhashed(tx, signature);
                (Self::Legacy(tx), buf)
            }
            OpTxType::Eip2930 => {
                let (tx, buf) = TxEip2930::from_compact(buf, buf.len());
                let tx = Signed::new_unhashed(tx, signature);
                (Self::Eip2930(tx), buf)
            }
            OpTxType::Eip1559 => {
                let (tx, buf) = TxEip1559::from_compact(buf, buf.len());
                let tx = Signed::new_unhashed(tx, signature);
                (Self::Eip1559(tx), buf)
            }
            OpTxType::Eip7702 => {
                let (tx, buf) = TxEip7702::from_compact(buf, buf.len());
                let tx = Signed::new_unhashed(tx, signature);
                (Self::Eip7702(tx), buf)
            }
            OpTxType::Deposit => {
                let (tx, buf) = TxDeposit::from_compact(buf, buf.len());
                let tx = Sealed::new(tx);
                (Self::Deposit(tx), buf)
            }
        }
    }
}

/// Placeholder signature for deposit transactions (they don't have real signatures).
const DEPOSIT_SIGNATURE: Signature = Signature::new(U256::ZERO, U256::ZERO, false);

impl Compact for OpTxEnvelope {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        use alloy_consensus::Transaction;

        let start = buf.as_mut().len();

        // Placeholder for bitflags.
        // The first byte uses 4 bits as flags: IsCompressed[1bit], TxType[2bits], Signature[1bit]
        buf.put_u8(0);

        let signature = self.signature().unwrap_or(&DEPOSIT_SIGNATURE);
        let sig_bit = signature.to_compact(buf) as u8;
        let zstd_bit = self.input().len() >= 32;

        let tx_bits = if zstd_bit {
            // Compress the tx prefixed with txtype
            let mut tx_buf = alloc::vec::Vec::with_capacity(256);
            let tx_bits = self.tx_type().to_compact(&mut tx_buf) as u8;
            self.to_tx_compact(&mut tx_buf);

            let compressed = reth_zstd_compressors::TRANSACTION_COMPRESSOR.with(|compressor| {
                let mut compressor = compressor.borrow_mut();
                compressor.compress(&tx_buf)
            });
            buf.put_slice(&compressed.expect("Failed to compress"));
            tx_bits
        } else {
            let tx_bits = self.tx_type().to_compact(buf) as u8;
            self.to_tx_compact(buf);
            tx_bits
        };

        let flags = sig_bit | (tx_bits << 1) | ((zstd_bit as u8) << 3);
        buf.as_mut()[start] = flags;

        buf.as_mut().len() - start
    }

    fn from_compact(mut buf: &[u8], _len: usize) -> (Self, &[u8]) {
        let flags = buf.get_u8() as usize;

        let sig_bit = flags & 1;
        let tx_bits = (flags & 0b110) >> 1;
        let zstd_bit = flags >> 3;

        let (signature, remaining) = Signature::from_compact(buf, sig_bit);
        buf = remaining;

        let (transaction, buf) = if zstd_bit != 0 {
            reth_zstd_compressors::TRANSACTION_DECOMPRESSOR.with(|decompressor| {
                let mut decompressor = decompressor.borrow_mut();
                let decompressed = decompressor.decompress(buf);

                let (tx_type, tx_buf) = OpTxType::from_compact(decompressed, tx_bits);
                let (tx, _) = Self::from_tx_compact(tx_buf, tx_type, signature);

                (tx, buf)
            })
        } else {
            let (tx_type, remaining) = OpTxType::from_compact(buf, tx_bits);
            Self::from_tx_compact(remaining, tx_type, signature)
        };

        (transaction, buf)
    }
}

// ============================================================================
// Compact implementation for OpReceipt
// ============================================================================

impl Compact for OpReceipt {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        use alloy_consensus::TxReceipt;

        let start = buf.as_mut().len();

        buf.put_u8(0);

        let tx_type_bits = self.tx_type().to_compact(buf);

        let status_bit = if self.status() { 1u8 } else { 0u8 };
        self.cumulative_gas_used().to_compact(buf);
        self.as_receipt().logs.to_compact(buf);

        let (deposit_nonce, deposit_version) = if let Self::Deposit(receipt) = self {
            (receipt.deposit_nonce, receipt.deposit_receipt_version)
        } else {
            (None, None)
        };

        let has_deposit_nonce = deposit_nonce.is_some();
        let has_deposit_version = deposit_version.is_some();

        if let Some(nonce) = deposit_nonce {
            nonce.to_compact(buf);
        }
        if let Some(version) = deposit_version {
            version.to_compact(buf);
        }

        let flags = status_bit
            | ((tx_type_bits as u8) << 1)
            | ((has_deposit_nonce as u8) << 3)
            | ((has_deposit_version as u8) << 4);
        buf.as_mut()[start] = flags;

        buf.as_mut().len() - start
    }

    fn from_compact(mut buf: &[u8], _len: usize) -> (Self, &[u8]) {
        use alloy_consensus::Receipt;

        let flags = buf.get_u8();

        let status_bit = (flags & 1) != 0;
        let tx_type_bits = ((flags >> 1) & 0b11) as usize;
        let has_deposit_nonce = ((flags >> 3) & 1) != 0;
        let has_deposit_version = ((flags >> 4) & 1) != 0;

        let (tx_type, remaining) = OpTxType::from_compact(buf, tx_type_bits);
        buf = remaining;

        let (cumulative_gas_used, remaining) = u64::from_compact(buf, buf.len());
        buf = remaining;

        let (logs, remaining) =
            alloc::vec::Vec::<alloy_primitives::Log>::from_compact(buf, buf.len());
        buf = remaining;

        let deposit_nonce = if has_deposit_nonce {
            let (nonce, remaining) = u64::from_compact(buf, buf.len());
            buf = remaining;
            Some(nonce)
        } else {
            None
        };

        let deposit_version = if has_deposit_version {
            let (version, remaining) = u64::from_compact(buf, buf.len());
            buf = remaining;
            Some(version)
        } else {
            None
        };

        let inner = Receipt { status: status_bit.into(), cumulative_gas_used, logs };

        let receipt = match tx_type {
            OpTxType::Legacy => Self::Legacy(inner),
            OpTxType::Eip2930 => Self::Eip2930(inner),
            OpTxType::Eip1559 => Self::Eip1559(inner),
            OpTxType::Eip7702 => Self::Eip7702(inner),
            OpTxType::Deposit => Self::Deposit(OpDepositReceipt {
                inner,
                deposit_nonce,
                deposit_receipt_version: deposit_version,
            }),
        };

        (receipt, buf)
    }
}

// ============================================================================
// Compress/Decompress implementations for database storage
// ============================================================================

impl Compress for OpTxEnvelope {
    type Compressed = alloc::vec::Vec<u8>;

    fn compress_to_buf<B: BufMut + AsMut<[u8]>>(&self, buf: &mut B) {
        let _ = Compact::to_compact(self, buf);
    }
}

impl Decompress for OpTxEnvelope {
    fn decompress(value: &[u8]) -> Result<Self, DatabaseError> {
        let (obj, _) = Compact::from_compact(value, value.len());
        Ok(obj)
    }
}

impl Compress for OpReceipt {
    type Compressed = alloc::vec::Vec<u8>;

    fn compress_to_buf<B: BufMut + AsMut<[u8]>>(&self, buf: &mut B) {
        let _ = Compact::to_compact(self, buf);
    }
}

impl Decompress for OpReceipt {
    fn decompress(value: &[u8]) -> Result<Self, DatabaseError> {
        let (obj, _) = Compact::from_compact(value, value.len());
        Ok(obj)
    }
}

// ============================================================================
// SerdeBincodeCompat implementations (requires serde-bincode-compat feature)
// ============================================================================

#[cfg(feature = "serde-bincode-compat")]
mod bincode_compat_impls {
    use reth_primitives_traits::serde_bincode_compat::SerdeBincodeCompat;

    use super::{OpReceipt, OpTxEnvelope};

    impl SerdeBincodeCompat for OpTxEnvelope {
        type BincodeRepr<'a> = crate::serde_bincode_compat::transaction::OpTxEnvelope<'a>;

        fn as_repr(&self) -> Self::BincodeRepr<'_> {
            self.into()
        }

        fn from_repr(repr: Self::BincodeRepr<'_>) -> Self {
            repr.into()
        }
    }

    impl SerdeBincodeCompat for OpReceipt {
        type BincodeRepr<'a> = crate::serde_bincode_compat::OpReceipt<'a>;

        fn as_repr(&self) -> Self::BincodeRepr<'_> {
            self.into()
        }

        fn from_repr(repr: Self::BincodeRepr<'_>) -> Self {
            repr.into()
        }
    }
}
