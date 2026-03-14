use axum::{
    Router,
    extract::ws::WebSocketUpgrade,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr}, path::PathBuf, time::Duration
};

pub mod build;
pub mod run;
pub mod uploaded;
pub mod workspace;

const UI_INDEX_HTML: &str = include_str!("../ui/index.html");
const UI_STYLES_CSS: &str = include_str!("../ui/styles.css");
const UI_APP_JS: &str = include_str!("../ui/app.js");

async fn serve_styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        UI_STYLES_CSS,
    )
}

async fn serve_app_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        UI_APP_JS,
    )
}

async fn serve_index() -> impl IntoResponse {
    Html(UI_INDEX_HTML)
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

#[derive(Clone, Debug)]
struct Config {
    ip: IpAddr,
    port: u16,
    update_ms: u64,
    workspace_ws: bool,
    workspace_src: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip: IpAddr::from([127, 0, 0, 1]),
            port: 8080,
            update_ms: 30,
            workspace_ws: false,
            workspace_src: "./src".into(),
        }
    }
}

fn parse_config_from_args() -> Result<Config, String> {
    let mut cfg = Config::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--" => break,
            "--ip" => {
                let value = args.next().ok_or("missing value for --ip")?;
                cfg.ip = value
                    .parse::<IpAddr>()
                    .map_err(|err| format!("invalid --ip `{value}`: {err}"))?;
            }
            "--port" => {
                let value = args.next().ok_or("missing value for --port")?;
                cfg.port = value
                    .parse::<u16>()
                    .map_err(|err| format!("invalid --port `{value}`: {err}"))?;
            }
            "--update-ms" => {
                let value = args.next().ok_or("missing value for --update-ms")?;
                cfg.update_ms = value
                    .parse::<u64>()
                    .map_err(|err| format!("invalid --update-ms `{value}`: {err}"))?;
            }
            "--workspace" => {
                cfg.workspace_ws = true;
            }
            "--workspace-src" => {
                cfg.workspace_ws = true;
                cfg.workspace_src = args.next().ok_or("missing value for --workspace-src")?.into();
            }
            "--help" | "-h" => {
                return Err(
                    "usage: relay [--ip <ip>] [--port <port>] [--update-ms <ms>] [--workspace]"
                        .into(),
                );
            }
            _ => {
                return Err(format!(
                    "unknown argument `{arg}`\nusage: relay [--ip <ip>] [--port <port>] [--update-ms <ms>] [--workspace]"
                ));
            }
        }
    }

    Ok(cfg)
}

#[tokio::main]
async fn main() {
    let cfg = match parse_config_from_args() {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    let update_interval = Duration::from_millis(cfg.update_ms);
    let mut app =
        Router::new()
            .route("/", get(move || async move { serve_index().await }))
            .route(
                "/index.html",
                get(move || async move { serve_index().await }),
            )
            .route("/styles.css", get(serve_styles))
            .route("/app.js", get(serve_app_js))
            .route(
                "/ws/uploaded",
                get(move |ws: WebSocketUpgrade| {
                    let update_interval = update_interval;
                    async move {
                        ws.on_upgrade(move |socket| uploaded::ws_handler(socket, update_interval))
                    }
                }),
            );

    if cfg.workspace_ws {
        app = app.route(
            "/ws/workspace",
            get(move |ws: WebSocketUpgrade| {
                let update_interval = update_interval;
                let workspace_src = cfg.workspace_src;
                async move {
                    ws.on_upgrade(move |socket| workspace::ws_handler(socket, workspace_src.clone(), update_interval))
                }
            }),
        );
    }

    app = app.fallback(get(not_found));

    let addr = SocketAddr::new(cfg.ip, cfg.port);
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
    Compiling,
    Start,
    Stop,
    Led(u32),
    Seg { value: u32, index: u32 },
}

pub type HResult<T> = Result<T, Box<dyn std::error::Error + Sync + Send>>;
