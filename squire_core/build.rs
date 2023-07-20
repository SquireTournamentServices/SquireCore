/* This build script ensure that everything needed to run the SquireCore server is in its place.
 * Primarily, this includes the static assets for the frontend, including the index, wasm app, and
 * JS bindings. Trunk is used to compile and generate the app and the JS bindings.
 */

use std::{env, process::Command};

fn main() -> Result<(), i32> {
    let wd = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Install external dependency (in the shuttle container only)
    if std::env::var("HOSTNAME")
        .unwrap_or_default()
        .contains("shuttle")
    {
        // Install the `wasm32-unknown-unknown` target
        if !std::process::Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status()
            .expect("failed to run rustup")
            .success()
        {
            panic!("failed to wasm32 target")
        }

        // Install `trunk` to compile the frontend
        if !std::process::Command::new("cargo")
            .args(["install", "trunk"])
            .status()
            .expect("failed to run cargo")
            .success()
        {
            panic!("failed to install trunk")
        }
    }

    let sw_path = format!("{wd}/../squire_web");
    let mut cmd = Command::new("trunk");
    cmd.args(["build", "-d", "../assets", "--filehash", "false"]);

    if Ok("release".to_owned()) == env::var("PROFILE") {
        cmd.arg("--release");
    }
    cmd.arg(format!("{sw_path}/index.html"));
    let status = cmd.status().map(|s| s.success());
    if status.unwrap_or(true) {
        return Err(1);
    }
    println!("cargo:rerun-if-changed={sw_path}");
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
