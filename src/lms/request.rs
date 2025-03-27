//! The functions to create the requests sent to the LMS server. The requests available are
//! described in [the LMS
//! documentation](https://raw.githack.com/Logitech/slimserver/public/8.4/HTML/EN/html/docs/cli-api.html)
use serde::Serialize;

/// This structure is serialized to JSON and sent to the LMS server.
#[derive(Debug, Serialize)]
pub struct LmsRequest {
    method: String,
    params: (String, Vec<String>),
}

impl LmsRequest {
    fn new(id: String) -> Self {
        Self {
            method: "slim.request".to_string(),
            params: (id, vec![]),
        }
    }

    fn add_param(mut self, param: String) -> Self {
        self.params.1.push(param);
        self
    }

    pub fn version() -> (Self, String) {
        Self::new("".to_string()).question("version".to_string())
    }

    fn question(self, key: String) -> (Self, String) {
        (
            self.add_param(key.clone()).add_param("?".to_string()),
            ("_".to_owned() + &key),
        )
    }

    pub fn connected(id: String) -> (Self, String) {
        Self::new(id).question("connected".to_string())
    }

    pub fn players() -> Self {
        Self::new("".to_string())
            .add_param("players".to_string())
            .add_param("0".to_string())
    }

    pub fn players_loop() -> (Self, String) {
        (Self::players(), "players_loop".to_string())
    }

    pub fn players_count() -> (Self, String) {
        (Self::players(), "count".to_string())
    }

    pub fn artist(id: String) -> (Self, String) {
        Self::new(id).question("artist".to_string())
    }

    pub fn title(id: String) -> (Self, String) {
        Self::new(id).question("title".to_string())
    }

    pub fn album(id: String) -> (Self, String) {
        Self::new(id).question("album".to_string())
    }

    pub fn mode(id: String) -> (Self, String) {
        Self::new(id).question("mode".to_string())
    }

    fn playlist(id: String) -> Self {
        Self::new(id).add_param("playlist".to_string())
    }

    pub fn shuffle(id: String) -> (Self, String) {
        Self::playlist(id).question("shuffle".to_string())
    }

    pub fn index(id: String) -> (Self, String) {
        Self::playlist(id).question("index".to_string())
    }

    pub fn track_count(id: String) -> (Self, String) {
        Self::playlist(id).question("tracks".to_string())
    }

    pub fn play(id: String) -> Self {
        Self::new(id).add_param("play".to_string())
    }

    pub fn stop(id: String) -> Self {
        Self::new(id).add_param("stop".to_string())
    }

    pub fn pause(id: String) -> Self {
        Self::new(id)
            .add_param("pause".to_string())
            .add_param("1".to_string())
    }

    pub fn play_pause(id: String) -> Self {
        Self::new(id).add_param("pause".to_string())
    }

    pub fn previous(id: String) -> Self {
        Self::playlist(id)
            .add_param("index".to_string())
            .add_param("-1".to_string())
    }

    pub fn next(id: String) -> Self {
        Self::playlist(id)
            .add_param("index".to_string())
            .add_param("+1".to_string())
    }
}
