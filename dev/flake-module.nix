{
  inputs,
  withSystem,
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

      # Combined rustdoc for all crates with cross-linking.
      # NOTE: nci.outputs.nix-bindings.docs uses doc-merge which doesn't support
      # rustdoc's new sharded search index format (Rust 1.78+).
      # See https://github.com/90-008/nix-cargo-integration/issues/198
      # Instead, we build all workspace crates together so rustdoc can link them.
      packages.docs =
        let
          # Use nix-bindings-flake (has most transitive deps) as base
          base = config.nci.outputs.nix-bindings-flake.packages.release;
          crates = [
            "nix-bindings-bdwgc-sys"
            "nix-bindings-util-sys"
            "nix-bindings-util"
            "nix-bindings-store-sys"
            "nix-bindings-store"
            "nix-bindings-expr-sys"
            "nix-bindings-expr"
            "nix-bindings-fetchers-sys"
            "nix-bindings-fetchers"
            "nix-bindings-flake-sys"
            "nix-bindings-flake"
          ];
          packageFlags = pkgs.lib.concatMapStringsSep " " (c: "-p ${c}") crates;
        in
        (base.extendModules {
          modules = [
            {
              mkDerivation = {
                # Build docs for all crates together (enabling cross-crate linking)
                buildPhase = pkgs.lib.mkForce ''
                  cargo doc $cargoBuildFlags --no-deps --profile $cargoBuildProfile ${packageFlags}
                '';
                checkPhase = pkgs.lib.mkForce ":";
                installPhase = pkgs.lib.mkForce ''
                  mv target/$CARGO_BUILD_TARGET/doc $out

                  # Find rustdoc assets (have hashes in filenames)
                  find_asset() {
                    local pattern="$1"
                    local matches=($out/static.files/$pattern)
                    if [[ ''${#matches[@]} -ne 1 || ! -e "''${matches[0]}" ]]; then
                      echo "Expected exactly one match for $pattern, found: ''${matches[*]}" >&2
                      exit 1
                    fi
                    basename "''${matches[0]}"
                  }
                  rustdoc_css=$(find_asset 'rustdoc-*.css')
                  normalize_css=$(find_asset 'normalize-*.css')
                  storage_js=$(find_asset 'storage-*.js')

                  cat > $out/index.html <<EOF
                  <!DOCTYPE html>
                  <html lang="en">
                  <head>
                    <meta charset="utf-8">
                    <title>nix-bindings-rust</title>
                    <link rel="stylesheet" href="static.files/$normalize_css">
                    <link rel="stylesheet" href="static.files/$rustdoc_css">
                    <script src="static.files/$storage_js"></script>
                    <style>
                      body { max-width: 800px; margin: 2em auto; padding: 0 1em; }
                      h1 { border-bottom: 1px solid var(--border-color); padding-bottom: 0.5em; }
                      ul { list-style: none; padding: 0; }
                      li { margin: 0.5em 0; display: flex; align-items: baseline; }
                      .crate {
                        display: inline-block;
                        min-width: 14em;
                        background-color: var(--code-block-background-color);
                        padding: 0.2em 0.5em;
                        border-radius: 3px;
                      }
                      .desc { color: var(--main-color); opacity: 0.7; margin-left: 1em; }
                      details { margin-top: 1.5em; }
                      summary { cursor: pointer; }
                    </style>
                  </head>
                  <body>
                    <h1>nix-bindings-rust</h1>
                    <p>Rust bindings for the Nix C API</p>
                    <h2>Crates</h2>
                    <ul>
                      <li><span class="crate"><a href="nix_bindings_store/index.html">nix_bindings_store</a></span><span class="desc">— Store operations</span></li>
                      <li><span class="crate"><a href="nix_bindings_expr/index.html">nix_bindings_expr</a></span><span class="desc">— Expression evaluation</span></li>
                      <li><span class="crate"><a href="nix_bindings_fetchers/index.html">nix_bindings_fetchers</a></span><span class="desc">— Fetcher operations</span></li>
                      <li><span class="crate"><a href="nix_bindings_flake/index.html">nix_bindings_flake</a></span><span class="desc">— Flake operations</span></li>
                      <li><span class="crate"><a href="nix_bindings_util/index.html">nix_bindings_util</a></span><span class="desc">— Utilities</span></li>
                    </ul>
                    <details>
                      <summary><h2 style="display: inline;">Low-level bindings</h2></summary>
                      <p>
                        These <code>-sys</code> crates provide raw FFI bindings generated by
                        <a href="https://rust-lang.github.io/rust-bindgen/">bindgen</a>.
                        They expose the C API directly without safety wrappers.
                        Most users should prefer the high-level crates above.
                      </p>
                      <ul>
                        <li><span class="crate"><a href="nix_bindings_store_sys/index.html">nix_bindings_store_sys</a></span><span class="desc">— nix-store-c</span></li>
                        <li><span class="crate"><a href="nix_bindings_expr_sys/index.html">nix_bindings_expr_sys</a></span><span class="desc">— nix-expr-c</span></li>
                        <li><span class="crate"><a href="nix_bindings_fetchers_sys/index.html">nix_bindings_fetchers_sys</a></span><span class="desc">— nix-fetchers-c</span></li>
                        <li><span class="crate"><a href="nix_bindings_flake_sys/index.html">nix_bindings_flake_sys</a></span><span class="desc">— nix-flake-c</span></li>
                        <li><span class="crate"><a href="nix_bindings_util_sys/index.html">nix_bindings_util_sys</a></span><span class="desc">— nix-util-c</span></li>
                        <li><span class="crate"><a href="nix_bindings_bdwgc_sys/index.html">nix_bindings_bdwgc_sys</a></span><span class="desc">— Boehm GC</span></li>
                      </ul>
                    </details>
                  </body>
                  </html>
                  EOF
                '';
              };
            }
          ];
        }).config.public;

      devShells.default = pkgs.mkShell {
        name = "nix-bindings-devshell";
        strictDeps = true;
        inputsFrom = [ config.nci.outputs.nix-bindings.devShell ];
        inherit (config.nci.outputs.nix-bindings.devShell.env)
          LIBCLANG_PATH
          NIX_CC_UNWRAPPED
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
          pkgs.valgrind
          pkgs.gdb
          pkgs.hci
          # TODO: set up cargo-valgrind in shell and build
          #       currently both this and `cargo install cargo-valgrind`
          #       produce a binary that says ENOENT.
          # pkgs.cargo-valgrind
        ];
        shellHook = ''
          ${config.pre-commit.shellHook}
          echo 1>&2 "Welcome to the development shell!"
        '';
        # rust-analyzer needs a NIX_PATH for some reason
        NIX_PATH = "nixpkgs=${inputs.nixpkgs}";
      };
    };
  herculesCI =
    hci@{ lib, ... }:
    {
      ciSystems = [ "x86_64-linux" ];
      onPush.default.outputs = {
        effects.pushDocs = lib.optionalAttrs (hci.config.repo.branch == "main") (
          withSystem "x86_64-linux" (
            { config, hci-effects, ... }:
            hci-effects.gitWriteBranch {
              git.checkout.remote.url = hci.config.repo.remoteHttpUrl;
              git.checkout.forgeType = "github";
              git.checkout.user = "x-access-token";
              git.update.branch = "gh-pages";
              contents = config.packages.docs;
              destination = "development"; # directory
            }
          )
        );
      };
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
