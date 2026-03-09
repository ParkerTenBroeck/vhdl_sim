use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
};
use tower_http::services::ServeDir;

pub mod build;
pub mod run;


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/ws",
            get(|ws: WebSocketUpgrade| async move { ws.on_upgrade(ws_handler) }),
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
struct ClientInput{
    /// bitfield of 32 switches
    switch: u32, 
    /// bitfield of 32 buttons
    buttons: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ServerMsg<'a> {
    Log { stream: &'a str, line: &'a str },
    Led(u32),
    Seg0(u32),
    Seg1(u32),
    Seg2(u32),
    Seg3(u32)
}

async fn ws_handler(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    let files = if let Some(Ok(Message::Text(msg))) = receiver.next().await
        && let Ok(files) = serde_json::from_str::<'_, HashMap<String, String>>(&msg)
    {  
        files
    } else {
        return;
    };


    let artifact_dir = match build::build(files).await{
        Ok(dir) => dir,
        Err(err) => {
            _ = sender.send(Message::Text(format!("Failed to build: {err}").into())).await;
            return;
        },
    };

    let mut process = match run::run(&artifact_dir).await{
        Ok(process) => process,
        Err(err) => {
            _ = sender.send(Message::Text(format!("Failed to run: {err}").into())).await;
            return;
        },
    };
    let mut sout = BufReader::new(process.stdout).lines();
    let mut serr = BufReader::new(process.stderr).lines();

    let artifact_prefix = artifact_dir.to_str().unwrap_or("\0\0NOPE");
    
    let result: Result<(), Box<dyn std::error::Error + Sync + Send>> = async {
        loop{
            tokio::select! {
                receive = receiver.next() => {
                    match receive{
                        Some(Ok(Message::Close(_))) => break,
                        Some(Ok(Message::Text(msg))) => {
                            let input = serde_json::from_str::<'_, ClientInput>(&msg)?;
                            use tokio::io::AsyncWriteExt;
                            process.stdin.write_all(format!("btn={}\n", input.buttons).as_bytes()).await?;
                            process.stdin.write_all(format!("sw={}\n", input.switch).as_bytes()).await?;
                        },
                        Some(Ok(_)) => {},
                        Some(Err(err)) => Err(err)?,
                        _ => break,
                    }
                }
                out = sout.next_line() => {
                    match out{
                        Ok(Some(line)) => {
                            
                            let msg = ServerMsg::Log {
                                stream: "stdout",
                                line: line.strip_prefix(artifact_prefix).unwrap_or(&line),
                            };
                            sender.send(Message::Text(serde_json::to_string(&msg)?.into())).await?;
                        },
                        Ok(None) => break,
                        Err(err) => {
                            Err(format!("Failed to read proccess sout: {err}"))?;
                        }
                    }
                }
                err = serr.next_line() => {
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
                                ServerMsg::Log {
                                    stream: "stderr",
                                    line: line.strip_prefix(artifact_prefix).unwrap_or(&line),
                                }
                            };
                            sender.send(Message::Text(serde_json::to_string(&msg)?.into())).await?;
                        },
                        Ok(None) => break,
                        Err(err) => {
                            Err(format!("Failed to read proccess serr: {err}"))?
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(30)) => {
                    use tokio::io::AsyncWriteExt;
                    process.stdin.write_all("\n".as_bytes()).await?;
                }
            }        
        }
        Ok(())
    }.await;

    match result{
        Ok(_) => {},
        Err(err) => {
            _ = sender.send(Message::Text(format!("{err}").into())).await;
        },
    }
}