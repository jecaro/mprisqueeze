use anyhow::{bail, Ok, Result};
use clap::{command, Parser};
use mpris::connect;
mod lms_client;
mod mpris;
use tokio::process::Command;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short = 'H', long)]
    hostname: String,
    #[arg(short = 'P', long, default_value_t = 9000)]
    port: u16,
    #[arg(short, long)]
    player_name: String,
    #[arg(last = true, default_values_t = vec!["squeezelite-pulse".to_string()])]
    player_command: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    let (player_command, player_args) = match options.player_command[..] {
        [] => bail!("No player command given"),
        [ref player_command, ref player_args @ ..] => Ok((player_command, player_args)),
    }?;

    let mut player_process = Command::new(player_command).args(player_args).spawn()?;

    let _connection = connect(options.hostname, options.port, options.player_name.clone()).await?;

    let exit_status = player_process.wait().await?;
    match exit_status.code() {
        Some(code) => bail!("Player exited with code {}", code),
        None => bail!("Player exited without code"),
    }
}
