use std::{path::Path};

use tokio::process::{Child, ChildStdin, ChildStdout, ChildStderr, Command};

pub struct Process{
    child: Child,
}

impl Drop for Process{
    fn drop(&mut self) {
        _ = self.child.start_kill()
    }
}

pub async fn run(artifact_dir: &Path)  -> Result<(Process, ChildStdin, ChildStdout, ChildStderr), Box<dyn std::error::Error + Send + Sync>>{
        let mut cmd = Command::new("ghdl");
    cmd.args(["-r", "--std=08", "tb", "--stop-delta=2147483647", "--unbuffered"]);
    cmd.current_dir(artifact_dir);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().ok_or("no stdin")?;
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    Ok((Process { child }, stdin, stdout, stderr))
}