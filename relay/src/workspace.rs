use axum::{
    Error,
    extract::ws::{Message, WebSocket},
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
    time::Instant,
};

use crate::{ClientMsg, ServerMsg, build, run};

struct Process {
    process: Child,

    stderr: Lines<BufReader<ChildStderr>>,
    stdout: Lines<BufReader<ChildStdout>>,
    stdin: ChildStdin,
}

struct Handler {
    sender: SplitSink<WebSocket, Message>,
    receiver: SplitStream<WebSocket>,

    build_dir: PathBuf,
    src_dir: PathBuf,

    process: Option<Process>,

    refresh_time: Duration,
}

impl Handler {
    fn workspace(socket: WebSocket, build: PathBuf, src: PathBuf, refresh_time: Duration) -> Self {
        let (sender, receiver) = socket.split();
        Self {
            sender,
            receiver,
            build_dir: build,
            src_dir: src,
            process: None,
            refresh_time,
        }
    }

    async fn print(&mut self, msg: impl AsRef<str>) {
        println!("stdout: {}", msg.as_ref());
        let msg = ServerMsg::Log {
            stream: "stdout",
            line: msg.as_ref(),
        };
        _ = self
            .sender
            .send(Message::Text(
                serde_json::to_string(&msg).unwrap_or_default().into(),
            ))
            .await;
    }

    pub async fn eprint(&mut self, msg: impl AsRef<str>) {
        println!("stderr: {}", msg.as_ref());
        let msg = ServerMsg::Log {
            stream: "stderr",
            line: msg.as_ref(),
        };
        _ = self
            .sender
            .send(Message::Text(
                serde_json::to_string(&msg).unwrap_or_default().into(),
            ))
            .await;
    }

    async fn stop_process(&mut self) {
        self.process = None;
        _ = self
            .sender
            .send(Message::Text(
                serde_json::to_string(&ServerMsg::Stop)
                    .unwrap_or_default()
                    .into(),
            ))
            .await;
    }

    async fn handle_websocket_msg(&mut self, msg: ClientMsg) {
        match msg {
            ClientMsg::Start => self.run_program().await,
            ClientMsg::Stop => self.stop_process().await,
            ClientMsg::Input { switch, buttons } => {
                if let Some(process) = &mut self.process {
                    use tokio::io::AsyncWriteExt;
                    _ = process
                        .stdin
                        .write_all(format!("btn={}\n", buttons).as_bytes())
                        .await;
                    _ = process
                        .stdin
                        .write_all(format!("sw={}\n", switch).as_bytes())
                        .await;
                }
            }
        }
    }

    async fn handle_websocket_receive(&mut self, msg: Option<Result<Message, Error>>) -> bool {
        match msg {
            Some(Ok(Message::Close(_))) => true,
            Some(Ok(Message::Text(msg))) => {
                let msg = match serde_json::from_str(msg.as_str()) {
                    Ok(msg) => msg,
                    Err(err) => {
                        self.eprint(format!("Client message error {err}")).await;
                        return false;
                    }
                };
                self.handle_websocket_msg(msg).await;
                false
            }
            Some(Ok(_)) => false,
            Some(Err(err)) => {
                self.eprint(format!("Client websocket error {err}")).await;
                true
            }
            None => true,
        }
    }

    async fn run(&mut self) {
        loop {
            if let Some(process) = &mut self.process {
                if let Ok(Some(_)) = process.process.try_wait() {
                    self.stop_process().await;
                    continue;
                }

                let mut print_deadline = Instant::now();

                tokio::select! {
                    receive = self.receiver.next() => {
                        if self.handle_websocket_receive(receive).await{
                            break;
                        }
                    }
                    out = process.stdout.next_line() => {
                        match out {
                            Ok(Some(line)) => self.print(line).await,
                            Ok(None) => self.stop_process().await,
                            Err(err) => {
                                self.eprint(format!("Failed to read proccess sout: {err}")).await;
                                self.stop_process().await;
                            }
                        }
                    }
                    err = process.stderr.next_line() => {
                        match err{
                            Ok(Some(line)) => {
                                let msg = if let Some(repr) = line.strip_prefix("led="){
                                    ServerMsg::Led(repr.parse().unwrap_or(0))
                                }else if let Some(repr) = line.strip_prefix("seg=")
                                    && let Some((val, idx)) = repr.split_once(";") {
                                    ServerMsg::Seg{
                                        value: val.parse().unwrap_or(0),
                                        index: idx.parse().unwrap_or(0)
                                    }
                                }else{
                                    self.eprint(line).await;
                                    continue;
                                };
                                _ = self.sender.send(Message::Text(serde_json::to_string(&msg).unwrap_or_default().into())).await;
                            },
                            Ok(None) => self.stop_process().await,
                            Err(err) => {
                                self.eprint(format!("Failed to read proccess serr: {err}")).await;
                                self.stop_process().await;
                            }
                        }
                    }
                    _ = tokio::time::sleep_until(print_deadline) => {
                        use tokio::io::AsyncWriteExt;
                        print_deadline += self.refresh_time;
                        _ = process.stdin.write_all("\n".as_bytes()).await;
                    }
                }
            } else {
                let res = self.receiver.next().await;
                if self.handle_websocket_receive(res).await {
                    break;
                }
            }
        }
    }

    async fn run_program(&mut self) {
        let artifact = match build::build(&self.build_dir, &self.src_dir).await {
            Ok(artifact) => artifact,
            Err(err) => {
                _ = self.eprint(format!("Failed to build: {err}")).await;
                return;
            }
        };

        let process = match run::run(&self.build_dir, &artifact).await {
            Ok(process) => process,
            Err(err) => {
                self.eprint(format!("Failed to run: {err}")).await;
                return;
            }
        };
        let stdout = BufReader::new(process.stdout).lines();
        let stderr = BufReader::new(process.stderr).lines();
        let stdin = process.stdin;

        _ = self
            .sender
            .send(Message::Text(
                serde_json::to_string(&ServerMsg::Start)
                    .unwrap_or_default()
                    .into(),
            ))
            .await;
        self.process = Some(Process {
            process: process.child,
            stderr,
            stdout,
            stdin,
        })
    }
}

pub async fn ws_handler(socket: WebSocket, workspace_src: PathBuf, refresh_time: Duration) {
    Handler::workspace(socket, "./target".into(), workspace_src, refresh_time)
        .run()
        .await;
}
