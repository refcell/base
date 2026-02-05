//! Contains error types for the [`crate::ConsolidateTask`].

use thiserror::Error;

use crate::{
    BuildTaskError, EngineTaskError, SealTaskError, SynchronizeTaskError,
    task_queue::tasks::{BuildAndSealError, task::EngineTaskErrorSeverity},
};

#[derive(Debug, Error)]
pub enum ConsolidateTaskError {
    #[error("Unsafe L2 block is missing {0}")]
    MissingUnsafeL2Block(u64),
    #[error("Failed to fetch the unsafe L2 block")]
    FailedToFetchUnsafeL2Block,
    #[error(transparent)]
    BuildTaskFailed(#[from] BuildTaskError),
    #[error(transparent)]
    SealTaskFailed(#[from] SealTaskError),
    #[error(transparent)]
    ForkchoiceUpdateFailed(#[from] SynchronizeTaskError),
}

impl From<BuildAndSealError> for ConsolidateTaskError {
    fn from(err: BuildAndSealError) -> Self {
        match err {
            BuildAndSealError::Build(e) => Self::BuildTaskFailed(e),
            BuildAndSealError::Seal(e) => Self::SealTaskFailed(e),
        }
    }
}

impl EngineTaskError for ConsolidateTaskError {
    fn severity(&self) -> EngineTaskErrorSeverity {
        match self {
            Self::MissingUnsafeL2Block(_) => EngineTaskErrorSeverity::Reset,
            Self::FailedToFetchUnsafeL2Block => EngineTaskErrorSeverity::Temporary,
            Self::BuildTaskFailed(inner) => inner.severity(),
            Self::SealTaskFailed(inner) => inner.severity(),
            Self::ForkchoiceUpdateFailed(inner) => inner.severity(),
        }
    }
}
