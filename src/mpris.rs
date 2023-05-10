use crate::lms::{LmsClient, Mode, Shuffle};
use log::{debug, info};
use std::{collections::HashMap, convert::TryFrom, result};
use zbus::{
    dbus_interface, fdo,
    zvariant::{ObjectPath, Value},
};
use zbus::{Connection, ConnectionBuilder};

/// Start the DBus server for a given player and expose an MPRIS interface for it. This interface
/// is specified in [the MPRIS
/// documentation](https://specifications.freedesktop.org/mpris-spec/latest/).
pub async fn start_dbus_server(
    client: LmsClient,
    player_name: String,
) -> anyhow::Result<Connection> {
    info!("Starting DBus server for player {}", player_name);
    let player = MprisPlayer {
        client,
        player_name: player_name.clone(),
    };

    let connection = ConnectionBuilder::session()?
        .name(format!("org.mpris.MediaPlayer2.{}", player_name))?
        .serve_at("/org/mpris/MediaPlayer2", MprisRoot {})?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build()
        .await?;

    info!("DBus server started for player {}", player_name);
    Ok(connection)
}

struct MprisRoot {}

#[dbus_interface(name = "org.mpris.MediaPlayer2")]
impl MprisRoot {
    async fn raise(&self) {
        debug!("MprisRoot::raise");
    }

    async fn quit(&self) {
        debug!("MprisRoot::quit");
    }

    #[dbus_interface(property)]
    async fn can_quit(&self) -> bool {
        debug!("MprisRoot::can_quit");
        false
    }
    #[dbus_interface(property)]
    async fn can_raise(&self) -> bool {
        debug!("MprisRoot::can_raise");
        false
    }

    #[dbus_interface(property)]
    async fn has_track_list(&self) -> bool {
        debug!("MprisRoot::has_track_list");
        false
    }

    #[dbus_interface(property)]
    async fn identity(&self) -> String {
        debug!("MprisRoot::identity");
        "squeezelite".to_string()
    }

    #[dbus_interface(property)]
    async fn supported_uri_schemes(&self) -> Vec<String> {
        debug!("MprisRoot::supported_uri_schemes");
        vec![]
    }

    #[dbus_interface(property)]
    async fn supported_mime_types(&self) -> Vec<String> {
        debug!("MprisRoot::supported_mime_types");
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
        debug!("MprisPlayer::next");
        self.client
            .next(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn previous(&self) -> Result<(), fdo::Error> {
        debug!("MprisPlayer::previous");
        self.client
            .previous(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn pause(&self) -> Result<(), fdo::Error> {
        debug!("MprisPlayer::pause");
        self.client
            .pause(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn play_pause(&self) -> Result<(), fdo::Error> {
        debug!("MprisPlayer::play_pause");
        self.client
            .play_pause(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn stop(&self) -> Result<(), fdo::Error> {
        debug!("MprisPlayer::stop");
        self.client
            .stop(self.player_name.clone())
            .await
            .map_err(to_fdo_error)
    }
    async fn play(&self) -> Result<(), fdo::Error> {
        debug!("MprisPlayer::play");
        let res = self
            .client
            .play(self.player_name.clone())
            .await
            .map_err(to_fdo_error);
        res
    }
    async fn seek(&self, offset: i64) {
        debug!("MprisPlayer::seek {}", offset);
    }
    async fn set_position(&self, track_id: String, position: i64) {
        debug!("MprisPlayer::set_position {} {}", track_id, position);
    }
    async fn open_uri(&self, uri: String) {
        debug!("MprisPlayer::open_uri {}", uri);
    }

    #[dbus_interface(property)]
    async fn playback_status(&self) -> result::Result<String, fdo::Error> {
        debug!("MprisPlayer::playback_status");
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
        debug!("MprisPlayer::loop_status");
        "None".to_string()
    }
    #[dbus_interface(property)]
    async fn rate(&self) -> f64 {
        1.0
    }
    #[dbus_interface(property)]
    async fn shuffle(&self) -> result::Result<bool, fdo::Error> {
        debug!("MprisPlayer::shuffle");
        let shuffle = self
            .client
            .get_shuffle(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;

        Ok(shuffle == Shuffle::Songs)
    }
    #[dbus_interface(property)]
    async fn metadata(&self) -> result::Result<HashMap<String, Value>, fdo::Error> {
        debug!("MprisPlayer::metadata");
        let track_count = self
            .client
            .get_track_count(self.player_name.clone())
            .await
            .map_err(to_fdo_error)?;
        if track_count == 0 {
            debug!("MprisPlayer::metadata no track");
            return Ok(HashMap::new());
        }
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
        artist.map(|artist| {
            hm.insert("xesam:artist".to_string(), vec![artist].into());
        });
        current_title.map(|title| {
            hm.insert("xesam:title".to_string(), title.into());
        });
        Ok(hm)
    }
    #[dbus_interface(property)]
    async fn volume(&self) -> f64 {
        debug!("MprisPlayer::volume");
        1.0
    }
    #[dbus_interface(property)]
    async fn position(&self) -> i64 {
        debug!("MprisPlayer::position");
        0
    }
    #[dbus_interface(property)]
    async fn minimum_rate(&self) -> f64 {
        debug!("MprisPlayer::minimum_rate");
        1.0
    }
    #[dbus_interface(property)]
    async fn maximum_rate(&self) -> f64 {
        debug!("MprisPlayer::maximum_rate");
        1.0
    }
    #[dbus_interface(property)]
    async fn can_go_next(&self) -> bool {
        debug!("MprisPlayer::can_go_next");
        true
    }
    #[dbus_interface(property)]
    async fn can_go_previous(&self) -> bool {
        debug!("MprisPlayer::can_go_previous");
        true
    }
    #[dbus_interface(property)]
    async fn can_play(&self) -> bool {
        debug!("MprisPlayer::can_play");
        true
    }
    #[dbus_interface(property)]
    async fn can_pause(&self) -> bool {
        debug!("MprisPlayer::can_pause");
        true
    }
    #[dbus_interface(property)]
    async fn can_seek(&self) -> bool {
        debug!("MprisPlayer::can_seek");
        false
    }
    #[dbus_interface(property)]
    async fn can_control(&self) -> bool {
        debug!("MprisPlayer::can_control");
        true
    }
}
