use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    match pkg_config::probe_library("cups") {
        Ok(_) => println!("Found CUPS using pkg-config"),
        Err(e) => {
            println!("cargo:warning=Failed to find CUPS with pkg-config: {}", e);
            println!("cargo:rustc-link-lib=cups");
        }
    }

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allow CUPS types and functions
        .allowlist_function("cups.*")
        .allowlist_type("cups_.*")
        .allowlist_var("CUPS_.*")
        // Allow HTTP types and functions (needed for http_t)
        .allowlist_function("http.*")
        .allowlist_type("http_.*")
        .allowlist_var("HTTP_.*")
        // Allow IPP types and functions
        .allowlist_function("ipp.*")
        .allowlist_type("ipp_.*")
        .allowlist_var("IPP_.*")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
