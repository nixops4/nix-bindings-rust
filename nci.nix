{
  perSystem =
    { config, ... }:
    let
      cfg = config.nix-bindings-rust;
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
            cfg.nciBuildConfig
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
    };
}
