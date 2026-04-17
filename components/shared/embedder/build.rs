/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rustc-check-cfg=cfg(servo_production)");
    println!("cargo::rustc-check-cfg=cfg(servo_do_not_use_in_production)");
    let out = std::env::var("OUT_DIR")?;
    let profile = Path::new(&out)
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .file_name().unwrap()
        .to_string_lossy()
        .into_owned();
    if profile == "production" || profile.starts_with("production-") {
        println!("cargo:rustc-cfg=servo_production");
    } else {
        println!("cargo:rustc-cfg=servo_do_not_use_in_production");
    }

    embedder_bridge_build::run(&out);
    Ok(())
}
