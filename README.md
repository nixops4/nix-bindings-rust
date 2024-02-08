
# `nix_bindings_*` crates

Use the Nix [C API] from Rust.

## Hacking

The following will open a shell with dependencies, and install pre-commit for automatic formatting.

```console
$ nix develop
```

### VSCode

#### rust-analyzer

If the rust-analyzer extension fails, make sure the devShell was loaded into VSCode via Nix Env Selector or direnv.

[C API]: https://nix.dev/manual/nix/latest/c-api.html
