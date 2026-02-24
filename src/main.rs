use anyhow::{Ok, Result, anyhow, bail};
use clap::Parser;
use discover::discover;
use lms::LmsClient;
use log::{debug, info};
use mpris::{poll_for_mode_changes, start_dbus_server};
use std::time::Duration;
use tokio::{
    pin,
    process::{Child, Command},
    select,
    time::{sleep, timeout},
};
mod discover;
mod lms;
mod mpris;

/// Check the player name given on the command line. It should be possible to use it to create a
/// dbus path hence it must conform to
/// https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-marshaling-object-path
fn is_player_name_valid(s: &str) -> Result<String> {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(s.to_string())
    } else {
        Err(anyhow!(
            "Player name must only contain characters a-z, A-Z, 0-9 and _"
        ))
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short = 'H', long, help = "LMS hostname")]
    hostname: Option<String>,
    #[arg(short = 'P', long, help = "LMS port", default_value_t = 9000)]
    port: u16,
    #[arg(
        short,
        long,
        default_value = "SqueezeLite",
        help = "Player name",
        value_parser = is_player_name_valid
    )]
    player_name: String,
    #[arg(
        short = 't',
        long,
        default_value_t = 3,
        help = "Timeout in seconds for squeezelite to be recognized by LMS"
    )]
    player_timeout: u64,
    #[arg(
        short = 'd',
        long,
        default_value_t = 3,
        help = "Timeout in seconds for LMS discovery"
    )]
    discover_timeout: u64,
    #[arg(
        short = 'r',
        long,
        default_value_t = 100,
        help = "Timeout in milliseconds for LMS to reply to the discovery message"
    )]
    discover_reply_timeout: u64,
    #[arg(
        last = true,
        default_values_t = vec!["squeezelite-pulse".to_string(), "-n".to_string(),
            "{name}".to_string(), "-s".to_string(), "{server}".to_string()],
        help = "Player command and arguments. The string '{name}' will be replaced with the player\
                name, '{server}' with the LMS server name."
    )]
    player_command: Vec<String>,
}

/// Wait for maximum `timeout` seconds for the player to be available
/// returns the player id
async fn wait_for_player(client: &LmsClient, player_name: &str, timeout: u64) -> Result<String> {
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
                    if let Some(player) = players.into_iter().find(|player| player.name == player_name) {
                        info!("Player {} is available", player_name);
                        break Ok(player.playerid);
                    }
                }
                count.map(|_| ())?
            }
        }
    }
}

/// Start the `squeezelite` process
fn start_squeezelite(options: &Options, server: &String) -> Result<Child> {
    let (player_command, player_args) = match options.player_command[..] {
        [] => bail!("No player command given"),
        [ref player_command, ref player_args @ ..] => Ok((player_command, player_args)),
    }?;

    if !player_args.iter().any(|arg| arg.contains("{name}")) {
        bail!("Player args must contain the string {{name}} to be replaced with the player name");
    }
    if !player_args.iter().any(|arg| arg.contains("{server}")) {
        bail!("Player args must contain the string {{server}} to be replaced with the server name");
    }
    let player_args_with_name = player_args
        .iter()
        .map(|arg| arg.replace("{name}", &options.player_name))
        .map(|arg| arg.replace("{server}", &server))
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
            port,
            ..
        } => (hostname.clone(), port),
        _ => {
            let (ip, reply) = timeout(
                Duration::from_secs(options.discover_timeout),
                discover(Duration::from_millis(options.discover_reply_timeout)),
            )
            .await??;

            (ip.to_string(), reply.port)
        }
    };

    // start squeezelite
    let mut player_process = start_squeezelite(&options, &hostname)?;

    let result: Result<()> = (|| async {
        // wait for the player to be available
        let (client, mut recv) = LmsClient::new(hostname, port);
        let player_id =
            wait_for_player(&client, &options.player_name, options.player_timeout).await?;

        // start the MPRIS server
        let (_connection, iface_ref) =
            start_dbus_server(client.clone(), options.player_name, player_id.clone()).await?;

        // Spawn a task to poll for mode changes and emit PropertiesChanged signals
        let poll_handle =
            tokio::spawn(async move { poll_for_mode_changes(iface_ref, client, player_id).await });

        select! {
            Some(error) = recv.recv() => bail!("Error from LMS: {:?}", error),
            result = poll_handle => {
                result??;
                bail!("Polling task exited unexpectedly")
            }
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
