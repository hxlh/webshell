use clap::Parser;
use cli::{Cli, cmd_run};
use tracing::error;
use webshell::logger;

mod cli;

#[tokio::main]
async fn main() {
    logger::init_logger(std::io::stdout);

    let args = Cli::parse();

    let res = match args.command {
        cli::Commands::Run { addr, timeout } => cmd_run(addr, timeout).await,
    };
    if let Err(e) = res {
        error!("{}", e);
    }
    return;
}
