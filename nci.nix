{
  perSystem =
    {
      config,
      pkgs,
      ...
    }:
    let
      cfg = config.nix-bindings-rust;
      nixLibs =
        if cfg.nixPackage ? libs then
          cfg.nixPackage.libs
        else
          # Fallback for older Nix versions without split libs
          {
            nix-util-c = cfg.nixPackage;
            nix-store-c = cfg.nixPackage;
            nix-expr-c = cfg.nixPackage;
            nix-fetchers-c = cfg.nixPackage;
            nix-flake-c = cfg.nixPackage;
          };
    in
    {
      # https://flake.parts/options/nix-cargo-integration
      nci.projects.nix-bindings = {
        path = ./.;
        profiles = {
          dev.drvConfig.env.RUSTFLAGS = "-D warnings";
          release.runTests = true;
        };
        drvConfig = {
          imports = [
            # Downstream projects import this into depsDrvConfig instead
            config.nix-bindings-rust.nciBuildConfig
          ];
          # Extra settings for running the tests
          mkDerivation = {
            # Prepare the environment for Nix to work.
            # Nix does not provide a suitable environment for running itself in
            # the sandbox - not by default. We configure it to use a relocated store.
            preCheck = ''
              # nix needs a home directory
              export HOME="$(mktemp -d $TMPDIR/home.XXXXXX)"

              # configure a relocated store
              store_data=$(mktemp -d $TMPDIR/store-data.XXXXXX)
              export NIX_REMOTE="$store_data"
              export NIX_BUILD_HOOK=
              export NIX_CONF_DIR=$store_data/etc
              export NIX_LOCALSTATE_DIR=$store_data/nix/var
              export NIX_LOG_DIR=$store_data/nix/var/log/nix
              export NIX_STATE_DIR=$store_data/nix/var/nix

              echo "Configuring relocated store at $NIX_REMOTE..."

              # Create nix.conf with experimental features enabled
              mkdir -p "$NIX_CONF_DIR"
              echo "experimental-features = ca-derivations flakes" > "$NIX_CONF_DIR/nix.conf"

              # Init ahead of time, because concurrent initialization is flaky
              ${cfg.nixPackage}/bin/nix-store --init

              echo "Store initialized."
            '';
          };
        };
      };

      # Per-crate configuration: only provide the specific Nix libs each crate needs
      # FIXME should use propagatedBuildInputs
      nci.crates.nix-bindings-bdwgc-sys.drvConfig.mkDerivation.buildInputs = [
        pkgs.boehmgc
      ];
      nci.crates.nix-bindings-util-sys.drvConfig.mkDerivation.buildInputs = [
        nixLibs.nix-util-c
      ];
      nci.crates.nix-bindings-util.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-util-sys.drvConfig.mkDerivation.buildInputs;
      nci.crates.nix-bindings-store-sys.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-util-sys.drvConfig.mkDerivation.buildInputs
        ++ [ nixLibs.nix-store-c ];
      nci.crates.nix-bindings-store.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-store-sys.drvConfig.mkDerivation.buildInputs;
      nci.crates.nix-bindings-expr-sys.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-store-sys.drvConfig.mkDerivation.buildInputs
        ++ [
          nixLibs.nix-expr-c
          pkgs.boehmgc
        ];
      nci.crates.nix-bindings-expr.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-expr-sys.drvConfig.mkDerivation.buildInputs;
      nci.crates.nix-bindings-fetchers-sys.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-expr-sys.drvConfig.mkDerivation.buildInputs
        ++ [ nixLibs.nix-fetchers-c ];
      nci.crates.nix-bindings-fetchers.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-fetchers-sys.drvConfig.mkDerivation.buildInputs;
      nci.crates.nix-bindings-flake-sys.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-fetchers-sys.drvConfig.mkDerivation.buildInputs
        ++ [ nixLibs.nix-flake-c ];
      nci.crates.nix-bindings-flake.drvConfig.mkDerivation.buildInputs =
        config.nci.crates.nix-bindings-flake-sys.drvConfig.mkDerivation.buildInputs;
    };
}
