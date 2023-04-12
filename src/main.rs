use anyhow::Result;
use clap::{command, Parser};
use mpris::connect;
mod lms_client;
mod mpris;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short = 'H', long)]
    hostname: String,
    #[arg(short = 'P', long, default_value_t = 9000)]
    port: u16,
    #[arg(short, long)]
    player_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    let _connection = connect(options.hostname, options.port, options.player_name.clone()).await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
