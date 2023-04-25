use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LmsRequest {
    method: String,
    params: (String, Vec<String>),
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

    pub fn version() -> (Self, String) {
        Self::new("".to_string()).question("version".to_string())
    }

    fn question(self, key: String) -> (Self, String) {
        (
            self.add_param(key.clone()).add_param("?".to_string()),
            ("_".to_owned() + &key),
        )
    }

    pub fn connected(name: String) -> (Self, String) {
        Self::new(name).question("connected".to_string())
    }

    pub fn players() -> (Self, String) {
        (
            Self::new("".to_string())
                .add_param("players".to_string())
                .add_param("0".to_string()),
            "players_loop".to_string(),
        )
    }

    pub fn artist(name: String) -> (Self, String) {
        Self::new(name).question("artist".to_string())
    }

    pub fn current_title(name: String) -> (Self, String) {
        Self::new(name).question("current_title".to_string())
    }

    pub fn mode(name: String) -> (Self, String) {
        Self::new(name).question("mode".to_string())
    }

    fn playlist(name: String) -> Self {
        Self::new(name).add_param("playlist".to_string())
    }

    pub fn shuffle(name: String) -> (Self, String) {
        Self::playlist(name).question("shuffle".to_string())
    }

    pub fn index(name: String) -> (Self, String) {
        Self::playlist(name).question("index".to_string())
    }

    pub fn play(name: String) -> Self {
        Self::new(name).add_param("play".to_string())
    }

    pub fn stop(name: String) -> Self {
        Self::new(name).add_param("stop".to_string())
    }

    pub fn pause(name: String) -> Self {
        Self::new(name)
            .add_param("pause".to_string())
            .add_param("1".to_string())
    }

    pub fn play_pause(name: String) -> Self {
        Self::new(name).add_param("pause".to_string())
    }

    pub fn previous(name: String) -> Self {
        Self::playlist(name)
            .add_param("index".to_string())
            .add_param("-1".to_string())
    }

    pub fn next(name: String) -> Self {
        Self::playlist(name)
            .add_param("index".to_string())
            .add_param("+1".to_string())
    }
}
