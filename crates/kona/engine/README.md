# base-engine

Engine API client for Base L2 node operations, including block building, payload management, and execution layer synchronization.

## Overview

This crate provides the core engine functionality for the Base rollup node:

- **Engine Client**: HTTP client for Engine API communication with JWT authentication
- **Task Queue**: Priority-based task execution for engine operations
- **State Management**: Engine sync state tracking and forkchoice management
- **Rollup Boost**: Optional integration with rollup-boost for enhanced block building

## Features

- `test-utils`: Enable test utilities for mocking engine components
- `default`: Standard features enabled

## License

MIT
