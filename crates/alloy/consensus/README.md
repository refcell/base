# `base-alloy-consensus`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

OP Stack consensus types for Base, including deposit transactions, receipts, and blocks.

## Overview

This crate provides consensus-layer types that differ from standard Ethereum types due to OP Stack modifications:

- **Transactions**: `OpTxEnvelope`, `TxDeposit`, `OpTypedTransaction`
- **Receipts**: `OpReceiptEnvelope`, `OpDepositReceipt`
- **Blocks**: `OpBlock`
- **EIP-1559**: Holocene and Jovian extra data encoding/decoding

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-consensus = { git = "https://github.com/base/base" }
```

## Features

- `std` (default): Enable standard library support
- `serde`: Serialization/deserialization support
- `k256`: Cryptographic signature support
- `arbitrary`: Property-based testing support

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
