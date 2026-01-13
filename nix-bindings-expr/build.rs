use nix_bindings_util::nix_version::emit_version_cfg;

fn main() {
    let nix_version = pkg_config::probe_library("nix-expr-c").unwrap().version;
    emit_version_cfg(&nix_version, &["2.26"]);
}
