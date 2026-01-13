use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=include/bdwgc.h");

    let mut args = Vec::new();
    for path in pkg_config::probe_library("bdw-gc")
        .unwrap()
        .include_paths
        .iter()
    {
        args.push(format!("-I{}", path.to_str().unwrap()));
    }

    let bindings = bindgen::Builder::default()
        .header("include/bdwgc.h")
        .clang_args(args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
