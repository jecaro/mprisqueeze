use crate::lms_client::{LmsClient, Mode, Shuffle};
use anyhow::Result;
use std::{collections::HashMap, convert::TryFrom};
use zbus::{
    dbus_interface,
    zvariant::{ObjectPath, Value},
};
use zbus::{Connection, ConnectionBuilder};

pub async fn connect(hostname: String, port: u16, player_name: String) -> Result<Connection> {
    let client = LmsClient::new(hostname, port);

    let root = MprisRoot {
        name: player_name.clone(),
    };

    let player = MprisPlayer {
        client,
        player_name: player_name.clone(),
    };

    let connection = ConnectionBuilder::session()?
        .name(format!("org.mpris.MediaPlayer2.{}", player_name))?
        .serve_at("/org/mpris/MediaPlayer2", root)?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build()
        .await?;

    Ok(connection)
}

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

struct MprisPlayer {
    client: LmsClient,
    player_name: String,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl MprisPlayer {
    async fn next(&self) {}
    async fn previous(&self) {}
    async fn pause(&self) {}
    async fn play_pause(&self) {}
    async fn stop(&self) {}
    async fn play(&self) {}
    async fn seek(&self, _offset: i64) {}
    async fn set_position(&self, _track_id: String, _position: i64) {}
    async fn open_uri(&self, _uri: String) {}

    #[dbus_interface(property)]
    async fn playback_status(&self) -> String {
        let mode = self
            .client
            .get_mode(self.player_name.clone())
            .await
            .unwrap_or(Mode::Pause);
        match mode {
            Mode::Play => "Playing",
            Mode::Pause => "Paused",
            Mode::Stop => "Stopped",
        }
        .to_string()
    }
    #[dbus_interface(property)]
    async fn loop_status(&self) -> String {
        "None".to_string()
    }
    #[dbus_interface(property)]
    async fn rate(&self) -> f64 {
        1.0
    }
    #[dbus_interface(property)]
    async fn shuffle(&self) -> bool {
        let shuffle = self
            .client
            .get_shuffle(self.player_name.clone())
            .await
            .unwrap_or(Shuffle::Off);

        shuffle == Shuffle::Songs
    }
    #[dbus_interface(property)]
    async fn metadata(&self) -> HashMap<String, Value> {
        let current_title = self
            .client
            .get_current_title(self.player_name.clone())
            .await
            .unwrap_or("".to_string());
        let artist = self
            .client
            .get_artist(self.player_name.clone())
            .await
            .unwrap_or("".to_string());
        let index = self
            .client
            .get_index(self.player_name.clone())
            .await
            .unwrap_or(0);
        let mut hm = HashMap::new();
        let op = ObjectPath::try_from(format!(
            "/org/mpris/MediaPlayer2/{0}/track/{index}",
            self.player_name
        ))
        .unwrap();
        hm.insert("mpris:trackid".to_string(), op.into());
        hm.insert("xesam:title".to_string(), current_title.into());
        hm.insert("xesam:artist".to_string(), artist.into());
        hm
    }
    #[dbus_interface(property)]
    async fn volume(&self) -> f64 {
        1.0
    }
    #[dbus_interface(property)]
    async fn position(&self) -> i64 {
        0
    }
    #[dbus_interface(property)]
    async fn minimum_rate(&self) -> f64 {
        1.0
    }
    #[dbus_interface(property)]
    async fn maximum_rate(&self) -> f64 {
        1.0
    }
    #[dbus_interface(property)]
    async fn can_go_next(&self) -> bool {
        true
    }
    #[dbus_interface(property)]
    async fn can_go_previous(&self) -> bool {
        true
    }
    #[dbus_interface(property)]
    async fn can_play(&self) -> bool {
        true
    }
    #[dbus_interface(property)]
    async fn can_pause(&self) -> bool {
        true
    }
    #[dbus_interface(property)]
    async fn can_seek(&self) -> bool {
        false
    }
    #[dbus_interface(property)]
    async fn can_control(&self) -> bool {
        true
    }
}
