/* This build script ensure that everything needed to run the SquireCore server is in its place.
 * Primarily, this includes the static assets for the frontend, including the index, wasm app, and
 * JS bindings. Trunk is used to compile and generate the app and the JS bindings.
 */

use std::{env, process::Command};
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use flate2::write::GzEncoder;
use flate2::Compression;

fn main() -> Result<(), i32> {
    if env::var("CARGO_FEATURE_IGNORE_FRONTEND").is_ok() {
        return Ok(())
    }

    let wd = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fe_path = format!("{wd}/../squire_web");

    println!("cargo:rerun-if-changed={fe_path}");

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
            panic!("failed to install wasm32 target")
        }

        // Install `trunk` to compile the frontend
        if !std::process::Command::new("cargo")
            .args(["install", "trunk"])
            .status()
            .expect("failed to run cargo install")
            .success()
        {
            panic!("failed to install trunk")
        }
    }

    let mut cmd = Command::new("trunk");
    cmd.args(["build", "-d", "../assets", "--filehash", "false"]);

    let is_release = env::var("PROFILE")
        .map(|v| v == "release")
        .unwrap_or_default();

    if is_release {
        cmd.arg("--release");
    }
    cmd.arg(format!("{fe_path}/index.html"));

    // Compresses the wasm package
    let wasm_package = format!("{}/../assets/squire_web_bg.wasm", wd);
    let mut wasm_file = File::open(&wasm_package).expect("Failed to open WASM file");
    let mut wasm_data = Vec::new();
    wasm_file.read_to_end(&mut wasm_data).unwrap();

    let output_path = format!("{}/../build_outputs/squire_web_bg.wasm.gz", wd);
    let output_file = File::create(&output_path).unwrap();  
    let mut encoder = GzEncoder::new(BufWriter::new(output_file), Compression::default());
    encoder.write_all(&wasm_data).unwrap();

    // If in debug mode, all for failed compilation of frontend.
    // In release mode, require that the frontend to be functional.
    if matches!(cmd.status().map(|s| s.success()), Ok(false) | Err(_)) && is_release {
        Err(1)
    } else {
        Ok(())
    }
}

