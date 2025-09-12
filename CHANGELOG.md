# Project Arete SDK Changelog

## [Unreleased]
### Added
- [#71](https://github.com/project-arete/sdk/issues/71) Expose Client addProvider and addConsumer fns
- [#69](https://github.com/project-arete/sdk/issues/69) Example services should register the "padi.light" profile with their context on the Arete Control Plane on startup
- [#56](https://github.com/project-arete/sdk/issues/56) (Rust) Support observing update event
- [#53](https://github.com/project-arete/sdk/issues/53) (Python) Track active requests, and wait for a transaction response
- [#52](https://github.com/project-arete/sdk/issues/52) (Rust) Implement a Light example service
- [#26](https://github.com/project-arete/sdk/issues/26) Example services should add their situation context to the Arete Control Plane on startup

### Changed
- [#60](https://github.com/project-arete/sdk/issues/60) (Rust) Rename Connection → Client for consistency with other language SDKs
- [#54](https://github.com/project-arete/sdk/issues/54) (Rust) Upgrade Rust 1.86.0 → 1.88.0
- [#28](https://github.com/project-arete/sdk/issues/28) Example Switch services should derive their connection key from node/context/role/profile/property values

### Fixed
- [#65](https://github.com/project-arete/sdk/issues/65) (Rust) The catalog entry at crates.io fails to link to this Git repo
- [#62](https://github.com/project-arete/sdk/issues/62) (Python) Examples fail to wait for open before system and node registration
- [#51](https://github.com/project-arete/sdk/issues/51) (Python) put is not implemented

## [0.1.3] - 2025-09-05
### Added
- [#46](https://github.com/project-arete/sdk/issues/46) Example services should add their system to the Arete Control Plane on startup
- [#31](https://github.com/project-arete/sdk/issues/31) Add a function to derive a System ID
- [#30](https://github.com/project-arete/sdk/issues/30) (Rust) Publish tag releases to crates.io and enable static analysis during CI
- [#29](https://github.com/project-arete/sdk/issues/29) (NodeJS) Publish tag releases to npmjs.com and enable static analysis during CI
- [#25](https://github.com/project-arete/sdk/issues/25) Example services should add themselves as a node to the Arete Control Plane on startup

### Fixed
- [#41](https://github.com/project-arete/sdk/issues/41) (Rust) SDK fails parsing websocket messages which are now sparsely populated
- [#39](https://github.com/project-arete/sdk/issues/39) (Python) Python SDK should not hard-code SSL options
- [#36](https://github.com/project-arete/sdk/issues/36) (Rust) Cannot call `to_string()` on Error: arete_sdk::Error doesn't implement std::fmt::Display

## [0.1.2] - 2025-08-30
### Added
- [#21](https://github.com/project-arete/sdk/issues/21) (Python) Provide an SDK for Python
- [#16](https://github.com/project-arete/sdk/issues/16) (Rust) Provide an SDK for Rust

### Changed
- [#14](https://github.com/project-arete/sdk/issues/14) Reorganize repo as a multi-language SDK with examples

## [0.1.1] - 2025-08-22
### Added
- [#10](https://github.com/project-arete/sdk/issues/10) (NodeJS) Wait for connection to Arete control plane, or fail startup
- [#8](https://github.com/project-arete/sdk/issues/8) (NodeJS) Synchronize switch setting to Arete control plane as desired state
- [#7](https://github.com/project-arete/sdk/issues/7) (NodeJS) Drive light using desired state coming from Arete control plane
- [#5](https://github.com/project-arete/sdk/issues/5) (NodeJS) Connect switch and light services to the Arete control plane on startup

## [0.1.0] - 2025-08-15
### Added
- [#2](https://github.com/project-arete/sdk/issues/2) (NodeJS) Implement writing the light bulb state via the GPIO hardware interface
- [#1](https://github.com/project-arete/sdk/issues/1) (NodeJS) Implement reading the switch via the GPIO hardware interface
