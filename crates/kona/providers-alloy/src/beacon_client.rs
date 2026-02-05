//! Contains an online implementation of the `BeaconClient` trait.

use std::{boxed::Box, collections::HashMap, format, string::String, vec::Vec};

use alloy_eips::eip4844::{IndexedBlobHash, env_settings::EnvKzgSettings, kzg_to_versioned_hash};
use alloy_primitives::{B256, FixedBytes};
use alloy_rpc_types_beacon::sidecar::GetBlobsResponse;
use async_trait::async_trait;
use c_kzg::Blob;
use reqwest::Client;
use thiserror::Error;

#[cfg(feature = "metrics")]
use crate::Metrics;
use crate::blobs::BoxedBlobWithIndex;

const SPEC_METHOD: &str = "eth/v1/config/spec";

const GENESIS_METHOD: &str = "eth/v1/beacon/genesis";

const BLOBS_METHOD_PREFIX: &str = "eth/v1/beacon/blobs";

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReducedGenesisData {
    #[serde(rename = "genesis_time")]
    #[serde(with = "alloy_serde::quantity")]
    pub genesis_time: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct APIGenesisResponse {
    pub data: ReducedGenesisData,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReducedConfigData {
    #[serde(rename = "SECONDS_PER_SLOT")]
    #[serde(with = "alloy_serde::quantity")]
    pub seconds_per_slot: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct APIConfigResponse {
    pub data: ReducedConfigData,
}

impl APIConfigResponse {
    pub const fn new(seconds_per_slot: u64) -> Self {
        Self { data: ReducedConfigData { seconds_per_slot } }
    }
}

impl APIGenesisResponse {
    pub const fn new(genesis_time: u64) -> Self {
        Self { data: ReducedGenesisData { genesis_time } }
    }
}

#[async_trait]
pub trait BeaconClient {
    type Error: core::fmt::Display;

    async fn slot_interval(&self) -> Result<APIConfigResponse, Self::Error>;

    async fn genesis_time(&self) -> Result<APIGenesisResponse, Self::Error>;

    async fn filtered_beacon_blobs(
        &self,
        slot: u64,
        blob_hashes: &[IndexedBlobHash],
    ) -> Result<Vec<BoxedBlobWithIndex>, Self::Error>;
}

const BLOB_SIZE: usize = 131072;

fn blob_versioned_hash(blob: &FixedBytes<BLOB_SIZE>) -> Result<B256, BeaconClientError> {
    let kzg_settings = EnvKzgSettings::Default;
    let kzg_blob = Blob::new(blob.0);
    let commitment = kzg_settings.get().blob_to_kzg_commitment(&kzg_blob)?;
    Ok(kzg_to_versioned_hash(commitment.as_slice()))
}

#[derive(Error, Debug)]
pub enum BeaconClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Blob hash not found in beacon response: {0}")]
    BlobNotFound(String),

    #[error("KZG error: {0}")]
    KZG(#[from] c_kzg::Error),
}

#[derive(Debug, Clone)]
pub struct OnlineBeaconClient {
    pub base: String,
    pub inner: Client,
    pub l1_slot_duration: Option<u64>,
}

impl OnlineBeaconClient {
    pub fn new_http(mut base: String) -> Self {
        if base.ends_with('/') {
            base.remove(base.len() - 1);
        }
        Self {
            base,
            inner: Client::builder().build().expect("Failed to create beacon client"),
            l1_slot_duration: None,
        }
    }

    pub const fn with_l1_slot_duration_override(mut self, l1_slot_duration: u64) -> Self {
        self.l1_slot_duration = Some(l1_slot_duration);
        self
    }

    async fn filtered_beacon_blobs(
        &self,
        slot: u64,
        blob_hashes: &[IndexedBlobHash],
    ) -> Result<Vec<BoxedBlobWithIndex>, BeaconClientError> {
        let params = blob_hashes.iter().map(|blob| blob.hash.to_string()).collect::<Vec<_>>();
        let response = self
            .inner
            .get(format!("{}/{}/{}", self.base, BLOBS_METHOD_PREFIX, slot))
            .query(&[("versioned_hashes", &params.join(","))])
            .send()
            .await?
            .error_for_status()?;
        let bundle = response.json::<GetBlobsResponse>().await?;

        let returned_blobs_mapped_by_hash = bundle
            .data
            .iter()
            .map(|data| -> Result<_, BeaconClientError> {
                let recomputed_hash = blob_versioned_hash(data)?;
                Ok((recomputed_hash, data))
            })
            .collect::<Result<HashMap<_, _>, BeaconClientError>>()?;

        blob_hashes
            .iter()
            .map(|blob_hash| -> Result<BoxedBlobWithIndex, BeaconClientError> {
                let matching_data = returned_blobs_mapped_by_hash
                    .get(&blob_hash.hash)
                    .ok_or(BeaconClientError::BlobNotFound(blob_hash.hash.to_string()))?;
                Ok(BoxedBlobWithIndex { blob: Box::new(**matching_data), index: blob_hash.index })
            })
            .collect::<Result<Vec<_>, BeaconClientError>>()
    }
}

#[async_trait]
impl BeaconClient for OnlineBeaconClient {
    type Error = BeaconClientError;

    async fn slot_interval(&self) -> Result<APIConfigResponse, Self::Error> {
        base_macros::inc!(gauge, Metrics::BEACON_CLIENT_REQUESTS, "method" => "spec");

        if let Some(l1_slot_duration) = self.l1_slot_duration {
            return Ok(APIConfigResponse::new(l1_slot_duration));
        }

        let result = async {
            let first = self.inner.get(format!("{}/{}", self.base, SPEC_METHOD)).send().await?;
            first.json::<APIConfigResponse>().await
        }
        .await;

        if result.is_err() {
            base_macros::inc!(gauge, Metrics::BEACON_CLIENT_ERRORS, "method" => "spec");
        }

        Ok(result?)
    }

    async fn genesis_time(&self) -> Result<APIGenesisResponse, Self::Error> {
        base_macros::inc!(gauge, Metrics::BEACON_CLIENT_REQUESTS, "method" => "genesis");

        let result = async {
            let first = self.inner.get(format!("{}/{}", self.base, GENESIS_METHOD)).send().await?;
            first.json::<APIGenesisResponse>().await
        }
        .await;

        if result.is_err() {
            base_macros::inc!(gauge, Metrics::BEACON_CLIENT_ERRORS, "method" => "genesis");
        }

        Ok(result?)
    }

    async fn filtered_beacon_blobs(
        &self,
        slot: u64,
        blob_hashes: &[IndexedBlobHash],
    ) -> Result<Vec<BoxedBlobWithIndex>, BeaconClientError> {
        base_macros::inc!(gauge, Metrics::BEACON_CLIENT_REQUESTS, "method" => "blobs");

        let result = self.filtered_beacon_blobs(slot, blob_hashes).await;

        if result.is_err() {
            base_macros::inc!(gauge, Metrics::BEACON_CLIENT_ERRORS, "method" => "blobs");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use alloy_consensus::Blob;
    use alloy_primitives::{FixedBytes, hex::FromHex};
    use httpmock::prelude::*;
    use serde_json::json;

    use super::*;

    const TEST_BLOB_DATA: Blob = FixedBytes::repeat_byte(1);
    const TEST_BLOB_HASH_HEX: &str =
        "0x016c357b8b3a6b3fd82386e7bebf77143d537cdb1c856509661c412602306a04";

    #[test]
    fn test_blob_versioned_hash() {
        let input: Blob = FixedBytes::repeat_byte(1);
        let test_blob_hash: FixedBytes<32> = FixedBytes::from_hex(TEST_BLOB_HASH_HEX).unwrap();
        assert_eq!(test_blob_hash, blob_versioned_hash(&input).unwrap());
    }

    #[tokio::test]
    async fn test_filtered_beacon_blobs() {
        let slot = 987654321;
        let slot_string = slot.to_string();
        let repeated_blob_data: Vec<Blob> = vec![TEST_BLOB_DATA, TEST_BLOB_DATA];
        let garbage_blob_data: Vec<Blob> = vec![FixedBytes::repeat_byte(2)];
        let required_query_param = format!("{TEST_BLOB_HASH_HEX},{TEST_BLOB_HASH_HEX}");
        let test_blob_hash: FixedBytes<32> = FixedBytes::from_hex(TEST_BLOB_HASH_HEX).unwrap();
        let requested_blob_hashes: Vec<IndexedBlobHash> = vec![
            IndexedBlobHash { index: 0, hash: test_blob_hash },
            IndexedBlobHash { index: 2, hash: test_blob_hash },
        ];

        struct TestCase {
            name: &'static str,
            mock_response_data: Vec<Blob>,
            want: Option<Vec<BoxedBlobWithIndex>>,
        }

        let test_cases = vec![
            TestCase {
                name: "Repeated Blob Data, expect success",
                mock_response_data: repeated_blob_data,
                want: Some(vec![
                    BoxedBlobWithIndex { index: 0, blob: Box::new(TEST_BLOB_DATA) },
                    BoxedBlobWithIndex { index: 2, blob: Box::new(TEST_BLOB_DATA) },
                ]),
            },
            TestCase {
                name: "Garbage Blob Data, expect error",
                mock_response_data: garbage_blob_data,
                want: None,
            },
        ];

        let server = MockServer::start();
        for test_case in test_cases {
            let mock_response = json!({
                "execution_optimistic": false,
                "finalized": false,
                "data": test_case.mock_response_data
            });
            let mut blobs_mock = server.mock(|when, then| {
                when.method(GET)
                    .path(format!("/eth/v1/beacon/blobs/{slot_string}"))
                    .query_param("versioned_hashes", required_query_param.clone());
                then.status(200).json_body(mock_response);
            });

            let client = OnlineBeaconClient::new_http(server.base_url());
            let response = client.filtered_beacon_blobs(slot, &requested_blob_hashes).await;
            blobs_mock.assert();
            match test_case.want {
                Some(s) => {
                    let r = response.unwrap();
                    assert_eq!(r.len(), s.len(), "length mismatch{}", test_case.name);
                    assert_eq!(r, s, "{}", test_case.name)
                }
                None => {
                    assert!(response.is_err(), "{}", test_case.name)
                }
            }
            blobs_mock.delete();
        }
    }
}
