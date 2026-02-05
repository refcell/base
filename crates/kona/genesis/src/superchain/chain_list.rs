//! List of OP Stack chains.

use alloc::{string::String, vec::Vec};

use alloy_chains::Chain as AlloyChain;

/// List of Chains.
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
pub struct ChainList {
    /// List of Chains.
    pub chains: Vec<Chain>,
}

impl ChainList {
    /// Fetch a [Chain] by its identifier.
    pub fn get_chain_by_ident(&self, identifier: &str) -> Option<&Chain> {
        self.chains.iter().find(|c| c.identifier.eq_ignore_ascii_case(identifier))
    }

    /// Returns all available [Chain] identifiers.
    pub fn chain_idents(&self) -> Vec<String> {
        self.chains.iter().map(|c| c.identifier.clone()).collect()
    }

    /// Fetch a [Chain] by its chain id.
    pub fn get_chain_by_id(&self, chain_id: u64) -> Option<&Chain> {
        self.chains.iter().find(|c| c.chain_id == chain_id)
    }

    /// Fetch a [Chain] by the corresponding [AlloyChain]
    pub fn get_chain_by_alloy_ident(&self, chain: &AlloyChain) -> Option<&Chain> {
        self.get_chain_by_id(chain.id())
    }

    /// Returns the number of chains.
    pub const fn len(&self) -> usize {
        self.chains.len()
    }

    /// Returns true if the list is empty.
    pub const fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }
}

/// A Chain Definition.
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "tabled", derive(tabled::Tabled))]
pub struct Chain {
    /// The name of the chain.
    pub name: String,
    /// Chain identifier.
    pub identifier: String,
    /// Chain ID.
    pub chain_id: u64,
    /// List of RPC Endpoints.
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub rpc: Vec<String>,
    /// List of Explorer Endpoints.
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub explorers: Vec<String>,
    /// The Superchain Level.
    pub superchain_level: u64,
    /// Governed by Optimism flag.
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub governed_by_optimism: Option<bool>,
    /// The data availability type.
    pub data_availability_type: String,
    /// The Superchain Parent.
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub parent: SuperchainParent,
    /// The gas paying token.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub gas_paying_token: Option<String>,
    /// Fault Proofs information.
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub fault_proofs: Option<FaultProofs>,
}

/// A Chain Parent
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct SuperchainParent {
    /// The parent type.
    pub r#type: String,
    /// The chain identifier.
    pub chain: String,
}

impl SuperchainParent {
    /// Returns the chain id for the parent.
    pub fn chain_id(&self) -> u64 {
        match self.chain.as_ref() {
            "mainnet" => 1,
            "sepolia" => 11155111,
            "sepolia-dev-0" => 11155421,
            _ => 10,
        }
    }
}

/// Fault Proofs information.
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct FaultProofs {
    /// The status of fault proofs.
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superchain_parent_chain_id() {
        let mainnet = SuperchainParent { r#type: "L2".into(), chain: "mainnet".into() };
        assert_eq!(mainnet.chain_id(), 1);

        let sepolia = SuperchainParent { r#type: "L2".into(), chain: "sepolia".into() };
        assert_eq!(sepolia.chain_id(), 11155111);

        let sepolia_dev = SuperchainParent { r#type: "L2".into(), chain: "sepolia-dev-0".into() };
        assert_eq!(sepolia_dev.chain_id(), 11155421);

        let other = SuperchainParent { r#type: "L2".into(), chain: "other".into() };
        assert_eq!(other.chain_id(), 10);
    }

    #[test]
    fn test_chain_list_operations() {
        let chains = vec![
            Chain {
                name: "Base".into(),
                identifier: "base".into(),
                chain_id: 8453,
                ..Default::default()
            },
            Chain {
                name: "Optimism".into(),
                identifier: "op".into(),
                chain_id: 10,
                ..Default::default()
            },
        ];
        let list = ChainList { chains };

        assert_eq!(list.len(), 2);
        assert!(!list.is_empty());

        let base = list.get_chain_by_ident("base").unwrap();
        assert_eq!(base.chain_id, 8453);

        let base_by_id = list.get_chain_by_id(8453).unwrap();
        assert_eq!(base_by_id.name, "Base");

        let idents = list.chain_idents();
        assert!(idents.contains(&"base".to_string()));
        assert!(idents.contains(&"op".to_string()));
    }
}
