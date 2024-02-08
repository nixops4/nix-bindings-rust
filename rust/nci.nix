{
  perSystem = { pkgs, config, ... }: {
    # https://flake.parts/options/nix-cargo-integration
    nci.projects.nix-bindings.path = ./.;
  };
}
