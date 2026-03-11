use std::path::PathBuf;
use std::process::Command;

fn main() {
    // silly hack of sorts because bindeps are unstable


    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR was not set by Cargo"),
    );
    let workspace_root = manifest_dir
        .parent()
        .expect("relay crate should live under a workspace root");

    println!("cargo:rerun-if-changed={}", workspace_root.join("conn").display());

    let isolated_target_dir = workspace_root.join("target").join("conn-build");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--package")
        .arg("libvhdl_conn")
        .arg("--release")
        .arg("--lib")
        .arg("--target-dir")
        .arg(&isolated_target_dir)
        .current_dir(workspace_root)
        .status()
        .expect("failed to spawn cargo build for conn");

    if !status.success() {
        panic!(
            "build script failed: `cargo build --package conn --release --lib` exited with {status}"
        );
    }

    // Copy the built staticlib into the workspace release target path used by relay/src/build.rs.
    let isolated_release_dir = isolated_target_dir.join("release");
    let out_release_dir = workspace_root.join("target").join("release");
    std::fs::create_dir_all(&out_release_dir).expect("failed to create workspace target/release");

    // conn currently builds as libconn.a; keep a compatibility alias for relay/src/build.rs (libvhdl_ui.a).
    let src_lib = isolated_release_dir.join("libconn.a");
    if !src_lib.exists() {
        panic!(
            "build script failed: expected static library not found at {}",
            src_lib.display()
        );
    }

    let dst_conn = out_release_dir.join("libvhdl_conn.a");
    std::fs::copy(&src_lib, &dst_conn).expect("failed to copy libconn.a into workspace target/release");

    let dst_compat = out_release_dir.join("libvhdl_conn.a");
    std::fs::copy(&src_lib, &dst_compat)
        .expect("failed to copy compatibility libvhdl_conn.a into workspace target/release");
}
