use std::path::PathBuf;

#[derive(Debug)]
struct StripNixPrefix;

impl bindgen::callbacks::ParseCallbacks for StripNixPrefix {
    fn item_name(&self, name: &str) -> Option<String> {
        name.strip_prefix("nix_").map(String::from)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=include/nix-c-fetchers.h");
    println!("cargo:rustc-link-lib=nixfetchersc");

    let mut args = Vec::new();
    for path in pkg_config::probe_library("nix-fetchers-c")
        .unwrap()
        .include_paths
        .iter()
    {
        args.push(format!("-I{}", path.to_str().unwrap()));
    }

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header("include/nix-c-fetchers.h")
        .clang_args(args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(StripNixPrefix))
        // Blocklist symbols from nix-bindings-util-sys
        .blocklist_file(".*nix_api_util\\.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
