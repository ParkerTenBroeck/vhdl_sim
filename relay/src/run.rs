use std::path::Path;

use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

use crate::build::{BuildArtifact, Simulator};

pub struct Process {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub stderr: ChildStderr,
}

pub async fn run(
    artifact_dir: &Path,
    artifact: &BuildArtifact,
) -> Result<Process, Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = match artifact.simulator {
        Simulator::Ghdl => {
            let mut cmd = Command::new("ghdl");
            cmd.args([
                "-r",
                "--std=08",
                "tb",
                "--stop-delta=4294967296",
                "--unbuffered",
                "--",
            ]);
            cmd
        }
        Simulator::Verilator => Command::new(&artifact.run_target),
    };

    cmd.args(std::env::args_os());
    cmd.current_dir(artifact_dir);
    cmd.kill_on_drop(true);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().ok_or("no stdin")?;
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    Ok(Process {
        child,
        stdin,
        stdout,
        stderr,
    })
}
