#![feature(let_chains)]

/* This build script ensure that everything needed to run the SquireCore server is in its place.
 * Primarily, this includes the static assets for the frontend, including the index, wasm app, and
 * JS bindings. Trunk is used to compile and generate the app and the JS bindings.
 */

use std::{env, process::Command};

fn main() -> Result<(), i32> {
    let wd = env::var("CARGO_MANIFEST_DIR").unwrap();
    let sw_path = format!("{wd}/../squire_web");
    let mut cmd = Command::new("trunk");
    cmd.args(["build", "-d", "../assets", "--filehash", "false"]);

    if Ok("release".to_owned()) == env::var("PROFILE") {
        cmd.arg("--release");
    }
    cmd.arg(format!("{sw_path}/index.html"));
    let status = cmd.status().map(|s| s.success());
    if let Ok(false) | Err(_) = status {
        return Err(1);
    }
    println!("cargo:rerun-if-changed={sw_path}");
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
