use std::{collections::HashMap, convert::TryFrom};

use anyhow::{anyhow, bail, Ok, Result};
use clap::{command, Parser};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use zbus::ConnectionBuilder;
use zbus::{
    dbus_interface,
    zvariant::{ObjectPath, Value},
};

struct MprisRoot {
    name: String,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2")]
impl MprisRoot {
    async fn raise(&self) {}

    async fn quit(&self) {}

    #[dbus_interface(property)]
    async fn can_quit(&self) -> bool {
        false
    }
    #[dbus_interface(property)]
    async fn can_raise(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    async fn has_track_list(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    async fn identity(&self) -> String {
        self.name.clone()
    }

    #[dbus_interface(property)]
    async fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }

    #[dbus_interface(property)]
    async fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }
}

struct MprisPlayer {}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl MprisPlayer {
    async fn next(&self) {
        println!("next");
    }
    async fn previous(&self) {
        println!("previous");
    }
    async fn pause(&self) {
        println!("pause");
    }
    async fn play_pause(&self) {
        println!("play_pause");
    }
    async fn stop(&self) {
        println!("stop");
    }
    async fn play(&self) {
        println!("play");
    }
    async fn seek(&self, _offset: i64) {}
    async fn set_position(&self, _track_id: String, _position: i64) {}
    async fn open_uri(&self, _uri: String) {}

    #[dbus_interface(property)]
    async fn playback_status(&self) -> String {
        println!("playback_status");
        "Playing".to_string()
    }
    #[dbus_interface(property)]
    async fn loop_status(&self) -> String {
        println!("loop_status");
        "None".to_string()
    }
    // #[dbus_interface(property)]
    // async fn rate(&self) -> f64 {
    //     1.0
    // }
    #[dbus_interface(property)]
    async fn shuffle(&self) -> bool {
        println!("shuffle");
        false
    }
    #[dbus_interface(property)]
    async fn metadata(&self) -> HashMap<String, Value> {
        println!("metadata");
        let mut hm = HashMap::new();
        let op = ObjectPath::try_from("/").unwrap();
        hm.insert("mpris:trackid".to_string(), op.into());
        hm.insert("xesam:title".to_string(), "the title".into());
        hm
    }
    #[dbus_interface(property)]
    async fn volume(&self) -> f64 {
        println!("volume");
        1.0
    }
    #[dbus_interface(property)]
    async fn position(&self) -> i64 {
        println!("position");
        0
    }
    #[dbus_interface(property)]
    async fn minimum_rate(&self) -> f64 {
        println!("minimum_rate");
        1.0
    }
    #[dbus_interface(property)]
    async fn maximum_rate(&self) -> f64 {
        println!("maximum_rate");
        1.0
    }
    #[dbus_interface(property)]
    async fn can_go_next(&self) -> bool {
        println!("can_go_next");
        true
    }
    #[dbus_interface(property)]
    async fn can_go_previous(&self) -> bool {
        println!("can_go_previous");
        true
    }
    #[dbus_interface(property)]
    async fn can_play(&self) -> bool {
        println!("can_play");
        true
    }
    #[dbus_interface(property)]
    async fn can_pause(&self) -> bool {
        println!("can_pause");
        true
    }
    #[dbus_interface(property)]
    async fn can_seek(&self) -> bool {
        println!("can_seek");
        false
    }
    #[dbus_interface(property)]
    async fn can_control(&self) -> bool {
        println!("can_control");
        true
    }
}

#[derive(Serialize, Debug)]
struct LmsRequest {
    method: String,
    params: (String, Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
struct LmsResponse {
    #[serde(rename = "method")]
    _method: String,
    #[serde(rename = "params")]
    _params: (String, Vec<String>),

    result: serde_json::Value,
}

impl LmsRequest {
    fn new_(name: String, params: Vec<String>) -> Self {
        Self {
            method: "slim.request".to_string(),
            params: (name, params),
        }
    }

    fn new(name: String, params: Vec<&str>) -> Self {
        Self::new_(name, params.into_iter().map(|s| s.to_string()).collect())
    }

    fn version() -> (Self, String) {
        let key = "version".to_string();
        (Self::question(Self::new("".to_string(), vec![&key])), key)
    }

    fn question(mut self) -> Self {
        self.params.1.push(String::from("?"));
        self
    }

    fn connected(name: String) -> (Self, String) {
        let key = "connected".to_string();
        (Self::question(Self::new(name, vec![&key])), key)
    }

    fn artist(name: String) -> (Self, String) {
        let key = "artist".to_string();
        (Self::question(Self::new(name, vec![&key])), key)
    }

    fn current_title(name: String) -> (Self, String) {
        let key = "current_title".to_string();
        (Self::question(Self::new(name, vec![&key])), key)
    }

    fn mode(name: String) -> (Self, String) {
        let key = "mode".to_string();
        (Self::question(Self::new(name, vec![&key])), key)
    }

    fn shuffle(name: String) -> (Self, String) {
        let key = "shuffle";
        (
            Self::question(Self::new(name, vec!["playlist", key])),
            key.to_string(),
        )
    }
}

#[derive(Debug)]
enum Mode {
    Stop,
    Play,
    Pause,
}

#[derive(Debug)]
enum Shuffle {
    Off,
    Songs,
    Albums,
}

fn as_bool(response: &LmsResponse, key: &String) -> Result<bool> {
    let res_value = result_key(response, key)?;
    match res_value {
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(|i| i != 0)
            .ok_or_else(|| anyhow!("{} is not an i64", n)),
        _ => bail!("Wrong top level type for bool: {:?}", response),
    }
}

fn as_string(response: &LmsResponse, key: &String) -> Result<String> {
    let res_value = result_key(response, key)?;
    match res_value {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => bail!("Wrong top level type for string: {:?}", res_value),
    }
}

fn as_mode(response: &LmsResponse, key: &String) -> Result<Mode> {
    let res_value = result_key(response, &key)?;
    match res_value {
        serde_json::Value::String(s) => match s.as_str() {
            "stop" => Ok(Mode::Stop),
            "play" => Ok(Mode::Play),
            "pause" => Ok(Mode::Pause),
            other => bail!("Expected stop, play or pause, got {}", other),
        },
        _ => bail!("Wrong top level type for mode: {:?}", response),
    }
}

fn as_shuffle(response: &LmsResponse, key: &String) -> Result<Shuffle> {
    let res_value = result_key(response, &key)?;
    match res_value {
        serde_json::Value::String(s) => match s.as_str() {
            "0" => Ok(Shuffle::Off),
            "1" => Ok(Shuffle::Songs),
            "2" => Ok(Shuffle::Albums),
            _ => bail!("Expected 0, 1 or 2, got {}", s),
        },
        _ => bail!("Wrong top level type for shuffle: {:?}", response),
    }
}

#[derive(Debug, Error)]
enum ResultError {
    #[error("The result key has the wrong type: {response:?}")]
    ResultHasWrongType { response: LmsResponse },
    #[error("Unable to get key {key} in {response:?}")]
    NoKey { response: LmsResponse, key: String },
}

fn result_key<'a>(response: &'a LmsResponse, key: &String) -> Result<&'a serde_json::Value> {
    let result = match &response.result {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err(anyhow!(ResultError::ResultHasWrongType {
            response: response.clone()
        })),
    }?;
    result.get(&("_".to_string() + key)).ok_or_else(|| {
        anyhow!(ResultError::NoKey {
            response: response.clone(),
            key: key.clone()
        })
    })
}

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

async fn get_version(client: &Client, url: &String) -> Result<String> {
    let (request, key) = LmsRequest::version();
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    as_string(&lms_response, &key)
}

async fn get_connected(client: &Client, url: &String, name: String) -> Result<bool> {
    let (request, key) = LmsRequest::connected(name);
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    as_bool(&lms_response, &key)
}

async fn get_shuffle(client: &Client, url: &String, name: String) -> Result<Shuffle> {
    let (request, key) = LmsRequest::shuffle(name);
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    as_shuffle(&lms_response, &key)
}

async fn get_mode(client: &Client, url: &String, name: String) -> Result<Mode> {
    let (request, key) = LmsRequest::mode(name);
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    as_mode(&lms_response, &key)
}

async fn get_artist(client: &Client, url: &String, name: String) -> Result<String> {
    let (request, key) = LmsRequest::artist(name);
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    // when listening a remote stream, the artist is not available. the key with the result is not
    // even in the json.
    as_string(&lms_response, &key)
        .map(|s| s.to_string())
        .or_else(|e| match e.downcast_ref::<ResultError>() {
            Some(ResultError::NoKey { .. }) => Ok("".to_string()),
            _ => Err(e),
        })
}

async fn get_current_title(client: &Client, url: &String, name: String) -> Result<String> {
    let (request, key) = LmsRequest::current_title(name);
    let response = client.post(url).json(&request).send().await?;
    let lms_response = response.json().await?;
    as_string(&lms_response, &key).map(|s| s.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    let url = format!("http://{}:{}/jsonrpc.js", options.hostname, options.port);
    let client = reqwest::Client::new();

    let version = get_version(&client, &url).await?;
    println!("Version: {:?}", version);

    let mode = get_mode(&client, &url, options.player_name.clone()).await?;
    println!("Mode: {:?}", mode);

    let shuffle = get_shuffle(&client, &url, options.player_name.clone()).await?;
    println!("Shuffle: {:?}", shuffle);

    let connected = get_connected(&client, &url, options.player_name.clone()).await?;
    println!("Connected: {:?}", connected);

    let current_title = get_current_title(&client, &url, options.player_name.clone()).await?;
    println!("current_title: {:?}", current_title);

    let artist = get_artist(&client, &url, options.player_name.clone()).await?;
    println!("artist: {:?}", artist);

    let root = MprisRoot {
        name: options.player_name.clone(),
    };

    let player = MprisPlayer {};
    let _connection = ConnectionBuilder::session()?
        .name(format!("org.mpris.MediaPlayer2.{}", options.player_name))?
        .serve_at("/org/mpris/MediaPlayer2", root)?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build()
        .await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
