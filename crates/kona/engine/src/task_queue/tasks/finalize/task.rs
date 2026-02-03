//! A task for finalizing an L2 block.

use crate::{
    EngineClient, EngineState, EngineTaskExt, FinalizeTaskError, SynchronizeTask,
    state::EngineSyncStateUpdate,
};
use async_trait::async_trait;
use derive_more::Constructor;
use base_genesis::RollupConfig;
use base_protocol::L2BlockInfo;
use std::{sync::Arc, time::Instant};

#[derive(Debug, Clone, Constructor)]
pub struct FinalizeTask<EngineClient_: EngineClient> {
    pub client: Arc<EngineClient_>,
    pub cfg: Arc<RollupConfig>,
    pub block_number: u64,
}

#[async_trait]
impl<EngineClient_: EngineClient> EngineTaskExt for FinalizeTask<EngineClient_> {
    type Output = ();

    type Error = FinalizeTaskError;

    async fn execute(&self, state: &mut EngineState) -> Result<(), FinalizeTaskError> {
        if state.sync_state.safe_head().block_info.number < self.block_number {
            return Err(FinalizeTaskError::BlockNotSafe);
        }

        let block_fetch_start = Instant::now();
        let block = self
            .client
            .get_l2_block(self.block_number.into())
            .full()
            .await
            .map_err(FinalizeTaskError::TransportError)?
            .ok_or(FinalizeTaskError::BlockNotFound(self.block_number))?
            .into_consensus();
        let block_info = L2BlockInfo::from_block_and_genesis(&block, &self.client.cfg().genesis)
            .map_err(FinalizeTaskError::FromBlock)?;
        let block_fetch_duration = block_fetch_start.elapsed();

        let fcu_start = Instant::now();
        SynchronizeTask::new(
            self.client.clone(),
            self.cfg.clone(),
            EngineSyncStateUpdate { finalized_head: Some(block_info), ..Default::default() },
        )
        .execute(state)
        .await?;
        let fcu_duration = fcu_start.elapsed();

        info!(
            target: "engine",
            hash = %block_info.block_info.hash,
            number = block_info.block_info.number,
            ?block_fetch_duration,
            ?fcu_duration,
            "Updated finalized head"
        );

        Ok(())
    }
}
