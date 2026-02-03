# `base-alloy-rpc-types-engine`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

OP Stack RPC types for the `engine` namespace for Base.

## Overview

This crate provides engine API types for OP Stack execution:

- **Payloads**: `OpExecutionPayload`, `OpExecutionPayloadV4`, `OpExecutionPayloadEnvelope`
- **Attributes**: `OpPayloadAttributes`
- **Sidecar**: `OpExecutionPayloadSidecar`
- **Superchain**: `ProtocolVersion`, `SuperchainSignal`
- **Flashblocks**: `OpFlashblockPayload`, `OpFlashblockPayloadDelta`

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-rpc-types-engine = { git = "https://github.com/base/base" }
```

## Features

- `std` (default): Enable standard library support with SSZ encoding
- `serde`: Serialization/deserialization support
- `k256`: Cryptographic signature support
- `arbitrary`: Property-based testing support

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
