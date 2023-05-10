use anyhow::{anyhow, bail, Ok, Result};
use clap::{command, Parser};
use discover::discover;
use lms::LmsClient;
use log::{debug, info};
use mpris::start_dbus_server;
use std::time::Duration;
use tokio::{
    pin,
    process::{Child, Command},
    select,
    time::sleep,
};
mod discover;
mod lms;
mod mpris;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short = 'H', long, help = "LMS hostname")]
    hostname: Option<String>,
    #[arg(short = 'P', long, help = "LMS port")]
    port: Option<u16>,
    #[arg(short, long, default_value = "SqueezeLite", help = "Player name")]
    player_name: String,
    #[arg(
        short,
        long,
        default_value_t = 3,
        help = "Timeout in seconds for squeezelite to be recognized by LMS"
    )]
    timeout: u64,
    #[arg(
        last = true,
        default_values_t = vec!["squeezelite".to_string(), "-n".to_string(), "{}".to_string()],
        help = "Player command and arguments. The string '{}' will be replaced with the player name."
    )]
    player_command: Vec<String>,
}

/// Wait for maximum `timeout` seconds for the player to be available
async fn wait_for_player(client: &LmsClient, player_name: &str, timeout: u64) -> Result<()> {
    info!("Waiting for player {} to be available", player_name);
    let sleep = sleep(Duration::from_secs(timeout));
    pin!(sleep);
    loop {
        select! {
            _ = &mut sleep => bail!("Player not available after {} seconds", timeout),
            count = client.get_player_count() =>
            {
                if let Result::Ok(true) = count.as_ref().map(|count| *count != 0) {
                    let players = client.get_players().await?;
                    if players.iter().any(|player| player.name == player_name) {
                        info!("Player {} is available", player_name);
                        break Ok(());
                    }
                }
                count.map(|_| ())?
            }
        }
    }
}

/// Start the `squeezelite` process
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

    info!(
        "Starting player: {} {:?}",
        player_command, player_args_with_name
    );
    Command::new(player_command)
        .args(player_args_with_name)
        .spawn()
        .map_err(|e| anyhow!("Failed to start player command {}: {}", player_command, e))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // parse the command line options
    let options = Options::parse();
    debug!("Options: {:?}", options);

    // get the hostname and port either from the command line or by discovering the server on the
    // network
    let (hostname, port) = match options {
        Options {
            hostname: Some(ref hostname),
            port: Some(port),
            ..
        } => (hostname.clone(), port),
        _ => {
            let reply = discover().await?;
            (reply.hostname, reply.port)
        }
    };

    // start squeezelite
    let mut player_process = start_squeezelite(&options)?;

    let result: Result<()> = (|| async {
        // wait for the player to be available
        let (client, mut recv) = LmsClient::new(hostname, port);
        wait_for_player(&client, &options.player_name, options.timeout).await?;

        // start the MPRIS server
        let _connection = start_dbus_server(client, options.player_name).await?;

        select! {
            Some (error) = recv.recv() => bail!("Error from LMS: {:?}", error),
            _ = player_process.wait() =>
            {
                let exit_status = player_process.wait().await?;
                match exit_status.code() {
                    Some(code) => bail!("Player exited with code {}", code),
                    None => bail!("Player exited without code"),
                }
            }
        }
    })()
    .await;

    // kill the player process if it is still running
    if player_process.id().is_some() {
        info!("Killing player process");
        player_process.kill().await?;
    }

    result
}
