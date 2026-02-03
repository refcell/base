//! Contains error types for the [crate::SynchronizeTask].

use crate::{EngineTaskError, InsertTaskError, task_queue::tasks::task::EngineTaskErrorSeverity};
use alloy_transport::{RpcError, TransportErrorKind};
use base_alloy_rpc_types_engine::OpExecutionPayloadEnvelope;
use base_protocol::FromBlockError;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum SealTaskError {
    #[error(transparent)]
    PayloadInsertionFailed(#[from] Box<InsertTaskError>),
    #[error(transparent)]
    GetPayloadFailed(RpcError<TransportErrorKind>),
    #[error("Deposit-only payload failed to import")]
    DepositOnlyPayloadFailed,
    #[error("Failed to re-attempt payload import with deposit-only payload")]
    DepositOnlyPayloadReattemptFailed,
    #[error("Invalid payload, must flush post-holocene")]
    HoloceneInvalidFlush,
    #[error(transparent)]
    FromBlock(#[from] FromBlockError),
    #[error(transparent)]
    MpscSend(
        #[from] Box<mpsc::error::SendError<Result<OpExecutionPayloadEnvelope, SealTaskError>>>,
    ),
    #[error("The clock went backwards")]
    ClockWentBackwards,
    #[error("Unsafe head changed between build and seal")]
    UnsafeHeadChangedSinceBuild,
}

impl EngineTaskError for SealTaskError {
    fn severity(&self) -> EngineTaskErrorSeverity {
        match self {
            Self::PayloadInsertionFailed(inner) => inner.severity(),
            Self::GetPayloadFailed(_) => EngineTaskErrorSeverity::Temporary,
            Self::HoloceneInvalidFlush => EngineTaskErrorSeverity::Flush,
            Self::DepositOnlyPayloadReattemptFailed => EngineTaskErrorSeverity::Critical,
            Self::DepositOnlyPayloadFailed => EngineTaskErrorSeverity::Critical,
            Self::FromBlock(_) => EngineTaskErrorSeverity::Critical,
            Self::MpscSend(_) => EngineTaskErrorSeverity::Critical,
            Self::ClockWentBackwards => EngineTaskErrorSeverity::Critical,
            Self::UnsafeHeadChangedSinceBuild => EngineTaskErrorSeverity::Critical,
        }
    }
}
