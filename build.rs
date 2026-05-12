use std::{env, process::Command};

fn main() {
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());
    let is_release = opt_level != "0";
    let needs_build = env::var("CARGO_FEATURE_WEB").is_ok();

    if needs_build {
        println!(
            "cargo:warning=Building frontend (opt-level: {})...",
            opt_level
        );

        let status = Command::new("yarn")
            .args(["install"])
            .current_dir("frontend")
            .status()
            .expect("Failed to run yarn. Is it installed?");

        if !status.success() {
            panic!("yarn install failed");
        }

        let status = Command::new("yarn")
            .args(["build"])
            .current_dir("frontend")
            .status()
            .expect("Failed to run yarn build");

        if !status.success() {
            panic!("yarn build failed");
        }
    }

    if is_release {
        println!("cargo:rerun-if-changed=frontend/src");
        println!("cargo:rerun-if-changed=frontend/public");
    }
}
