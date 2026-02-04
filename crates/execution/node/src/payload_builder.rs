//! The implementation of the [`PayloadAttributesBuilder`] for local payload building.

use alloy_consensus::BlockHeader;
use alloy_primitives::{Address, B256};
use base_alloy_rpc_types_engine::OpPayloadAttributes;
use reth_chainspec::EthChainSpec;
use reth_node_api::PayloadAttributesBuilder;
use reth_primitives_traits::SealedHeader;
use std::sync::Arc;

/// Base local payload attributes builder for Optimism-specific payloads.
#[derive(Debug)]
#[non_exhaustive]
pub struct BaseLocalPayloadAttributesBuilder<ChainSpec> {
    /// The chainspec
    pub chain_spec: Arc<ChainSpec>,

    /// Whether to enforce increasing timestamp.
    pub enforce_increasing_timestamp: bool,
}

impl<ChainSpec> BaseLocalPayloadAttributesBuilder<ChainSpec> {
    /// Creates a new instance of the builder.
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { chain_spec, enforce_increasing_timestamp: true }
    }

    /// Creates a new instance of the builder without enforcing increasing timestamps.
    pub fn without_increasing_timestamp(self) -> Self {
        Self { enforce_increasing_timestamp: false, ..self }
    }
}

impl<ChainSpec> PayloadAttributesBuilder<OpPayloadAttributes, ChainSpec::Header>
    for BaseLocalPayloadAttributesBuilder<ChainSpec>
where
    ChainSpec: EthChainSpec + 'static,
{
    fn build(&self, parent: &SealedHeader<ChainSpec::Header>) -> OpPayloadAttributes {
        let mut timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if self.enforce_increasing_timestamp {
            timestamp = std::cmp::max(parent.timestamp().saturating_add(1), timestamp);
        }

        OpPayloadAttributes {
            payload_attributes: alloy_rpc_types_engine::PayloadAttributes {
                timestamp,
                prev_randao: B256::random(),
                suggested_fee_recipient: Address::random(),
                withdrawals: None,
                parent_beacon_block_root: None,
            },
            // Add dummy system transaction
            transactions: Some(vec![
                base_chainspec::constants::TX_SET_L1_BLOCK_OP_MAINNET_BLOCK_124665056
                    .as_slice()
                    .to_vec()
                    .into(),
            ]),
            no_tx_pool: None,
            gas_limit: None,
            eip_1559_params: None,
            min_base_fee: None,
        }
    }
}
