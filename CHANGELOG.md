# Project Arete SDK Changelog

## [Unreleased]
### Added
- [#31](https://github.com/project-arete/sdk/issues/31) Add a function to derive a System ID
- [#30](https://github.com/project-arete/sdk/issues/30) (Rust) Publish tag releases to crates.io and enable static analysis during CI
- [#29](https://github.com/project-arete/sdk/issues/29) (NodeJS) Publish tag releases to npmjs.com and enable static analysis during CI

### Fixed
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
