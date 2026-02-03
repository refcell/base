#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(missing_docs)]

mod metrics;
pub use beacon_client::BeaconClientError;
pub use metrics::Metrics;

mod beacon_client;
pub use beacon_client::{
    APIConfigResponse, APIGenesisResponse, BeaconClient, OnlineBeaconClient, ReducedConfigData,
    ReducedGenesisData,
};

mod blobs;
pub use blobs::{BoxedBlobWithIndex, OnlineBlobProvider};

mod chain_provider;
pub use chain_provider::{AlloyChainProvider, AlloyChainProviderError};

mod l2_chain_provider;
pub use l2_chain_provider::{AlloyL2ChainProvider, AlloyL2ChainProviderError};

mod pipeline;
pub use pipeline::OnlinePipeline;
