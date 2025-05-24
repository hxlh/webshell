use std::{io::Write, sync::Arc};

use axum::{Extension, Router, http::Request, routing::get};
use clap::Parser;
use local_ip_address::local_ip;
use tokio::{net::TcpListener, time};
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
    user = user.trim_end_matches("\n").into();

    stdio.write("请输入密码：".as_bytes())?;
    stdio.flush()?;
    let pwd = rpassword::read_password()?;

    let state = Arc::new(AppState::new(
        &addr,
        &user,
        &pwd,
        time::Duration::from_secs(timeout),
    ));

    let ip = local_ip().unwrap();
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    info!("serve on http://{}:{}/static/shell.html", ip, port);

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(state))
        .nest_service("/static", ServeDir::new("web"));

    let handle = tokio::spawn(async {
        axum::serve(listener, app).await.unwrap();
    });

    let start = time::Instant::now();
    while start.elapsed() < time::Duration::from_secs(timeout) {
        time::sleep(time::Duration::from_secs(1)).await;
    }
    handle.abort();

    info!("server exit because of timeout");
    Ok(())
}
