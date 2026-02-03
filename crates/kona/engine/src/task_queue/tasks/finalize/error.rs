//! Contains error types for the [crate::FinalizeTask].

use crate::{
    EngineTaskError, SynchronizeTaskError, task_queue::tasks::task::EngineTaskErrorSeverity,
};
use alloy_transport::{RpcError, TransportErrorKind};
use base_protocol::FromBlockError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinalizeTaskError {
    #[error("Attempted to finalize a block that is not yet safe")]
    BlockNotSafe,
    #[error("The block to finalize was not found: Number {0}")]
    BlockNotFound(u64),
    #[error(transparent)]
    FromBlock(#[from] FromBlockError),
    #[error(transparent)]
    TransportError(#[from] RpcError<TransportErrorKind>),
    #[error(transparent)]
    ForkchoiceUpdateFailed(#[from] SynchronizeTaskError),
}

impl EngineTaskError for FinalizeTaskError {
    fn severity(&self) -> EngineTaskErrorSeverity {
        match self {
            Self::BlockNotSafe => EngineTaskErrorSeverity::Critical,
            Self::BlockNotFound(_) => EngineTaskErrorSeverity::Critical,
            Self::FromBlock(_) => EngineTaskErrorSeverity::Critical,
            Self::TransportError(_) => EngineTaskErrorSeverity::Temporary,
            Self::ForkchoiceUpdateFailed(inner) => inner.severity(),
        }
    }
}
