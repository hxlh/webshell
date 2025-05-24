use std::{sync::Arc, time};

use russh::client::{Config, Handler};
use tokio::{runtime::Handle, task};
use tracing::debug;

struct Client {}

impl Handler for Client {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        async { Ok(true) }
    }
}

pub struct Session {
    client: russh::client::Handle<Client>,
    pub channel: russh::Channel<russh::client::Msg>,
    expired_at: time::Instant,
}

impl Session {
    async fn create(
        addr: &str,
        user: &str,
        password: &str,
        timeout: time::Duration,
    ) -> Result<Self, anyhow::Error> {
        let config = Arc::new(Config {
            inactivity_timeout: Some(timeout),
            ..Default::default()
        });
        let mut client = russh::client::connect(config, addr, Client {}).await?;
        let auth_res = client.authenticate_password(user, password).await?;
        debug!("auth_res: {:?}", auth_res);
        let channle = client.channel_open_session().await?;

        Ok(Self {
            client: client,
            channel: channle,
            expired_at: time::Instant::now() + timeout,
        })
    }

    pub async fn close(&self) {
        _ = self.channel.close().await;
        _ = self
            .client
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await;
    }

    pub fn is_expired(&self) -> bool {
        self.expired_at < time::Instant::now()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        task::block_in_place(|| {
            Handle::current().block_on(self.close());
        });
    }
}

pub struct AppState {
    addr: String,
    user: String,
    password: String,
    timeout: time::Duration,
}

impl AppState {
    pub fn new(addr: &str, user: &str, pwd: &str, timeout: time::Duration) -> Self {
        Self {
            addr: addr.into(),
            user: user.into(),
            password: pwd.into(),
            timeout: timeout,
        }
    }

    pub async fn create_session(&self) -> Result<Session, anyhow::Error> {
        Session::create(&self.addr, &self.user, &self.password, self.timeout).await
    }
}

#[cfg(test)]
mod tests {
    use russh::client::Msg;
    use tracing::info;

    use crate::logger::init_logger;

    use super::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_session() {
        init_logger(std::io::stdout);
        let mut s = Session::create(
            "127.0.0.1:22",
            "root",
            "123456",
            time::Duration::from_secs(10),
        )
        .await
        .unwrap();
        s.channel.exec(true, "pwd".as_bytes()).await.unwrap();
        while let Some(msg) = s.channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { data } => {
                    info!("data: {:?}", String::from_utf8_lossy(&data));
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    info!("exit_status: {:?}", exit_status);
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(s.is_expired(), false);
        tokio::time::sleep(time::Duration::from_secs(10)).await;
        assert_eq!(s.is_expired(), true);
    }
}
