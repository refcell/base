//! A task for importing a block that has already been started.
use super::SealTaskError;
use crate::{
    EngineClient, EngineGetPayloadVersion, EngineState, EngineTaskExt, InsertTask,
    InsertTaskError::{self},
    task_queue::build_and_seal,
};
use alloy_rpc_types_engine::{ExecutionPayload, PayloadId};
use async_trait::async_trait;
use derive_more::Constructor;
use base_genesis::RollupConfig;
use base_protocol::{L2BlockInfo, OpAttributesWithParent};
use base_alloy_rpc_types_engine::{OpExecutionPayload, OpExecutionPayloadEnvelope};
use std::{sync::Arc, time::Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Constructor)]
pub struct SealTask<EngineClient_: EngineClient> {
    pub engine: Arc<EngineClient_>,
    pub cfg: Arc<RollupConfig>,
    pub payload_id: PayloadId,
    pub attributes: OpAttributesWithParent,
    pub is_attributes_derived: bool,
    pub result_tx: Option<mpsc::Sender<Result<OpExecutionPayloadEnvelope, SealTaskError>>>,
}

impl<EngineClient_: EngineClient> SealTask<EngineClient_> {
    async fn seal_payload(
        &self,
        cfg: &RollupConfig,
        engine: &EngineClient_,
        payload_id: PayloadId,
        payload_attrs: OpAttributesWithParent,
    ) -> Result<OpExecutionPayloadEnvelope, SealTaskError> {
        let payload_timestamp = payload_attrs.attributes().payload_attributes.timestamp;

        debug!(
            target: "engine",
            payload_id = payload_id.to_string(),
            l2_time = payload_timestamp,
            "Sealing payload"
        );

        let get_payload_version = EngineGetPayloadVersion::from_cfg(cfg, payload_timestamp);
        let payload_envelope = match get_payload_version {
            EngineGetPayloadVersion::V4 => {
                let payload = engine.get_payload_v4(payload_id).await.map_err(|e| {
                    error!(target: "engine", "Payload fetch failed: {e}");
                    SealTaskError::GetPayloadFailed(e)
                })?;

                OpExecutionPayloadEnvelope {
                    parent_beacon_block_root: Some(payload.parent_beacon_block_root),
                    execution_payload: OpExecutionPayload::V4(payload.execution_payload),
                }
            }
            EngineGetPayloadVersion::V3 => {
                let payload = engine.get_payload_v3(payload_id).await.map_err(|e| {
                    error!(target: "engine", "Payload fetch failed: {e}");
                    SealTaskError::GetPayloadFailed(e)
                })?;

                OpExecutionPayloadEnvelope {
                    parent_beacon_block_root: Some(payload.parent_beacon_block_root),
                    execution_payload: OpExecutionPayload::V3(payload.execution_payload),
                }
            }
            EngineGetPayloadVersion::V2 => {
                let payload = engine.get_payload_v2(payload_id).await.map_err(|e| {
                    error!(target: "engine", "Payload fetch failed: {e}");
                    SealTaskError::GetPayloadFailed(e)
                })?;

                OpExecutionPayloadEnvelope {
                    parent_beacon_block_root: None,
                    execution_payload: match payload.execution_payload.into_payload() {
                        ExecutionPayload::V1(payload) => OpExecutionPayload::V1(payload),
                        ExecutionPayload::V2(payload) => OpExecutionPayload::V2(payload),
                        _ => unreachable!("the response should be a V1 or V2 payload"),
                    },
                }
            }
        };

        Ok(payload_envelope)
    }

    async fn insert_payload(
        &self,
        state: &mut EngineState,
        new_payload: OpExecutionPayloadEnvelope,
    ) -> Result<(), SealTaskError> {
        match InsertTask::new(
            Arc::clone(&self.engine),
            self.cfg.clone(),
            new_payload.clone(),
            self.is_attributes_derived,
        )
        .execute(state)
        .await
        {
            Err(InsertTaskError::UnexpectedPayloadStatus(e))
                if self.attributes.is_deposits_only() =>
            {
                error!(target: "engine", error = ?e, "Critical: Deposit-only payload import failed");
                return Err(SealTaskError::DepositOnlyPayloadFailed);
            }
            Err(InsertTaskError::UnexpectedPayloadStatus(e))
                if self.cfg.is_holocene_active(
                    self.attributes.attributes().payload_attributes.timestamp,
                ) =>
            {
                warn!(target: "engine", error = ?e, "Re-attempting payload import with deposits only.");

                let deposits_only_attrs = self.attributes.as_deposits_only();

                return match build_and_seal(
                    state,
                    self.engine.clone(),
                    self.cfg.clone(),
                    deposits_only_attrs.clone(),
                    self.is_attributes_derived,
                )
                .await
                {
                    Ok(_) => {
                        info!(target: "engine", "Successfully imported deposits-only payload");
                        Err(SealTaskError::HoloceneInvalidFlush)
                    }
                    Err(_) => Err(SealTaskError::DepositOnlyPayloadReattemptFailed),
                }
            }
            Err(e) => {
                error!(target: "engine", "Payload import failed: {e}");
                return Err(Box::new(e).into());
            }
            Ok(_) => {
                info!(target: "engine", "Successfully imported payload")
            }
        }

        Ok(())
    }

    async fn seal_and_canonicalize_block(
        &self,
        state: &mut EngineState,
    ) -> Result<OpExecutionPayloadEnvelope, SealTaskError> {
        let block_import_start_time = Instant::now();
        let new_payload = self
            .seal_payload(&self.cfg, &self.engine, self.payload_id, self.attributes.clone())
            .await?;

        let new_block_ref = L2BlockInfo::from_payload_and_genesis(
            new_payload.execution_payload.clone(),
            self.attributes.attributes().payload_attributes.parent_beacon_block_root,
            &self.cfg.genesis,
        )
        .map_err(SealTaskError::FromBlock)?;

        self.insert_payload(state, new_payload.clone()).await?;

        let block_import_duration = block_import_start_time.elapsed();

        info!(
            target: "engine",
            l2_number = new_block_ref.block_info.number,
            l2_time = new_block_ref.block_info.timestamp,
            block_import_duration = ?block_import_duration,
            "Built and imported new {} block",
            if self.is_attributes_derived { "safe" } else { "unsafe" },
        );

        Ok(new_payload)
    }

    async fn send_channel_result_or_get_error(
        &self,
        res: Result<OpExecutionPayloadEnvelope, SealTaskError>,
    ) -> Result<(), SealTaskError> {
        if let Some(tx) = &self.result_tx {
            tx.send(res).await.map_err(|e| SealTaskError::MpscSend(Box::new(e)))?;
        } else if let Err(x) = res {
            return Err(x)
        }

        Ok(())
    }
}

#[async_trait]
impl<EngineClient_: EngineClient> EngineTaskExt for SealTask<EngineClient_> {
    type Output = ();

    type Error = SealTaskError;

    async fn execute(&self, state: &mut EngineState) -> Result<(), SealTaskError> {
        debug!(
            target: "engine",
            txs = self.attributes.attributes().transactions.as_ref().map_or(0, |txs| txs.len()),
            is_deposits = self.attributes.is_deposits_only(),
            "Starting new seal job"
        );

        let unsafe_block_info = state.sync_state.unsafe_head().block_info;
        let parent_block_info = self.attributes.parent.block_info;

        let res = if unsafe_block_info.hash != parent_block_info.hash ||
            unsafe_block_info.number != parent_block_info.number
        {
            info!(
                target: "engine",
                unsafe_block_info = ?unsafe_block_info,
                parent_block_info = ?parent_block_info,
                "Seal attributes parent does not match unsafe head, returning rebuild error"
            );
            Err(SealTaskError::UnsafeHeadChangedSinceBuild)
        } else {
            self.seal_and_canonicalize_block(state).await
        };

        self.send_channel_result_or_get_error(res).await?;

        Ok(())
    }
}
