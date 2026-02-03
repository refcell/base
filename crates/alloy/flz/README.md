# `base-alloy-flz`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

Tiny FastLZ compression library for L2 transaction cost estimation.

## Overview

This crate provides utilities for estimating the compressed size of transactions using FastLZ compression, which is used for calculating Optimism L1 data availability costs post-Fjord upgrade.

Ported from [`op-alloy-flz`](https://github.com/alloy-rs/flz).

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-flz = { git = "https://github.com/base/base" }
```

## Example

```rust
use base_alloy_flz::tx_estimated_size_fjord_bytes;

let tx_data = [0u8; 100];
let estimated_size = tx_estimated_size_fjord_bytes(&tx_data);
```

## Features

- `compress` - Enable actual FastLZ compression via `fastlz` crate
- `decompress` - Enable FastLZ decompression via `fastlz` crate

## Functions

- `tx_estimated_size_fjord_bytes(input)` - Returns estimated compressed size in bytes
- `tx_estimated_size_fjord(input)` - Returns estimated size scaled by 1e6
- `data_gas_fjord(input)` - Returns L1 data gas cost for the transaction
- `flz_compress_len(input)` - Returns estimated FastLZ compressed length

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
