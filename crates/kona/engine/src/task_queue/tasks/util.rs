//! Utility functions for task execution.

use std::sync::Arc;

use base_genesis::RollupConfig;
use base_protocol::OpAttributesWithParent;

use super::{BuildTask, BuildTaskError, EngineTaskExt, SealTask, SealTaskError};
use crate::{EngineClient, EngineState};

/// Error type for build and seal operations.
#[derive(Debug, thiserror::Error)]
pub(in crate::task_queue) enum BuildAndSealError {
    /// An error occurred during the build phase.
    #[error(transparent)]
    Build(#[from] BuildTaskError),
    /// An error occurred during the seal phase.
    #[error(transparent)]
    Seal(#[from] SealTaskError),
}

/// Builds and seals a payload in sequence.
pub(in crate::task_queue) async fn build_and_seal<EngineClient_: EngineClient>(
    state: &mut EngineState,
    engine: Arc<EngineClient_>,
    cfg: Arc<RollupConfig>,
    attributes: OpAttributesWithParent,
    is_attributes_derived: bool,
) -> Result<(), BuildAndSealError> {
    let payload_id = BuildTask::new(engine.clone(), cfg.clone(), attributes.clone(), None)
        .execute(state)
        .await?;

    SealTask::new(engine, cfg, payload_id, attributes, is_attributes_derived, None)
        .execute(state)
        .await?;

    Ok(())
}
