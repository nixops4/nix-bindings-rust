{
  description = "Rust bindings for the Nix C API";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix.url = "github:NixOS/nix/ed129267dcd7dd2cce48c09b17aefd6cfc488bcd"; # 2.24-pre, before splitting libnixflake
    nix.inputs.nixpkgs.follows = "nixpkgs";
    nix-cargo-integration.url = "github:yusdacra/nix-cargo-integration";
    nix-cargo-integration.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake
      { inherit inputs; }
      ({ lib, ... }: {
        imports = [
          inputs.nix-cargo-integration.flakeModule
          inputs.flake-parts.flakeModules.partitions
          ./rust/nci.nix
        ];
        systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
        perSystem = { config, self', inputs', pkgs, ... }: {
          packages.nix = inputs'.nix.packages.nix;
        };

        partitionedAttrs.devShells = "dev";
        partitionedAttrs.checks = "dev";
        partitionedAttrs.herculesCI = "dev";
        partitions.dev.extraInputsFlake = ./dev;
        partitions.dev.module = {
          imports = [ ./dev/flake-module.nix ];
        };
      });
}
