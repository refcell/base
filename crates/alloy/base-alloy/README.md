# `base-alloy`

<a href="https://github.com/base/base/actions/workflows/ci.yml"><img src="https://github.com/base/base/actions/workflows/ci.yml/badge.svg?label=ci" alt="CI"></a>
<a href="https://github.com/base/base/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-d1d1f6.svg?label=license&labelColor=2a2f35" alt="MIT License"></a>

Connect applications to the OP Stack for Base.

## Overview

This is a meta-crate that re-exports all base-alloy crates:

- `base-alloy-consensus` - OP Stack consensus types
- `base-alloy-network` - OP Stack network types
- `base-alloy-provider` - OP Stack provider extensions
- `base-alloy-rpc-types` - OP Stack RPC types
- `base-alloy-rpc-types-engine` - OP Stack engine RPC types
- `base-alloy-rpc-jsonrpsee` - OP Stack JSON-RPC traits

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
base-alloy = { git = "https://github.com/base/base", features = ["full"] }
```

## Features

- `full` - Enable all sub-crates
- `consensus` - Enable consensus types
- `network` - Enable network types
- `provider` - Enable provider extensions
- `rpc-types` - Enable RPC types
- `rpc-types-engine` - Enable engine RPC types
- `rpc-jsonrpsee` - Enable JSON-RPC traits

## License

Licensed under the [MIT License](https://github.com/base/base/blob/main/LICENSE).
