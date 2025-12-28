{
  description = "Rust bindings for the Nix C API";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix.url = "github:NixOS/nix";
    nix.inputs.nixpkgs.follows = "nixpkgs";
    nix-cargo-integration.url = "github:90-008/nix-cargo-integration";
    nix-cargo-integration.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      toplevel@{
        lib,
        ...
      }:
      let
        /**
          Makes perSystem.nix-bindings-rust available.
        */
        flake-parts-modules.basic =
          {
            flake-parts-lib,
            ...
          }:
          {
            _file = ./flake.nix;
            options.perSystem = flake-parts-lib.mkPerSystemOption (
              { config, pkgs, ... }:
              let
                cfg = config.nix-bindings-rust;
              in
              {
                options.nix-bindings-rust = {
                  nixPackage = lib.mkOption {
                    type = lib.types.package;
                    default = pkgs.nix;
                    defaultText = lib.literalMD "pkgs.nix";
                    description = ''
                      The Nix package to use when building the `nix-bindings-...` crates.
                    '';
                  };
                  nciBuildConfig = lib.mkOption {
                    type = lib.types.deferredModule;
                    description = ''
                      A module to load into your nix-cargo-integration
                      [`perSystem.nci.projects.<name>.depsDrvConfig`](https://flake.parts/options/nix-cargo-integration.html#opt-perSystem.nci.projects._name_.depsDrvConfig) or similar such options.

                      Example:
                      ```nix
                      perSystem = perSystem@{ config, ... }: {
                        nci.projects."my_project".depsDrvConfig = perSystem.config.nix-bindings-rust.nciBuildConfig;
                      }
                      ```
                    '';
                  };
                };
                config.nix-bindings-rust = {
                  nciBuildConfig = {
                    mkDerivation = rec {
                      buildInputs = [
                        # stdbool.h
                        pkgs.stdenv.cc
                      ]
                      ++ (
                        if cfg.nixPackage ? libs then
                          let
                            l = cfg.nixPackage.libs;
                          in
                          [
                            l.nix-expr-c
                            l.nix-store-c
                            l.nix-util-c
                            l.nix-fetchers-c or null # Nix >= 2.29
                            l.nix-flake-c
                          ]
                        else
                          [ cfg.nixPackage ]
                      );
                      nativeBuildInputs = [
                        pkgs.pkg-config
                      ];
                      # bindgen uses clang to generate bindings, but it doesn't know where to
                      # find our stdenv cc's headers, so when it's gcc, we need to tell it.
                      postConfigure = lib.optionalString pkgs.stdenv.cc.isGNU ''
                        source ${./bindgen-gcc.sh}
                      '';
                      shellHook = postConfigure;
                    };
                    # NOTE: duplicated in flake.nix devShell
                    env = {
                      LIBCLANG_PATH = lib.makeLibraryPath [ pkgs.buildPackages.llvmPackages.clang-unwrapped ];
                      BINDGEN_EXTRA_CLANG_ARGS =
                        # Work around missing [[deprecated]] in clang
                        "-x c++ -std=c++2a";
                    }
                    // lib.optionalAttrs pkgs.stdenv.cc.isGNU {
                      # Avoid cc wrapper, because we only need to add the compiler/"system" dirs
                      NIX_CC_UNWRAPPED = "${pkgs.stdenv.cc.cc}/bin/gcc";
                    };
                  };
                };
              }
            );
          };

        /**
          A flake-parts module for dependents to import. Also dogfooded locally
          (extra, not required for normal CI).

          Adds flake checks that test the nix-bindings crates with the
          dependent's nix package.

          See https://github.com/nixops4/nix-bindings-rust?tab=readme-ov-file#integration-with-nix-projects
        */
        flake-parts-modules.tested =
          # Consumer toplevel
          { ... }:
          {
            _file = ./flake.nix;
            imports = [ flake-parts-modules.basic ];
            config.perSystem =
              # Consumer perSystem
              {
                lib,
                config,
                system,
                pkgs,
                ...
              }:
              let
                # nix-bindings-rust's perSystem, but with the consumer's `pkgs`
                nix-bindings-rust-perSystemConfig =
                  # Extending our own perSystem, not the consumer's perSystem!
                  toplevel.config.partitions.testing-support.module.nix-bindings-rust.internalWithSystem system
                    ({ extendModules, ... }: extendModules)
                    {
                      modules = [
                        {
                          config = {
                            # Overriding our `perSystem` to use the consumer's `pkgs`
                            _module.args.pkgs = lib.mkForce pkgs;
                            # ... and `nixPackage`
                            nix-bindings-rust.nixPackage = lib.mkForce config.nix-bindings-rust.nixPackage;
                          };
                        }
                      ];
                    };
              in
              {
                key = "nix-bindings-rust-add-checks";
                # Exclude clippy checks; those are part of this repo's local CI.
                # This module is for dependents (and local dogfooding), which
                # don't need to run clippy on nix-bindings-rust.
                config.checks = lib.concatMapAttrs (
                  k: v:
                  lib.optionalAttrs (lib.strings.hasPrefix "nix-bindings-" k && !lib.strings.hasSuffix "-clippy" k) {
                    "dependency-${k}" = v;
                  }
                ) nix-bindings-rust-perSystemConfig.config.checks;
              };
          };

        flake-parts-modules.default = flake-parts-modules.tested;

      in
      {
        imports = [
          inputs.nix-cargo-integration.flakeModule
          inputs.flake-parts.flakeModules.partitions
          inputs.flake-parts.flakeModules.modules
          # dogfood
          flake-parts-modules.tested
          ./nci.nix
        ];
        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ];
        perSystem =
          {
            inputs',
            ...
          }:
          {
            packages.nix = inputs'.nix.packages.nix;
          };

        partitionedAttrs.devShells = "dev";
        partitionedAttrs.checks = "dev";
        partitionedAttrs.herculesCI = "dev";
        # Packages are basically just checks in this project; a library by
        # itself is not useful. That's just not how the Rust integration works.
        # By taking `packages` from `dev` we benefit from this dev-only definition:
        #      nix-bindings-rust.nixPackage = inputs'.nix.packages.default;
        partitionedAttrs.packages = "dev";

        partitions.dev.extraInputsFlake = ./dev;
        partitions.dev.module = {
          imports = [ ./dev/flake-module.nix ];
        };

        # A partition that doesn't dogfood the flake-parts-modules.tested module
        # so that we can actually retrieve `checks` without infinite recursions
        # from trying to include the dogfooded attrs.
        partitions.testing-support.module =
          { withSystem, ... }:
          {
            # Make a clean withSystem available for consumers
            options.nix-bindings-rust.internalWithSystem = lib.mkOption { internal = true; };
            config = {
              nix-bindings-rust.internalWithSystem = withSystem;
              perSystem = {
                # Remove dogfooded checks. This configuration's checks are
                # *consumed* by nix-bindings-rust-add-checks, so they should
                # *NOT* also be *produced* by it.
                disabledModules = [ { key = "nix-bindings-rust-add-checks"; } ];
              };
            };
          };

        # flake output attributes
        flake = {
          modules.flake = flake-parts-modules;
        };
      }
    );
}
