use super::{Action, HomeAction, Layout, PlayerAction, Screen, SettingsAction};
pub use keys::{KeyModifier, KeyPress, KeyStore};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AppTheme {
    Light,
    Dark,
    SolarizedLight,
    Nord,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    KanagawaWave,
    #[default]
    TokyoNight,
    TokyoNightLight,
    Moonfly,
}

impl AppTheme {
    pub const ALL: [Self; 11] = [
        AppTheme::Light,
        AppTheme::Dark,
        AppTheme::SolarizedLight,
        AppTheme::Nord,
        AppTheme::GruvboxLight,
        AppTheme::GruvboxDark,
        AppTheme::CatppuccinLatte,
        AppTheme::KanagawaWave,
        AppTheme::TokyoNight,
        AppTheme::TokyoNightLight,
        AppTheme::Moonfly,
    ];
}

impl From<AppTheme> for iced::Theme {
    fn from(value: AppTheme) -> Self {
        use iced::Theme;

        match value {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
            AppTheme::SolarizedLight => Theme::SolarizedLight,
            AppTheme::Nord => Theme::Nord,
            AppTheme::GruvboxLight => Theme::GruvboxLight,
            AppTheme::GruvboxDark => Theme::GruvboxDark,
            AppTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            AppTheme::KanagawaWave => Theme::KanagawaWave,
            AppTheme::TokyoNight => Theme::TokyoNight,
            AppTheme::TokyoNightLight => Theme::TokyoNightLight,
            AppTheme::Moonfly => Theme::Moonfly,
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => "Light",
                Self::Dark => "Dark",
                Self::SolarizedLight => "Solarized Light",
                Self::Nord => "Nord",
                Self::GruvboxLight => "Gruvbox Light",
                Self::GruvboxDark => "Gruvbox Dark",
                Self::CatppuccinLatte => "Catppuccin Latte",
                Self::KanagawaWave => "Kanagawa Wave",
                Self::TokyoNight => "Tokyo Night",
                Self::TokyoNightLight => "Tokyo Night Light",
                Self::Moonfly => "Moonfly",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
// todo filters, subtitles
pub struct VideoSettings {
    pub thumbnail_interval: u32,
    pub volume: f64,
    pub speed: f64,
    pub gamma: f64,
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
}

impl VideoSettings {
    fn defaults() -> Self {
        Self {
            thumbnail_interval: 10,
            volume: 1.0,
            speed: 1.0,
            gamma: 1.3,
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
        }
    }
}

// todo: API key,
#[derive(Debug, Clone, Copy)]
pub struct GeneralSettings {
    pub layout: Layout,
    pub refresh_interval: std::time::Duration,
    //todo: Scripted Collections should remove need for this
    pub recents_limit: Option<i32>,
    pub search_limit: Option<i32>,
    pub theme: AppTheme,
}

impl GeneralSettings {
    fn defaults() -> Self {
        Self {
            layout: Layout::default(),
            refresh_interval: std::time::Duration::from_secs(600),
            theme: AppTheme::default(),
            recents_limit: Some(5),
            search_limit: Some(5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub video: VideoSettings,
    pub general: GeneralSettings,
    pub keystore: KeyStore,
}

impl Config {
    pub fn defaults() -> Self {
        Self {
            video: VideoSettings::defaults(),
            general: GeneralSettings::defaults(),
            keystore: KeyStore::defaults(),
        }
    }

    pub fn theme(&self) -> iced::Theme {
        self.general.theme.into()
    }

    pub fn layout(&self) -> Layout {
        self.general.layout
    }

    pub fn refresh_interval(&self) -> std::time::Duration {
        self.general.refresh_interval
    }

    pub fn search_limit(&self) -> Option<i32> {
        self.general.search_limit
    }
}

mod keys {
    use super::*;
    use iced::keyboard;
    use std::{collections::hash_map::{HashMap, Iter}, hash::Hash};

    #[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
    pub enum KeyModifier {
        #[default]
        None,
        Shift,
        Control,
        Alt,
        ShiftControl,
        ShiftAlt,
        ControlAlt,
    }

    impl From<keyboard::Modifiers> for KeyModifier {
        fn from(value: keyboard::Modifiers) -> Self {
            if value.shift() && value.command() {
                KeyModifier::ShiftControl
            } else if value.shift() && value.alt() {
                KeyModifier::ShiftAlt
            } else if value.command() && value.alt() {
                KeyModifier::ControlAlt
            } else if value.shift() {
                KeyModifier::Shift
            } else if value.command() {
                KeyModifier::Control
            } else if value.alt() {
                KeyModifier::Alt
            } else {
                KeyModifier::None
            }
        }
    }

    impl From<KeyModifier> for keyboard::Modifiers {
        fn from(value: KeyModifier) -> Self {
            use keyboard::Modifiers;

            match value {
                KeyModifier::None => Modifiers::empty(),
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
                    Self::None => "",
                    Self::Control => "Ctrl",
                    Self::ControlAlt => "Ctrl + Alt",
                    Self::Shift => "Shift",
                    Self::ShiftAlt => "Shift + Alt",
                    Self::ShiftControl => "Shift + Ctrl",
                    Self::Alt => "Alt",
                }
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct KeyPress {
        pub key: keyboard::Key,
        pub modifiers: KeyModifier,
    }

    impl std::fmt::Display for KeyPress {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use keyboard::key::{Key, Named};

            let has_modifiers = !matches!(self.modifiers, KeyModifier::None);
            let modifiers = if has_modifiers {
                format!(" + {}", self.modifiers)
            } else {
                String::default()
            };

            write!(
                f,
                "{}{modifiers}",
                match &self.key {
                    Key::Named(named) => format!("{named:?}"),
                    Key::Character(character) => format!("{character}"),
                    Key::Unidentified => "Unidentified".to_owned(),
                }
            )
        }
    }

    impl KeyPress {
        pub fn new(key: keyboard::Key, modifiers: KeyModifier) -> Self {
            Self { key, modifiers }
        }
    }

    #[derive(Debug, Clone)]
    struct KeyStoreInner<A> {
        keys: HashMap<KeyPress, A>,
        actions: HashMap<A, Vec<KeyPress>>,
    }

    impl<A> KeyStoreInner<A>
    where
        A: Hash + Eq + Copy,
    {
        fn new() -> Self {
            Self {
                keys: HashMap::default(),
                actions: HashMap::default(),
            }
        }

        fn get(&self, key: &KeyPress) -> Option<&A>{
            self.keys.get(key)
        }

        fn get_action(&self, action: A) -> Option<&Vec<KeyPress>>{
            self.actions.get(&action)
        }

        fn iter_action(&self) -> Iter<'_, A, Vec<KeyPress>> {
            self.actions.iter()
        }

        fn iter_keys(&self) -> Iter<'_, KeyPress, A> {
            self.keys.iter()
        }

        fn insert(&mut self, key: KeyPress, action: A) {
            if let Some(previous) = self.keys.insert(key.clone(), action) {
                if let Some(keys) = self.actions.get_mut(&previous) {
                    keys.retain(|curr| key != *curr);
                }
            }

            let keys = self.actions.entry(action).or_default();
            keys.push(key);
        }

        fn extend(&mut self, iter: impl IntoIterator<Item = (KeyPress, A)>) {
            for (key, action) in iter {
                self.insert(key, action);
            }
        }

        fn remove(&mut self, key: KeyPress) {
            if let Some(action) = self.keys.remove(&key) {
                if let Some(keys) = self.actions.get_mut(&action) {
                    keys.retain(|curr| key != *curr)
                }
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

    #[derive(Debug, Clone)]
    pub struct KeyStore {
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
                home: home().collect(),
                player: player().collect(),
                settings: settings().collect(),
            }
        }

        pub fn action(
            &self,
            keypress: KeyPress,
            screen: Screen,
        ) -> Option<Action> {

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

        pub fn get_home(&self, key: &KeyPress) -> Option<&HomeAction>{
            self.home.get(key)
        }

        pub fn get_player(&self, key: &KeyPress) -> Option<&PlayerAction>{
            self.player.get(key)
        }

        pub fn get_settings(&self, key: &KeyPress) -> Option<&SettingsAction>{
            self.settings.get(key)
        }

        pub fn insert_home(&mut self, key: KeyPress, action: HomeAction){
            self.home.insert(key, action);
        }

        pub fn insert_player(&mut self, key: KeyPress, action: PlayerAction){
            self.player.insert(key, action);
        }

        pub fn insert_settings(&mut self, key: KeyPress, action: SettingsAction){
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

        pub fn clear_home(&mut self, action: HomeAction){
            self.home.clear(action);
        }

        pub fn clear_player(&mut self, action: PlayerAction){
            self.player.clear(action);
        }

        pub fn clear_settings(&mut self, action: SettingsAction){
            self.settings.clear(action);
        }
    }

    fn home() -> impl Iterator<Item = (KeyPress, HomeAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (
                key(Key::Named(Named::ArrowLeft), KeyModifier::Alt),
                HomeAction::Back,
            ),
            (
                key(Key::Named(Named::ArrowRight), KeyModifier::Alt),
                HomeAction::Forward,
            ),
            (
                key(Key::Named(Named::NavigateNext), KeyModifier::None),
                HomeAction::Forward,
            ),
            (
                key(Key::Named(Named::BrowserForward), KeyModifier::None),
                HomeAction::Forward,
            ),
            (
                key(Key::Named(Named::NavigatePrevious), KeyModifier::None),
                HomeAction::Back,
            ),
            (
                key(Key::Named(Named::BrowserBack), KeyModifier::None),
                HomeAction::Back,
            ),
            (
                key(Key::Character("l".into()), KeyModifier::None),
                HomeAction::LayoutToggle,
            ),
            (
                key(Key::Character("r".into()), KeyModifier::Shift),
                HomeAction::Refresh,
            ),
            (
                key(Key::Character("r".into()), KeyModifier::None),
                HomeAction::RefreshContent,
            ),
            (
                key(Key::Character("s".into()), KeyModifier::Control),
                HomeAction::SettingsOpen,
            ),
            (
                key(Key::Character("/".into()), KeyModifier::None),
                HomeAction::SearchToggle,
            ),
            (
                key(Key::Character("f".into()), KeyModifier::Control),
                HomeAction::SearchToggle,
            ),
            (
                key(Key::Named(Named::Escape), KeyModifier::None),
                HomeAction::CloseModal,
            ),
        ]
        .into_iter()
    }

    fn player() -> impl Iterator<Item = (KeyPress, PlayerAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (
                key(Key::Named(Named::ArrowLeft), KeyModifier::Alt),
                PlayerAction::Back,
            ),
            (
                key(Key::Named(Named::NavigatePrevious), KeyModifier::None),
                PlayerAction::Back,
            ),
            (
                key(Key::Named(Named::BrowserBack), KeyModifier::None),
                PlayerAction::Back,
            ),
            (
                key(Key::Named(Named::Space), KeyModifier::None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::MediaPlayPause), KeyModifier::None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::ArrowLeft), KeyModifier::Control),
                PlayerAction::PlayPrevious,
            ),
            (
                key(Key::Named(Named::MediaTrackPrevious), KeyModifier::None),
                PlayerAction::PlayPrevious,
            ),
            (
                key(Key::Named(Named::ArrowRight), KeyModifier::Control),
                PlayerAction::PlayNext,
            ),
            (
                key(Key::Named(Named::MediaTrackNext), KeyModifier::None),
                PlayerAction::PlayToggle,
            ),
            (
                key(Key::Named(Named::Enter), KeyModifier::None),
                PlayerAction::FullscreenToggle,
            ),
            (
                key(Key::Named(Named::Escape), KeyModifier::None),
                PlayerAction::Exit,
            ),
            (
                key(Key::Character("f".into()), KeyModifier::None),
                PlayerAction::FullscreenToggle,
            ),
            (
                key(Key::Named(Named::ArrowLeft), KeyModifier::Shift),
                PlayerAction::SeekBackShift,
            ),
            (
                key(Key::Named(Named::ArrowLeft), KeyModifier::None),
                PlayerAction::SeekBack,
            ),
            (
                key(Key::Named(Named::ArrowRight), KeyModifier::Shift),
                PlayerAction::SeekFrontShift,
            ),
            (
                key(Key::Named(Named::ArrowRight), KeyModifier::None),
                PlayerAction::SeekFront,
            ),
            (
                key(Key::Named(Named::ArrowUp), KeyModifier::None),
                PlayerAction::VolumeIncrease,
            ),
            (
                key(Key::Named(Named::ArrowDown), KeyModifier::None),
                PlayerAction::VolumeDecrease,
            ),
            (
                key(Key::Character("m".into()), KeyModifier::None),
                PlayerAction::MuteToggle,
            ),
            (
                key(Key::Character("c".into()), KeyModifier::None),
                PlayerAction::SpeedIncrease,
            ),
            (
                key(Key::Named(Named::PlaySpeedUp), KeyModifier::None),
                PlayerAction::SpeedIncrease,
            ),
            (
                key(Key::Character("x".into()), KeyModifier::None),
                PlayerAction::SpeedDecrease,
            ),
            (
                key(Key::Named(Named::PlaySpeedDown), KeyModifier::None),
                PlayerAction::SpeedDecrease,
            ),
            (
                key(Key::Character("z".into()), KeyModifier::None),
                PlayerAction::SpeedReset,
            ),
            (
                key(Key::Named(Named::PlaySpeedReset), KeyModifier::None),
                PlayerAction::SpeedReset,
            ),
            (
                key(Key::Character("s".into()), KeyModifier::Control),
                PlayerAction::VideoConfig,
            ),
            (
                key(Key::Character("s".into()), KeyModifier::None),
                PlayerAction::SubtitlesToggle,
            ),
            (
                key(Key::Named(Named::Subtitle), KeyModifier::None),
                PlayerAction::SubtitlesToggle,
            ),
            (
                key(Key::Character("b".into()), KeyModifier::None),
                PlayerAction::VideoComment,
            ),
            (
                key(Key::Character("p".into()), KeyModifier::None),
                PlayerAction::PlaylistToggle,
            ),
        ]
        .into_iter()
    }

    fn settings() -> impl Iterator<Item = (KeyPress, SettingsAction)> {
        use keyboard::{Key, key::Named};
        let key = KeyPress::new;

        [
            (
                key(Key::Named(Named::Escape), KeyModifier::None),
                SettingsAction::Cancel,
            ),
            (
                key(Key::Named(Named::ArrowUp), KeyModifier::None),
                SettingsAction::Up,
            ),
            (
                key(Key::Named(Named::ArrowDown), KeyModifier::None),
                SettingsAction::Down,
            ),
        ]
        .into_iter()
    }
}
