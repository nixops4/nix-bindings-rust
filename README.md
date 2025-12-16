# Nix Bindings for Rust

Rust bindings for the Nix [C API], providing safe, idiomatic Rust interfaces to Nix's core functionality including store operations, expression evaluation, and flake management.

## Overview

This workspace provides multiple crates that wrap different layers of the Nix C API:

- **`nix-bindings-util`** - Utility types and helpers (settings, context, version detection, string handling)
- **`nix-bindings-store`** - Store operations (paths, derivations, store management)
- **`nix-bindings-expr`** - Expression evaluation and type extraction
- **`nix-bindings-flake`** - Flake operations
- **`nix-bindings-fetchers`** - Fetcher functionality (requires Nix ≥ 2.29)

The `nix-bindings-bindgen-raw` crate contains the generated FFI bindings and is not intended for direct use.

## Features

- **Nix evaluation** - Evaluate Nix expressions and create and extract values
- **Store integration** - Interact with the Nix store, manage paths, build derivations
- **Threading** - GC registration and memory management via `Drop`
- **Lazy evaluation** - Fine-grained control over evaluation strictness
- **Version compatibility** - Conditional compilation for different Nix versions

## Quick Start

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
nix-bindings-store = { git = "https://github.com/nixops4/nix-bindings-rust" }
nix-bindings-expr = { git = "https://github.com/nixops4/nix-bindings-rust" }
```

Basic example:

```rust
use nix_bindings_expr::eval_state::{EvalState, init, gc_register_my_thread};
use nix_bindings_store::store::Store;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // Initialize Nix library and register thread with GC
    init()?;
    let guard = gc_register_my_thread()?;

    // Open a store connection and create an evaluation state
    let store = Store::open(None, HashMap::new())?;
    let mut eval_state = EvalState::new(store, [])?;

    // Evaluate a Nix expression
    let value = eval_state.eval_from_string("[1 2 3]", "<example>")?;

    // Extract typed values
    let elements: Vec<_> = eval_state.require_list_strict(&value)?;
    for element in elements {
        let num = eval_state.require_int(&element)?;
        println!("Element: {}", num);
    }

    drop(guard);
    Ok(())
}
```

## Usage Examples

### Evaluating Nix Expressions

```rust
use nix_bindings_expr::eval_state::EvalState;

// Evaluate and extract different types
let int_value = eval_state.eval_from_string("42", "<example>")?;
let num = eval_state.require_int(&int_value)?;

let str_value = eval_state.eval_from_string("\"hello\"", "<example>")?;
let text = eval_state.require_string(&str_value)?;

let attr_value = eval_state.eval_from_string("{ x = 1; y = 2; }", "<example>")?;
let attrs = eval_state.require_attrs(&attr_value)?;
```

### Working with Lists

```rust
let list_value = eval_state.eval_from_string("[1 2 3 4 5]", "<example>")?;

// Lazy: check size without evaluating elements
let size = eval_state.require_list_size(&list_value)?;

// Selective: evaluate only accessed elements
if let Some(first) = eval_state.require_list_select_idx_strict(&list_value, 0)? {
    let value = eval_state.require_int(&first)?;
}

// Strict: evaluate all elements
let all_elements: Vec<_> = eval_state.require_list_strict(&list_value)?;
```

### Thread Safety

Before using `EvalState` in a thread, register with the garbage collector:

```rust
use nix_bindings_expr::eval_state::{init, gc_register_my_thread};

init()?;  // Once per process
let guard = gc_register_my_thread()?;  // Once per thread
// ... use EvalState ...
drop(guard);  // Unregister when done
```

For more examples, see the documentation in each crate's source code.

## Nix Version Compatibility

The crates use conditional compilation to support multiple Nix versions:

- **`nix-bindings-fetchers`** requires Nix ≥ 2.29
- Some features in other crates require specific Nix versions

The build system automatically detects the Nix version and enables appropriate features.

## Integration with Nix Projects

These crates use [nix-cargo-integration] for seamless integration with Nix builds. To use them in your Nix project:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-cargo-integration.url = "github:90-008/nix-cargo-integration";
    nix-bindings-rust.url = "github:nixops4/nix-bindings-rust";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.nix-cargo-integration.flakeModule
        inputs.nix-bindings-rust.modules.flake.default
      ];

      perSystem = { config, pkgs, ... }: {
        # Optional: override Nix package
        nix-bindings-rust.nixPackage = pkgs.nix;

        nci.projects."myproject" = {
          depsDrvConfig = {
            imports = [ config.nix-bindings-rust.nciBuildConfig ];
          };
        };
      };
    };
}
```

See the [nix-cargo-integration documentation][nix-cargo-integration] for more options.

## Development

### Getting Started

```console
$ nix develop
```

### Building

```bash
# Build specific crates (release mode)
nix build .#nix-bindings-store-release
nix build .#nix-bindings-expr-release

# Build with Cargo (in dev shell)
cargo build
cargo build --release
```

### Testing

```bash
# Run tests for specific crates via Nix (recommended - includes proper store setup)
nix build .#checks.x86_64-linux.nix-bindings-store-tests
nix build .#checks.x86_64-linux.nix-bindings-expr-tests

# Run all checks (tests + clippy + formatting)
nix flake check

# Run tests with Cargo (in dev shell)
cargo test

# Run specific test
cargo test test_name
```

### Memory Testing

For FFI memory leak testing with valgrind, see [doc/hacking/test-ffi.md](doc/hacking/test-ffi.md).

### Code Formatting

```bash
treefmt
```

### IDE Setup

For VSCode, load the dev shell via Nix Env Selector extension or direnv.

## Documentation

- [Nix C API Reference][C API]
- [nix-cargo-integration][nix-cargo-integration]
- [Hacking Guide](doc/hacking/test-ffi.md)

## License

See [LICENSE](LICENSE) file in the repository.

[C API]: https://nix.dev/manual/nix/latest/c-api.html
[nix-cargo-integration]: https://github.com/90-008/nix-cargo-integration#readme
