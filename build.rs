#[cfg(feature = "ffi")]
extern crate cbindgen;

#[cfg(feature = "ffi")]
use std::env;

#[cfg(feature = "ffi")]
fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("squire_core.h");
}

#[cfg(not(feature = "ffi"))]
fn main() {}
