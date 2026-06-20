use std::{env, process::Command};

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_release = profile == "release";
    let needs_build = env::var("CARGO_FEATURE_WEB").is_ok();

    if needs_build {
        println!("cargo:warning=Building frontend (profile: {})...", profile);

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
