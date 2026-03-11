use axum::{
    Router,
    extract::ws::WebSocketUpgrade,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub mod build;
pub mod run;
pub mod local;
pub mod remote;

const UI_INDEX_HTML: &str = include_str!("../ui/index.html");
const UI_STYLES_CSS: &str = include_str!("../ui/styles.css");
const UI_APP_JS: &str = include_str!("../ui/app.js");

async fn serve_index() -> impl IntoResponse {
    Html(UI_INDEX_HTML)
}

async fn serve_styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        UI_STYLES_CSS,
    )
}

async fn serve_app_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        UI_APP_JS,
    )
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_styles))
        .route("/app.js", get(serve_app_js))
        .route(
            "/ws/remote",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(local::ws_handler) }),
        )
        .route(
            "/ws/local",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(remote::ws_handler) }),
        )
        .fallback(get(not_found));

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
