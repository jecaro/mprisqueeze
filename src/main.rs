use anyhow::{bail, Ok, Result};
use clap::{command, Parser};
use lms_client::LmsClient;
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
    #[arg(short, long, default_value = "SqueezeLite")]
    player_name: String,
    #[arg(short, long, default_value_t = 3)]
    timeout: u64,
    #[arg(last = true, default_values_t = vec!["squeezelite".to_string()])]
    player_command: Vec<String>,
}

async fn wait_for_player(client: &LmsClient, player_name: &str, timeout: u64) -> Result<()> {
    let sleep = tokio::time::sleep(std::time::Duration::from_secs(timeout));
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => bail!("Player not available after {} seconds", timeout),
            connected = client.get_connected(player_name.to_string()) =>
            {
                match connected {
                    Result::Ok(true) => break Ok(()),
                    Result::Ok(false) => continue,
                    Err(e) => bail!("Error while waiting for player: {}", e),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // parse the command line options
    let options = Options::parse();
    let (player_command, player_args) = match options.player_command[..] {
        [] => bail!("No player command given"),
        [ref player_command, ref player_args @ ..] => Ok((player_command, player_args)),
    }?;

    // start squeezelite
    let mut player_process = Command::new(player_command)
        .args(player_args)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start player command {}: {}", player_command, e))?;

    // wait for the player to be available
    let client = LmsClient::new(options.hostname.clone(), options.port);
    wait_for_player(&client, &options.player_name, options.timeout).await?;

    // start the MPRIS server
    let _connection = connect(client, options.player_name.clone()).await?;

    // wait for squeezelite to exit
    let exit_status = player_process.wait().await?;
    match exit_status.code() {
        Some(code) => bail!("Player exited with code {}", code),
        None => bail!("Player exited without code"),
    }
}
