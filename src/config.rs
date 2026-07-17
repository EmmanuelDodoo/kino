use crate::theme::Theme;
use crate::utils::icons;
use core::variants;
use devutils::source::SourceSet;
use iced::Color;
pub use keys::{KeyPress, KeyStore};
use serde::{Deserialize, Serialize, de, ser};
use std::{path::PathBuf, time::Duration};
pub use subtitles::SubtitleDescription;

mod keys;

pub fn se_color<S: ser::Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    let color = color.to_string();

    s.serialize_str(&color)
}

pub fn de_color<'de, D: de::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    use de::Visitor;

    struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a valid Color string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<Color>().map_err(|error| de::Error::custom(error))
        }
    }

    d.deserialize_str(ColorVisitor)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VideoFilters {
    pub brightness: f64,
    pub contrast: f64,
    pub hue: f64,
    pub saturation: f64,
    pub gamma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "VideoSettings::defaults", rename = "Player")]
pub struct VideoSettings {
    pub thumbnail_interval: u32,
    pub volume: f64,
    pub speed: f64,
    pub volume_change_amt: f64,
    pub seek_change_amt: f64,
    pub seek_shift_change_amt: f64,
    pub speed_change_amt: f64,
    pub show_subtitles: bool,
    pub muted: bool,
    /// Whether a loaded video automatically starts playing
    pub auto_start: bool,
    /// Whether the next video in a playlist is automatically loaded and played.
    pub auto_next: bool,
    /// The percentage at which a video is considered as 'watched'.
    pub completion_point: f64,
    /// The percentage watch time at which a video is considered 'watched'.
    pub completion_watch_time: f64,

    pub subtitles: SubtitleDescription,

    pub filters: VideoFilters,

    pub comment_span: u64,
}

impl VideoSettings {
    fn defaults() -> Self {
        Self {
            thumbnail_interval: 10,
            volume: 1.0,
            speed: 1.0,
            volume_change_amt: 0.05,
            seek_change_amt: 5.0,
            seek_shift_change_amt: 10.0,
            speed_change_amt: 0.1,
            show_subtitles: true,
            muted: false,
            auto_start: true,
            auto_next: true,
            completion_point: 0.95,
            completion_watch_time: 0.75,
            subtitles: SubtitleDescription::defaults(),
            filters: VideoFilters {
                brightness: 0.00,
                contrast: 1.0,
                hue: 0.0,
                saturation: 1.0,
                gamma: 1.0,
            },
            comment_span: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default = "GeneralSettings::defaults", rename = "General")]
pub struct GeneralSettings {
    pub layout: Layout,
    pub refresh_interval: Duration,
    pub recents_limit: Option<i32>,
    pub search_limit: Option<i32>,
    pub theme: Theme,
    #[serde(skip)]
    pub themes: Vec<Theme>,
    pub scan_discoverer: bool,
    pub auth_token: String,
    pub movie_depth: u8,
    pub fetching_interval: Duration,
    pub restore_deleted: bool,
    pub preferred_subtitle_codec: Option<String>,
    pub preferred_audio_codec: Option<String>,
    pub tmdb_rating: bool,
    pub default_source: SourceSet,
    pub show_dirs: bool,
}

impl GeneralSettings {
    fn defaults() -> Self {
        Self {
            layout: Layout::default(),
            refresh_interval: Duration::from_secs(600),
            theme: Theme::default(),
            themes: vec![],
            recents_limit: Some(5),
            search_limit: Some(5),
            scan_discoverer: true,
            auth_token: String::default(),
            movie_depth: 2,
            fetching_interval: Duration::from_secs(300),
            restore_deleted: true,
            preferred_subtitle_codec: Some("en".into()),
            preferred_audio_codec: Some("en".into()),
            tmdb_rating: true,
            default_source: SourceSet::Tmdb,
            show_dirs: false,
        }
    }

    fn debug_defaults() -> Self {
        Self {
            layout: Layout::default(),
            refresh_interval: Duration::from_secs(120),
            theme: Theme::default(),
            themes: vec![],
            recents_limit: Some(5),
            search_limit: Some(5),
            scan_discoverer: false,
            auth_token: String::default(),
            movie_depth: 0,
            fetching_interval: Duration::from_secs(30),
            restore_deleted: true,
            preferred_subtitle_codec: Some("en".into()),
            preferred_audio_codec: Some("en".into()),
            tmdb_rating: true,
            default_source: SourceSet::Tmdb,
            show_dirs: true,
        }
    }

    fn load_themes(&mut self) {
        self.themes = self.theme.all()
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default = "Config::defaults")]
pub struct Config {
    pub video: VideoSettings,
    pub general: GeneralSettings,
    #[serde(rename = "keybindings")]
    pub keystore: KeyStore,
    #[serde(skip)]
    config_dir: Option<PathBuf>,
    #[serde(skip)]
    pub span_writer: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        let Config {
            video,
            general,
            keystore,
            config_dir,
            span_writer: _writer,
        } = self;

        Self {
            video: video.clone(),
            general: general.clone(),
            keystore: keystore.clone(),
            config_dir: config_dir.clone(),
            span_writer: None,
        }
    }
}

impl Config {
    const CONFIG_PATH: &str = "config.toml";
    const DEV_PATH: &str = ".dev";
    const LOG_FILE: &str = "kino.log";

    pub fn load() -> (Self, Vec<String>) {
        use directories::ProjectDirs;
        use std::fs::{create_dir_all, read_to_string};

        tracing::debug!("Loading Config");

        let mut errors = Vec::with_capacity(3);

        let project = ProjectDirs::from("", "", "kino").expect("Cannot create project directory");
        let config_dir = project.config_local_dir();
        let images = config_dir.join("images");

        create_dir_all(images).expect("Cannot create project directory structure");

        let config_path = config_dir.join(Self::CONFIG_PATH);

        tracing::debug!("Reading config contents");
        let config = match read_to_string(config_path).map_err(|error| error.kind()) {
            Ok(config) => config,
            Err(std::io::ErrorKind::NotFound) => {
                return (Config::defaults().prep(config_dir), errors);
            }
            Err(error) => {
                errors.push(format!("Config file reading error.\n{error}"));

                return (Config::defaults().prep(config_dir), errors);
            }
        };

        match toml::from_str::<Config>(&config) {
            Ok(config) => (config.prep(config_dir), errors),
            Err(error) => {
                errors.push(format!("Config file loading error.\n{error}"));

                (Config::defaults().prep(config_dir), errors)
            }
        }
    }

    fn prep(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        use std::fs::OpenOptions;
        use tracing_subscriber::EnvFilter;

        tracing::debug!("preping config");
        let dir = dir.as_ref();
        let log = dir.join(Self::LOG_FILE);
        self.config_dir = Some(dir.to_path_buf());
        self.general.load_themes();

        match OpenOptions::new().append(true).create(true).open(log) {
            Ok(log) => {
                let (non_blocking, writer_guard) = tracing_appender::non_blocking(log);
                let filter = EnvFilter::new("error,kino=debug");

                self.span_writer = Some(writer_guard);

                tracing_subscriber::fmt()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_env_filter(filter)
                    .init();
            }
            Err(error) => {
                tracing::error!("Error opening log file: \n{error}");
            }
        };

        self
    }

    pub fn save(&self) -> Result<(), String> {
        use std::fs::write;

        let path = match &self.config_dir {
            Some(dir) => dir.join(Self::CONFIG_PATH),
            None => [Self::DEV_PATH, Self::CONFIG_PATH].iter().collect(),
        };

        let config = toml::to_string_pretty(self).map_err(|error| error.to_string())?;

        write(path, config).map_err(|error| error.to_string())
    }

    pub fn defaults() -> Self {
        Self {
            video: VideoSettings::defaults(),
            general: GeneralSettings::defaults(),
            keystore: KeyStore::defaults(),
            config_dir: None,
            span_writer: None,
        }
    }

    pub fn dev() -> Self {
        tracing::debug!("Loading dev config");
        let new = Self {
            general: GeneralSettings::debug_defaults(),
            video: VideoSettings::defaults(),
            keystore: KeyStore::defaults(),
            config_dir: Some(PathBuf::from(Self::DEV_PATH)),
            span_writer: None,
        };

        new.prep(Self::DEV_PATH)
    }

    pub fn theme(&self) -> Theme {
        self.general.theme.clone()
    }

    pub fn theme_ref(&self) -> &Theme {
        &self.general.theme
    }

    pub fn layout(&self) -> Layout {
        self.general.layout
    }

    pub fn refresh_interval(&self) -> Duration {
        self.general.refresh_interval
    }

    pub fn search_limit(&self) -> Option<i32> {
        self.general.search_limit
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        let path = self
            .config_dir
            .as_ref()
            .map(|dir| dir.join(Self::CONFIG_PATH))?;

        match path.canonicalize() {
            Ok(path) => Some(path),
            Err(error) => {
                let error = error.to_string();
                tracing::error!(error);
                Some(path)
            }
        }
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        let path = self
            .config_dir
            .as_ref()
            .map(|dir| dir.join(Self::LOG_FILE))?;

        match path.canonicalize() {
            Ok(path) => Some(path),
            Err(error) => {
                let error = error.to_string();
                tracing::error!(error);
                Some(path)
            }
        }
    }

    pub fn db_path(&self) -> PathBuf {
        self.config_dir
            .as_ref()
            .map(|dir| dir.join("kino.db"))
            .unwrap_or_else(|| [Self::DEV_PATH, "kino.db"].iter().collect())
    }

    pub fn images_path(&self) -> PathBuf {
        self.config_dir
            .as_ref()
            .map(|dir| dir.join("images"))
            .unwrap_or_else(|| [Self::DEV_PATH, "images"].iter().collect())
    }

    pub fn fetching_interval(&self) -> Duration {
        self.general.fetching_interval
    }

    pub fn tmdb_auth(&self) -> String {
        self.general.auth_token.clone()
    }
}

mod subtitles {
    use super::{de_color, se_color};
    use crate::utils::typo::DEFAULT_SUBTITLE_FONT_NAME;
    use iced::{Color, color};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(default = "SubtitleDescription::defaults")]
    pub struct SubtitleDescription {
        pub size: u32,
        #[serde(serialize_with = "se_color", deserialize_with = "de_color")]
        pub color: Color,
        pub font: String,
        #[serde(serialize_with = "se_color", deserialize_with = "de_color")]
        pub background_color: Color,
    }

    impl SubtitleDescription {
        pub(super) fn defaults() -> Self {
            Self {
                size: 20,
                color: color!(0xff8243),
                font: DEFAULT_SUBTITLE_FONT_NAME.to_owned(),
                background_color: Color::BLACK.scale_alpha(0.69),
            }
        }
    }
}
#[derive(Clone, Debug, Copy)]
pub enum Screen {
    Home,
    Player,
    Settings,
    // Log,
}

variants! {
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum Layout {
        #[default]
        Grid,
        List,
        Compact,
    }

}

impl Layout {
    pub fn icon(&self) -> char {
        match self {
            Self::Grid => icons::GRID,
            Self::List => icons::LIST,
            Self::Compact => icons::COMPACT_LIST,
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Grid => "Grid",
                Self::List => "List",
                Self::Compact => "Compact list",
            }
        )
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum HomeAction {
        SettingsOpen,
        CloseModal,
        LayoutToggle,
        RefreshContent,
        /// Refreshes both content and the side menu
        #[serde(rename = "RefreshAll")]
        Refresh,
        SearchToggle,
        Back,
        Forward,
        WishNew,
        SelectionStart,
    }
}

impl HomeAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::SettingsOpen => "Opens the settings screen",
            Self::CloseModal => "Closes the current modal",
            Self::LayoutToggle => "Toggles the content layout",
            Self::RefreshContent => "Refreshes only the current content",
            Self::Refresh => "Refreshes both the current content and collections",
            Self::SearchToggle => "Opens the search dialog",
            Self::Back => "Navigates back to the previous page",
            Self::Forward => "Navigates forward to the next page",
            Self::SelectionStart => "Enters selection mode, adding clicked media to a new playlist",
            Self::WishNew => "Navigates to the wishlist page and opens the new wishlist dialog",
        }
    }
}

impl std::fmt::Display for HomeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Back => "Back",
                Self::Forward => "Forward",
                Self::SearchToggle => "Search Toggle",
                Self::Refresh => "Refresh",
                Self::RefreshContent => "Refresh Content",
                Self::LayoutToggle => "Layout Toggle",
                Self::CloseModal => "Close Modal",
                Self::SettingsOpen => "Settings Open",
                Self::SelectionStart => "Selection Start",
                Self::WishNew => "Wish New",
            }
        )
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum PlayerAction {
        PlayToggle,
        PlayNext,
        PlayPrevious,
        FullscreenToggle,
        /// Used for either exiting fullscreen or closing modal windows
        Exit,
        SeekBack,
        SeekBackShift,
        SeekFront,
        SeekFrontShift,
        VolumeIncrease,
        VolumeDecrease,
        MuteToggle,
        SpeedIncrease,
        SpeedDecrease,
        SpeedReset,
        VideoConfig,
        VideoComment,
        VideoCommentNew,
        SubtitlesToggle,
        #[serde(rename = "CollectionAdd")]
        Add,
        CloseView,
        Back,
        PlaylistToggle,
    }

}

impl PlayerAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::PlayToggle => "Toggles Play/Pause",
            Self::PlayNext => "Plays the next video",
            Self::PlayPrevious => "Plays the previous video",
            Self::FullscreenToggle => "Toggles full screen mode",
            Self::Exit => "Exits either full screen mode or closes the current modal",
            Self::SeekBack => "Rewind video",
            Self::SeekBackShift => "Rewind video",
            Self::SeekFront => "Fast forward video",
            Self::SeekFrontShift => "Fast forward video",
            Self::VolumeIncrease => "Increases the volume",
            Self::VolumeDecrease => "Decreases the volume",
            Self::MuteToggle => "Toggles mute",
            Self::SpeedIncrease => "Increase playback rate",
            Self::SpeedDecrease => "Decrease playback rate",
            Self::SpeedReset => "Reset playback rate",
            Self::VideoConfig => "Opens the video configuration menu",
            Self::VideoComment => "Opens the video comment dialog",
            Self::SubtitlesToggle => "Toggles video subtitles",
            Self::Add => "Opens the add to collection dialog",
            Self::CloseView => "Closes the current modal",
            Self::Back => "Exits the video player",
            Self::PlaylistToggle => "Toggles the playlist view",
            Self::VideoCommentNew => "Starts a new comment on the current playback",
        }
    }
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PlayToggle => "Play Toggle",
                Self::PlayNext => "Play Next",
                Self::PlayPrevious => "Play Previous",
                Self::FullscreenToggle => "Fullscreen Toggle",
                Self::Exit => "Exit",
                Self::SeekBack => "Seek Back",
                Self::SeekBackShift => "Seek Back Shift",
                Self::SeekFront => "Seek Front",
                Self::SeekFrontShift => "Seek Front Shift",
                Self::VolumeIncrease => "Volume Increase",
                Self::VolumeDecrease => "Volume Decrease",
                Self::MuteToggle => "Mute Toggle",
                Self::SpeedIncrease => "Speed Increase",
                Self::SpeedDecrease => "Speed Decrease",
                Self::SpeedReset => "Speed Reset",
                Self::VideoConfig => "Video Config",
                Self::VideoComment => "Video Comment",
                Self::SubtitlesToggle => "Subtitles Toggle",
                Self::Add => "Add",
                Self::CloseView => "Close View",
                Self::Back => "Back",
                Self::PlaylistToggle => "Playlist Toggle",
                Self::VideoCommentNew => "Video Comment New",
            }
        )
    }
}

variants! {

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum SettingsAction {
        Up,
        Down,
        Cancel,
    }
}

impl SettingsAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::Cancel => "Discards changes and exits the settings screen",
            Self::Up => "Navigates to the next sub-menu",
            Self::Down => "Navigates to the previous sub-menu",
        }
    }
}

impl std::fmt::Display for SettingsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cancel => "Cancel",
                Self::Down => "Down",
                Self::Up => "Up",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Home(HomeAction),
    Player(PlayerAction),
    Settings(SettingsAction),
}

impl Action {
    pub fn descr(&self) -> &str {
        match self {
            Self::Home(action) => action.descr(),
            Self::Player(action) => action.descr(),
            Self::Settings(action) => action.descr(),
        }
    }
}

impl From<PlayerAction> for Action {
    fn from(value: PlayerAction) -> Self {
        Self::Player(value)
    }
}

impl From<HomeAction> for Action {
    fn from(value: HomeAction) -> Self {
        Self::Home(value)
    }
}

impl From<SettingsAction> for Action {
    fn from(value: SettingsAction) -> Self {
        Self::Settings(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_de() -> Result<(), toml::de::Error> {
        let config = include_str!("../resources/docs/config.toml");

        toml::from_str::<Config>(config).map(|_| {})
    }

    #[test]
    fn config_se() -> Result<(), toml::ser::Error> {
        let config = Config::dev();

        toml::to_string_pretty(&config).map(|_| {})
    }
}
