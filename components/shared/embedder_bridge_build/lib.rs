/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared build-script helper for the embedder bridge codegen.
//!
//! Each crate whose enum needs bridge variants (embedder_traits,
//! constellation_traits, script_traits) adds this as a build-dependency
//! and calls `embedder_bridge_build::run()` from its `build.rs`.

use std::path::PathBuf;
use std::process::Command;

// The path to this helper crate's own manifest directory, baked in at
// compile time. The Python script lives two levels up from here:
//   components/shared/embedder_bridge_build/  ← this crate
//   components/script_bindings/codegen/embedder_bridge.py
const SELF_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// Run the embedder bridge Python codegen, writing output files into `out_dir`.
///
/// Emits the necessary `cargo::rerun-if-*` directives automatically.
/// Exits with a clear error if `SERVO_BRIDGE_WEBIDL` is not set.
pub fn run(out_dir: &str) {
    let script = PathBuf::from(SELF_MANIFEST_DIR)
        .join("../../script_bindings/codegen/embedder_bridge.py");

    println!("cargo::rerun-if-changed={}", script.display());
    println!("cargo::rerun-if-env-changed=SERVO_BRIDGE_WEBIDL");

    // Also watch the templates so changes to .j2 files trigger a rebuild.
    let templates_dir = PathBuf::from(SELF_MANIFEST_DIR)
        .join("../../script_bindings/codegen/bridge/templates");
    println!("cargo::rerun-if-changed={}", templates_dir.display());

    let bridge_webidl = match std::env::var("SERVO_BRIDGE_WEBIDL") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!(
                "error[embedder_bridge_build]: SERVO_BRIDGE_WEBIDL is not set.\n\
                 Set it to the path of your bridge WebIDL file before building.\n\
                 Example: SERVO_BRIDGE_WEBIDL=/path/to/bridge.webidl cargo build"
            );
            std::process::exit(1);
        },
    };
    println!("cargo::rerun-if-changed={bridge_webidl}");

    let status = find_python()
        .arg(&script)
        .arg(out_dir)
        .status()
        .expect("failed to launch Python");

    if !status.success() {
        std::process::exit(1);
    }
}

fn find_python() -> Command {
    if try_python("uv", &["run", "--frozen", "python"]) {
        let mut cmd = Command::new("uv");
        cmd.args(["run", "--frozen", "python"]);
        return cmd;
    }
    if try_python("python3", &[]) {
        return Command::new("python3");
    }
    if try_python("python", &[]) {
        return Command::new("python");
    }
    panic!("No suitable python found! Tried: `uv run python`, `python3`, `python`.");
}

fn try_python(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
