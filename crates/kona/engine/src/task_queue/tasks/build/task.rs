//! A task for building a new block and importing it.
use std::{sync::Arc, time::Instant};

use alloy_rpc_types_engine::{PayloadId, PayloadStatusEnum};
use async_trait::async_trait;
use base_genesis::RollupConfig;
use base_protocol::OpAttributesWithParent;
use derive_more::Constructor;
use tokio::sync::mpsc;

use super::BuildTaskError;
use crate::{
    EngineClient, EngineForkchoiceVersion, EngineState, EngineTaskExt,
    state::EngineSyncStateUpdate, task_queue::tasks::build::error::EngineBuildError,
};

#[derive(Debug, Clone, Constructor)]
pub struct BuildTask<EngineClient_: EngineClient> {
    pub engine: Arc<EngineClient_>,
    pub cfg: Arc<RollupConfig>,
    pub attributes: OpAttributesWithParent,
    pub payload_id_tx: Option<mpsc::Sender<PayloadId>>,
}

impl<EngineClient_: EngineClient> BuildTask<EngineClient_> {
    fn validate_forkchoice_status(status: PayloadStatusEnum) -> Result<(), BuildTaskError> {
        match status {
            PayloadStatusEnum::Valid => Ok(()),
            PayloadStatusEnum::Invalid { validation_error } => {
                error!(target: "engine_builder", "Forkchoice update failed: {}", validation_error);
                Err(BuildTaskError::EngineBuildError(EngineBuildError::InvalidPayload(
                    validation_error,
                )))
            }
            PayloadStatusEnum::Syncing => {
                warn!(target: "engine_builder", "Forkchoice update failed temporarily: EL is syncing");
                Err(BuildTaskError::EngineBuildError(EngineBuildError::EngineSyncing))
            }
            PayloadStatusEnum::Accepted => Err(BuildTaskError::EngineBuildError(
                EngineBuildError::UnexpectedPayloadStatus(status),
            )),
        }
    }

    pub(super) async fn start_build(
        &self,
        state: &EngineState,
        engine_client: &EngineClient_,
        attributes_envelope: OpAttributesWithParent,
    ) -> Result<PayloadId, BuildTaskError> {
        if state.sync_state.unsafe_head().block_info.number
            < state.sync_state.finalized_head().block_info.number
        {
            return Err(BuildTaskError::EngineBuildError(
                EngineBuildError::FinalizedAheadOfUnsafe(
                    state.sync_state.unsafe_head().block_info.number,
                    state.sync_state.finalized_head().block_info.number,
                ),
            ));
        }

        let new_forkchoice = state
            .sync_state
            .apply_update(EngineSyncStateUpdate {
                unsafe_head: Some(attributes_envelope.parent),
                ..Default::default()
            })
            .create_forkchoice_state();

        let forkchoice_version = EngineForkchoiceVersion::from_cfg(
            &self.cfg,
            attributes_envelope.attributes.payload_attributes.timestamp,
        );
        let update = match forkchoice_version {
            EngineForkchoiceVersion::V3 => {
                engine_client
                    .fork_choice_updated_v3(new_forkchoice, Some(attributes_envelope.attributes))
                    .await
            }
            EngineForkchoiceVersion::V2 => {
                engine_client
                    .fork_choice_updated_v2(new_forkchoice, Some(attributes_envelope.attributes))
                    .await
            }
        }
        .map_err(|e| {
            error!(target: "engine_builder", "Forkchoice update failed: {}", e);
            BuildTaskError::EngineBuildError(EngineBuildError::AttributesInsertionFailed(e))
        })?;

        Self::validate_forkchoice_status(update.payload_status.status)?;

        debug!(
            target: "engine_builder",
            unsafe_hash = new_forkchoice.head_block_hash.to_string(),
            safe_hash = new_forkchoice.safe_block_hash.to_string(),
            finalized_hash = new_forkchoice.finalized_block_hash.to_string(),
            "Forkchoice update with attributes successful"
        );

        update
            .payload_id
            .ok_or(BuildTaskError::EngineBuildError(EngineBuildError::MissingPayloadId))
    }
}

#[async_trait]
impl<EngineClient_: EngineClient> EngineTaskExt for BuildTask<EngineClient_> {
    type Output = PayloadId;

    type Error = BuildTaskError;

    async fn execute(&self, state: &mut EngineState) -> Result<PayloadId, BuildTaskError> {
        debug!(
            target: "engine_builder",
            txs = self.attributes.attributes().transactions.as_ref().map_or(0, |txs| txs.len()),
            is_deposits = self.attributes.is_deposits_only(),
            "Starting new build job"
        );

        let fcu_start_time = Instant::now();
        let payload_id = self.start_build(state, &self.engine, self.attributes.clone()).await?;
        let fcu_duration = fcu_start_time.elapsed();

        info!(
            target: "engine_builder",
            fcu_duration = ?fcu_duration,
            "block build started"
        );

        if let Some(tx) = &self.payload_id_tx {
            tx.send(payload_id).await.map_err(Box::new)?;
        }

        Ok(payload_id)
    }
}
