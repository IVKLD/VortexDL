use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());
    let is_release = opt_level != "0";
    let frontend_dist = "frontend/dist/voltexdl/browser";
    let needs_build = !Path::new(frontend_dist).exists() || is_release;

    if needs_build {
        println!("cargo:warning=Building frontend (opt-level: {})...", opt_level);
        
        let status = Command::new("yarn")
            .args(["install"])
            .current_dir("frontend")
            .status()
            .expect("Failed to run yarn install");

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
