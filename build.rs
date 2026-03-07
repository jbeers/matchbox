use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 1. Get Git commit hash
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 2. Get build date
    let date = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT={}", commit);
    println!("cargo:rustc-env=BUILD_DATE={}", date);

    // 3. Build the runner stub (Independent of Workspace)
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let runner_dir = Path::new(&root_dir).join("crates/matchbox-runner");
    let stub_dest_dir = Path::new(&root_dir).join("stubs");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not found");
    let stub_target_dir = Path::new(&out_dir).join("runner_target");

    // Ensure stubs directory exists
    if !stub_dest_dir.exists() {
        fs::create_dir_all(&stub_dest_dir).expect("Failed to create stubs directory");
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    
    println!("cargo:rerun-if-changed=crates/matchbox-runner/src/main.rs");
    println!("cargo:rerun-if-changed=crates/matchbox-runner/Cargo.toml");

    // Helper closure to build and copy a stub
    let build_stub = |target: Option<&str>, dest_name: &str, src_name: &str| {
        let dest_path = stub_dest_dir.join(dest_name);
        
        let mut cmd = Command::new(&cargo);
        cmd.arg("build").arg("--release")
           .current_dir(&runner_dir)
           .env("CARGO_TARGET_DIR", &stub_target_dir);
           
        if let Some(t) = target {
            cmd.arg("--target").arg(t);
        }

        let output = cmd.output();

        let mut success = false;
        if let Ok(out) = output {
            if out.status.success() {
                let mut src_path = stub_target_dir.clone();
                if let Some(t) = target {
                    src_path = src_path.join(t);
                }
                src_path = src_path.join("release").join(src_name);
                
                if fs::copy(&src_path, &dest_path).is_ok() {
                    success = true;
                    println!("cargo:warning=Runner stub built and copied to {}", dest_path.display());
                } else {
                    println!("cargo:warning=Failed to copy stub from {} to {}", src_path.display(), dest_path.display());
                }
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                println!("cargo:warning=Failed to build stub: {}. Error: {}", dest_name, stderr);
                if !stdout.is_empty() {
                    println!("cargo:warning=Stdout: {}", stdout);
                }
            }
        } else if let Err(e) = output {
            println!("cargo:warning=Failed to execute build command for {}: {}", dest_name, e);
        }

        if !success {
            println!("cargo:warning=Using dummy file for stub: {}.", dest_name);
            if !dest_path.exists() {
                let _ = fs::write(&dest_path, b"");
            }
        }
    };

    let native_src_name = if cfg!(windows) { "matchbox_runner.exe" } else { "matchbox_runner" };
    build_stub(None, "runner_stub_native", native_src_name);
    build_stub(Some("wasm32-wasip1"), "runner_stub_wasip1.wasm", "matchbox_runner.wasm");
}
