//! Build the web page with trunk so the server can embed it.

use std::env;
use std::path::PathBuf;
use std::process::Command;

const INPUTS: [&str; 10] = [
    "web/src",
    "web/index.html",
    "web/style.css",
    "web/icons",
    "web/manifest.webmanifest",
    "web/sw.js",
    "web/Cargo.toml",
    "web/Trunk.toml",
    "vestaboard/src",
    "vestaboard/Cargo.toml",
];

fn main() {
    for input in INPUTS {
        println!("cargo::rerun-if-changed={input}");
    }

    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let mut trunk = Command::new("trunk");
    trunk.arg("build").current_dir(root.join("web"));
    // Keep this build's cargo settings out of trunk's separate wasm build.
    trunk.env_clear();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
    ] {
        if let Some(value) = env::var_os(key) {
            trunk.env(key, value);
        }
    }
    // A separate target directory avoids waiting on the lock this build already holds.
    trunk.env("CARGO_TARGET_DIR", root.join("target/web"));
    if env::var("PROFILE").as_deref() == Ok("release") {
        trunk.arg("--release");
    }

    let output = trunk.output().unwrap_or_else(|err| {
        panic!(
            "couldn't run trunk ({err}): install it with `brew install trunk` and \
             `rustup target add wasm32-unknown-unknown`"
        )
    });
    if !output.status.success() {
        panic!(
            "trunk build failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
