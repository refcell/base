//! Contains an online derivation pipeline.

use core::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use base_derive::{
    DerivationPipeline, EthereumDataSource, IndexedAttributesQueueStage, L2ChainProvider,
    OriginProvider, Pipeline, PipelineBuilder, PipelineErrorKind, PipelineResult,
    PolledAttributesQueueStage, ResetSignal, Signal, SignalReceiver, StatefulAttributesBuilder,
    StepResult,
};
use base_genesis::{L1ChainConfig, RollupConfig, SystemConfig};
use base_protocol::{BlockInfo, L2BlockInfo, OpAttributesWithParent};

use crate::{AlloyChainProvider, AlloyL2ChainProvider, OnlineBeaconClient, OnlineBlobProvider};

type OnlinePolledDerivationPipeline = DerivationPipeline<
    PolledAttributesQueueStage<
        OnlineDataProvider,
        AlloyChainProvider,
        AlloyL2ChainProvider,
        OnlineAttributesBuilder,
    >,
    AlloyL2ChainProvider,
>;

type OnlineManagedDerivationPipeline = DerivationPipeline<
    IndexedAttributesQueueStage<
        OnlineDataProvider,
        AlloyChainProvider,
        AlloyL2ChainProvider,
        OnlineAttributesBuilder,
    >,
    AlloyL2ChainProvider,
>;

type OnlineDataProvider =
    EthereumDataSource<AlloyChainProvider, OnlineBlobProvider<OnlineBeaconClient>>;

type OnlineAttributesBuilder = StatefulAttributesBuilder<AlloyChainProvider, AlloyL2ChainProvider>;

#[derive(Debug)]
pub enum OnlinePipeline {
    Polled(OnlinePolledDerivationPipeline),
    Managed(OnlineManagedDerivationPipeline),
}

impl OnlinePipeline {
    pub async fn new(
        cfg: Arc<RollupConfig>,
        l1_cfg: Arc<L1ChainConfig>,
        l2_safe_head: L2BlockInfo,
        l1_origin: BlockInfo,
        blob_provider: OnlineBlobProvider<OnlineBeaconClient>,
        chain_provider: AlloyChainProvider,
        mut l2_chain_provider: AlloyL2ChainProvider,
    ) -> PipelineResult<Self> {
        let mut pipeline = Self::new_polled(
            Arc::clone(&cfg),
            Arc::clone(&l1_cfg),
            blob_provider,
            chain_provider,
            l2_chain_provider.clone(),
        );

        pipeline
            .signal(
                ResetSignal {
                    l2_safe_head,
                    l1_origin,
                    system_config: l2_chain_provider
                        .system_config_by_number(l2_safe_head.block_info.number, Arc::clone(&cfg))
                        .await
                        .ok(),
                }
                .signal(),
            )
            .await?;

        Ok(pipeline)
    }

    pub fn new_polled(
        cfg: Arc<RollupConfig>,
        l1_cfg: Arc<L1ChainConfig>,
        blob_provider: OnlineBlobProvider<OnlineBeaconClient>,
        chain_provider: AlloyChainProvider,
        l2_chain_provider: AlloyL2ChainProvider,
    ) -> Self {
        let attributes = StatefulAttributesBuilder::new(
            Arc::clone(&cfg),
            l1_cfg,
            l2_chain_provider.clone(),
            chain_provider.clone(),
        );
        let dap = EthereumDataSource::new_from_parts(chain_provider.clone(), blob_provider, &cfg);

        let pipeline = PipelineBuilder::new()
            .rollup_config(cfg)
            .dap_source(dap)
            .l2_chain_provider(l2_chain_provider)
            .chain_provider(chain_provider)
            .builder(attributes)
            .origin(BlockInfo::default())
            .build_polled();

        Self::Polled(pipeline)
    }

    pub fn new_indexed(
        cfg: Arc<RollupConfig>,
        l1_cfg: Arc<L1ChainConfig>,
        blob_provider: OnlineBlobProvider<OnlineBeaconClient>,
        chain_provider: AlloyChainProvider,
        l2_chain_provider: AlloyL2ChainProvider,
    ) -> Self {
        let attributes = StatefulAttributesBuilder::new(
            Arc::clone(&cfg),
            l1_cfg,
            l2_chain_provider.clone(),
            chain_provider.clone(),
        );
        let dap = EthereumDataSource::new_from_parts(chain_provider.clone(), blob_provider, &cfg);

        let pipeline = PipelineBuilder::new()
            .rollup_config(cfg)
            .dap_source(dap)
            .l2_chain_provider(l2_chain_provider)
            .chain_provider(chain_provider)
            .builder(attributes)
            .origin(BlockInfo::default())
            .build_indexed();

        Self::Managed(pipeline)
    }
}

#[async_trait]
impl SignalReceiver for OnlinePipeline {
    async fn signal(&mut self, signal: Signal) -> PipelineResult<()> {
        match self {
            Self::Polled(pipeline) => pipeline.signal(signal).await,
            Self::Managed(pipeline) => pipeline.signal(signal).await,
        }
    }
}

impl OriginProvider for OnlinePipeline {
    fn origin(&self) -> Option<BlockInfo> {
        match self {
            Self::Polled(pipeline) => pipeline.origin(),
            Self::Managed(pipeline) => pipeline.origin(),
        }
    }
}

impl Iterator for OnlinePipeline {
    type Item = OpAttributesWithParent;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Polled(pipeline) => pipeline.next(),
            Self::Managed(pipeline) => pipeline.next(),
        }
    }
}

#[async_trait]
impl Pipeline for OnlinePipeline {
    fn peek(&self) -> Option<&OpAttributesWithParent> {
        match self {
            Self::Polled(pipeline) => pipeline.peek(),
            Self::Managed(pipeline) => pipeline.peek(),
        }
    }

    async fn step(&mut self, cursor: L2BlockInfo) -> StepResult {
        match self {
            Self::Polled(pipeline) => pipeline.step(cursor).await,
            Self::Managed(pipeline) => pipeline.step(cursor).await,
        }
    }

    fn rollup_config(&self) -> &RollupConfig {
        match self {
            Self::Polled(pipeline) => pipeline.rollup_config(),
            Self::Managed(pipeline) => pipeline.rollup_config(),
        }
    }

    async fn system_config_by_number(
        &mut self,
        number: u64,
    ) -> Result<SystemConfig, PipelineErrorKind> {
        match self {
            Self::Polled(pipeline) => pipeline.system_config_by_number(number).await,
            Self::Managed(pipeline) => pipeline.system_config_by_number(number).await,
        }
    }
}
