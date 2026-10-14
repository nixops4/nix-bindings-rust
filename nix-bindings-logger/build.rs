use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/shim.cc");
    println!("cargo:rerun-if-changed=include/nix-bindings-logger/shim.hh");

    let nix_util = pkg_config::Config::new()
        .probe("nix-util")
        .expect("nix-util not found via pkg-config");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut build = cxx_build::bridge("src/bridge.rs");
    for path in &nix_util.include_paths {
        build.include(path);
    }
    build.include(manifest_dir.join("include"));
    build.file("src/shim.cc");
    build.std("c++23");
    build.flag_if_supported("-Wno-unused-parameter");
    build.compile("nix-bindings-logger-cxx");

    for lib in &nix_util.libs {
        println!("cargo:rustc-link-lib={lib}");
    }
    for path in &nix_util.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
}
