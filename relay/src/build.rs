use std::{
    collections::HashMap,
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};
use tokio::process::{Child, Command};

use crate::HResult;

const EMBEDDED_VHDL_UI_LIB: &[u8] = include_bytes!(env!("EMBEDDED_VHDL_CONN_LIB_PATH"));
const EMBEDDED_TB_VHDL: &str = include_str!("../shim/shim.vhdl");
const EMBEDDED_TB_VERILATOR: &str = include_str!("../shim/verilog.c");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Simulator {
    Ghdl,
    Verilator,
}

#[derive(Debug, Clone)]
pub struct BuildArtifact {
    pub simulator: Simulator,
    pub run_target: PathBuf,
}

async fn ensure_ok(child: Child) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = child.wait_with_output().await?;
    if !result.status.success() {
        return Err(format!(
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ))?;
    }
    Ok(())
}

fn source_files(src: &Path) -> HResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for file in src.read_dir()?.flatten() {
        let path = file.path();
        if path.is_file() {
            files.push(path.canonicalize()?);
        }
    }
    files.sort();
    Ok(files)
}

fn detect_simulator(files: &[PathBuf]) -> HResult<Simulator> {
    let mut has_vhdl = false;
    let mut has_verilog = false;

    for file in files {
        match file.extension().and_then(OsStr::to_str) {
            Some("vhdl" | "vhd") => has_vhdl = true,
            Some("v" | "sv") => has_verilog = true,
            _ => {}
        }
    }

    match (has_vhdl, has_verilog) {
        (true, false) => Ok(Simulator::Ghdl),
        (false, true) => Ok(Simulator::Verilator),
        (true, true) => Err("mixed VHDL and Verilog sources are not supported yet".into()),
        (false, false) => Err("no VHDL or Verilog source files found".into()),
    }
}

async fn build_with_ghdl(
    build: &Path,
    files: &[PathBuf],
    embedded_lib_path: &Path,
) -> HResult<BuildArtifact> {
    let embedded_tb_path = build.join("tb.vhdl");
    std::fs::write(&embedded_tb_path, EMBEDDED_TB_VHDL)?;

    let mut cmd = Command::new("ghdl");
    cmd.kill_on_drop(true);
    cmd.args(["-i", "-g", "--std=08"]);

    for file in files {
        if matches!(
            file.extension().and_then(OsStr::to_str),
            Some("vhdl" | "vhd")
        ) {
            cmd.arg(file);
        }
    }
    cmd.arg(&embedded_tb_path.canonicalize()?);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    cmd.current_dir(build);
    ensure_ok(cmd.spawn()?).await?;

    let mut cmd = Command::new("ghdl");
    cmd.kill_on_drop(true);
    cmd.args(["-m", "--std=08"]);
    cmd.arg(format!(
        "-Wl,{}",
        embedded_lib_path.canonicalize()?.display()
    ));
    cmd.arg("tb");
    cmd.current_dir(build);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    ensure_ok(cmd.spawn()?).await?;

    Ok(BuildArtifact {
        simulator: Simulator::Ghdl,
        run_target: build.join("tb"),
    })
}

async fn build_with_verilator(
    build: &Path,
    files: &[PathBuf],
    embedded_lib_path: &Path,
) -> HResult<BuildArtifact> {
    let embedded_tb_path = build.join("tb.cpp");
    let obj_dir = build.join("obj_dir");
    std::fs::write(&embedded_tb_path, EMBEDDED_TB_VERILATOR)?;
    std::fs::create_dir_all(&obj_dir)?;

    let mut cmd = Command::new("verilator");
    cmd.kill_on_drop(true);
    cmd.args(["--cc", "--exe", "--top-module", "circuit", "--Mdir"]);
    cmd.arg(&obj_dir);
    cmd.args(["-o", "tb"]);
    cmd.args([
        "-LDFLAGS",
        &embedded_lib_path.canonicalize()?.display().to_string(),
    ]);
    cmd.arg(&embedded_tb_path);

    for file in files {
        if matches!(file.extension().and_then(OsStr::to_str), Some("v" | "sv")) {
            cmd.arg(file);
        }
    }

    cmd.current_dir(build);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    ensure_ok(cmd.spawn()?).await?;

    let mut cmd = Command::new("make");
    cmd.kill_on_drop(true);
    cmd.args(["-C"]);
    cmd.arg(&obj_dir);
    cmd.args(["-f", "Vcircuit.mk", "-j", "1"]);
    cmd.current_dir(build);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    ensure_ok(cmd.spawn()?).await?;

    Ok(BuildArtifact {
        simulator: Simulator::Verilator,
        run_target: obj_dir.join("tb"),
    })
}

pub struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.0)
    }
}

impl Deref for TempDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PathBuf> for TempDir {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}
impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

pub struct TempBuild {
    pub dir: TempDir,
    pub artifact: BuildArtifact,
}

pub async fn copy_and_build(files: HashMap<String, String>) -> HResult<TempBuild> {
    use std::hash::*;
    let mut hasher = std::hash::DefaultHasher::default();
    for (key, value) in &files {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }
    let hash = hasher.finish();

    let mut work_dir = std::env::temp_dir();
    work_dir.push(format!("hdl-relay-{hash:x?}"));
    std::fs::create_dir_all(&work_dir)?;
    let work_dir = TempDir(work_dir);

    for (name, contents) in &files {
        let mut path = work_dir.clone();
        path.push(name);
        std::fs::write(path, contents)?;
    }

    let artifact = build(&work_dir, &work_dir).await?;

    Ok(TempBuild {
        dir: work_dir,
        artifact,
    })
}

pub async fn build(build: &Path, src: &Path) -> HResult<BuildArtifact> {
    let build = build.canonicalize()?;
    let src = src.canonicalize()?;
    std::fs::create_dir_all(&build)?;
    let embedded_lib_path = build.join("libvhdl_conn.a");
    std::fs::write(&embedded_lib_path, EMBEDDED_VHDL_UI_LIB)?;
    let files = source_files(&src)?;
    match detect_simulator(&files)? {
        Simulator::Ghdl => build_with_ghdl(&build, &files, &embedded_lib_path).await,
        Simulator::Verilator => build_with_verilator(&build, &files, &embedded_lib_path).await,
    }
}
