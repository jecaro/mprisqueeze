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

    pub async fn get_players(&self) -> Result<Vec<Player>> {
        let (request, key) = LmsRequest::players();
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        let value = result_key(&lms_response, &key)?.clone();
        serde_json::from_value(value.to_owned()).map_err(|e| e.into())
    }

    pub async fn get_index(&self, name: String) -> Result<u16> {
        let (request, key) = LmsRequest::index(name);
        let response = self.post(&request).await?;
        let lms_response = response.json().await?;
        as_u16(&lms_response, &key)
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

    async fn post_no_result(&self, request: &LmsRequest) -> Result<()> {
        let response = self.post(&request).await?;
        response
            .json::<LmsResponse>()
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    pub async fn play(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::play(name)).await
    }

    pub async fn stop(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::stop(name)).await
    }

    pub async fn pause(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::pause(name)).await
    }

    pub async fn play_pause(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::play_pause(name)).await
    }

    pub async fn previous(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::previous(name)).await
    }

    pub async fn next(&self, name: String) -> Result<()> {
        self.post_no_result(&LmsRequest::next(name)).await
    }
}

#[derive(Debug, Serialize)]
struct LmsRequest {
    method: String,
    params: (String, Vec<String>),
}

#[derive(Clone, Debug, Deserialize)]
struct LmsResponse {
    #[allow(dead_code)]
    method: String,
    #[allow(dead_code)]
    params: (String, Vec<String>),

    result: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Player {
    pub name: String,
}

impl LmsRequest {
    fn new(name: String) -> Self {
        Self {
            method: "slim.request".to_string(),
            params: (name, vec![]),
        }
    }

    fn add_param(mut self, param: String) -> Self {
        self.params.1.push(param);
        self
    }

    fn version() -> (Self, String) {
        Self::new("".to_string()).question("version".to_string())
    }

    fn question(self, key: String) -> (Self, String) {
        (
            self.add_param(key.clone()).add_param("?".to_string()),
            ("_".to_owned() + &key),
        )
    }

    fn connected(name: String) -> (Self, String) {
        Self::new(name).question("connected".to_string())
    }

    fn players() -> (Self, String) {
        (
            Self::new("".to_string())
                .add_param("players".to_string())
                .add_param("0".to_string()),
            "players_loop".to_string(),
        )
    }

    fn artist(name: String) -> (Self, String) {
        Self::new(name).question("artist".to_string())
    }

    fn current_title(name: String) -> (Self, String) {
        Self::new(name).question("current_title".to_string())
    }

    fn mode(name: String) -> (Self, String) {
        Self::new(name).question("mode".to_string())
    }

    fn playlist(name: String) -> Self {
        Self::new(name).add_param("playlist".to_string())
    }

    fn shuffle(name: String) -> (Self, String) {
        Self::playlist(name).question("shuffle".to_string())
    }

    fn index(name: String) -> (Self, String) {
        Self::playlist(name).question("index".to_string())
    }

    fn play(name: String) -> Self {
        Self::new(name).add_param("play".to_string())
    }

    fn stop(name: String) -> Self {
        Self::new(name).add_param("stop".to_string())
    }

    fn pause(name: String) -> Self {
        Self::new(name)
            .add_param("pause".to_string())
            .add_param("1".to_string())
    }

    fn play_pause(name: String) -> Self {
        Self::new(name).add_param("pause".to_string())
    }

    fn previous(name: String) -> Self {
        Self::playlist(name)
            .add_param("index".to_string())
            .add_param("-1".to_string())
    }

    fn next(name: String) -> Self {
        Self::playlist(name)
            .add_param("index".to_string())
            .add_param("+1".to_string())
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

fn as_u16(response: &LmsResponse, key: &String) -> Result<u16> {
    let res_value = result_key(response, key)?;
    match res_value {
        serde_json::Value::String(n) => n.parse::<u16>().map_err(|e| anyhow!(e)),
        _ => bail!("Wrong top level type for u16: {:?}", response),
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
    result.get(key).ok_or_else(|| {
        anyhow!(ResultError::NoKey {
            response: response.clone(),
            key: key.clone()
        })
    })
}
