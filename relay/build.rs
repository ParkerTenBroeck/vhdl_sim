use std::env;

fn main() {
    let artifact_path = env::var_os("CARGO_STATICLIB_FILE_LIBVHDL_CONN")
        .expect("missing staticlib artifact for build-dependency `libvhdl_conn`");

    println!("cargo:rerun-if-changed={}", artifact_path.display());
    println!(
        "cargo:rustc-env=EMBEDDED_VHDL_CONN_LIB_PATH={}",
        artifact_path.display()
    );
}
