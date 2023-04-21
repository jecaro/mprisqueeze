use crate::lms_client::{LmsClient, Mode, Shuffle};
use std::{collections::HashMap, convert::TryFrom, result};
use zbus::{
    dbus_interface, fdo,
    zvariant::{ObjectPath, Value},
};
use zbus::{Connection, ConnectionBuilder};

pub async fn start_dbus_server(
    client: LmsClient,
    player_name: String,
) -> anyhow::Result<Connection> {
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

fn to_fdo_error(err: anyhow::Error) -> fdo::Error {
    fdo::Error::Failed(err.to_string())
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl MprisPlayer {
    async fn next(&self) -> Result<(), fdo::Error> {
        self.client
            .next(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn previous(&self) -> Result<(), fdo::Error> {
        self.client
            .previous(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn pause(&self) -> Result<(), fdo::Error> {
        self.client
            .pause(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn play_pause(&self) -> Result<(), fdo::Error> {
        self.client
            .play_pause(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn stop(&self) -> Result<(), fdo::Error> {
        self.client
            .stop(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn play(&self) -> Result<(), fdo::Error> {
        let res = self
            .client
            .play(self.player_name.clone())
            .await
            .map_err(to_fdo_error);
        res
    }
    async fn seek(&self, _offset: i64) {}
    async fn set_position(&self, _track_id: String, _position: i64) {}
    async fn open_uri(&self, _uri: String) {}

    #[dbus_interface(property)]
    async fn playback_status(&self) -> result::Result<String, fdo::Error> {
        let mode = self
            .client
            .get_mode(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;
        Ok(match mode {
            Mode::Play => "Playing",
            Mode::Pause => "Paused",
            Mode::Stop => "Stopped",
        }
        .to_string())
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
    async fn shuffle(&self) -> result::Result<bool, fdo::Error> {
        let shuffle = self
            .client
            .get_shuffle(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;

        Ok(shuffle == Shuffle::Songs)
    }
    #[dbus_interface(property)]
    async fn metadata(&self) -> result::Result<HashMap<String, Value>, fdo::Error> {
        let current_title = self
            .client
            .get_current_title(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;
        let artist = self
            .client
            .get_artist(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;
        let index = self
            .client
            .get_index(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;
        let mut hm = HashMap::new();
        let op = ObjectPath::try_from(format!(
            "/org/mpris/MediaPlayer2/{0}/track/{index}",
            self.player_name
        ))
        .unwrap();
        hm.insert("mpris:trackid".to_string(), op.into());
        hm.insert("xesam:title".to_string(), current_title.into());
        hm.insert("xesam:artist".to_string(), artist.into());
        Ok(hm)
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
