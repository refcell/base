# `base-alloy-rpc-types`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

OP Stack RPC types for Base, including transaction receipts, transaction requests, and genesis configuration.

## Overview

This crate provides RPC-layer types that wrap consensus types with additional fields:

- **Transactions**: `Transaction`, `OpTransactionRequest`, `OpTransactionFields`
- **Receipts**: `OpTransactionReceipt`, `OpTransactionReceiptFields`, `L1BlockInfo`
- **Genesis**: `OpChainInfo`, `OpGenesisInfo`, `OpBaseFeeInfo`
- **Errors**: `SuperchainDAError`

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-rpc-types = { git = "https://github.com/base/base" }
```

## Features

- `std` (default): Enable standard library support
- `serde`: Serialization/deserialization support
- `k256`: Cryptographic signature support
- `arbitrary`: Property-based testing support
- `jsonrpsee`: JSON-RPC support

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
