//! The [`Engine`] is a task queue that receives and executes [`EngineTask`]s.

use std::{collections::BinaryHeap, sync::Arc};

use alloy_rpc_types_eth::Transaction;
use base_alloy_consensus::OpTxEnvelope;
use base_genesis::{RollupConfig, SystemConfig};
use base_protocol::{BlockInfo, L2BlockInfo, OpBlockConversionError, to_system_config};
use thiserror::Error;
use tokio::sync::watch::Sender;

use super::EngineTaskExt;
use crate::{
    EngineClient, EngineState, EngineSyncStateUpdate, EngineTask, EngineTaskError,
    EngineTaskErrorSeverity, Metrics, SyncStartError, SynchronizeTask, SynchronizeTaskError,
    find_starting_forkchoice, task_queue::EngineTaskErrors,
};

/// The [`Engine`] task queue.
#[derive(Debug)]
pub struct Engine<EngineClient_: EngineClient> {
    state: EngineState,
    state_sender: Sender<EngineState>,
    task_queue_length: Sender<usize>,
    tasks: BinaryHeap<EngineTask<EngineClient_>>,
}

impl<EngineClient_: EngineClient> Engine<EngineClient_> {
    /// Creates a new [`Engine`] with an empty task queue and the passed initial [`EngineState`].
    pub fn new(
        initial_state: EngineState,
        state_sender: Sender<EngineState>,
        task_queue_length: Sender<usize>,
    ) -> Self {
        Self { state: initial_state, state_sender, task_queue_length, tasks: BinaryHeap::default() }
    }

    /// Returns a reference to the inner [`EngineState`].
    pub const fn state(&self) -> &EngineState {
        &self.state
    }

    /// Returns a receiver that can be used to listen to engine state updates.
    pub fn state_subscribe(&self) -> tokio::sync::watch::Receiver<EngineState> {
        self.state_sender.subscribe()
    }

    /// Returns a receiver that can be used to listen to engine queue length updates.
    pub fn queue_length_subscribe(&self) -> tokio::sync::watch::Receiver<usize> {
        self.task_queue_length.subscribe()
    }

    /// Enqueues a new [`EngineTask`] for execution.
    pub fn enqueue(&mut self, task: EngineTask<EngineClient_>) {
        self.tasks.push(task);
        self.task_queue_length.send_replace(self.tasks.len());
    }

    /// Resets the engine by finding a plausible sync starting point via
    /// [`find_starting_forkchoice`].
    pub async fn reset(
        &mut self,
        client: Arc<EngineClient_>,
        config: Arc<RollupConfig>,
    ) -> Result<(L2BlockInfo, BlockInfo, SystemConfig), EngineResetError> {
        self.clear();

        let mut start = find_starting_forkchoice(&config, client.as_ref()).await?;

        while let Err(err) = SynchronizeTask::new(
            client.clone(),
            config.clone(),
            EngineSyncStateUpdate {
                unsafe_head: Some(start.un_safe),
                cross_unsafe_head: Some(start.un_safe),
                local_safe_head: Some(start.safe),
                safe_head: Some(start.safe),
                finalized_head: Some(start.finalized),
            },
        )
        .execute(&mut self.state)
        .await
        {
            match err.severity() {
                EngineTaskErrorSeverity::Temporary
                | EngineTaskErrorSeverity::Flush
                | EngineTaskErrorSeverity::Reset => {
                    warn!(target: "engine", ?err, "Forkchoice update failed during reset. Trying again...");
                    start = find_starting_forkchoice(&config, client.as_ref()).await?;
                }
                EngineTaskErrorSeverity::Critical => {
                    return Err(EngineResetError::Forkchoice(err));
                }
            }
        }

        let origin_block = start
            .safe
            .l1_origin
            .number
            .saturating_sub(config.channel_timeout(start.safe.block_info.timestamp));
        let l1_origin_info: BlockInfo = client
            .get_l1_block(origin_block.into())
            .await
            .map_err(SyncStartError::RpcError)?
            .ok_or(SyncStartError::BlockNotFound(origin_block.into()))?
            .into_consensus()
            .into();
        let l2_safe_block = client
            .get_l2_block(start.safe.block_info.hash.into())
            .full()
            .await
            .map_err(SyncStartError::RpcError)?
            .ok_or(SyncStartError::BlockNotFound(origin_block.into()))?
            .into_consensus()
            .map_transactions(|t| <Transaction<OpTxEnvelope> as Clone>::clone(&t).into_inner());
        let system_config = to_system_config(&l2_safe_block, &config)?;

        base_macros::inc!(counter, Metrics::ENGINE_RESET_COUNT);

        Ok((start.safe, l1_origin_info, system_config))
    }

    /// Clears the task queue.
    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    /// Attempts to drain the queue by executing all [`EngineTask`]s in-order.
    pub async fn drain(&mut self) -> Result<(), EngineTaskErrors> {
        while let Some(task) = self.tasks.peek() {
            task.execute(&mut self.state).await?;
            self.state_sender.send_replace(self.state);
            self.tasks.pop();
            self.task_queue_length.send_replace(self.tasks.len());
        }

        Ok(())
    }
}

/// An error occurred while attempting to reset the [`Engine`].
#[derive(Debug, Error)]
pub enum EngineResetError {
    /// An error that occurred while updating the forkchoice state.
    #[error(transparent)]
    Forkchoice(#[from] SynchronizeTaskError),
    /// An error occurred while traversing the L1 for the sync starting point.
    #[error(transparent)]
    SyncStart(#[from] SyncStartError),
    /// An error occurred while constructing the SystemConfig for the new safe head.
    #[error(transparent)]
    SystemConfigConversion(#[from] OpBlockConversionError),
}
