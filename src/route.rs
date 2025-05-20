use std::sync::Arc;

use axum::{
    Extension,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use russh::ChannelMsg;
use tokio::io::AsyncReadExt;
use tracing::{error, info};

use crate::session::AppState;

pub async fn websocket_handler(
    Extension(state): Extension<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_callback(socket, state))
}

async fn websocket_callback(mut socket: WebSocket, state: Arc<AppState>) {
    // connect ssh
    let mut session = match state.create_session().await {
        Ok(s) => s,
        Err(e) => {
            error!("create session error: {e}");
            // Send error message and close connection
            if socket
                .send(Message::Text(
                    format!("Error creating session: {}", e).into(),
                ))
                .await
                .is_err()
            {
                error!("Failed to send error message to client");
            }
            if socket.send(Message::Close(None)).await.is_err() {
                error!("Failed to send close message to client");
            }
            return;
        }
    };

    session
        .channel
        .request_pty(
            true,
            "xterm",
            80 as u32,
            24 as u32,
            0,
            0,
            &[], // ideally you want to pass the actual terminal modes here
        )
        .await
        .unwrap();

    session.channel.request_shell(true).await.unwrap();
    // session.channel.exec(true, "pwd").await.unwrap();

    // Use tokio::select! to concurrently handle messages from WebSocket and SSH channel
    loop {
        tokio::select! {
            // Read from WebSocket
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(msg)) => {
                        info!("msg:{:?}",msg);
                        match msg {
                            Message::Text(t) => {
                                // Send text message to SSH channel
                                info!("Received text message from client: {}", String::from_utf8_lossy(t.as_bytes()));
                                if let Err(e) = session.channel.data(t.as_bytes()).await {
                                    error!("Failed to send data to SSH channel: {e}");
                                    break;
                                }
                            },
                            Message::Close(c) => {
                                info!("Client sent close: {:?}", c);
                                break;
                            },
                            _ => {

                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket receive error: {e}");
                        break;
                    }
                    None => {
                        info!("WebSocket connection closed");
                        break;
                    }
                }
            }
            // Read from SSH channel
            ssh_msg = session.channel.wait() => {
                match ssh_msg {
                    Some(data) => {
                        match data{ChannelMsg::Data { data } => {
                            if let Err(e)=socket.send(
                                Message::Text(
                                    String::from_utf8_lossy(&data).into_owned().into()
                                )
                            ).await{
                                error!("Failed to send data to WebSocket: {e}");
                                break;
                            }
                        },
                            _ => {},
                        }
                    }
                    None => {
                        info!("SSH channel closed");
                        break;
                    }
                }
            }
        }
    }

    // Ensure connections are closed when the loop breaks
    info!("Closing connections");
    if socket.send(Message::Close(None)).await.is_err() {
        error!("Failed to send close message to client");
    }
    session.close().await;
}
