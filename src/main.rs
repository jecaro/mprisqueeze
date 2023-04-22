use anyhow::{anyhow, bail, Ok, Result};
use clap::{command, Parser};
use lms_client::LmsClient;
use mpris::start_dbus_server;
use std::time::Duration;
use tokio::{
    pin,
    process::{Child, Command},
    select,
    time::sleep,
};
mod lms_client;
mod mpris;

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
    #[arg(last = true, default_values_t = vec!["squeezelite".to_string(), "-n".to_string(),
          "{}".to_string()])]
    player_command: Vec<String>,
}

async fn wait_for_player(client: &LmsClient, player_name: &str, timeout: u64) -> Result<()> {
    let sleep = sleep(Duration::from_secs(timeout));
    pin!(sleep);
    loop {
        select! {
            _ = &mut sleep => bail!("Player not available after {} seconds", timeout),
            players = client.get_players() =>
            {
                let found = players.map(|players| {
                    players.iter().any(|player| player.name == player_name)
                })?;
                if found {
                    break Ok(());
                }
            }
        }
    }
}

fn start_squeezelite(options: &Options) -> Result<Child> {
    let (player_command, player_args) = match options.player_command[..] {
        [] => bail!("No player command given"),
        [ref player_command, ref player_args @ ..] => Ok((player_command, player_args)),
    }?;

    // put the player name into the command line arguments
    if !player_args.iter().any(|arg| arg.contains("{}")) {
        bail!("Player args must contain the string {{}} to be replaced with the player name");
    }
    let player_args_with_name = player_args
        .iter()
        .map(|arg| arg.replace("{}", &options.player_name))
        .collect::<Vec<_>>();

    Command::new(player_command)
        .args(player_args_with_name)
        .spawn()
        .map_err(|e| anyhow!("Failed to start player command {}: {}", player_command, e))
}

#[tokio::main]
async fn main() -> Result<()> {
    // parse the command line options
    let options = Options::parse();

    // start squeezelite
    let mut player_process = start_squeezelite(&options)?;

    // wait for the player to be available
    let client = LmsClient::new(options.hostname.clone(), options.port);
    wait_for_player(&client, &options.player_name, options.timeout).await?;

    // get a listener for errors before the client is moved into the MPRIS server
    let error_listener = client.error.listen();

    // start the MPRIS server
    let _connection = start_dbus_server(client, options.player_name).await?;

    select! {
        _ = error_listener => bail!("Error from LMS"),
        _ = player_process.wait() =>
        {
            let exit_status = player_process.wait().await?;
            match exit_status.code() {
                Some(code) => bail!("Player exited with code {}", code),
                None => bail!("Player exited without code"),
            }
        }
    }
}
