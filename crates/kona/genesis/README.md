# base-genesis

OP Stack genesis and chain configuration types.

Ported from [`kona-genesis`](https://github.com/ethereum-optimism/kona).

## Features

- `std` - Standard library support
- `serde` - Serialization/deserialization support
- `arbitrary` - Property-based testing support
- `revm` - REVM integration for `TxKind` conversion
- `tabled` - Table display support

## Usage

```toml
[dependencies]
base-genesis = { workspace = true }
```
