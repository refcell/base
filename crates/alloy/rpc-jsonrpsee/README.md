# `base-alloy-rpc-jsonrpsee`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

Low-level OP Stack JSON-RPC server and client implementations for Base.

## Overview

This crate provides JSON-RPC traits for OP Stack admin and miner APIs:

- **Admin API**: Sequencer control endpoints
- **Miner API**: Block production settings

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy-rpc-jsonrpsee = { git = "https://github.com/base/base" }
```

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
