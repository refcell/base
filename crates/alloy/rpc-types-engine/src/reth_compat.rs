//! Reth trait implementations for base-alloy-rpc-types-engine types.

use crate::{OpExecutionData, OpPayloadAttributes};
use alloc::vec::Vec;
use alloy_eips::eip4895::Withdrawal;
use alloy_primitives::{B256, Bytes};
use reth_payload_primitives::{ExecutionPayload, PayloadAttributes};

impl ExecutionPayload for OpExecutionData {
    fn parent_hash(&self) -> B256 {
        self.parent_hash()
    }

    fn block_hash(&self) -> B256 {
        self.block_hash()
    }

    fn block_number(&self) -> u64 {
        self.block_number()
    }

    fn withdrawals(&self) -> Option<&Vec<Withdrawal>> {
        self.payload.as_v2().map(|p| &p.withdrawals)
    }

    fn block_access_list(&self) -> Option<&Bytes> {
        None
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.sidecar.parent_beacon_block_root()
    }

    fn timestamp(&self) -> u64 {
        self.payload.as_v1().timestamp
    }

    fn gas_used(&self) -> u64 {
        self.payload.as_v1().gas_used
    }

    fn transaction_count(&self) -> usize {
        self.payload.as_v1().transactions.len()
    }
}

impl PayloadAttributes for OpPayloadAttributes {
    fn timestamp(&self) -> u64 {
        self.payload_attributes.timestamp
    }

    fn withdrawals(&self) -> Option<&Vec<Withdrawal>> {
        self.payload_attributes.withdrawals.as_ref()
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.payload_attributes.parent_beacon_block_root
    }
}
