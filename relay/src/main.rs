use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, time::Duration};
use tokio::{
    io::{BufReader, Lines},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
};
use tower_http::services::ServeDir;

use crate::build::TempDir;

pub mod build;
pub mod run;
pub mod local;
pub mod remote;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/ws",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(remote::ws_handler) }),
        )
        .fallback_service(ServeDir::new("ui"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Open UI: http://{}/", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
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
pub enum ServerMsg<'a> {
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

struct Handler {
    sender: SplitSink<WebSocket, Message>,
    receiver: SplitStream<WebSocket>,

    build_dir: TempDir,
    src_dir: PathBuf,

    program: Option<PathBuf>,
    process: Option<Process>,

    refresh_time: Duration,
}

pub type HResult<T> = Result<T, Box<dyn std::error::Error + Sync + Send>>;