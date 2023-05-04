use crate::lms::request::LmsRequest;
use anyhow::bail;
use anyhow::{anyhow, Ok, Result};
use reqwest::Client;
use reqwest::Response;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc;

mod request;

#[derive(Debug)]
pub enum Mode {
    Stop,
    Play,
    Pause,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Shuffle {
    Off,
    Songs,
    Albums,
}

#[derive(Debug)]
pub struct LmsClient {
    client: Client,
    url: String,
    sender: mpsc::Sender<anyhow::Error>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Player {
    pub name: String,
}

impl LmsClient {
    pub fn new(hostname: String, port: u16) -> (Self, mpsc::Receiver<anyhow::Error>) {
        let client = Client::new();
        let url = format!("http://{}:{}/jsonrpc.js", hostname, port);
        let (sender, receiver) = mpsc::channel::<anyhow::Error>(1);

        (
            Self {
                client,
                url,
                sender,
            },
            receiver,
        )
    }

    #[allow(dead_code)]
    pub async fn get_version(&self) -> Result<String> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::version();
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_string(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_version"),
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn get_connected(&self, name: String) -> Result<bool> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::connected(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_bool(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_connected"),
        )
        .await
    }

    pub async fn get_player_count(&self) -> Result<u64> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::player_count();
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_u64(lms_response, &key)
            })()
            .await,
            anyhow!("Error player_count"),
        )
        .await
    }

    pub async fn get_players(&self) -> Result<Vec<Player>> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::players();
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                let value = result_key(lms_response, &key)?.clone();
                serde_json::from_value(value.to_owned()).map_err(|e| e.into())
            })()
            .await,
            anyhow!("Error get_players"),
        )
        .await
    }

    pub async fn get_index(&self, name: String) -> Result<u64> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::index(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_u64(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_index"),
        )
        .await
    }

    pub async fn get_track_count(&self, name: String) -> Result<u64> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::track_count(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_u64(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_track_count"),
        )
        .await
    }

    pub async fn get_shuffle(&self, name: String) -> Result<Shuffle> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::shuffle(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_shuffle(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_shuffle"),
        )
        .await
    }

    pub async fn get_mode(&self, name: String) -> Result<Mode> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::mode(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_mode(lms_response, &key)
            })()
            .await,
            anyhow!("Error get_mode"),
        )
        .await
    }

    // When the playlist is empty, the key is not here. The "result" field contains an empty
    // object.
    pub async fn get_artist(&self, name: String) -> Result<Option<String>> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::artist(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_string(lms_response, &key).map(Some).or_else(|e| {
                    match e.downcast_ref::<ResultError>() {
                        Some(ResultError::NoKey { .. }) => Ok(None),
                        _ => Err(e),
                    }
                })
            })()
            .await,
            anyhow!("Error get_artist"),
        )
        .await
    }

    // Same remark as get_artist
    pub async fn get_current_title(&self, name: String) -> Result<Option<String>> {
        self.handle_error(
            (|| async {
                let (request, key) = LmsRequest::current_title(name);
                let response = self.post(&request).await?;
                let lms_response = response.json().await?;
                as_string(lms_response, &key).map(Some).or_else(|e| {
                    match e.downcast_ref::<ResultError>() {
                        Some(ResultError::NoKey { .. }) => Ok(None),
                        _ => Err(e),
                    }
                })
            })()
            .await,
            anyhow!("Error get_current_title"),
        )
        .await
    }

    pub async fn play(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::play(name)).await,
            anyhow!("Error play"),
        )
        .await
    }

    pub async fn stop(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::stop(name)).await,
            anyhow!("Error stop"),
        )
        .await
    }

    pub async fn pause(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::pause(name)).await,
            anyhow!("Error pause"),
        )
        .await
    }

    pub async fn play_pause(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::play_pause(name)).await,
            anyhow!("Error play_pause"),
        )
        .await
    }

    pub async fn previous(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::previous(name)).await,
            anyhow!("Error previous"),
        )
        .await
    }

    pub async fn next(&self, name: String) -> Result<()> {
        self.handle_error(
            self.post_no_result(&LmsRequest::next(name)).await,
            anyhow!("Error next"),
        )
        .await
    }

    // The error is not passed to the client but send to the error channel.
    async fn handle_error<T>(&self, result: Result<T>, error: anyhow::Error) -> Result<T> {
        match result {
            Result::Ok(s) => Ok(s),
            Err(error_from_result) => {
                self.sender.send(error_from_result).await?;
                Err(error)
            }
        }
    }

    async fn post(&self, request: &LmsRequest) -> Result<Response> {
        self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.into())
    }

    async fn post_no_result(&self, request: &LmsRequest) -> Result<()> {
        let response = self.post(&request).await?;
        response
            .json::<LmsResponse>()
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LmsResponse {
    #[allow(dead_code)]
    method: String,
    #[allow(dead_code)]
    params: (String, Vec<String>),

    result: serde_json::Value,
}

fn as_bool(response: LmsResponse, key: &String) -> Result<bool> {
    let value = result_key(response, key)?;
    match value {
        Value::Number(n) => n
            .as_i64()
            .map(|i| i != 0)
            .ok_or_else(|| anyhow!("{} is not an i64", n)),
        _ => bail!("Wrong top level type for bool: {:?}", value),
    }
}

fn as_u64(response: LmsResponse, key: &String) -> Result<u64> {
    let value = result_key(response, key)?;
    match value {
        Value::String(n) => n.parse::<u64>().map_err(|e| e.into()),
        Value::Number(n) => n.as_u64().ok_or_else(|| anyhow!("{} is not an u64", n)),
        _ => bail!("Wrong top level type for u64: {:?}", value),
    }
}

fn as_string(response: LmsResponse, key: &String) -> Result<String> {
    let value = result_key(response, key)?;
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => bail!("Wrong top level type for string: {:?}", value),
    }
}

fn as_mode(response: LmsResponse, key: &String) -> Result<Mode> {
    let value = result_key(response, &key)?;
    match value {
        Value::String(s) => match s.as_str() {
            "stop" => Ok(Mode::Stop),
            "play" => Ok(Mode::Play),
            "pause" => Ok(Mode::Pause),
            other => bail!("Expected stop, play or pause, got {}", other),
        },
        _ => bail!("Wrong top level type for mode: {:?}", value),
    }
}

fn as_shuffle(response: LmsResponse, key: &String) -> Result<Shuffle> {
    fn wrong_value<T: std::fmt::Display>(value: T) -> anyhow::Error {
        anyhow!("Expected 0, 1 or 2, got {}", value)
    }

    let value = result_key(response, &key)?;
    match value {
        Value::String(s) => match s.as_str() {
            "0" => Ok(Shuffle::Off),
            "1" => Ok(Shuffle::Songs),
            "2" => Ok(Shuffle::Albums),
            _ => Err(wrong_value(s)),
        },
        Value::Number(n) => match n.as_u64() {
            Some(0) => Ok(Shuffle::Off),
            Some(1) => Ok(Shuffle::Songs),
            Some(2) => Ok(Shuffle::Albums),
            _ => Err(wrong_value(n)),
        },
        _ => bail!("Wrong top level type for shuffle: {:?}", value),
    }
}

#[derive(Debug, Error)]
enum ResultError {
    #[error("The result key has the wrong type: {response:?}")]
    ResultHasWrongType { response: LmsResponse },
    #[error("Unable to get key {key} in {response:?}")]
    NoKey { response: LmsResponse, key: String },
}

fn result_key(response: LmsResponse, key: &String) -> Result<Value> {
    let mut result = match response.result {
        Value::Object(ref map) => Ok(map.clone()),
        _ => Err(anyhow!(ResultError::ResultHasWrongType {
            response: response.clone()
        })),
    }?;
    result.remove(key).ok_or_else(|| {
        anyhow!(ResultError::NoKey {
            response: response,
            key: key.clone()
        })
    })
}
