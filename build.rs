use std::{env, path::Path, process::Command};

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let dist_exists = Path::new("frontend/dist").exists();

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/public");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/yarn.lock");
    println!("cargo:rerun-if-changed=build.rs");

    if profile == "release" || !dist_exists {
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
    } else {
        println!("cargo:warning=Skipping frontend build in debug profile (dist folder exists).");
    }
}
