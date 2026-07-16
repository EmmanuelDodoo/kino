use super::{Action, HomeAction, Layout, PlayerAction, Screen, SettingsAction};
use crate::theme::Theme;
use core::variants;
use devutils::source::SourceSet;
use iced::Color;
pub use keys::{KeyModifier, KeyPress, KeyStore};
use serde::{Deserialize, Serialize, de, ser};
use std::{path::PathBuf, time::Duration};
pub use subtitles::SubtitleDescription;

pub fn se_color<S: ser::Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    use ser::Serializer;
    let color = color.to_string();

    s.serialize_str(&color)
}

pub fn de_color<'de, D: de::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    use de::{Deserializer, Visitor};

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
            use de::Error;

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

mod keys {
    use super::*;
    use iced::keyboard;
    use serde::{de::VariantAccess, ser::SerializeStruct};
    use std::{
        collections::hash_map::{HashMap, Iter},
        hash::Hash,
    };

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum KeyModifier {
        #[default]
        Shift = 1,
        Control = 2,
        Alt = 3,
        ShiftControl = 4,
        ShiftAlt = 5,
        ControlAlt = 6,
    }

    impl KeyModifier {
        const NAME: &str = "modifiers";
        const SHIFT: &str = "Shift";
        const CONTROL: &str = "Ctrl";
        const ALT: &str = "Alt";
        const SHIFT_CONTROL: &str = "Shift + Ctrl";
        const SHIFT_ALT: &str = "Shift + Alt";
        const CONTROL_ALT: &str = "Ctrl + Alt";
        const VARIANTS: &[&str] = &[
            Self::SHIFT,
            Self::CONTROL,
            Self::ALT,
            Self::SHIFT_CONTROL,
            Self::SHIFT_ALT,
            Self::CONTROL_ALT,
        ];

        pub fn from_modifiers(value: keyboard::Modifiers) -> Option<Self> {
            if value.shift() && value.command() {
                Some(KeyModifier::ShiftControl)
            } else if value.shift() && value.alt() {
                Some(KeyModifier::ShiftAlt)
            } else if value.command() && value.alt() {
                Some(KeyModifier::ControlAlt)
            } else if value.shift() {
                Some(KeyModifier::Shift)
            } else if value.command() {
                Some(KeyModifier::Control)
            } else if value.alt() {
                Some(KeyModifier::Alt)
            } else {
                None
            }
        }
    }

    impl From<KeyModifier> for keyboard::Modifiers {
        fn from(value: KeyModifier) -> Self {
            use keyboard::Modifiers;

            match value {
                KeyModifier::Alt => Modifiers::ALT,
                KeyModifier::Shift => Modifiers::SHIFT,
                KeyModifier::Control => Modifiers::COMMAND,
                KeyModifier::ShiftControl => Modifiers::SHIFT & Modifiers::COMMAND,
                KeyModifier::ShiftAlt => Modifiers::SHIFT & Modifiers::ALT,
                KeyModifier::ControlAlt => Modifiers::COMMAND & Modifiers::ALT,
            }
        }
    }

    impl std::fmt::Display for KeyModifier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Control => Self::CONTROL,
                    Self::ControlAlt => Self::CONTROL_ALT,
                    Self::Shift => Self::SHIFT,
                    Self::ShiftAlt => Self::SHIFT_ALT,
                    Self::ShiftControl => Self::SHIFT_CONTROL,
                    Self::Alt => Self::ALT,
                }
            )
        }
    }

    impl Serialize for KeyModifier {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Shift => {
                    serializer.serialize_unit_variant(Self::NAME, *self as _, Self::SHIFT)
                }
                Self::Control => {
                    serializer.serialize_unit_variant(Self::NAME, *self as _, Self::CONTROL)
                }
                Self::Alt => serializer.serialize_unit_variant(Self::NAME, *self as _, Self::ALT),
                Self::ShiftControl => {
                    serializer.serialize_unit_variant(Self::NAME, *self as _, Self::SHIFT_CONTROL)
                }
                Self::ShiftAlt => {
                    serializer.serialize_unit_variant(Self::NAME, *self as _, Self::SHIFT_ALT)
                }
                Self::ControlAlt => {
                    serializer.serialize_unit_variant(Self::NAME, *self as _, Self::CONTROL_ALT)
                }
            }
        }
    }

    impl<'de> Deserialize<'de> for KeyModifier {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::{self, Visitor};

            struct ModifierVisitor;

            impl<'de> Visitor<'de> for ModifierVisitor {
                type Value = KeyModifier;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "{} variants", KeyModifier::NAME)
                }

                fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::EnumAccess<'de>,
                {
                    let (variant, access) = data.variant::<String>()?;

                    match variant.as_str() {
                        KeyModifier::SHIFT => {
                            access.unit_variant()?;
                            Ok(KeyModifier::Shift)
                        }
                        KeyModifier::CONTROL => {
                            access.unit_variant()?;
                            Ok(KeyModifier::Control)
                        }
                        KeyModifier::ALT => {
                            access.unit_variant()?;
                            Ok(KeyModifier::Alt)
                        }
                        KeyModifier::SHIFT_CONTROL => {
                            access.unit_variant()?;
                            Ok(KeyModifier::ShiftControl)
                        }
                        KeyModifier::SHIFT_ALT => {
                            access.unit_variant()?;
                            Ok(KeyModifier::ShiftAlt)
                        }
                        KeyModifier::CONTROL_ALT => {
                            access.unit_variant()?;
                            Ok(KeyModifier::ControlAlt)
                        }
                        _ => Err(de::Error::unknown_variant(&variant, KeyModifier::VARIANTS)),
                    }
                }
            }

            deserializer.deserialize_enum(Self::NAME, Self::VARIANTS, ModifierVisitor)
        }
    }

    fn serialize_named<S: serde::Serializer>(
        serializer: S,
        named: keyboard::key::Named,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{named:?}"))
    }

    fn _deserialize_named<'de, D>(deserializer: D) -> Result<keyboard::key::Named, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_named(&s).map_err(serde::de::Error::custom)
    }

    fn _deserialize_unidentified<'de, D>(deserializer: D) -> Result<keyboard::Key, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "Unidentified" => Ok(keyboard::Key::Unidentified),
            _ => Err(serde::de::Error::custom("unknown key")),
        }
    }

    fn parse_named(s: &str) -> Result<keyboard::key::Named, String> {
        use keyboard::key::Named;

        // todo: Not my best, I admit
        match s {
            "Alt" => Ok(Named::Alt),
            "AltGraph" => Ok(Named::AltGraph),
            "CapsLock" => Ok(Named::CapsLock),
            "Control" => Ok(Named::Control),
            "Fn" => Ok(Named::Fn),
            "FnLock" => Ok(Named::FnLock),
            "NumLock" => Ok(Named::NumLock),
            "ScrollLock" => Ok(Named::ScrollLock),
            "Shift" => Ok(Named::Shift),
            "Symbol" => Ok(Named::Symbol),
            "SymbolLock" => Ok(Named::SymbolLock),
            "Meta" => Ok(Named::Meta),
            "Hyper" => Ok(Named::Hyper),
            "Super" => Ok(Named::Super),
            "Enter" => Ok(Named::Enter),
            "Tab" => Ok(Named::Tab),
            "Space" => Ok(Named::Space),
            "ArrowDown" => Ok(Named::ArrowDown),
            "ArrowLeft" => Ok(Named::ArrowLeft),
            "ArrowRight" => Ok(Named::ArrowRight),
            "ArrowUp" => Ok(Named::ArrowUp),
            "End" => Ok(Named::End),
            "Home" => Ok(Named::Home),
            "PageDown" => Ok(Named::PageDown),
            "PageUp" => Ok(Named::PageUp),
            "Backspace" => Ok(Named::Backspace),
            "Clear" => Ok(Named::Clear),
            "Copy" => Ok(Named::Copy),
            "CrSel" => Ok(Named::CrSel),
            "Cut" => Ok(Named::Cut),
            "Delete" => Ok(Named::Delete),
            "EraseEof" => Ok(Named::EraseEof),
            "ExSel" => Ok(Named::ExSel),
            "Insert" => Ok(Named::Insert),
            "Paste" => Ok(Named::Paste),
            "Redo" => Ok(Named::Redo),
            "Undo" => Ok(Named::Undo),
            "Accept" => Ok(Named::Accept),
            "Again" => Ok(Named::Again),
            "Attn" => Ok(Named::Attn),
            "Cancel" => Ok(Named::Cancel),
            "ContextMenu" => Ok(Named::ContextMenu),
            "Escape" => Ok(Named::Escape),
            "Execute" => Ok(Named::Execute),
            "Find" => Ok(Named::Find),
            "Help" => Ok(Named::Help),
            "Pause" => Ok(Named::Pause),
            "Play" => Ok(Named::Play),
            "Props" => Ok(Named::Props),
            "Select" => Ok(Named::Select),
            "ZoomIn" => Ok(Named::ZoomIn),
            "ZoomOut" => Ok(Named::ZoomOut),
            "BrightnessDown" => Ok(Named::BrightnessDown),
            "BrightnessUp" => Ok(Named::BrightnessUp),
            "Eject" => Ok(Named::Eject),
            "LogOff" => Ok(Named::LogOff),
            "Power" => Ok(Named::Power),
            "PowerOff" => Ok(Named::PowerOff),
            "PrintScreen" => Ok(Named::PrintScreen),
            "Hibernate" => Ok(Named::Hibernate),
            "Standby" => Ok(Named::Standby),
            "WakeUp" => Ok(Named::WakeUp),
            "AllCandidates" => Ok(Named::AllCandidates),
            "Alphanumeric" => Ok(Named::Alphanumeric),
            "CodeInput" => Ok(Named::CodeInput),
            "Compose" => Ok(Named::Compose),
            "Convert" => Ok(Named::Convert),
            "FinalMode" => Ok(Named::FinalMode),
            "GroupFirst" => Ok(Named::GroupFirst),
            "GroupLast" => Ok(Named::GroupLast),
            "GroupNext" => Ok(Named::GroupNext),
            "GroupPrevious" => Ok(Named::GroupPrevious),
            "ModeChange" => Ok(Named::ModeChange),
            "NextCandidate" => Ok(Named::NextCandidate),
            "NonConvert" => Ok(Named::NonConvert),
            "PreviousCandidate" => Ok(Named::PreviousCandidate),
            "Process" => Ok(Named::Process),
            "SingleCandidate" => Ok(Named::SingleCandidate),
            "HangulMode" => Ok(Named::HangulMode),
            "HanjaMode" => Ok(Named::HanjaMode),
            "JunjaMode" => Ok(Named::JunjaMode),
            "Eisu" => Ok(Named::Eisu),
            "Hankaku" => Ok(Named::Hankaku),
            "Hiragana" => Ok(Named::Hiragana),
            "HiraganaKatakana" => Ok(Named::HiraganaKatakana),
            "KanaMode" => Ok(Named::KanaMode),
            "KanjiMode" => Ok(Named::KanjiMode),
            "Katakana" => Ok(Named::Katakana),
            "Romaji" => Ok(Named::Romaji),
            "Zenkaku" => Ok(Named::Zenkaku),
            "ZenkakuHankaku" => Ok(Named::ZenkakuHankaku),
            "Soft1" => Ok(Named::Soft1),
            "Soft2" => Ok(Named::Soft2),
            "Soft3" => Ok(Named::Soft3),
            "Soft4" => Ok(Named::Soft4),
            "ChannelDown" => Ok(Named::ChannelDown),
            "ChannelUp" => Ok(Named::ChannelUp),
            "Close" => Ok(Named::Close),
            "MailForward" => Ok(Named::MailForward),
            "MailReply" => Ok(Named::MailReply),
            "MailSend" => Ok(Named::MailSend),
            "MediaClose" => Ok(Named::MediaClose),
            "MediaFastForward" => Ok(Named::MediaFastForward),
            "MediaPause" => Ok(Named::MediaPause),
            "MediaPlay" => Ok(Named::MediaPlay),
            "MediaPlayPause" => Ok(Named::MediaPlayPause),
            "MediaRecord" => Ok(Named::MediaRecord),
            "MediaRewind" => Ok(Named::MediaRewind),
            "MediaStop" => Ok(Named::MediaStop),
            "MediaTrackNext" => Ok(Named::MediaTrackNext),
            "MediaTrackPrevious" => Ok(Named::MediaTrackPrevious),
            "New" => Ok(Named::New),
            "Open" => Ok(Named::Open),
            "Print" => Ok(Named::Print),
            "Save" => Ok(Named::Save),
            "SpellCheck" => Ok(Named::SpellCheck),
            "Key11" => Ok(Named::Key11),
            "Key12" => Ok(Named::Key12),
            "AudioBalanceLeft" => Ok(Named::AudioBalanceLeft),
            "AudioBalanceRight" => Ok(Named::AudioBalanceRight),
            "AudioBassBoostDown" => Ok(Named::AudioBassBoostDown),
            "AudioBassBoostToggle" => Ok(Named::AudioBassBoostToggle),
            "AudioBassBoostUp" => Ok(Named::AudioBassBoostUp),
            "AudioFaderFront" => Ok(Named::AudioFaderFront),
            "AudioFaderRear" => Ok(Named::AudioFaderRear),
            "AudioSurroundModeNext" => Ok(Named::AudioSurroundModeNext),
            "AudioTrebleDown" => Ok(Named::AudioTrebleDown),
            "AudioTrebleUp" => Ok(Named::AudioTrebleUp),
            "AudioVolumeDown" => Ok(Named::AudioVolumeDown),
            "AudioVolumeUp" => Ok(Named::AudioVolumeUp),
            "AudioVolumeMute" => Ok(Named::AudioVolumeMute),
            "MicrophoneToggle" => Ok(Named::MicrophoneToggle),
            "MicrophoneVolumeDown" => Ok(Named::MicrophoneVolumeDown),
            "MicrophoneVolumeUp" => Ok(Named::MicrophoneVolumeUp),
            "MicrophoneVolumeMute" => Ok(Named::MicrophoneVolumeMute),
            "SpeechCorrectionList" => Ok(Named::SpeechCorrectionList),
            "SpeechInputToggle" => Ok(Named::SpeechInputToggle),
            "LaunchApplication1" => Ok(Named::LaunchApplication1),
            "LaunchApplication2" => Ok(Named::LaunchApplication2),
            "LaunchCalendar" => Ok(Named::LaunchCalendar),
            "LaunchContacts" => Ok(Named::LaunchContacts),
            "LaunchMail" => Ok(Named::LaunchMail),
            "LaunchMediaPlayer" => Ok(Named::LaunchMediaPlayer),
            "LaunchMusicPlayer" => Ok(Named::LaunchMusicPlayer),
            "LaunchPhone" => Ok(Named::LaunchPhone),
            "LaunchScreenSaver" => Ok(Named::LaunchScreenSaver),
            "LaunchSpreadsheet" => Ok(Named::LaunchSpreadsheet),
            "LaunchWebBrowser" => Ok(Named::LaunchWebBrowser),
            "LaunchWebCam" => Ok(Named::LaunchWebCam),
            "LaunchWordProcessor" => Ok(Named::LaunchWordProcessor),
            "BrowserBack" => Ok(Named::BrowserBack),
            "BrowserFavorites" => Ok(Named::BrowserFavorites),
            "BrowserForward" => Ok(Named::BrowserForward),
            "BrowserHome" => Ok(Named::BrowserHome),
            "BrowserRefresh" => Ok(Named::BrowserRefresh),
            "BrowserSearch" => Ok(Named::BrowserSearch),
            "BrowserStop" => Ok(Named::BrowserStop),
            "AppSwitch" => Ok(Named::AppSwitch),
            "Call" => Ok(Named::Call),
            "Camera" => Ok(Named::Camera),
            "CameraFocus" => Ok(Named::CameraFocus),
            "EndCall" => Ok(Named::EndCall),
            "GoBack" => Ok(Named::GoBack),
            "GoHome" => Ok(Named::GoHome),
            "HeadsetHook" => Ok(Named::HeadsetHook),
            "LastNumberRedial" => Ok(Named::LastNumberRedial),
            "Notification" => Ok(Named::Notification),
            "MannerMode" => Ok(Named::MannerMode),
            "VoiceDial" => Ok(Named::VoiceDial),
            "TV" => Ok(Named::TV),
            "TV3DMode" => Ok(Named::TV3DMode),
            "TVAntennaCable" => Ok(Named::TVAntennaCable),
            "TVAudioDescription" => Ok(Named::TVAudioDescription),
            "TVAudioDescriptionMixDown" => Ok(Named::TVAudioDescriptionMixDown),
            "TVAudioDescriptionMixUp" => Ok(Named::TVAudioDescriptionMixUp),
            "TVContentsMenu" => Ok(Named::TVContentsMenu),
            "TVDataService" => Ok(Named::TVDataService),
            "TVInput" => Ok(Named::TVInput),
            "TVInputComponent1" => Ok(Named::TVInputComponent1),
            "TVInputComponent2" => Ok(Named::TVInputComponent2),
            "TVInputComposite1" => Ok(Named::TVInputComposite1),
            "TVInputComposite2" => Ok(Named::TVInputComposite2),
            "TVInputHDMI1" => Ok(Named::TVInputHDMI1),
            "TVInputHDMI2" => Ok(Named::TVInputHDMI2),
            "TVInputHDMI3" => Ok(Named::TVInputHDMI3),
            "TVInputHDMI4" => Ok(Named::TVInputHDMI4),
            "TVInputVGA1" => Ok(Named::TVInputVGA1),
            "TVMediaContext" => Ok(Named::TVMediaContext),
            "TVNetwork" => Ok(Named::TVNetwork),
            "TVNumberEntry" => Ok(Named::TVNumberEntry),
            "TVPower" => Ok(Named::TVPower),
            "TVRadioService" => Ok(Named::TVRadioService),
            "TVSatellite" => Ok(Named::TVSatellite),
            "TVSatelliteBS" => Ok(Named::TVSatelliteBS),
            "TVSatelliteCS" => Ok(Named::TVSatelliteCS),
            "TVSatelliteToggle" => Ok(Named::TVSatelliteToggle),
            "TVTerrestrialAnalog" => Ok(Named::TVTerrestrialAnalog),
            "TVTerrestrialDigital" => Ok(Named::TVTerrestrialDigital),
            "TVTimer" => Ok(Named::TVTimer),
            "AVRInput" => Ok(Named::AVRInput),
            "AVRPower" => Ok(Named::AVRPower),
            "ColorF0Red" => Ok(Named::ColorF0Red),
            "ColorF1Green" => Ok(Named::ColorF1Green),
            "ColorF2Yellow" => Ok(Named::ColorF2Yellow),
            "ColorF3Blue" => Ok(Named::ColorF3Blue),
            "ColorF4Grey" => Ok(Named::ColorF4Grey),
            "ColorF5Brown" => Ok(Named::ColorF5Brown),
            "ClosedCaptionToggle" => Ok(Named::ClosedCaptionToggle),
            "Dimmer" => Ok(Named::Dimmer),
            "DisplaySwap" => Ok(Named::DisplaySwap),
            "DVR" => Ok(Named::DVR),
            "Exit" => Ok(Named::Exit),
            "FavoriteClear0" => Ok(Named::FavoriteClear0),
            "FavoriteClear1" => Ok(Named::FavoriteClear1),
            "FavoriteClear2" => Ok(Named::FavoriteClear2),
            "FavoriteClear3" => Ok(Named::FavoriteClear3),
            "FavoriteRecall0" => Ok(Named::FavoriteRecall0),
            "FavoriteRecall1" => Ok(Named::FavoriteRecall1),
            "FavoriteRecall2" => Ok(Named::FavoriteRecall2),
            "FavoriteRecall3" => Ok(Named::FavoriteRecall3),
            "FavoriteStore0" => Ok(Named::FavoriteStore0),
            "FavoriteStore1" => Ok(Named::FavoriteStore1),
            "FavoriteStore2" => Ok(Named::FavoriteStore2),
            "FavoriteStore3" => Ok(Named::FavoriteStore3),
            "Guide" => Ok(Named::Guide),
            "GuideNextDay" => Ok(Named::GuideNextDay),
            "GuidePreviousDay" => Ok(Named::GuidePreviousDay),
            "Info" => Ok(Named::Info),
            "InstantReplay" => Ok(Named::InstantReplay),
            "Link" => Ok(Named::Link),
            "ListProgram" => Ok(Named::ListProgram),
            "LiveContent" => Ok(Named::LiveContent),
            "Lock" => Ok(Named::Lock),
            "MediaApps" => Ok(Named::MediaApps),
            "MediaAudioTrack" => Ok(Named::MediaAudioTrack),
            "MediaLast" => Ok(Named::MediaLast),
            "MediaSkipBackward" => Ok(Named::MediaSkipBackward),
            "MediaSkipForward" => Ok(Named::MediaSkipForward),
            "MediaStepBackward" => Ok(Named::MediaStepBackward),
            "MediaStepForward" => Ok(Named::MediaStepForward),
            "MediaTopMenu" => Ok(Named::MediaTopMenu),
            "NavigateIn" => Ok(Named::NavigateIn),
            "NavigateNext" => Ok(Named::NavigateNext),
            "NavigateOut" => Ok(Named::NavigateOut),
            "NavigatePrevious" => Ok(Named::NavigatePrevious),
            "NextFavoriteChannel" => Ok(Named::NextFavoriteChannel),
            "NextUserProfile" => Ok(Named::NextUserProfile),
            "OnDemand" => Ok(Named::OnDemand),
            "Pairing" => Ok(Named::Pairing),
            "PinPDown" => Ok(Named::PinPDown),
            "PinPMove" => Ok(Named::PinPMove),
            "PinPToggle" => Ok(Named::PinPToggle),
            "PinPUp" => Ok(Named::PinPUp),
            "PlaySpeedDown" => Ok(Named::PlaySpeedDown),
            "PlaySpeedReset" => Ok(Named::PlaySpeedReset),
            "PlaySpeedUp" => Ok(Named::PlaySpeedUp),
            "RandomToggle" => Ok(Named::RandomToggle),
            "RcLowBattery" => Ok(Named::RcLowBattery),
            "RecordSpeedNext" => Ok(Named::RecordSpeedNext),
            "RfBypass" => Ok(Named::RfBypass),
            "ScanChannelsToggle" => Ok(Named::ScanChannelsToggle),
            "ScreenModeNext" => Ok(Named::ScreenModeNext),
            "Settings" => Ok(Named::Settings),
            "SplitScreenToggle" => Ok(Named::SplitScreenToggle),
            "STBInput" => Ok(Named::STBInput),
            "STBPower" => Ok(Named::STBPower),
            "Subtitle" => Ok(Named::Subtitle),
            "Teletext" => Ok(Named::Teletext),
            "VideoModeNext" => Ok(Named::VideoModeNext),
            "Wink" => Ok(Named::Wink),
            "ZoomToggle" => Ok(Named::ZoomToggle),
            "F1" => Ok(Named::F1),
            "F2" => Ok(Named::F2),
            "F3" => Ok(Named::F3),
            "F4" => Ok(Named::F4),
            "F5" => Ok(Named::F5),
            "F6" => Ok(Named::F6),
            "F7" => Ok(Named::F7),
            "F8" => Ok(Named::F8),
            "F9" => Ok(Named::F9),
            "F10" => Ok(Named::F10),
            "F11" => Ok(Named::F11),
            "F12" => Ok(Named::F12),
            "F13" => Ok(Named::F13),
            "F14" => Ok(Named::F14),
            "F15" => Ok(Named::F15),
            "F16" => Ok(Named::F16),
            "F17" => Ok(Named::F17),
            "F18" => Ok(Named::F18),
            "F19" => Ok(Named::F19),
            "F20" => Ok(Named::F20),
            "F21" => Ok(Named::F21),
            "F22" => Ok(Named::F22),
            "F23" => Ok(Named::F23),
            "F24" => Ok(Named::F24),
            "F25" => Ok(Named::F25),
            "F26" => Ok(Named::F26),
            "F27" => Ok(Named::F27),
            "F28" => Ok(Named::F28),
            "F29" => Ok(Named::F29),
            "F30" => Ok(Named::F30),
            "F31" => Ok(Named::F31),
            "F32" => Ok(Named::F32),
            "F33" => Ok(Named::F33),
            "F34" => Ok(Named::F34),
            "F35" => Ok(Named::F35),
            _ => Err("unknown key".to_owned()),
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct KeyPress {
        pub key: keyboard::Key,
        pub modifiers: Option<KeyModifier>,
    }

    impl std::fmt::Display for KeyPress {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use keyboard::key::Key;

            let modifiers = match self.modifiers {
                Some(modifiers) => {
                    format!("{} + ", modifiers)
                }
                None => String::default(),
            };

            write!(
                f,
                "{modifiers}{}",
                match &self.key {
                    Key::Named(named) => format!("{named:?}"),
                    Key::Character(character) => format!("{character}"),
                    Key::Unidentified => "Unidentified".to_owned(),
                }
            )
        }
    }

    impl KeyPress {
        const NAME: &str = "keybinding";
        const KEY: &str = "key";
        const MODIFIERS: &str = KeyModifier::NAME;
        const FIELDS: &[&str] = &[Self::KEY, Self::MODIFIERS];

        pub fn new(key: keyboard::Key, modifiers: Option<KeyModifier>) -> Self {
            Self { key, modifiers }
        }

        pub fn with_modifiers(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Self {
            Self::new(key, KeyModifier::from_modifiers(modifiers))
        }
    }

    impl Serialize for KeyPress {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::Serializer;

            struct KeyRef<'a>(&'a keyboard::Key);

            impl<'a> Serialize for KeyRef<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    use keyboard::key::Key;

                    match self.0 {
                        Key::Unidentified => serializer.serialize_str("Unidentified"),
                        Key::Named(named) => serialize_named(serializer, *named),
                        Key::Character(c) => c.serialize(serializer),
                    }
                }
            }

            let mut strt = serializer.serialize_struct(Self::NAME, Self::FIELDS.len())?;

            let key = KeyRef(&self.key);
            strt.serialize_field(Self::KEY, &key)?;

            strt.serialize_field(Self::MODIFIERS, &self.modifiers)?;

            strt.end()
        }
    }

    impl<'de> Deserialize<'de> for KeyPress {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::{self, Visitor};

            struct KeyOwned(keyboard::Key);

            impl<'de> Deserialize<'de> for KeyOwned {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct KeyVisitor;

                    impl<'de> Visitor<'de> for KeyVisitor {
                        type Value = KeyOwned;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            formatter.write_str(KeyPress::KEY)
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            if v == "Unidentified" {
                                return Ok(KeyOwned(keyboard::Key::Unidentified));
                            }

                            if let Ok(named) = parse_named(v) {
                                return Ok(KeyOwned(keyboard::Key::Named(named)));
                            }

                            // todo: Ideally, I'd know any invariant Key::Character has so I can
                            // check for that
                            let c = smol_str::SmolStr::from(v);

                            Ok(KeyOwned(keyboard::Key::Character(c)))
                        }
                    }

                    deserializer.deserialize_str(KeyVisitor)
                }
            }

            enum Fields {
                Key,
                Modifiers,
            }

            impl<'de> Deserialize<'de> for Fields {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> Visitor<'de> for FieldVisitor {
                        type Value = Fields;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(
                                formatter,
                                "{} and/or {}",
                                KeyPress::KEY,
                                KeyPress::MODIFIERS
                            )
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            match v {
                                KeyPress::KEY => Ok(Fields::Key),
                                KeyPress::MODIFIERS => Ok(Fields::Modifiers),
                                _ => Err(de::Error::unknown_field(v, KeyPress::FIELDS)),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct KeypressVisitor;

            impl<'de> Visitor<'de> for KeypressVisitor {
                type Value = KeyPress;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "struct {}", KeyPress::NAME)
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let key: KeyOwned = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    let modifiers = seq.next_element()?;

                    Ok(KeyPress {
                        key: key.0,
                        modifiers,
                    })
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut key = None;
                    let mut modifiers = None;

                    while let Some(val) = map.next_key()? {
                        match val {
                            Fields::Key => {
                                if key.is_some() {
                                    return Err(de::Error::duplicate_field(KeyPress::KEY));
                                }

                                key = Some(map.next_value()?);
                            }
                            Fields::Modifiers => {
                                if modifiers.is_some() {
                                    return Err(de::Error::duplicate_field(KeyPress::MODIFIERS));
                                }

                                modifiers = Some(map.next_value()?);
                            }
                        }
                    }

                    let key: KeyOwned =
                        key.ok_or_else(|| de::Error::missing_field(KeyPress::KEY))?;

                    Ok(KeyPress {
                        key: key.0,
                        modifiers,
                    })
                }
            }

            deserializer.deserialize_struct(Self::NAME, Self::FIELDS, KeypressVisitor)
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(from = "DeInner<A>")]
    struct KeyStoreInner<A>
    where
        A: Hash + Eq + Copy,
    {
        keys: HashMap<KeyPress, A>,
        actions: HashMap<A, Vec<KeyPress>>,
        defaults: bool,
    }

    impl<A> KeyStoreInner<A>
    where
        A: Hash + Eq + Copy,
    {
        const NAME: &str = "bindings";
        const ACTIONS: &str = "actions";
        const DEFAULTS: &str = "defaults";
        const FIELDS: &[&str] = &[Self::ACTIONS, Self::DEFAULTS];

        fn new() -> Self {
            Self {
                keys: HashMap::default(),
                actions: HashMap::default(),
                defaults: false,
            }
        }

        fn defaults(actions: impl IntoIterator<Item = (KeyPress, A)>) -> Self {
            let mut new: KeyStoreInner<_> = actions.into_iter().collect();
            new.defaults = true;

            new
        }

        fn get(&self, key: &KeyPress) -> Option<&A> {
            self.keys.get(key)
        }

        fn get_action(&self, action: A) -> Option<&Vec<KeyPress>> {
            self.actions.get(&action)
        }

        fn iter_action(&self) -> Iter<'_, A, Vec<KeyPress>> {
            self.actions.iter()
        }

        fn iter_keys(&self) -> Iter<'_, KeyPress, A> {
            self.keys.iter()
        }

        fn insert(&mut self, key: KeyPress, action: A) {
            if let Some(previous) = self.keys.insert(key.clone(), action)
                && let Some(keys) = self.actions.get_mut(&previous)
            {
                keys.retain(|curr| key != *curr);
            }

            let keys = self.actions.entry(action).or_default();
            keys.push(key);

            self.defaults = false;
        }

        fn extend(&mut self, iter: impl IntoIterator<Item = (KeyPress, A)>) {
            for (key, action) in iter {
                self.insert(key, action);
            }
        }

        fn remove(&mut self, key: KeyPress) {
            if let Some(action) = self.keys.remove(&key)
                && let Some(keys) = self.actions.get_mut(&action)
            {
                keys.retain(|curr| key != *curr);
                self.defaults = false;
            }
        }

        fn clear(&mut self, action: A) {
            if let Some(keys) = self.actions.remove(&action) {
                for key in keys {
                    self.remove(key)
                }
            };
        }
    }

    impl<A> FromIterator<(KeyPress, A)> for KeyStoreInner<A>
    where
        A: Eq + Hash + Copy,
    {
        fn from_iter<T: IntoIterator<Item = (KeyPress, A)>>(iter: T) -> Self {
            let mut new = KeyStoreInner::new();

            new.extend(iter);

            new
        }
    }

    impl<A> Serialize for KeyStoreInner<A>
    where
        A: Hash + Eq + Copy + Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut strt = serializer.serialize_struct(Self::NAME, 2)?;
            strt.serialize_field(Self::ACTIONS, &self.actions)?;
            strt.serialize_field(Self::DEFAULTS, &(!self.defaults).then_some(false))?;
            strt.end()
        }
    }

    struct DeInner<A>
    where
        A: Eq + Hash + Copy,
    {
        actions: Option<HashMap<A, Vec<KeyPress>>>,
        defaults: Option<bool>,
    }

    impl<A> From<DeInner<A>> for KeyStoreInner<A>
    where
        A: Eq + Hash + Copy,
    {
        fn from(value: DeInner<A>) -> Self {
            let DeInner { actions, defaults } = value;

            let defaults = defaults.unwrap_or(true);

            let Some(actions) = actions else {
                if defaults {
                    return KeyStoreInner::defaults(std::iter::empty());
                } else {
                    return KeyStoreInner::new();
                }
            };

            let actions = actions
                .into_iter()
                .flat_map(|(action, keys)| keys.into_iter().map(move |key| (key, action)));

            if defaults {
                KeyStoreInner::defaults(actions)
            } else {
                actions.collect()
            }
        }
    }

    impl<'de, A: Deserialize<'de>> Deserialize<'de> for DeInner<A>
    where
        A: Eq + Hash + Copy + Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::{self, Visitor};

            #[derive(Deserialize)]
            #[serde(field_identifier, rename_all = "lowercase")]
            enum Field {
                Actions,
                Defaults,
            }

            struct InnerVisitor<A>
            where
                A: Eq + Hash + Copy,
            {
                _phantom: std::marker::PhantomData<A>,
            }

            impl<'de, Act> Visitor<'de> for InnerVisitor<Act>
            where
                Act: Eq + Hash + Copy + Deserialize<'de>,
            {
                type Value = DeInner<Act>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(KeyStoreInner::<Act>::NAME)
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let actions = seq.next_element()?;
                    let defaults = seq.next_element()?;

                    Ok(DeInner { actions, defaults })
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut actions = None;
                    let mut defaults = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Actions => {
                                if actions.is_some() {
                                    return Err(de::Error::duplicate_field(
                                        KeyStoreInner::<Act>::ACTIONS,
                                    ));
                                }

                                actions = Some(map.next_value()?);
                            }
                            Field::Defaults => {
                                if defaults.is_some() {
                                    return Err(de::Error::duplicate_field(
                                        KeyStoreInner::<Act>::DEFAULTS,
                                    ));
                                }

                                defaults = Some(map.next_value()?);
                            }
                        }
                    }

                    Ok(DeInner { actions, defaults })
                }
            }

            deserializer.deserialize_struct(
                KeyStoreInner::<A>::NAME,
                KeyStoreInner::<A>::FIELDS,
                InnerVisitor {
                    _phantom: std::marker::PhantomData,
                },
            )
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(default = "KeyStore::defaults")]
    pub struct KeyStore {
        #[serde(rename = "general")]
        home: KeyStoreInner<HomeAction>,
        player: KeyStoreInner<PlayerAction>,
        settings: KeyStoreInner<SettingsAction>,
    }

    impl KeyStore {
        fn new() -> Self {
            Self {
                home: KeyStoreInner::new(),
                player: KeyStoreInner::new(),
                settings: KeyStoreInner::new(),
            }
        }

        pub(super) fn defaults() -> Self {
            Self {
                home: KeyStoreInner::defaults(home()),
                player: KeyStoreInner::defaults(player()),
                settings: KeyStoreInner::defaults(settings()),
            }
        }

        pub fn action(&self, keypress: KeyPress, screen: Screen) -> Option<Action> {
            match screen {
                Screen::Home => self.home.get(&keypress).copied().map(Action::Home),
                Screen::Player => self.player.get(&keypress).copied().map(Action::Player),
                Screen::Settings => self.settings.get(&keypress).copied().map(Action::Settings),
            }
        }

        pub fn home(&self) -> Iter<'_, HomeAction, Vec<KeyPress>> {
            self.home.iter_action()
        }

        pub fn player(&self) -> Iter<'_, PlayerAction, Vec<KeyPress>> {
            self.player.iter_action()
        }

        pub fn settings(&self) -> Iter<'_, SettingsAction, Vec<KeyPress>> {
            self.settings.iter_action()
        }

        pub fn get_home(&self, key: &KeyPress) -> Option<&HomeAction> {
            self.home.get(key)
        }

        pub fn get_player(&self, key: &KeyPress) -> Option<&PlayerAction> {
            self.player.get(key)
        }

        pub fn get_settings(&self, key: &KeyPress) -> Option<&SettingsAction> {
            self.settings.get(key)
        }

        pub fn insert_home(&mut self, key: KeyPress, action: HomeAction) {
            self.home.insert(key, action);
        }

        pub fn insert_player(&mut self, key: KeyPress, action: PlayerAction) {
            self.player.insert(key, action);
        }

        pub fn insert_settings(&mut self, key: KeyPress, action: SettingsAction) {
            self.settings.insert(key, action);
        }

        pub fn remove_home(&mut self, key: KeyPress) {
            self.home.remove(key);
        }

        pub fn remove_player(&mut self, key: KeyPress) {
            self.player.remove(key);
        }

        pub fn remove_settings(&mut self, key: KeyPress) {
            self.settings.remove(key);
        }

        pub fn clear_home(&mut self, action: HomeAction) {
            self.home.clear(action);
        }

        pub fn clear_player(&mut self, action: PlayerAction) {
            self.player.clear(action);
        }

        pub fn clear_settings(&mut self, action: SettingsAction) {
            self.settings.clear(action);
        }
    }

    fn home() -> impl Iterator<Item = (KeyPress, HomeAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (key(Key::Named(Named::F5), None), HomeAction::RefreshContent),
            (
                key(Key::Named(Named::F5), Some(KeyModifier::Shift)),
                HomeAction::Refresh,
            ),
            (
                key(Key::Named(Named::ArrowLeft), Some(KeyModifier::Alt)),
                HomeAction::Back,
            ),
            (
                key(Key::Named(Named::ArrowRight), Some(KeyModifier::Alt)),
                HomeAction::Forward,
            ),
            (
                key(Key::Named(Named::BrowserForward), None),
                HomeAction::Forward,
            ),
            (key(Key::Named(Named::BrowserBack), None), HomeAction::Back),
            (
                key(Key::Character("l".into()), None),
                HomeAction::LayoutToggle,
            ),
            (
                key(Key::Character("r".into()), Some(KeyModifier::Shift)),
                HomeAction::Refresh,
            ),
            (
                key(Key::Character("r".into()), None),
                HomeAction::RefreshContent,
            ),
            (
                key(Key::Character("s".into()), Some(KeyModifier::Control)),
                HomeAction::SettingsOpen,
            ),
            (
                key(Key::Character("/".into()), None),
                HomeAction::SearchToggle,
            ),
            (
                key(Key::Character("f".into()), Some(KeyModifier::Control)),
                HomeAction::SearchToggle,
            ),
            (key(Key::Named(Named::Escape), None), HomeAction::CloseModal),
            (
                key(Key::Character("p".into()), None),
                HomeAction::SelectionStart,
            ),
            (
                key(Key::Character("w".into()), Some(KeyModifier::Shift)),
                HomeAction::WishNew,
            ),
        ]
        .into_iter()
    }

    fn player() -> impl Iterator<Item = (KeyPress, PlayerAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (
                key(Key::Named(Named::ArrowLeft), Some(KeyModifier::Alt)),
                PlayerAction::Back,
            ),
            (
                key(Key::Named(Named::BrowserBack), None),
                PlayerAction::Back,
            ),
            (
                key(Key::Named(Named::Space), None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::MediaPlayPause), None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::ArrowLeft), Some(KeyModifier::Control)),
                PlayerAction::PlayPrevious,
            ),
            (
                key(Key::Named(Named::MediaTrackPrevious), None),
                PlayerAction::PlayPrevious,
            ),
            (
                key(Key::Named(Named::ArrowRight), Some(KeyModifier::Control)),
                PlayerAction::PlayNext,
            ),
            (
                key(Key::Named(Named::MediaTrackNext), None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::Enter), None),
                PlayerAction::FullscreenToggle,
            ),
            (key(Key::Named(Named::Escape), None), PlayerAction::Exit),
            (
                key(Key::Character("f".into()), None),
                PlayerAction::FullscreenToggle,
            ),
            (
                key(Key::Named(Named::ArrowLeft), Some(KeyModifier::Shift)),
                PlayerAction::SeekBackShift,
            ),
            (
                key(Key::Named(Named::ArrowLeft), None),
                PlayerAction::SeekBack,
            ),
            (
                key(Key::Named(Named::ArrowRight), Some(KeyModifier::Shift)),
                PlayerAction::SeekFrontShift,
            ),
            (
                key(Key::Named(Named::ArrowRight), None),
                PlayerAction::SeekFront,
            ),
            (
                key(Key::Named(Named::ArrowUp), None),
                PlayerAction::VolumeIncrease,
            ),
            (
                key(Key::Named(Named::ArrowDown), None),
                PlayerAction::VolumeDecrease,
            ),
            (
                key(Key::Character("m".into()), None),
                PlayerAction::MuteToggle,
            ),
            (
                key(Key::Character("c".into()), None),
                PlayerAction::SpeedIncrease,
            ),
            (
                key(Key::Named(Named::PlaySpeedUp), None),
                PlayerAction::SpeedIncrease,
            ),
            (
                key(Key::Character("x".into()), None),
                PlayerAction::SpeedDecrease,
            ),
            (
                key(Key::Named(Named::PlaySpeedDown), None),
                PlayerAction::SpeedDecrease,
            ),
            (
                key(Key::Character("z".into()), None),
                PlayerAction::SpeedReset,
            ),
            (
                key(Key::Named(Named::PlaySpeedReset), None),
                PlayerAction::SpeedReset,
            ),
            (
                key(Key::Character("s".into()), Some(KeyModifier::Control)),
                PlayerAction::VideoConfig,
            ),
            (
                key(Key::Character("s".into()), None),
                PlayerAction::SubtitlesToggle,
            ),
            (
                key(Key::Named(Named::Subtitle), None),
                PlayerAction::SubtitlesToggle,
            ),
            (
                key(Key::Character("b".into()), Some(KeyModifier::Shift)),
                PlayerAction::VideoCommentNew,
            ),
            (
                key(Key::Character("b".into()), None),
                PlayerAction::VideoComment,
            ),
            (
                key(Key::Character("p".into()), None),
                PlayerAction::PlaylistToggle,
            ),
        ]
        .into_iter()
    }

    fn settings() -> impl Iterator<Item = (KeyPress, SettingsAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (key(Key::Named(Named::Escape), None), SettingsAction::Cancel),
            (key(Key::Named(Named::ArrowUp), None), SettingsAction::Up),
            (
                key(Key::Named(Named::ArrowDown), None),
                SettingsAction::Down,
            ),
        ]
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_de() -> Result<(), toml::de::Error> {
        let config = include_str!("../../resources/docs/config.toml");

        toml::from_str::<Config>(config).map(|_| {})
    }

    #[test]
    fn config_se() -> Result<(), toml::ser::Error> {
        let config = Config::dev();

        toml::to_string_pretty(&config).map(|_| {})
    }
}
