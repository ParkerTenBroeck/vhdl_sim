use axum::{
    Router,
    extract::ws::WebSocketUpgrade,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub mod build;
pub mod run;
pub mod local;
pub mod remote;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/ws/local",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(remote::ws_handler) }),
        )
        .route(
            "/ws/remote",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(local::ws_handler) }),
        )
        .fallback_service(ServeDir::new("ui"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Open UI: http://{}/", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMsg {
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

pub type HResult<T> = Result<T, Box<dyn std::error::Error + Sync + Send>>;
