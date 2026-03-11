use axum::{
    extract::ws::{Message, WebSocket},
};
use futures_util::{
    SinkExt, StreamExt,
};
use std::{collections::HashMap, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader}, time::Instant,
};

use crate::{ClientMsg, HResult, ServerMsg, build, run};

pub async fn ws_handler(socket: WebSocket, refresh_time: Duration) {
    let (mut sender, mut receiver) = socket.split();

    let files = if let Some(Ok(Message::Text(msg))) = receiver.next().await
        && let Ok(files) = serde_json::from_str::<'_, HashMap<String, String>>(&msg)
    {  
        files
    } else {
        return;
    };


    let artifact_dir = match build::copy_and_build(files).await{
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

    let mut print_deadline = Instant::now();
    
    let result: HResult<()> = async {
        loop{
            tokio::select! {
                receive = receiver.next() => {
                    match receive{
                        Some(Ok(Message::Close(_))) => break,
                        Some(Ok(Message::Text(msg))) => {
                            let input = serde_json::from_str::<'_, ClientMsg>(&msg)?;
                            match input{
                                ClientMsg::Start => {},
                                ClientMsg::Stop => break,
                                ClientMsg::Input { switch, buttons } => {
                                    use tokio::io::AsyncWriteExt;
                                    process.stdin.write_all(format!("btn={}\n", buttons).as_bytes()).await?;
                                    process.stdin.write_all(format!("sw={}\n", switch).as_bytes()).await?;
                                },
                            }
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
                _ = tokio::time::sleep_until(print_deadline) => {
                    use tokio::io::AsyncWriteExt;
                    print_deadline += refresh_time;
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
