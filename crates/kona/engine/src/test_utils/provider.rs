use alloy_network::Ethereum;
use alloy_provider::Provider;
use async_trait::async_trait;
use base_alloy_network::Optimism;

#[derive(Debug, Clone)]
pub struct MockL1Provider;

#[async_trait]
impl Provider<Ethereum> for MockL1Provider {
    fn root(&self) -> &alloy_provider::RootProvider<Ethereum> {
        unimplemented!("MockL1Provider does not support root()")
    }
}

#[derive(Debug, Clone)]
pub struct MockL2Provider;

#[async_trait]
impl Provider<Optimism> for MockL2Provider {
    fn root(&self) -> &alloy_provider::RootProvider<Optimism> {
        unimplemented!("MockL2Provider does not support root()")
    }
}
