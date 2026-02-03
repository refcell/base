# `base-derive`

A `no_std` compatible implementation of the OP Stack's derivation pipeline.

This crate provides the core L2 block derivation logic for the Base network.

## Usage

The intended way of working with `base-derive` is to use the `DerivationPipeline` which implements the `Pipeline` trait. To create an instance of the `DerivationPipeline`, it's recommended to use the `PipelineBuilder`.

```rust,ignore
use std::sync::Arc;
use base_genesis::RollupConfig;
use base_derive::EthereumDataSource;
use base_derive::PipelineBuilder;
use base_derive::StatefulAttributesBuilder;

let chain_provider = todo!();
let l2_chain_provider = todo!();
let blob_provider = todo!();
let l1_origin = todo!();

let cfg = Arc::new(RollupConfig::default());
let attributes = StatefulAttributesBuilder::new(
   cfg.clone(),
   l2_chain_provider.clone(),
   chain_provider.clone(),
);
let dap = EthereumDataSource::new(
   chain_provider.clone(),
   blob_provider,
   cfg.as_ref()
);

// Construct a new derivation pipeline.
let pipeline = PipelineBuilder::new()
   .rollup_config(cfg)
   .dap_source(dap)
   .l2_chain_provider(l2_chain_provider)
   .chain_provider(chain_provider)
   .builder(attributes)
   .origin(l1_origin)
   .build();
```

## Features

- `serde`: Serialization and Deserialization support for `base-derive` types.
- `test-utils`: Test utilities for downstream libraries.
- `metrics`: Metrics support for the derivation pipeline.
