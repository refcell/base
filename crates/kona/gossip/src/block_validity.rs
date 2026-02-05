#[cfg(feature = "metrics")]
use std::time::Instant;
use std::time::SystemTime;

use alloy_consensus::Block;
use alloy_eips::eip7685::EMPTY_REQUESTS_HASH;
use alloy_primitives::{Address, B256};
use alloy_rpc_types_engine::{ExecutionPayloadV3, PayloadError};
use base_alloy_consensus::OpTxEnvelope;
use base_alloy_rpc_types_engine::{
    OpExecutionPayload, OpExecutionPayloadV4, OpNetworkPayloadEnvelope, OpPayloadError,
};
use base_genesis::RollupConfig;
use libp2p::gossipsub::MessageAcceptance;

use super::BlockHandler;
#[cfg(feature = "metrics")]
use crate::Metrics;

/// Error that can occur when validating a block.
#[derive(Debug, thiserror::Error)]
pub enum BlockInvalidError {
    /// The block has an invalid timestamp.
    #[error("Invalid timestamp. Current: {current}, Received: {received}")]
    Timestamp {
        /// The current timestamp.
        current: u64,
        /// The received timestamp.
        received: u64,
    },
    /// The block has an invalid base fee per gas.
    #[error("Base fee per gas overflow")]
    BaseFeePerGasOverflow(#[from] PayloadError),
    /// The block has an invalid hash.
    #[error("Invalid block hash. Expected: {expected}, Received: {received}")]
    BlockHash {
        /// The expected block hash.
        expected: B256,
        /// The received block hash.
        received: B256,
    },
    /// The block has an invalid signature.
    #[error("Invalid signature.")]
    Signature,
    /// The block has an invalid signer.
    #[error("Invalid signer, expected: {expected}, received: {received}")]
    Signer {
        /// The expected signer.
        expected: Address,
        /// The received signer.
        received: Address,
    },
    /// Invalid block.
    #[error(transparent)]
    InvalidBlock(#[from] OpPayloadError),
    /// The block has an invalid parent beacon block root.
    #[error("Payload is on v3+ topic, but has empty parent beacon root")]
    ParentBeaconRoot,
    /// The block has an invalid blob gas used.
    #[error("Payload is on v3+ topic, but has non-zero blob gas used")]
    BlobGasUsed,
    /// The block has an invalid excess blob gas.
    #[error("Payload is on v3+ topic, but has non-zero excess blob gas")]
    ExcessBlobGas,
    /// The block has an invalid withdrawals root.
    #[error("Payload is on v4+ topic, but has non-empty withdrawals root")]
    WithdrawalsRoot,
    /// Too many blocks were validated for the same height.
    #[error("Too many blocks seen for height {height}")]
    TooManyBlocks {
        /// The height of the block.
        height: u64,
    },
    /// The block has already been seen.
    #[error("Block seen before")]
    BlockSeen {
        /// The hash of the block.
        block_hash: B256,
    },
}

impl From<BlockInvalidError> for MessageAcceptance {
    fn from(value: BlockInvalidError) -> Self {
        match value {
            BlockInvalidError::BlockSeen { block_hash: _ } => Self::Ignore,
            _ => Self::Reject,
        }
    }
}

impl BlockHandler {
    /// The maximum number of blocks to keep in the seen hashes map.
    pub const SEEN_HASH_CACHE_SIZE: usize = 1_000;

    /// The maximum number of blocks to keep per height.
    const MAX_BLOCKS_TO_KEEP: usize = 5;

    /// Determines if a block is valid.
    pub fn block_valid(
        &mut self,
        envelope: &OpNetworkPayloadEnvelope,
    ) -> Result<(), BlockInvalidError> {
        #[cfg(feature = "metrics")]
        let validation_start = Instant::now();

        #[cfg(feature = "metrics")]
        base_macros::inc!(counter, Metrics::BLOCK_VALIDATION_TOTAL);

        #[cfg(feature = "metrics")]
        {
            let version = match &envelope.payload {
                OpExecutionPayload::V1(_) => "v1",
                OpExecutionPayload::V2(_) => "v2",
                OpExecutionPayload::V3(_) => "v3",
                OpExecutionPayload::V4(_) => "v4",
            };
            base_macros::inc!(counter, Metrics::BLOCK_VERSION, "version" => version);
        }

        let validation_result = self.validate_block_internal(envelope);

        #[cfg(feature = "metrics")]
        {
            let duration = validation_start.elapsed();
            base_macros::record!(
                histogram,
                Metrics::BLOCK_VALIDATION_DURATION_SECONDS,
                duration.as_secs_f64()
            );
        }

        match &validation_result {
            Ok(()) => {
                #[cfg(feature = "metrics")]
                base_macros::inc!(counter, Metrics::BLOCK_VALIDATION_SUCCESS);
            }
            Err(_err) => {
                #[cfg(feature = "metrics")]
                {
                    let reason = match _err {
                        BlockInvalidError::Timestamp { current, received } => {
                            if *received > *current + 5 {
                                "timestamp_future"
                            } else {
                                "timestamp_past"
                            }
                        }
                        BlockInvalidError::BlockHash { .. } => "invalid_hash",
                        BlockInvalidError::Signature => "invalid_signature",
                        BlockInvalidError::Signer { .. } => "invalid_signer",
                        BlockInvalidError::TooManyBlocks { .. } => "too_many_blocks",
                        BlockInvalidError::BlockSeen { .. } => "block_seen",
                        BlockInvalidError::InvalidBlock(_)
                        | BlockInvalidError::BaseFeePerGasOverflow(_) => "invalid_block",
                        BlockInvalidError::ParentBeaconRoot => "parent_beacon_root",
                        BlockInvalidError::BlobGasUsed => "blob_gas_used",
                        BlockInvalidError::ExcessBlobGas => "excess_blob_gas",
                        BlockInvalidError::WithdrawalsRoot => "withdrawals_root",
                    };
                    base_macros::inc!(counter, Metrics::BLOCK_VALIDATION_FAILED, "reason" => reason);
                }
            }
        }

        validation_result
    }

    fn validate_block_internal(
        &mut self,
        envelope: &OpNetworkPayloadEnvelope,
    ) -> Result<(), BlockInvalidError> {
        let current_timestamp =
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let is_future = envelope.payload.timestamp() > current_timestamp + 5;
        let is_past = envelope.payload.timestamp() < current_timestamp - 60;

        if is_future || is_past {
            return Err(BlockInvalidError::Timestamp {
                current: current_timestamp,
                received: envelope.payload.timestamp(),
            });
        }

        let expected = envelope.payload.block_hash();
        let mut block: Block<OpTxEnvelope> = envelope.payload.clone().try_into_block()?;
        block.header.parent_beacon_block_root = envelope.parent_beacon_block_root;
        if self.rollup_config.is_isthmus_active(envelope.payload.timestamp()) {
            block.header.requests_hash = Some(EMPTY_REQUESTS_HASH);
        }
        let received = block.header.hash_slow();
        if received != expected {
            return Err(BlockInvalidError::BlockHash { expected, received });
        }

        self.validate_version_specific_payload(envelope)?;

        if let Some(seen_hashes_at_height) =
            self.seen_hashes.get_mut(&envelope.payload.block_number())
        {
            if seen_hashes_at_height.len() > Self::MAX_BLOCKS_TO_KEEP {
                return Err(BlockInvalidError::TooManyBlocks {
                    height: envelope.payload.block_number(),
                });
            }

            if seen_hashes_at_height.contains(&envelope.payload.block_hash()) {
                return Err(BlockInvalidError::BlockSeen {
                    block_hash: envelope.payload.block_hash(),
                });
            }
        }

        let msg = envelope.payload_hash.signature_message(self.rollup_config.l2_chain_id.id());
        let block_signer = *self.signer_recv.borrow();

        let Ok(msg_signer) = envelope.signature.recover_address_from_prehash(&msg) else {
            return Err(BlockInvalidError::Signature);
        };

        if msg_signer != block_signer {
            return Err(BlockInvalidError::Signer { expected: block_signer, received: msg_signer });
        }

        self.seen_hashes
            .entry(envelope.payload.block_number())
            .or_default()
            .insert(envelope.payload.block_hash());

        if self.seen_hashes.len() >= Self::SEEN_HASH_CACHE_SIZE {
            self.seen_hashes.pop_first();
        }

        Ok(())
    }

    fn validate_version_specific_payload(
        &self,
        envelope: &OpNetworkPayloadEnvelope,
    ) -> Result<(), BlockInvalidError> {
        fn validate_v3(
            rollup_config: &RollupConfig,
            block: &ExecutionPayloadV3,
            parent_beacon_block_root: Option<B256>,
        ) -> Result<(), BlockInvalidError> {
            if !rollup_config.is_jovian_active(block.timestamp()) && block.blob_gas_used != 0 {
                return Err(BlockInvalidError::BlobGasUsed);
            }

            if block.excess_blob_gas != 0 {
                return Err(BlockInvalidError::ExcessBlobGas);
            }

            if parent_beacon_block_root.is_none() {
                return Err(BlockInvalidError::ParentBeaconRoot);
            }

            Ok(())
        }

        fn validate_v4(
            rollup_config: &RollupConfig,
            block: &OpExecutionPayloadV4,
            parent_beacon_block_root: Option<B256>,
        ) -> Result<(), BlockInvalidError> {
            validate_v3(rollup_config, &block.payload_inner, parent_beacon_block_root)
        }

        match &envelope.payload {
            OpExecutionPayload::V1(_) | OpExecutionPayload::V2(_) => Ok(()),
            OpExecutionPayload::V3(payload) => {
                validate_v3(&self.rollup_config, payload, envelope.parent_beacon_block_root)
            }
            OpExecutionPayload::V4(payload) => {
                validate_v4(&self.rollup_config, payload, envelope.parent_beacon_block_root)
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {

    use alloy_chains::Chain;
    use alloy_consensus::{Block, EMPTY_OMMER_ROOT_HASH};
    use alloy_eips::{eip2718::Encodable2718, eip4895::Withdrawal};
    use alloy_primitives::{Address, B256, Bytes, Signature};
    use alloy_rlp::BufMut;
    use alloy_rpc_types_engine::{ExecutionPayloadV1, ExecutionPayloadV2, ExecutionPayloadV3};
    use arbitrary::{Arbitrary, Unstructured};
    use base_alloy_consensus::OpTxEnvelope;
    use base_alloy_rpc_types_engine::{OpExecutionPayload, OpExecutionPayloadV4, PayloadHash};
    use base_genesis::RollupConfig;

    use super::*;

    fn valid_block() -> Block<OpTxEnvelope> {
        let mut data = vec![0; 1024 * 1024];
        let mut rng = rand::rng();
        rand::Rng::fill(&mut rng, &mut data[..]);

        let u = Unstructured::new(&data);

        let mut block: Block<OpTxEnvelope> = Block::arbitrary_take_rest(u).unwrap();

        let transactions: Vec<Bytes> =
            block.body.transactions().map(|tx| tx.encoded_2718().into()).collect();

        let transactions_root =
            alloy_consensus::proofs::ordered_trie_root_with_encoder(&transactions, |item, buf| {
                buf.put_slice(item)
            });

        block.header.transactions_root = transactions_root;

        block.header.base_fee_per_gas =
            Some(block.header.base_fee_per_gas.unwrap_or_default().saturating_add(1));

        let current_timestamp =
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        block.header.timestamp = current_timestamp;

        block
    }

    fn v1_valid_block() -> Block<OpTxEnvelope> {
        let mut block = valid_block();
        block.header.withdrawals_root = None;
        block.header.blob_gas_used = None;
        block.header.excess_blob_gas = None;
        block.header.parent_beacon_block_root = None;
        block.header.requests_hash = None;
        block.header.ommers_hash = EMPTY_OMMER_ROOT_HASH;
        block.header.difficulty = Default::default();
        block.header.nonce = Default::default();

        block
    }

    pub(crate) fn v2_valid_block() -> Block<OpTxEnvelope> {
        let mut block = v1_valid_block();

        block.body.withdrawals = Some(vec![].into());
        let withdrawals_root = alloy_consensus::proofs::calculate_withdrawals_root(
            &block.body.withdrawals.clone().unwrap_or_default(),
        );

        block.header.withdrawals_root = Some(withdrawals_root);

        block
    }

    pub(crate) fn v3_valid_block() -> Block<OpTxEnvelope> {
        let mut block = valid_block();

        block.body.withdrawals = Some(vec![].into());
        let withdrawals_root = alloy_consensus::proofs::calculate_withdrawals_root(
            &block.body.withdrawals.clone().unwrap_or_default(),
        );
        block.header.withdrawals_root = Some(withdrawals_root);

        block.header.blob_gas_used = Some(0);
        block.header.excess_blob_gas = Some(0);
        block.header.parent_beacon_block_root =
            Some(block.header.parent_beacon_block_root.unwrap_or_default());

        block.header.requests_hash = None;
        block.header.ommers_hash = EMPTY_OMMER_ROOT_HASH;
        block.header.difficulty = Default::default();
        block.header.nonce = Default::default();

        block
    }

    pub(crate) fn v4_valid_block() -> Block<OpTxEnvelope> {
        v3_valid_block()
    }

    #[test]
    fn test_block_valid() {
        let block = v1_valid_block();

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
    }

    #[test]
    fn test_block_invalid_timestamp_early() {
        let mut block = v1_valid_block();

        block.header.timestamp -= 61;

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::Timestamp { .. })));
    }

    #[test]
    fn test_block_invalid_timestamp_too_far() {
        let mut block = v1_valid_block();

        block.header.timestamp += 6;

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::Timestamp { .. })));
    }

    #[test]
    fn test_block_invalid_hash() {
        let block = v1_valid_block();

        let mut v1 = ExecutionPayloadV1::from_block_slow(&block);

        v1.block_hash = B256::ZERO;

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::BlockHash { .. })));
    }

    #[test]
    fn test_cannot_validate_same_block_twice() {
        let block = v1_valid_block();

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::BlockSeen { .. })));
    }

    #[test]
    fn test_cannot_have_too_many_blocks_for_the_same_height() {
        let first_block = v1_valid_block();

        let initial_height = first_block.header.number;

        let v1 = ExecutionPayloadV1::from_block_slow(&first_block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());

        let next_payloads = (0..=BlockHandler::MAX_BLOCKS_TO_KEEP)
            .map(|_| {
                let mut block = v1_valid_block();
                block.header.number = initial_height;

                let v1 = ExecutionPayloadV1::from_block_slow(&block);

                let payload = OpExecutionPayload::V1(v1);
                OpNetworkPayloadEnvelope {
                    payload,
                    signature: Signature::test_signature(),
                    payload_hash: PayloadHash(B256::ZERO),
                    parent_beacon_block_root: None,
                }
            })
            .collect::<Vec<_>>();

        for envelope in &next_payloads[..next_payloads.len() - 1] {
            assert!(handler.block_valid(envelope).is_ok());
        }

        assert!(matches!(
            handler.block_valid(next_payloads.last().unwrap()),
            Err(BlockInvalidError::TooManyBlocks { .. })
        ));
    }

    #[test]
    fn test_invalid_signature() {
        let block = v1_valid_block();

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let mut envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        let mut signature_bytes = envelope.signature.as_bytes();
        signature_bytes[0] = !signature_bytes[0];
        envelope.signature = Signature::from_raw_array(&signature_bytes).unwrap();

        assert!(handler.seen_hashes.is_empty());
        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::Signature)));
    }

    #[test]
    fn test_invalid_signer() {
        let block = v1_valid_block();

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let (_, unsafe_signer) = tokio::sync::watch::channel(Address::default());
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::Signer { .. })));
    }

    #[test]
    fn test_v1_v2_block_invalid_parent_beacon_block_root() {
        let block = v1_valid_block();

        let v1 = ExecutionPayloadV1::from_block_slow(&block);

        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(B256::ZERO),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::BlockHash { .. })));

        let block = v2_valid_block();

        let v2 = ExecutionPayloadV2::from_block_slow(&block);

        let payload = OpExecutionPayload::V2(v2);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(B256::ZERO),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::BlockHash { .. })));
    }

    #[test]
    fn test_v2_block() {
        let block = v2_valid_block();

        let v2 = ExecutionPayloadV2::from_block_slow(&block);

        let payload = OpExecutionPayload::V2(v2);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
    }

    #[test]
    fn test_v2_non_empty_withdrawals() {
        let mut block = v2_valid_block();
        block.body.withdrawals = Some(vec![Withdrawal::default()].into());
        let withdrawals_root = alloy_consensus::proofs::calculate_withdrawals_root(
            &block.body.withdrawals.clone().unwrap_or_default(),
        );
        block.header.withdrawals_root = Some(withdrawals_root);

        let v2 = ExecutionPayloadV2::from_block_slow(&block);

        let payload = OpExecutionPayload::V2(v2);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(
            handler.block_valid(&envelope),
            Err(BlockInvalidError::InvalidBlock(OpPayloadError::NonEmptyL1Withdrawals))
        ));
    }

    #[test]
    fn test_v3_block() {
        let block = v3_valid_block();

        let v3 = ExecutionPayloadV3::from_block_slow(&block);

        let payload = OpExecutionPayload::V3(v3);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(
                block.header.parent_beacon_block_root.unwrap_or_default(),
            ),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
    }

    #[test]
    fn test_v3_non_empty_withdrawals() {
        let mut block = v3_valid_block();
        block.body.withdrawals = Some(vec![Withdrawal::default()].into());
        let withdrawals_root = alloy_consensus::proofs::calculate_withdrawals_root(
            &block.body.withdrawals.clone().unwrap_or_default(),
        );
        block.header.withdrawals_root = Some(withdrawals_root);

        let v3 = ExecutionPayloadV3::from_block_slow(&block);

        let payload = OpExecutionPayload::V3(v3);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(
                block.header.parent_beacon_block_root.unwrap_or_default(),
            ),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(
            handler.block_valid(&envelope),
            Err(BlockInvalidError::InvalidBlock(OpPayloadError::NonEmptyL1Withdrawals))
        ));
    }

    #[test]
    fn test_v3_gas_params() {
        let mut block = v3_valid_block();
        block.header.blob_gas_used = Some(1);

        let v3 = ExecutionPayloadV3::from_block_slow(&block);

        let payload = OpExecutionPayload::V3(v3);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(
                block.header.parent_beacon_block_root.unwrap_or_default(),
            ),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::BlobGasUsed)));

        block.header.blob_gas_used = Some(0);
        block.header.excess_blob_gas = Some(1);

        let v3 = ExecutionPayloadV3::from_block_slow(&block);

        let payload = OpExecutionPayload::V3(v3);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(
                block.header.parent_beacon_block_root.unwrap_or_default(),
            ),
        };

        assert!(matches!(handler.block_valid(&envelope), Err(BlockInvalidError::ExcessBlobGas)));
    }

    #[test]
    fn test_v4_block() {
        let block = v4_valid_block();

        let v3 = ExecutionPayloadV3::from_block_slow(&block);
        let v4 = OpExecutionPayloadV4::from_v3_with_withdrawals_root(
            v3,
            block.withdrawals_root.unwrap(),
        );

        let payload = OpExecutionPayload::V4(v4);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: Some(
                block.header.parent_beacon_block_root.unwrap_or_default(),
            ),
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
    }

    #[test]
    #[cfg(feature = "metrics")]
    fn test_metrics_instrumentation() {
        use crate::Metrics;

        Metrics::init();

        let block = v1_valid_block();
        let v1 = ExecutionPayloadV1::from_block_slow(&block);
        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());

        let mut invalid_block = v1_valid_block();
        invalid_block.header.timestamp = 0;

        let v1_invalid = ExecutionPayloadV1::from_block_slow(&invalid_block);
        let payload_invalid = OpExecutionPayload::V1(v1_invalid);
        let envelope_invalid = OpNetworkPayloadEnvelope {
            payload: payload_invalid,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        assert!(handler.block_valid(&envelope_invalid).is_err());
    }

    #[test]
    fn test_metrics_feature_gating() {
        let block = v1_valid_block();
        let v1 = ExecutionPayloadV1::from_block_slow(&block);
        let payload = OpExecutionPayload::V1(v1);
        let envelope = OpNetworkPayloadEnvelope {
            payload,
            signature: Signature::test_signature(),
            payload_hash: PayloadHash(B256::ZERO),
            parent_beacon_block_root: None,
        };

        let msg = envelope.payload_hash.signature_message(10);
        let signer = envelope.signature.recover_address_from_prehash(&msg).unwrap();
        let (_, unsafe_signer) = tokio::sync::watch::channel(signer);
        let mut handler = BlockHandler::new(
            RollupConfig { l2_chain_id: Chain::optimism_mainnet(), ..Default::default() },
            unsafe_signer,
        );

        assert!(handler.block_valid(&envelope).is_ok());
    }
}
