//! Contains error types for the [crate::SynchronizeTask].

use crate::{EngineTaskError, task_queue::tasks::task::EngineTaskErrorSeverity};
use alloy_rpc_types_engine::{PayloadId, PayloadStatusEnum};
use alloy_transport::{RpcError, TransportErrorKind};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum EngineBuildError {
    #[error("Finalized head is ahead of unsafe head")]
    FinalizedAheadOfUnsafe(u64, u64),
    #[error("Failed to build payload attributes in the engine. Forkchoice RPC error: {0}")]
    AttributesInsertionFailed(#[from] RpcError<TransportErrorKind>),
    #[error("The inserted payload is invalid: {0}")]
    InvalidPayload(String),
    #[error("The inserted payload status is unexpected: {0}")]
    UnexpectedPayloadStatus(PayloadStatusEnum),
    #[error("The inserted payload ID is missing")]
    MissingPayloadId,
    #[error("The engine is syncing")]
    EngineSyncing,
}

#[derive(Debug, Error)]
pub enum BuildTaskError {
    #[error("An error occurred when building the payload attributes to the engine.")]
    EngineBuildError(EngineBuildError),
    #[error(transparent)]
    MpscSend(#[from] Box<mpsc::error::SendError<PayloadId>>),
}

impl EngineTaskError for BuildTaskError {
    fn severity(&self) -> EngineTaskErrorSeverity {
        match self {
            Self::EngineBuildError(EngineBuildError::FinalizedAheadOfUnsafe(_, _)) => {
                EngineTaskErrorSeverity::Critical
            }
            Self::EngineBuildError(EngineBuildError::AttributesInsertionFailed(_)) => {
                EngineTaskErrorSeverity::Temporary
            }
            Self::EngineBuildError(EngineBuildError::InvalidPayload(_)) => {
                EngineTaskErrorSeverity::Temporary
            }
            Self::EngineBuildError(EngineBuildError::UnexpectedPayloadStatus(_)) => {
                EngineTaskErrorSeverity::Temporary
            }
            Self::EngineBuildError(EngineBuildError::MissingPayloadId) => {
                EngineTaskErrorSeverity::Temporary
            }
            Self::EngineBuildError(EngineBuildError::EngineSyncing) => {
                EngineTaskErrorSeverity::Temporary
            }
            Self::MpscSend(_) => EngineTaskErrorSeverity::Critical,
        }
    }
}
