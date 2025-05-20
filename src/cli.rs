use std::{io::Write, sync::Arc, time};

use axum::{Extension, Router, http::Request, routing::get};
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;
use webshell::{logger, route::websocket_handler, session::AppState};

#[derive(Parser)]
pub enum Commands {
    #[clap(about = "启动一个新的会话")]
    Run { addr: String, timeout: u64 },
}

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

pub async fn cmd_run(addr: String, timeout: u64) -> Result<(), anyhow::Error> {
    let mut stdio = std::io::stdout();
    let stdin = std::io::stdin();
    stdio.write("请输入账号：".as_bytes())?;
    stdio.flush()?;
    let mut user = String::new();
    stdin.read_line(&mut user)?;
    stdio.write("请输入密码：".as_bytes())?;
    stdio.flush()?;
    let pwd = rpassword::read_password()?;

    info!(
        "addr: {},user: {}, pwd: {}",
        addr,
        user.strip_suffix("\n").unwrap(),
        pwd
    );

    let state = Arc::new(AppState::new(
        "127.0.0.1:22",
        "root",
        "123456",
        time::Duration::from_secs(timeout),
    ));
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(state))
        .nest_service("/static", ServeDir::new("web"));
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
