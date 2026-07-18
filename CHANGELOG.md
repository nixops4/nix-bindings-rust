# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- `EvalState::store()`. Consumers need to track the `Store` themselves.

## [0.2.1] - 2026-05-16

### Added

- `StorePath` bindings: hash access and `Display` impl. ([#42] by [@Ericson2314])
- `Derivation::to_json_string`. ([#47] by [@Ericson2314])
- Experimental [harmonia](https://github.com/nix-community/harmonia) interop behind the `harmonia` feature flag: conversions for `StorePath` and `Derivation`. ([#43] by [@artemist]; see [issue #64])
- `primop::RecoverableError` for primop errors that should not be memoized in the thunk, allowing retry on next force. Required by Nix >= 2.34 ([release note](https://nix.dev/manual/nix/2.34/release-notes/rl-2.34.html#c-api-changes)) for recoverable errors to remain recoverable, as Nix 2.34 memoizes errors by default.

### Fixed

- aarch64 support for nix-bindings-util, nix-bindings-expr, and nix-bindings-store. ([#52] by [@RossComputerGuy], [#63] by [@dramforever] and [@roberth])
- Dev shell and build on Darwin. ([#52] by [@RossComputerGuy])
- `cargo doc` generation. ([#41])

### Changed

- Replace `lazy_static` with `std::sync::LazyLock`. ([#51] by [@RossComputerGuy])

## [0.2.0] - 2026-01-13

### Added

- Workaround for automatic C library input propagation in downstream Nix builds. ([#27] by [@roberth])
- `EvalStateBuilder::load_ambient_settings()` to control whether global Nix settings are loaded. ([#36] by [@roberth])

### Fixed

- Path coercion failing with "path does not exist" errors due to missing `eval_state_builder_load()` call. ([#36] by [@aanderse])

### Changed

- Split `nix-bindings-util-sys` (which contained all low-level FFI bindings) into separate per-library `*-sys` crates. ([#27] by [@Ericson2314])
  This allows downstream crates to depend on just the low-level bindings they need without pulling in higher-level crates.

## [0.1.0] - 2026-01-12

Initial release, extracted from the [nixops4 repository](https://github.com/nixops4/nixops4).

### Added

- `nix-bindings-store`: Rust bindings for Nix store operations
  - Store opening (auto, from URI, from environment)
  - Store path parsing and manipulation
  - `Store::get_fs_closure` ([#12] by [@RossComputerGuy], [@roberth])
  - `Clone` for `Derivation` ([#25] by [@Ericson2314])
  - Store deduplication workaround for [nix#11979]
  - aarch64 ABI support ([#26] by [@RossComputerGuy])
- `nix-bindings-expr`: Rust bindings for Nix expression evaluation
  - `EvalState` for evaluating Nix expressions
  - Value creation (int, string, attrs, thunks, primops, etc.)
  - Value inspection/extraction (`require_*` functions)
  - Attribute selection and manipulation
  - Thread registration for GC safety
- `nix-bindings-fetchers`: Rust bindings for Nix fetchers
- `nix-bindings-flake`: Rust bindings for Nix flake operations
  - Flake locking
  - Flake overriding
- `nix-bindings-util`: Shared utilities
  - Context management for Nix C API error handling
  - Settings access
- `nix-bindings-util-sys`: Low-level FFI bindings for all Nix C libraries

### Contributors

Thanks to everyone who contributed to the initial development, some of whom may not be listed with individual changes above:

- [@aanderse]
- [@Ericson2314]
- [@ErinvanderVeen]
- [@numinit]
- [@prednaz]
- [@Radvendii]
- [@roberth]
- [@RossComputerGuy]

<!-- end of 0.1.0 release section -->

[@aanderse]: https://github.com/aanderse
[@artemist]: https://github.com/artemist
[@dramforever]: https://github.com/dramforever
[@Ericson2314]: https://github.com/Ericson2314
[@ErinvanderVeen]: https://github.com/ErinvanderVeen
[@numinit]: https://github.com/numinit
[@prednaz]: https://github.com/prednaz
[@Radvendii]: https://github.com/Radvendii
[@roberth]: https://github.com/roberth
[@RossComputerGuy]: https://github.com/RossComputerGuy

[#12]: https://github.com/nixops4/nix-bindings-rust/pull/12
[#25]: https://github.com/nixops4/nix-bindings-rust/pull/25
[#26]: https://github.com/nixops4/nix-bindings-rust/pull/26
[#27]: https://github.com/nixops4/nix-bindings-rust/pull/27
[#36]: https://github.com/nixops4/nix-bindings-rust/pull/36
[#41]: https://github.com/nixops4/nix-bindings-rust/pull/41
[#42]: https://github.com/nixops4/nix-bindings-rust/pull/42
[#43]: https://github.com/nixops4/nix-bindings-rust/pull/43
[#47]: https://github.com/nixops4/nix-bindings-rust/pull/47
[#51]: https://github.com/nixops4/nix-bindings-rust/pull/51
[#52]: https://github.com/nixops4/nix-bindings-rust/pull/52
[#63]: https://github.com/nixops4/nix-bindings-rust/pull/63
[issue #64]: https://github.com/nixops4/nix-bindings-rust/issues/64
[Unreleased]: https://github.com/nixops4/nix-bindings-rust/compare/0.2.1...HEAD
[0.2.1]: https://github.com/nixops4/nix-bindings-rust/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/nixops4/nix-bindings-rust/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/nixops4/nix-bindings-rust/releases/tag/0.1.0
[nix#11979]: https://github.com/NixOS/nix/issues/11979
