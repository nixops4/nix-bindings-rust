
# `nix_bindings_*` crates

Use the Nix [C API] from Rust.

## Build with `nix-cargo-integration`

The development environment and building with Nix are taken care of by [nix-cargo-integration](https://github.com/90-008/nix-cargo-integration#readme) ([options](https://flake.parts/options/nix-cargo-integration.html)).

The dependency on Nix is taken care of with the [`nix-bindings-rust` flake-parts module]().

Example usage:

```nix
{
  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; }
    {
      imports = [
        inputs.nix-cargo-integration.flakeModule
        inputs.nix-bindings-rust.modules.flake.default
      ];

      perSystem = { config, pkgs, ... }: {
        # optional:
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

## Hacking

The following will open a shell with dependencies, and install pre-commit for automatic formatting.

```console
$ nix develop
```

### VSCode

#### rust-analyzer

If the rust-analyzer extension fails, make sure the devShell was loaded into VSCode via Nix Env Selector or direnv.

[C API]: https://nix.dev/manual/nix/latest/c-api.html
