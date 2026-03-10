use axum::{
    Error, extract::ws::{Message, WebSocket}
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
};

use crate::{build, run};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    Compile(Option<HashMap<String, String>>),
    Start,
    Stop,
    Input {
        /// bitfield of 32 switches
        switch: u32,
        /// bitfield of 32 buttons
        buttons: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ServerMsg<'a> {
    Log { stream: &'a str, line: &'a str },
    Start,
    Stop,
    Led(u32),
    Seg0(u32),
    Seg1(u32),
    Seg2(u32),
    Seg3(u32),
}

struct Process {
    process: Child,

    stderr: Lines<BufReader<ChildStderr>>,
    stdout: Lines<BufReader<ChildStdout>>,
    stdin: ChildStdin,
}

enum Mode {
    SingleLocal,
    Remote,
}

struct Handler {
    sender: SplitSink<WebSocket, Message>,
    receiver: SplitStream<WebSocket>,

    build_dir: PathBuf,
    src_dir: PathBuf,

    program: Option<PathBuf>,
    process: Option<Process>,

    refresh_time: Duration,
}

type HResult<T> = Result<T, Box<dyn std::error::Error + Sync + Send>>;

impl Handler {

    async fn local(socket: WebSocket, build: PathBuf, src: PathBuf) -> Self {
        let (sender, receiver) = socket.split();
        Self {
            sender,
            receiver,
            build_dir: build,
            src_dir: src,
            program: None,
            process: None,
            refresh_time: Duration::from_millis(30),
        }
    }

    async fn print(&mut self, msg: impl AsRef<str>) {
        println!("stdout: {}", msg.as_ref());
        let msg = ServerMsg::Log {
            stream: "stdout",
            line: msg.as_ref(),
        };
        _ = self.sender.send(Message::Text(serde_json::to_string(&msg).unwrap_or_default().into())).await;
    }

    pub async fn eprint(&mut self, msg: impl AsRef<str>) {
        println!("stderr: {}", msg.as_ref());
        let msg = ServerMsg::Log {
            stream: "stderr",
            line: msg.as_ref(),
        };
        _ = self.sender.send(Message::Text(serde_json::to_string(&msg).unwrap_or_default().into())).await;
    }

    async fn stop_process(&mut self) {
        self.process = None;
        _ = self.sender.send(Message::Text(serde_json::to_string(&ServerMsg::Stop).unwrap_or_default().into())).await;
    }

    async fn handle_websocket_msg(&mut self, msg: ClientMsg) -> HResult<bool> {
        match msg{
            ClientMsg::Compile(_) => todo!(),
            ClientMsg::Start => self.run_program().await,
            ClientMsg::Stop => self.stop_process().await,
            ClientMsg::Input { switch, buttons } => {
                if let Some(process) = &mut self.process{
                    use tokio::io::AsyncWriteExt;
                    process.stdin.write_all(format!("btn={}\n", buttons).as_bytes()).await?;
                    process.stdin.write_all(format!("sw={}\n", switch).as_bytes()).await?;
                }
            },
        }
        Ok(false)
    }

    async fn handle_websocket_receive(
        &mut self,
        msg: Option<Result<Message, Error>>,
    ) -> bool {
        match msg {
            Some(Ok(Message::Close(_))) => true,
            Some(Ok(Message::Text(msg))) => {
                let msg = match serde_json::from_str(msg.as_str()){
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
            },
            None => true,
        }
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        loop {
            if let Some(process) = &mut self.process {
                if let Ok(Some(_)) = process.process.try_wait(){
                    self.stop_process().await;
                    continue;
                }
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
                                }else if let Some(repr) = line.strip_prefix("seg0="){
                                    ServerMsg::Seg0(repr.parse().unwrap_or(0))
                                }else if let Some(repr) = line.strip_prefix("seg1="){
                                    ServerMsg::Seg1(repr.parse().unwrap_or(0))
                                }else if let Some(repr) = line.strip_prefix("seg2="){
                                    ServerMsg::Seg2(repr.parse().unwrap_or(0))
                                }else if let Some(repr) = line.strip_prefix("seg3="){
                                    ServerMsg::Seg3(repr.parse().unwrap_or(0))
                                }else{
                                    self.eprint(line).await;
                                    continue;
                                };
                                self.sender.send(Message::Text(serde_json::to_string(&msg)?.into())).await?;
                            },
                            Ok(None) => self.stop_process().await,
                            Err(err) => {
                                self.eprint(format!("Failed to read proccess serr: {err}")).await;
                                self.stop_process().await;
                            }
                        }
                    }
                    _ = tokio::time::sleep(self.refresh_time) => {
                        use tokio::io::AsyncWriteExt;
                        _ = process.stdin.write_all("\n".as_bytes()).await;
                    }
                }
            }else{
                let res = self.receiver.next().await;
                if self.handle_websocket_receive(res).await{
                    break;
                }
            }
        }
        Ok(())
    }

    async fn build_program(&mut self) {
        let files = if let Some(Ok(Message::Text(msg))) = self.receiver.next().await
            && let Ok(files) = serde_json::from_str::<'_, HashMap<String, String>>(&msg)
        {
            files
        } else {
            return;
        };

        let artifact_dir = match build::build(files).await {
            Ok(dir) => dir,
            Err(err) => {
                _ = self
                    .sender
                    .send(Message::Text(format!("Failed to build: {err}").into()))
                    .await;
                return;
            }
        };
    }

    async fn run_program(&mut self) {
        let process = match run::run(&self.build_dir).await {
            Ok(process) => process,
            Err(err) => {
                self.eprint(format!("Failed to run: {err}")).await;
                return;
            }
        };
        let stdout = BufReader::new(process.stdout).lines();
        let stderr = BufReader::new(process.stderr).lines();
        let stdin = process.stdin;

        self.process = Some(
            Process { process: process.child, stderr, stdout, stdin }
        )
    }
}

pub async fn ws_handler(socket: WebSocket) {
    Handler::local(socket, "target".into(), "src".into());
}
