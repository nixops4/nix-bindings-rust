{
  inputs,
  ...
}:
{
  imports = [
    inputs.pre-commit-hooks-nix.flakeModule
    inputs.hercules-ci-effects.flakeModule
    inputs.treefmt-nix.flakeModule
  ];
  perSystem =
    {
      config,
      pkgs,
      inputs',
      ...
    }:
    {
      nix-bindings-rust.nixPackage = inputs'.nix.packages.default;

      treefmt = {
        # Used to find the project root
        projectRootFile = "flake.lock";

        programs.rustfmt = {
          enable = true;
          edition = "2021";
        };
        programs.nixfmt.enable = true;
        programs.deadnix.enable = true;
        #programs.clang-format.enable = true;
      };

      pre-commit.settings.hooks.treefmt.enable = true;
      # Temporarily disable rustfmt due to configuration issues
      # pre-commit.settings.hooks.rustfmt.enable = true;
      pre-commit.settings.settings.rust.cargoManifestPath = "./Cargo.toml";

      # Check that we're using ///-style doc comments in Rust code.
      #
      # Unfortunately, rustfmt won't do this for us yet - at least not
      # without nightly, and it might do too much.
      pre-commit.settings.hooks.rust-doc-comments = {
        enable = true;
        files = "\\.rs$";
        entry = "${pkgs.writeScript "rust-doc-comments" ''
          #!${pkgs.runtimeShell}
          set -uxo pipefail
          grep -n -C3 --color=always -F '/**' "$@"
          r=$?
          set -e
          if [ $r -eq 0 ]; then
            echo "Please replace /**-style comments by /// style comments in Rust code."
            exit 1
          fi
        ''}";
      };

      devShells.default = pkgs.mkShell (
        {
          name = "nix-bindings-devshell";
          strictDeps = true;
          inputsFrom = [ config.nci.outputs.nix-bindings.devShell ];
          inherit (config.nci.outputs.nix-bindings.devShell.env)
            LIBCLANG_PATH
            ;
          NIX_DEBUG_INFO_DIRS =
            let
              # TODO: add to Nixpkgs lib
              getDebug =
                pkg:
                if pkg ? debug then
                  pkg.debug
                else if pkg ? lib then
                  pkg.lib
                else
                  pkg;
            in
            "${getDebug config.packages.nix}/lib/debug";
          buildInputs = [
            config.packages.nix
          ];
          nativeBuildInputs = [
            config.treefmt.build.wrapper

            pkgs.rust-analyzer
            pkgs.nixfmt
            pkgs.rustfmt
            pkgs.pkg-config
            pkgs.clang-tools # clangd
            pkgs.gdb
            pkgs.hci
            # TODO: set up cargo-valgrind in shell and build
            #       currently both this and `cargo install cargo-valgrind`
            #       produce a binary that says ENOENT.
            # pkgs.cargo-valgrind
          ]
          ++ pkgs.lib.optionals (pkgs.stdenv.cc.isGNU) [
            pkgs.valgrind
          ];
          shellHook = ''
            ${config.pre-commit.shellHook}
            echo 1>&2 "Welcome to the development shell!"
          '';
          # rust-analyzer needs a NIX_PATH for some reason
          NIX_PATH = "nixpkgs=${inputs.nixpkgs}";
        }
        // pkgs.lib.optionalAttrs (pkgs.stdenv.cc.isGNU) {
          inherit (config.nci.outputs.nix-bindings.devShell.env)
            NIX_CC_UNWRAPPED
            ;
        }
      );
    };
  herculesCI =
    { ... }:
    {
      ciSystems = [ "x86_64-linux" ];
    };
  hercules-ci.flake-update = {
    enable = true;
    baseMerge.enable = true;
    autoMergeMethod = "merge";
    when = {
      dayOfMonth = 1;
    };
    flakes = {
      "." = { };
      "dev" = { };
    };
  };
  hercules-ci.cargo-publish = {
    enable = true;
    secretName = "crates-io";
    assertVersions = true;
  };
  flake = { };
}
