//! Contains error types for the [crate::SynchronizeTask].

use crate::{EngineTaskError, task_queue::tasks::task::EngineTaskErrorSeverity};
use alloy_rpc_types_engine::PayloadStatusEnum;
use alloy_transport::{RpcError, TransportErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynchronizeTaskError {
    #[error("Forkchoice update engine api call failed due to an RPC error: {0}")]
    ForkchoiceUpdateFailed(RpcError<TransportErrorKind>),
    #[error("Invalid forkchoice state: unsafe head {0} is ahead of finalized head {1}")]
    FinalizedAheadOfUnsafe(u64, u64),
    #[error("Invalid forkchoice state")]
    InvalidForkchoiceState,
    #[error("Unexpected payload status: {0}")]
    UnexpectedPayloadStatus(PayloadStatusEnum),
}

impl EngineTaskError for SynchronizeTaskError {
    fn severity(&self) -> EngineTaskErrorSeverity {
        match self {
            Self::FinalizedAheadOfUnsafe(_, _) => EngineTaskErrorSeverity::Critical,
            Self::ForkchoiceUpdateFailed(_) => EngineTaskErrorSeverity::Temporary,
            Self::UnexpectedPayloadStatus(_) => EngineTaskErrorSeverity::Temporary,
            Self::InvalidForkchoiceState => EngineTaskErrorSeverity::Reset,
        }
    }
}
