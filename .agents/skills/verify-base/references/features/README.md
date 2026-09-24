# Feature Map

This is the index an agent reads before using Base's existing tools. Each linked page explains a behavior, how to exercise it, and what evidence establishes success.

| Feature | Expected behavior | Tools |
| --- | --- | --- |
| [Local devnet transaction inclusion](local-devnet-transaction-inclusion.md) | A funded ETH transfer sent directly to the local sequencer succeeds and appears in its reported block. | `just devnet up-single`, `cast send`, `cast block` |
| [Isolated factory devnet and restart](autonomous-local-devnet.md) | Source-built sequencer inclusion agrees with an independent validator; receipts survive a scoped process restart and new blocks advance. | Experiment Compose overrides, `cast`, assertion scripts |

This is an evolving inventory, not exhaustive coverage. The map records commands, assertions and limitations; uncatalogued behavior is not evidence that tests do not exist. See `docs/autonomous/BEHAVIOR_INVENTORY.md` for the broader experiment coverage backlog.
