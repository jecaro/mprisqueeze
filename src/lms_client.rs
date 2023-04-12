use anyhow::{anyhow, bail, Ok, Result};
use reqwest::Client;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

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
}

impl LmsClient {
    pub fn new(hostname: String, port: u16) -> Self {
        let client = Client::new();
        let url = format!("http://{}:{}/jsonrpc.js", hostname, port);
        Self { client, url }
    }

    async fn post(&self, request: &LmsRequest) -> Result<Response> {
        self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.into())
    }

    #[allow(dead_code)]
    pub async fn get_version(&self) -> Result<String> {
        let (request, key) = LmsRequest::version();
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_string(&lms_response, &key)
    }

    #[allow(dead_code)]
    pub async fn get_connected(&self, name: String) -> Result<bool> {
        let (request, key) = LmsRequest::connected(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_bool(&lms_response, &key)
    }

    pub async fn get_shuffle(&self, name: String) -> Result<Shuffle> {
        let (request, key) = LmsRequest::shuffle(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_shuffle(&lms_response, &key)
    }

    pub async fn get_mode(&self, name: String) -> Result<Mode> {
        let (request, key) = LmsRequest::mode(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_mode(&lms_response, &key)
    }

    pub async fn get_artist(&self, name: String) -> Result<String> {
        let (request, key) = LmsRequest::artist(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        // When listening a remote stream, the artist is not available. The key with the result is not
        // even in the json.
        as_string(&lms_response, &key)
            .map(|s| s.to_string())
            .or_else(|e| match e.downcast_ref::<ResultError>() {
                Some(ResultError::NoKey { .. }) => Ok("".to_string()),
                _ => Err(e),
            })
    }

    pub async fn get_current_title(&self, name: String) -> Result<String> {
        let (request, key) = LmsRequest::current_title(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_string(&lms_response, &key).map(|s| s.to_string())
    }
}

#[derive(Serialize, Debug)]
struct LmsRequest {
    method: String,
    params: (String, Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
struct LmsResponse {
    #[allow(dead_code)]
    method: String,
    #[allow(dead_code)]
    params: (String, Vec<String>),

    result: serde_json::Value,
}

impl LmsRequest {
    fn new(name: String, params: Vec<&str>) -> Self {
        let params = params.into_iter().map(|s| s.to_string()).collect();
        Self {
            method: "slim.request".to_string(),
            params: (name, params),
        }
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
