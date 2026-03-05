use std::{collections::HashMap, ops::Deref, path::{Path, PathBuf}};
use tokio::{
    process::{Child, Command},
};

async fn ensure_ok(child: Child) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let result = child.wait_with_output().await?;
    if !result.status.success(){
        return Err(format!("{}\n{}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)))?
    }
    Ok(())
}

pub struct TempDir(PathBuf);
impl Drop for TempDir{
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.0)
    }
}

impl Deref for TempDir{
    type Target = PathBuf;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PathBuf> for TempDir{
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}
impl AsRef<Path> for TempDir{
    fn as_ref(&self) -> &Path {
        &self.0
    }
}


pub async fn build(files: HashMap<String, String>) -> Result<TempDir, Box<dyn std::error::Error + Send + Sync>>{
        use std::hash::*;
        let mut hasher = std::hash::DefaultHasher::default();
        for (key, value) in &files{
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        let hash = hasher.finish();

        let mut work_dir = std::env::temp_dir();
        work_dir.push(format!("ghdl-relay-{hash:x?}"));
        _ = std::fs::create_dir(&work_dir);
        let work_dir = TempDir(work_dir);


        for (name, contents) in &files{
            let mut path = work_dir.clone();
            path.push(name);
            std::fs::write(path, contents)?;
        }
        
        
        let mut cmd = Command::new("ghdl");
        cmd.kill_on_drop(true);
        cmd.args(["-a", "-g", "--std=08"]);
        for name in files.keys(){
            let mut path = work_dir.clone();
            path.push(name);
            cmd.arg(path);
        }
        cmd.arg(std::fs::canonicalize("../rtl/tb.vhdl")?);
        
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        
        cmd.current_dir(&work_dir);
        ensure_ok(cmd.spawn()?).await?;

        let mut cmd = Command::new("ghdl");
        cmd.kill_on_drop(true);
        cmd.args(["-e", "--std=08"]);
        cmd.arg(format!("-Wl,{}", std::fs::canonicalize("../conn/target/release/libvhdl_ui.a")?.display()));
        cmd.arg("tb");
        cmd.current_dir(&work_dir);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        ensure_ok(cmd.spawn()?).await?;

        Ok(work_dir)
}