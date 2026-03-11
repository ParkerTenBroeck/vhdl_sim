use std::{
    collections::HashMap, ffi::OsStr, ops::Deref, path::{Path, PathBuf}
};
use tokio::process::{Child, Command};

use crate::HResult;

const EMBEDDED_VHDL_UI_LIB: &[u8] = include_bytes!(env!("EMBEDDED_VHDL_CONN_LIB_PATH"));

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

pub async fn copy_and_build(
    files: HashMap<String, String>,
) -> HResult<TempDir> {
    use std::hash::*;
    let mut hasher = std::hash::DefaultHasher::default();
    for (key, value) in &files {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }
    let hash = hasher.finish();

    let mut work_dir = std::env::temp_dir();
    work_dir.push(format!("ghdl-relay-{hash:x?}"));
    std::fs::create_dir_all(&work_dir)?;
    let work_dir = TempDir(work_dir);

    for (name, contents) in &files {
        let mut path = work_dir.clone();
        path.push(name);
        std::fs::write(path, contents)?;
    }

    build(&work_dir, &work_dir).await?;

    Ok(work_dir)
}


pub async fn build(path: &Path, src: &Path) -> HResult<()>{
    std::fs::create_dir_all(path)?;
    let embedded_lib_path = path.join("libvhdl_conn.a");
    std::fs::write(&embedded_lib_path, EMBEDDED_VHDL_UI_LIB)?;

        let mut cmd = Command::new("ghdl");
    cmd.kill_on_drop(true);
    cmd.args(["-i", "-g", "--std=08"]);

    for file in src.read_dir().unwrap().flatten(){
        if Path::new(&file.file_name()).extension() == Some(OsStr::new("vhdl")) {
            cmd.arg(file.path());
        }
    }

    cmd.arg(std::fs::canonicalize("../rtl/tb.vhdl")?);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    cmd.current_dir(path);
    ensure_ok(cmd.spawn()?).await?;





    let mut cmd = Command::new("ghdl");
    cmd.kill_on_drop(true);
    cmd.args(["-m", "--std=08"]);
    cmd.arg(format!(
        "-Wl,{}",
        embedded_lib_path.display()
    ));
    cmd.arg("tb");
    cmd.current_dir(path);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    ensure_ok(cmd.spawn()?).await?;
    
    Ok(())
}
