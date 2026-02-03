//! A task for the `engine_forkchoiceUpdated` method, with no attributes.

use crate::{
    EngineClient, EngineState, EngineTaskExt, SynchronizeTaskError, state::EngineSyncStateUpdate,
};
use alloy_rpc_types_engine::{INVALID_FORK_CHOICE_STATE_ERROR, PayloadStatusEnum};
use async_trait::async_trait;
use derive_more::Constructor;
use base_genesis::RollupConfig;
use std::sync::Arc;
use tokio::time::Instant;

#[derive(Debug, Clone, Constructor)]
pub struct SynchronizeTask<EngineClient_: EngineClient> {
    pub client: Arc<EngineClient_>,
    pub rollup: Arc<RollupConfig>,
    pub state_update: EngineSyncStateUpdate,
}

impl<EngineClient_: EngineClient> SynchronizeTask<EngineClient_> {
    fn check_forkchoice_updated_status(
        &self,
        state: &mut EngineState,
        status: &PayloadStatusEnum,
    ) -> Result<(), SynchronizeTaskError> {
        match status {
            PayloadStatusEnum::Valid => {
                if !state.el_sync_finished {
                    info!(
                        target: "engine",
                        "Finished execution layer sync."
                    );
                    state.el_sync_finished = true;
                }

                Ok(())
            }
            PayloadStatusEnum::Syncing => {
                debug!(target: "engine", "Attempting to update forkchoice state while EL syncing");
                Ok(())
            }
            s => {
                Err(SynchronizeTaskError::UnexpectedPayloadStatus(s.clone()))
            }
        }
    }
}

#[async_trait]
impl<EngineClient_: EngineClient> EngineTaskExt for SynchronizeTask<EngineClient_> {
    type Output = ();
    type Error = SynchronizeTaskError;

    async fn execute(&self, state: &mut EngineState) -> Result<Self::Output, SynchronizeTaskError> {
        let new_sync_state = state.sync_state.apply_update(self.state_update);

        if state.sync_state != Default::default() && state.sync_state == new_sync_state {
            debug!(target: "engine", ?new_sync_state, "No forkchoice update needed");
            return Ok(());
        }

        if new_sync_state.unsafe_head().block_info.number <
            new_sync_state.finalized_head().block_info.number
        {
            return Err(SynchronizeTaskError::FinalizedAheadOfUnsafe(
                new_sync_state.unsafe_head().block_info.number,
                new_sync_state.finalized_head().block_info.number,
            ));
        }

        let fcu_time_start = Instant::now();

        let forkchoice = new_sync_state.create_forkchoice_state();

        let response = self.client.fork_choice_updated_v3(forkchoice, None).await;

        let valid_response = response.map_err(|e| {
            let error = e
                .as_error_resp()
                .and_then(|e| {
                    (e.code == INVALID_FORK_CHOICE_STATE_ERROR as i64)
                        .then_some(SynchronizeTaskError::InvalidForkchoiceState)
                })
                .unwrap_or_else(|| SynchronizeTaskError::ForkchoiceUpdateFailed(e));

            debug!(target: "engine", error = ?error, "Unexpected forkchoice update error");

            error
        })?;

        self.check_forkchoice_updated_status(state, &valid_response.payload_status.status)?;

        state.sync_state = new_sync_state;

        let fcu_duration = fcu_time_start.elapsed();
        debug!(
            target: "engine",
            fcu_duration = ?fcu_duration,
            forkchoice = ?forkchoice,
            response = ?valid_response,
            "Forkchoice updated"
        );

        Ok(())
    }
}
