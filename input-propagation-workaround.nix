# Workaround for missing native input propagation in nix-cargo-integration
#
# Automatically adds Nix C library build inputs based on which nix-bindings
# crates are direct dependencies of the crate being built. The mapping is
# recursive, so depending on nix-bindings-flake will also bring in the
# transitive C library dependencies (nix-fetchers-c, nix-expr-c, etc.).
#
# Note: For multi-crate workspaces, if your crate A depends on your crate B
# which depends on nix-bindings, you'll need to add an A -> B mapping to
# `crateInputMapping` so that A also gets B's nix-bindings inputs.
{
  perSystem =
    {
      lib,
      config,
      pkgs,
      ...
    }:
    let
      cfg = config.nix-bindings-rust.inputPropagationWorkaround;
      nixPackage = config.nix-bindings-rust.nixPackage;

      nixLibs =
        if nixPackage ? libs then
          nixPackage.libs
        else
          # Fallback for older Nix versions without split libs
          {
            nix-util-c = nixPackage;
            nix-store-c = nixPackage;
            nix-expr-c = nixPackage;
            nix-fetchers-c = nixPackage;
            nix-flake-c = nixPackage;
          };

      # A module for nciBuildConfig that sets buildInputs based on nix-bindings dependencies.
      # Uses options inspection to detect drvConfig vs depsDrvConfig context.
      workaroundModule =
        {
          lib,
          config,
          options,
          ...
        }:
        let
          # rust-cargo-lock exists in drvConfig but not depsDrvConfig
          isDrvConfig = options ? rust-cargo-lock;

          dreamLock = config.rust-cargo-lock.dreamLock;
          depsList = dreamLock.dependencies.${config.name}.${config.version} or [ ];

          # Convert list of deps to attrset keyed by name for efficient lookup
          deps = builtins.listToAttrs (
            map (dep: {
              name = dep.name;
              value = dep;
            }) depsList
          );

          # Inputs for the crate itself if it's in the mapping
          selfInputs = cfg.crateInputMapping.${config.name} or [ ];

          # Inputs for direct dependencies that have mappings
          depInputs = lib.concatLists (lib.attrValues (lib.intersectAttrs deps cfg.crateInputMapping));

          allInputs = selfInputs ++ depInputs;
        in
        {
          config = lib.optionalAttrs isDrvConfig {
            mkDerivation.buildInputs = allInputs;
            rust-crane.depsDrv.mkDerivation.buildInputs = allInputs;
          };
        };
    in
    {
      options.nix-bindings-rust.inputPropagationWorkaround = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = ''
            Whether to automatically add Nix C library build inputs based on
            which nix-bindings crates are direct dependencies.

            Set to `false` to disable automatic detection and specify buildInputs manually.
          '';
        };

        crateInputMapping = lib.mkOption {
          type = lib.types.lazyAttrsOf (lib.types.listOf lib.types.package);
          description = ''
            Mapping from crate names to build inputs. Entries can reference
            other entries for transitive dependencies.

            The input propagation workaround can see direct dependencies, so
            if you have `my-crate -> nix-bindings`, that works out of the box.
            If you have `my-other-crate -> my-crate -> nix-bindings`, then you
            need to specify `my-other-crate -> my-crate` as follows:

            ```nix
            nix-bindings-rust.inputPropagationWorkaround.crateInputMapping."my-other-crate" =
              config.nix-bindings-rust.inputPropagationWorkaround.crateInputMapping."my-crate";
            ```
          '';
          default = { };
        };
      };

      config = lib.mkIf cfg.enable {
        nix-bindings-rust.inputPropagationWorkaround.crateInputMapping = {
          # -sys crates with their transitive dependencies
          "nix-bindings-bdwgc-sys" = [ pkgs.boehmgc ];
          "nix-bindings-util-sys" = [ nixLibs.nix-util-c.dev ];
          "nix-bindings-store-sys" = [
            nixLibs.nix-store-c.dev
          ]
          ++ cfg.crateInputMapping."nix-bindings-util-sys";
          "nix-bindings-expr-sys" = [
            nixLibs.nix-expr-c.dev
          ]
          ++ cfg.crateInputMapping."nix-bindings-store-sys"
          ++ cfg.crateInputMapping."nix-bindings-bdwgc-sys";
          "nix-bindings-fetchers-sys" = [
            nixLibs.nix-fetchers-c.dev
          ]
          ++ cfg.crateInputMapping."nix-bindings-expr-sys";
          "nix-bindings-flake-sys" = [
            nixLibs.nix-flake-c.dev
          ]
          ++ cfg.crateInputMapping."nix-bindings-fetchers-sys"
          ++ cfg.crateInputMapping."nix-bindings-bdwgc-sys";
          # High-level crates reference their -sys counterparts
          "nix-bindings-bdwgc" = cfg.crateInputMapping."nix-bindings-bdwgc-sys";
          "nix-bindings-util" = cfg.crateInputMapping."nix-bindings-util-sys";
          "nix-bindings-store" = cfg.crateInputMapping."nix-bindings-store-sys";
          "nix-bindings-expr" = cfg.crateInputMapping."nix-bindings-expr-sys";
          "nix-bindings-fetchers" = cfg.crateInputMapping."nix-bindings-fetchers-sys";
          "nix-bindings-flake" = cfg.crateInputMapping."nix-bindings-flake-sys";
        };

        nix-bindings-rust.nciBuildConfig.imports = [ workaroundModule ];
      };
    };
}
