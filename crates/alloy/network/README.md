# `base-alloy-network`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

OP Stack blockchain RPC behavior abstraction for Base.

## Overview

This crate contains a simple abstraction of the RPC behavior of an OP Stack blockchain. It provides:

- **Network**: `Optimism` network type implementing the alloy `Network` trait
- **Transaction Builder**: Implementation of `TransactionBuilder` for OP transactions
- **Wallet**: `EthereumWallet` implementation for `NetworkWallet<Optimism>`

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-network = { git = "https://github.com/base/base" }
```

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
