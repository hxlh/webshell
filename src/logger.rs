use std::sync::Arc;

use tracing_subscriber::{
    EnvFilter,
    fmt::{self, MakeWriter},
};

pub fn init_logger<W>(w: W)
where
    W: for<'writer> MakeWriter<'writer> + 'static,
{
    let filter = EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt::fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .init();
}
