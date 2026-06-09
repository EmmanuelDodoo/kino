use crate::app::Message;
use crate::db::Operation;
use crate::utils::{
    self, AppTheme, Config, FontState, GeneralSettings, HomeAction, KeyPress, Layout, PlayerAction,
    Scroll, SettingsAction, SubtitleDescription, VideoFilters, VideoSettings, cancel_btn,
    convert_color_str, empty, icons, icons::sized_button, modal::modal, modal_container,
    picklist_handle, save_btn, styles, toggler, tooltip, trim_path, typo::*,
};
use devutils::source::SourceSet;
use iced::{
    Border, Element, Length, Task, Theme,
    alignment::{Horizontal, Vertical},
    font::Family,
    widget::{
        button, center_x, checkbox, column, combo_box, container, operation, pick_list, rich_text,
        row, rule, scrollable, slider, space, span, table, text, text_input, tooltip::Tooltip,
    },
};
use registry::models::{Directory, DirectoryId, MediaType, humanize_datetime};
use widgets::{expandable, marquee};

use std::path::{Path, PathBuf};
use std::time::Duration;

const PADDING: [f32; 2] = [20.0, 24.0];
const TEXT_SIZE: f32 = P;
const INPUT_WIDTH: f32 = 56.0;
const SPACING: f32 = 6.0;
const INPUT_PADDING: [f32; 2] = [3.5, 5.0];
const LIST_PADDING: [f32; 2] = [5.0, 10.0];
const ACTIONS_PADDING: [f32; 2] = [1.5, 1.5];
const ACTIONS_SIZE: f32 = 10.0;
const ACTIONS_SPACING: f32 = 4.0;
const SECTION_SPACING: f32 = 16.0;
const SLIDER_WIDTH: f32 = 200.0;
const SLIDER_SPACING: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub enum KeyAction {
    General(Option<HomeAction>),
    Video(Option<PlayerAction>),
    Settings(Option<SettingsAction>),
}

impl KeyAction {
    fn none(&self) -> Self {
        match self {
            Self::Video(_) => Self::Video(None),
            Self::General(_) => Self::General(None),
            Self::Settings(_) => Self::Settings(None),
        }
    }

    fn is_some(&self) -> bool {
        matches!(
            self,
            Self::Video(Some(_)) | Self::Settings(Some(_)) | Self::General(Some(_))
        )
    }
}

impl From<HomeAction> for KeyAction {
    fn from(value: HomeAction) -> Self {
        Self::General(Some(value))
    }
}

impl From<PlayerAction> for KeyAction {
    fn from(value: PlayerAction) -> Self {
        Self::Video(Some(value))
    }
}

impl From<SettingsAction> for KeyAction {
    fn from(value: SettingsAction) -> Self {
        Self::Settings(Some(value))
    }
}

#[derive(Debug, Clone)]
struct ScrollState {
    general: Scroll,
    video: Scroll,
    keybinds: Scroll,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            general: Scroll::new(),
            video: Scroll::new(),
            keybinds: Scroll::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum View {
    FolderSelection {
        path: PathBuf,
        kind: MediaType,
    },
    CaptureKey {
        action: KeyAction,
        key: Option<KeyPress>,
        conflict: KeyAction,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Page {
    #[default]
    General,
    Video,
    Keybinds,
}

#[derive(Debug, Clone, Copy)]
pub enum FolderSelectionMessage {
    Cancel,
    Reselect,
    Kind(MediaType),
    Submit,
}

#[derive(Debug, Clone)]
pub enum VideoFilterMessage {
    Gamma(f64),
    Brightness(f64),
    Contrast(f64),
    Hue(f64),
    Saturation(f64),
}

#[derive(Debug, Clone)]
pub enum GeneralMessage {
    Refresh(String),
    IncrRefresh,
    DecrRefresh,
    Recents(String),
    IncrRecents,
    DecrRecents,
    Search(String),
    IncrSearch,
    DecrSearch,
    ShowDirs(bool),
    ToggleShowDirs,
}

#[derive(Debug, Clone)]
pub enum AppearanceMessage {
    Layout(Layout),
    Theme(AppTheme),
}

#[derive(Debug, Clone)]
pub enum MediaMessage {
    MovieDepth(String),
    ScanDiscoverer(bool),
    ToggleScanDiscover,
    RestoreDeleted(bool),
    ToggleRestore,
    ToggleDirShow(bool),
    Scan(DirectoryId, bool),
    ScanAll,
    ToggleDirectoryAdd(DirectoryId),
    ToggleDirKind(DirectoryId),
    DirSource(DirectoryId, SourceSet),
    AddFolder,
    IncrMovieDepth,
    DecrMovieDepth,
    PreferredSub(String),
    PreferredAudio(String),
    None,
}

#[derive(Debug, Clone)]
pub enum MetadataMessage {
    TMDBRating(bool),
    ToggleTMDBRating,
    Auth(String),
    Fetch(String),
    IncrFetch,
    DecrFetch,
    Source(SourceSet),
}

#[derive(Debug, Clone)]
pub enum PlaybackMessage {
    AutoStart(bool),
    AutoNext(bool),
    ToggleAutoStart,
    ToggleAutoNext,
    Volume(f64),
    VolumeAmt(String),
    IncrVolAmt,
    DecrVolAmt,
    Speed(f64),
    SpeedAmt(String),
    IncrSpeedAmt,
    DecrSpeedAmt,
    CompletionPoint(String),
    IncrComplPoint,
    DecrComplPoint,
    CompletionTime(String),
    IncrComplTime,
    DecrComplTime,
}

#[derive(Debug, Clone)]
pub enum SeekingMessage {
    ThumbnailInterval(String),
    IncrThumbInterval,
    DecrThumbInterval,
    Seek(String),
    IncrSeek,
    DecrSeek,
    SeekShift(String),
    IncrSeekShift,
    DecrSeekShift,
    Span(String),
    IncrSpan,
    DecrSpan,
}

#[derive(Debug, Clone)]
pub enum SubtitleMessage {
    Dummy(String),
    Subtitles(bool),
    ToggleSubtitles,
    SubSizeIncr,
    SubSizeDecr,
    SubSize(String),
    SubColor(String),
    SubBackground(String),
    Font(Family),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Goto(Page),
    Scroll(scrollable::Viewport),
    OpenLog,
    OpenConfig,
    General(GeneralMessage),
    Appearance(AppearanceMessage),
    Media(MediaMessage),
    Metadata(MetadataMessage),
    Playback(PlaybackMessage),
    Seeking(SeekingMessage),
    VideoFilters(VideoFilterMessage),
    Subtitles(SubtitleMessage),
    Save,
    Cancel,
    FolderSelected(Option<PathBuf>),
    FolderSelection(FolderSelectionMessage),
    ClearAllBindings(KeyAction),
    ClearBinding(Page, KeyPress),
    NewKeyPress(KeyAction),
    KeyAction(KeyAction),
    SaveKeyBinding,
    None,
}

#[derive(Debug, Clone)]
struct Dir {
    dir: Directory,
    toggled: Option<bool>,
    operation: Operation,
    original_media: MediaType,
    original_source: SourceSet,
    scan: bool,
}

impl Dir {
    fn new(dir: Directory) -> Self {
        Self {
            toggled: None,
            operation: Operation::Insert,
            original_media: dir.media_type,
            original_source: SourceSet::from_str(&dir.source),
            scan: true,
            dir,
        }
    }

    fn fetched(dir: Directory) -> Self {
        Self {
            toggled: Some(false),
            operation: Operation::Update,
            original_media: dir.media_type,
            original_source: SourceSet::from_str(&dir.source),
            scan: false,
            dir,
        }
    }

    fn toggle_add(&mut self) {
        self.operation = match self.operation {
            Operation::Update => Operation::Delete,
            Operation::Insert => Operation::Delete,
            Operation::Delete if self.toggled.is_none() => Operation::Insert,
            Operation::Delete => Operation::Update,
        };
    }

    fn toggle_kind(&mut self) {
        self.dir.media_type = match self.dir.media_type {
            MediaType::Shows => MediaType::Movies,
            MediaType::Movies => MediaType::Shows,
        };

        if let Some(toggled) = self.toggled.as_mut() {
            self.scan = self.dir.media_type != self.original_media;
            *toggled = !*toggled;
        }
    }

    fn select_source(&mut self, source: SourceSet) {
        self.dir.source = source.to_str().to_owned();
        self.scan = self.dir.source != self.original_source.to_str();
    }

    fn save(self) -> Option<(Directory, Option<Operation>, bool, bool)> {
        let source_changed = self.dir.source != self.original_source.to_str();
        match self.toggled {
            // Pre-existing not being scanned or deleted or source changed
            Some(false)
                if matches!(self.operation, Operation::Update) && !self.scan && !source_changed =>
            {
                None
            }
            None if matches!(self.operation, Operation::Delete) => None,
            _ => {
                let operation = if self.original_media != self.dir.media_type
                    || source_changed
                    || matches!(self.operation, Operation::Insert)
                    || (self.toggled.is_some() && matches!(self.operation, Operation::Delete))
                {
                    Some(self.operation)
                } else {
                    None
                };

                Some((self.dir, operation, self.scan, source_changed))
            }
        }
    }
}

#[derive(Debug)]
pub struct Settings {
    pub config: Config,
    subtitle_state: FontState,

    page: Page,
    view: Option<View>,

    scroll_state: ScrollState,

    text_color: String,
    background_color: String,

    directories: Vec<Dir>,
    directories_shown: bool,

    subtitle_dummy: String,
}

impl Settings {
    pub fn boot(config: Config, fonts: Vec<Family>) -> (Self, Task<Message>) {
        let mut new = Self::new(config, fonts);
        let scroll = new.update_scroll();
        let dirs = Task::done(Message::FetchDirectories);

        let tasks = Task::batch([scroll, dirs]);

        (new, tasks)
    }

    fn new(config: Config, fonts: Vec<Family>) -> Self {
        let text_color = format!("#{:08x}", config.video.subtitles.color);
        let background_color = format!("#{:08x}", config.video.subtitles.background_color);

        let subtitle_state = FontState::new(fonts, &config.video.subtitles.font);

        Self {
            config,
            subtitle_state,
            page: Page::default(),
            view: None,
            scroll_state: ScrollState::new(),
            directories: Vec::default(),
            text_color,
            background_color,
            directories_shown: false,
            subtitle_dummy: "An example subtitle".to_owned(),
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<Message> {
        match message {
            SettingsMessage::None => Task::none(),
            SettingsMessage::Goto(page) => self.goto(page),
            SettingsMessage::Scroll(viewport) => {
                let offset = viewport.absolute_offset();
                match self.page {
                    Page::General => self.scroll_state.general.offset = offset,
                    Page::Video => self.scroll_state.video.offset = offset,
                    Page::Keybinds => self.scroll_state.keybinds.offset = offset,
                }

                Task::none()
            }
            SettingsMessage::Cancel => self.cancel(),
            SettingsMessage::Save => Task::done(Message::SaveSettings),
            SettingsMessage::General(gsg) => match gsg {
                GeneralMessage::Refresh(interval) => {
                    let interval = interval.trim();
                    if interval.is_empty() {
                        self.config.general.refresh_interval = Duration::ZERO;
                        return Task::none();
                    }

                    let Ok(interval) = interval.parse::<u64>() else {
                        let msg = Message::error(format!("Invalid input: {interval}"), true);
                        return Task::done(msg);
                    };

                    self.config.general.refresh_interval = Duration::from_secs(interval);

                    Task::none()
                }
                GeneralMessage::IncrRefresh => {
                    self.config.general.refresh_interval = self
                        .config
                        .general
                        .refresh_interval
                        .saturating_add(Duration::from_secs(1));
                    Task::none()
                }
                GeneralMessage::DecrRefresh => {
                    self.config.general.refresh_interval = self
                        .config
                        .general
                        .refresh_interval
                        .saturating_sub(Duration::from_secs(1));
                    Task::none()
                }
                GeneralMessage::Search(searches) => {
                    let search = searches.trim();
                    if search.is_empty() {
                        self.config.general.search_limit = None;
                        return Task::none();
                    }

                    let Ok(searches) = searches.parse::<i32>() else {
                        let msg = Message::error(format!("Invalid input: {searches}"), true);
                        return Task::done(msg);
                    };

                    self.config.general.search_limit = Some(searches);

                    Task::none()
                }
                GeneralMessage::IncrSearch => {
                    let value = self.config.general.search_limit.unwrap_or(0) + 1;
                    self.config.general.search_limit = Some(value);
                    Task::none()
                }
                GeneralMessage::DecrSearch => {
                    let value = self.config.general.search_limit.unwrap_or(0) - 1;
                    self.config.general.search_limit = Some(value.max(-1));
                    Task::none()
                }
                GeneralMessage::Recents(recents) => {
                    let recents = recents.trim();
                    if recents.is_empty() {
                        self.config.general.recents_limit = None;
                        return Task::none();
                    }

                    let Ok(recents) = recents.parse::<i32>() else {
                        let msg = Message::error(format!("Invalid input: {recents}"), true);
                        return Task::done(msg);
                    };

                    self.config.general.recents_limit = Some(recents);

                    Task::none()
                }
                GeneralMessage::IncrRecents => {
                    let value = self.config.general.recents_limit.unwrap_or(0) + 1;
                    self.config.general.recents_limit = Some(value);
                    Task::none()
                }
                GeneralMessage::DecrRecents => {
                    let value = self.config.general.recents_limit.unwrap_or(0) - 1;
                    self.config.general.recents_limit = Some(value.max(-1));
                    Task::none()
                }
                GeneralMessage::ShowDirs(show) => {
                    self.config.general.show_dirs = show;
                    Task::none()
                }
                GeneralMessage::ToggleShowDirs => {
                    self.config.general.show_dirs = !self.config.general.show_dirs;
                    Task::none()
                }
            },
            SettingsMessage::Appearance(asg) => match asg {
                AppearanceMessage::Theme(theme) => {
                    self.config.general.set_theme(theme);
                    Task::none()
                }
                AppearanceMessage::Layout(layout) => {
                    self.config.general.layout = layout;

                    Task::none()
                }
            },
            SettingsMessage::Media(msg) => match msg {
                MediaMessage::None => Task::none(),
                MediaMessage::AddFolder => pick_task(),
                MediaMessage::MovieDepth(depth) => {
                    let depth = depth.trim();
                    if depth.is_empty() {
                        self.config.general.movie_depth = 0;
                        return Task::none();
                    }

                    let Ok(depth) = depth.parse::<u8>() else {
                        let msg = Message::error(format!("Invalid input: {depth}"), true);
                        return Task::done(msg);
                    };

                    self.config.general.movie_depth = depth;
                    Task::none()
                }
                MediaMessage::IncrMovieDepth => {
                    self.config.general.movie_depth =
                        self.config.general.movie_depth.saturating_add(1);
                    Task::none()
                }
                MediaMessage::DecrMovieDepth => {
                    self.config.general.movie_depth =
                        self.config.general.movie_depth.saturating_sub(1);
                    Task::none()
                }
                MediaMessage::ToggleDirShow(show) => {
                    self.directories_shown = show;
                    Task::none()
                }
                MediaMessage::ToggleDirectoryAdd(id) => {
                    if let Some(dir) = self.directories.iter_mut().find(|dir| dir.dir.id == id) {
                        dir.toggle_add();
                    };

                    Task::none()
                }
                MediaMessage::ToggleDirKind(id) => {
                    if let Some(dir) = self.directories.iter_mut().find(|dir| dir.dir.id == id) {
                        dir.toggle_kind();
                    }

                    Task::none()
                }
                MediaMessage::DirSource(id, source) => {
                    if let Some(dir) = self.directories.iter_mut().find(|dir| dir.dir.id == id) {
                        dir.select_source(source);
                    }

                    Task::none()
                }
                MediaMessage::ToggleRestore => {
                    self.config.general.restore_deleted = !self.config.general.restore_deleted;
                    Task::none()
                }
                MediaMessage::ToggleScanDiscover => {
                    self.config.general.scan_discoverer = !self.config.general.scan_discoverer;
                    Task::none()
                }
                MediaMessage::Scan(id, scan) => {
                    if let Some(dir) = self.directories.iter_mut().find(|dir| dir.dir.id == id) {
                        dir.scan = scan;
                    }

                    Task::none()
                }
                MediaMessage::ScanAll => {
                    for dir in self.directories.iter_mut() {
                        dir.scan = true;
                    }

                    self.directories_shown = true;
                    Task::none()
                }
                MediaMessage::ScanDiscoverer(enable) => {
                    self.config.general.scan_discoverer = enable;

                    Task::none()
                }
                MediaMessage::RestoreDeleted(enable) => {
                    self.config.general.restore_deleted = enable;

                    Task::none()
                }
                MediaMessage::PreferredSub(preferred) => {
                    self.config.general.preferred_subtitle_codec =
                        (!preferred.is_empty()).then_some(preferred);

                    Task::none()
                }
                MediaMessage::PreferredAudio(preferred) => {
                    self.config.general.preferred_audio_codec =
                        (!preferred.is_empty()).then_some(preferred);

                    Task::none()
                }
            },
            SettingsMessage::Metadata(msg) => match msg {
                MetadataMessage::Auth(auth) => {
                    self.config.general.auth_token = auth;
                    Task::none()
                }
                MetadataMessage::Fetch(interval) => {
                    let interval = interval.trim();
                    if interval.is_empty() {
                        self.config.general.fetching_interval = Duration::ZERO;
                        return Task::none();
                    }

                    let Ok(interval) = interval.parse::<u64>() else {
                        let msg = Message::error(format!("Invalid input: {interval}"), true);
                        return Task::done(msg);
                    };

                    self.config.general.fetching_interval = Duration::from_secs(interval);

                    Task::none()
                }
                MetadataMessage::TMDBRating(enable) => {
                    self.config.general.tmdb_rating = enable;
                    Task::none()
                }
                MetadataMessage::ToggleTMDBRating => {
                    self.config.general.tmdb_rating = !self.config.general.tmdb_rating;
                    Task::none()
                }
                MetadataMessage::IncrFetch => {
                    self.config.general.fetching_interval = self
                        .config
                        .general
                        .fetching_interval
                        .saturating_add(Duration::from_secs(1));
                    Task::none()
                }
                MetadataMessage::DecrFetch => {
                    self.config.general.fetching_interval = self
                        .config
                        .general
                        .fetching_interval
                        .saturating_sub(Duration::from_secs(1));
                    Task::none()
                }
                MetadataMessage::Source(source) => {
                    self.config.general.default_source = source;
                    Task::none()
                }
            },
            SettingsMessage::Playback(psg) => match psg {
                PlaybackMessage::VolumeAmt(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.volume_change_amt = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.volume_change_amt = amt.min(1.0);

                    Task::none()
                }
                PlaybackMessage::IncrVolAmt => {
                    self.config.video.volume_change_amt =
                        (self.config.video.volume_change_amt + 0.05).min(1.0);
                    Task::none()
                }
                PlaybackMessage::DecrVolAmt => {
                    self.config.video.volume_change_amt =
                        (self.config.video.volume_change_amt - 0.05).max(0.0);
                    Task::none()
                }
                PlaybackMessage::SpeedAmt(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.speed_change_amt = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.speed_change_amt = amt;

                    Task::none()
                }
                PlaybackMessage::IncrSpeedAmt => {
                    self.config.video.speed_change_amt =
                        (self.config.video.speed_change_amt + 0.1).min(1.0);
                    Task::none()
                }
                PlaybackMessage::DecrSpeedAmt => {
                    self.config.video.speed_change_amt =
                        (self.config.video.speed_change_amt - 0.1).max(0.0);
                    Task::none()
                }
                PlaybackMessage::CompletionPoint(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.completion_point = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.completion_point = amt.min(1.0);

                    Task::none()
                }
                PlaybackMessage::IncrComplPoint => {
                    self.config.video.completion_point =
                        (self.config.video.completion_point + 0.05).min(1.0);
                    Task::none()
                }
                PlaybackMessage::DecrComplPoint => {
                    self.config.video.completion_point =
                        (self.config.video.completion_point - 0.05).max(0.0);
                    Task::none()
                }
                PlaybackMessage::CompletionTime(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.completion_watch_time = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.completion_watch_time = amt.min(1.0);

                    Task::none()
                }
                PlaybackMessage::IncrComplTime => {
                    self.config.video.completion_watch_time =
                        (self.config.video.completion_watch_time + 0.05).min(1.0);
                    Task::none()
                }
                PlaybackMessage::DecrComplTime => {
                    self.config.video.completion_watch_time =
                        (self.config.video.completion_watch_time - 0.05).max(0.0);
                    Task::none()
                }
                PlaybackMessage::Volume(new) => {
                    self.config.video.volume = new;
                    Task::none()
                }
                PlaybackMessage::Speed(new) => {
                    self.config.video.speed = new;
                    Task::none()
                }
                PlaybackMessage::AutoStart(toggle) => {
                    self.config.video.auto_start = toggle;
                    Task::none()
                }
                PlaybackMessage::ToggleAutoStart => {
                    self.config.video.auto_start = !self.config.video.auto_start;
                    Task::none()
                }
                PlaybackMessage::AutoNext(toggle) => {
                    self.config.video.auto_next = toggle;
                    Task::none()
                }
                PlaybackMessage::ToggleAutoNext => {
                    self.config.video.auto_next = !self.config.video.auto_next;
                    Task::none()
                }
            },
            SettingsMessage::Seeking(ssg) => match ssg {
                SeekingMessage::ThumbnailInterval(interval) => {
                    let interval = interval.trim();
                    if interval.is_empty() {
                        self.config.video.thumbnail_interval = 0;
                        return Task::none();
                    }

                    let Ok(interval) = interval.parse::<u32>() else {
                        let msg = Message::error(format!("Invalid input: {interval}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.thumbnail_interval = interval;

                    Task::none()
                }
                SeekingMessage::IncrThumbInterval => {
                    self.config.video.thumbnail_interval =
                        self.config.video.thumbnail_interval.saturating_add(1);
                    Task::none()
                }
                SeekingMessage::DecrThumbInterval => {
                    self.config.video.thumbnail_interval =
                        self.config.video.thumbnail_interval.saturating_sub(1);
                    Task::none()
                }
                SeekingMessage::Seek(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.seek_change_amt = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.seek_change_amt = amt;

                    Task::none()
                }
                SeekingMessage::IncrSeek => {
                    self.config.video.seek_change_amt += 1.0;
                    Task::none()
                }
                SeekingMessage::DecrSeek => {
                    self.config.video.seek_change_amt =
                        (self.config.video.seek_change_amt - 1.0).max(0.0);
                    Task::none()
                }
                SeekingMessage::SeekShift(amt) => {
                    let amt = amt.trim();
                    if amt.is_empty() {
                        self.config.video.seek_shift_change_amt = 0.0;
                        return Task::none();
                    }

                    let Ok(amt) = amt.parse::<f64>() else {
                        let msg = Message::error(format!("Invalid input: {amt}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.seek_shift_change_amt = amt;

                    Task::none()
                }
                SeekingMessage::IncrSeekShift => {
                    self.config.video.seek_shift_change_amt += 1.0;
                    Task::none()
                }
                SeekingMessage::DecrSeekShift => {
                    self.config.video.seek_shift_change_amt =
                        (self.config.video.seek_shift_change_amt - 1.0).max(0.0);
                    Task::none()
                }
                SeekingMessage::Span(span) => {
                    let span = span.trim();
                    if span.is_empty() {
                        self.config.video.comment_span = 0;
                        return Task::none();
                    }

                    let Ok(span) = span.parse::<u64>() else {
                        let msg = Message::error(format!("Invalid input: {span}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.comment_span = span;

                    Task::none()
                }
                SeekingMessage::IncrSpan => {
                    self.config.video.comment_span += 1;
                    Task::none()
                }
                SeekingMessage::DecrSpan => {
                    self.config.video.comment_span =
                        self.config.video.comment_span.saturating_sub(1);
                    Task::none()
                }
            },
            SettingsMessage::VideoFilters(vsg) => match vsg {
                VideoFilterMessage::Gamma(value) => {
                    self.config.video.filters.gamma = value;
                    Task::none()
                }
                VideoFilterMessage::Hue(value) => {
                    self.config.video.filters.hue = value;
                    Task::none()
                }
                VideoFilterMessage::Brightness(value) => {
                    self.config.video.filters.brightness = value;
                    Task::none()
                }
                VideoFilterMessage::Contrast(value) => {
                    self.config.video.filters.contrast = value;
                    Task::none()
                }
                VideoFilterMessage::Saturation(value) => {
                    self.config.video.filters.saturation = value;
                    Task::none()
                }
            },
            SettingsMessage::Subtitles(ssg) => match ssg {
                SubtitleMessage::Subtitles(show) => {
                    self.config.video.show_subtitles = show;
                    Task::none()
                }
                SubtitleMessage::ToggleSubtitles => {
                    self.config.video.show_subtitles = !self.config.video.show_subtitles;
                    Task::none()
                }
                SubtitleMessage::SubSizeIncr => {
                    self.config.video.subtitles.size =
                        (self.config.video.subtitles.size + 1).min(60);
                    Task::none()
                }
                SubtitleMessage::SubSizeDecr => {
                    self.config.video.subtitles.size =
                        (self.config.video.subtitles.size - 1).max(5);
                    Task::none()
                }
                SubtitleMessage::SubSize(size) => {
                    let size = size.trim();
                    if size.is_empty() {
                        self.config.video.subtitles.size = 5;
                        return Task::none();
                    }

                    let Ok(size) = size.parse::<u32>() else {
                        let msg = Message::error(format!("Invalid input: {size}"), true);
                        return Task::done(msg);
                    };

                    self.config.video.subtitles.size = size.max(5);

                    Task::none()
                }
                SubtitleMessage::SubColor(color) => {
                    if let Some(color) = convert_color_str(&color) {
                        self.config.video.subtitles.color = color;
                    };

                    self.text_color = color;

                    Task::none()
                }
                SubtitleMessage::SubBackground(color) => {
                    if let Some(color) = convert_color_str(&color) {
                        self.config.video.subtitles.background_color = color;
                    };

                    self.background_color = color;

                    Task::none()
                }
                SubtitleMessage::Dummy(dummy) => {
                    self.subtitle_dummy = dummy;

                    Task::none()
                }
                SubtitleMessage::Font(family) => {
                    self.config.video.subtitles.font = family.to_string();
                    self.subtitle_state.selected = Some(family);
                    Task::none()
                }
            },
            SettingsMessage::FolderSelected(folder) => {
                let Some(folder) = folder else {
                    return Task::none();
                };

                self.view = Some(View::FolderSelection {
                    path: folder,
                    kind: MediaType::Movies,
                });

                self.update_scroll()
            }
            SettingsMessage::FolderSelection(fsg) => match fsg {
                FolderSelectionMessage::Cancel => self.cancel(),
                FolderSelectionMessage::Reselect => pick_task(),
                FolderSelectionMessage::Kind(new) => {
                    if let Some(View::FolderSelection { kind, .. }) = self.view.as_mut() {
                        *kind = new;
                    }

                    Task::none()
                }
                FolderSelectionMessage::Submit => {
                    let Some(View::FolderSelection { path, kind }) = self.view.take() else {
                        return self.update_scroll();
                    };

                    let path = path.canonicalize().unwrap().display().to_string();
                    let path = path
                        .strip_prefix(r"\\?\")
                        .map(ToOwned::to_owned)
                        .unwrap_or(path);

                    if self.directories.iter().any(|dir| dir.dir.path == path) {
                        return self.update_scroll();
                    }

                    let dir = Directory::new(
                        path,
                        kind,
                        true,
                        self.config.general.default_source.to_str().to_owned(),
                    );

                    let dir = Dir::new(dir);

                    self.directories.push(dir);
                    self.directories_shown = true;

                    self.update_scroll()
                }
            },
            SettingsMessage::ClearAllBindings(action) => {
                match action {
                    KeyAction::General(Some(action)) => {
                        self.config.keystore.clear_home(action);
                    }
                    KeyAction::Video(Some(action)) => {
                        self.config.keystore.clear_player(action);
                    }
                    KeyAction::Settings(Some(action)) => {
                        self.config.keystore.clear_settings(action);
                    }
                    _ => {}
                }

                Task::none()
            }
            SettingsMessage::ClearBinding(page, keypress) => {
                match page {
                    Page::General => {
                        self.config.keystore.remove_home(keypress);
                    }
                    Page::Video => {
                        self.config.keystore.remove_player(keypress);
                    }
                    Page::Keybinds => {
                        self.config.keystore.remove_settings(keypress);
                    }
                }

                Task::none()
            }
            SettingsMessage::NewKeyPress(action) => {
                self.view = Some(View::CaptureKey {
                    conflict: action.none(),
                    action,
                    key: None,
                });
                Task::batch([Task::done(Message::CaptureKeys(true)), self.update_scroll()])
            }
            SettingsMessage::KeyAction(action) => {
                if let Some(View::CaptureKey { action: old, .. }) = self.view.as_mut() {
                    *old = action
                }

                Task::none()
            }
            SettingsMessage::SaveKeyBinding => {
                let Some(View::CaptureKey {
                    action,
                    key,
                    conflict: _unused,
                }) = self.view.take()
                else {
                    return Task::done(Message::CaptureKeys(false));
                };

                match action {
                    KeyAction::General(action) => {
                        if let Some((action, key)) = action.zip(key) {
                            self.config.keystore.insert_home(key, action);
                        }
                    }
                    KeyAction::Video(action) => {
                        if let Some((action, key)) = action.zip(key) {
                            self.config.keystore.insert_player(key, action);
                        }
                    }
                    KeyAction::Settings(action) => {
                        if let Some((action, key)) = action.zip(key) {
                            self.config.keystore.insert_settings(key, action);
                        }
                    }
                }

                Task::batch([
                    Task::done(Message::CaptureKeys(false)),
                    self.update_scroll(),
                ])
            }
            SettingsMessage::OpenLog => {
                let Some(path) = self.config.log_path() else {
                    return Task::none();
                };

                match open::that(path) {
                    Ok(_) => Task::none(),
                    Err(error) => Task::done(Message::error(error, true)),
                }
            }
            SettingsMessage::OpenConfig => {
                let Some(path) = self.config.config_path() else {
                    return Task::none();
                };

                match open::that(path) {
                    Ok(_) => Task::none(),
                    Err(error) => Task::done(Message::error(error, true)),
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let content = container(
            row!(self.side(), self.content_area())
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(4),
        );

        match &self.view {
            None => content.into(),
            Some(View::FolderSelection { path, kind }) => {
                let overlay = draw_folder_selection(path, kind);

                modal(content, overlay, SettingsMessage::Cancel)
            }
            Some(View::CaptureKey {
                action,
                key,
                conflict,
            }) => {
                let overlay = draw_capture_key(action, key, conflict);

                modal(content, overlay, SettingsMessage::Cancel)
            }
        }
    }

    fn side(&self) -> Element<'_, SettingsMessage> {
        let header = {
            let text = h4("Settings");

            container(text).padding([5, 10]).align_y(Vertical::Center)
        };

        let options = column!(
            side_button(
                "General",
                SettingsMessage::Goto(Page::General),
                matches!(self.page, Page::General)
            ),
            side_button(
                "Video Player",
                SettingsMessage::Goto(Page::Video),
                matches!(self.page, Page::Video)
            ),
            side_button(
                "Key Bindings",
                SettingsMessage::Goto(Page::Keybinds),
                matches!(self.page, Page::Keybinds)
            ),
        )
        .spacing(20);

        let content = column!(header, space::vertical().height(20.0), options)
            .padding(iced::Padding::ZERO.left(12))
            .width(275.0)
            .height(Length::Fill);

        let content = container(content).style(|theme| {
            let default = styles::container::bw3(theme);
            let border = default.border.rounded(2.5);

            container::Style { border, ..default }
        });

        content.into()
    }

    fn content_area(&self) -> Element<'_, SettingsMessage> {
        let title = match self.page {
            Page::General => "General Settings",
            Page::Video => "Video Settings",
            Page::Keybinds => "Keybindings",
        };

        let title = container(h6(title)).height(28.0).center_x(Length::Fill);

        let content: Element<'_, SettingsMessage> = match self.page {
            Page::General => self.general(),
            Page::Video => self.video(),
            Page::Keybinds => self.keybinds(),
        };

        let scroll = match self.page {
            Page::General => self.scroll_state.general.id.clone(),
            Page::Video => self.scroll_state.video.id.clone(),
            Page::Keybinds => self.scroll_state.keybinds.id.clone(),
        };

        let top = column!(title, horizontal_rule(), space::vertical().height(10.0));

        let content = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(24.0)
            .id(scroll)
            .on_scroll(SettingsMessage::Scroll);

        let actions = {
            let save = save_btn().on_press(SettingsMessage::Save);
            let cancel = cancel_btn().on_press(SettingsMessage::Cancel);

            let actions = row!(save, cancel).spacing(100.0).align_y(Vertical::Center);

            container(actions)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
        };

        let content = column!(top, content, actions)
            .height(Length::Fill)
            .spacing(16)
            .padding([40, 80]);

        container(content)
            .clip(true)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn general(&self) -> Element<'_, SettingsMessage> {
        let size = TEXT_SIZE;

        let GeneralSettings {
            layout,
            refresh_interval,
            recents_limit,
            search_limit,
            theme,
            theme_iced: _unused,
            scan_discoverer,
            auth_token,
            movie_depth,
            fetching_interval,
            restore_deleted,
            tmdb_rating,
            preferred_subtitle_codec,
            preferred_audio_codec,
            default_source,
            show_dirs,
        } = &self.config.general;

        let general = draw_general(refresh_interval, recents_limit, search_limit, *show_dirs)
            .map(SettingsMessage::General);

        let appearance = draw_appearance(layout, theme).map(SettingsMessage::Appearance);

        let media = draw_media(
            &self.directories,
            self.directories_shown,
            *scan_discoverer,
            *restore_deleted,
            *movie_depth,
            preferred_subtitle_codec,
            preferred_audio_codec,
        )
        .map(SettingsMessage::Media);

        let metadata = draw_metadata(auth_token, *default_source, fetching_interval, *tmdb_rating)
            .map(SettingsMessage::Metadata);

        let open = {
            let config = {
                let label = label_maker("Config File");
                let icon = icons::icon(icons::EXTERNAL).size(size / RATIO);

                let label = row!(label, icon).spacing(4).align_y(Vertical::Center);

                button(label)
                    .padding(0)
                    .on_press(SettingsMessage::OpenConfig)
                    .style(styles::button::text_primary)
            };

            let log = {
                let label = label_maker("Log File");
                let icon = icons::icon(icons::EXTERNAL).size(size / RATIO);

                let label = row!(label, icon).spacing(4).align_y(Vertical::Center);

                button(label)
                    .padding(0)
                    .on_press(SettingsMessage::OpenLog)
                    .style(styles::button::text_primary)
            };

            column!(config, log).spacing(12)
        };

        let content = column!(general, appearance, media, metadata, open,)
            .spacing(36.0)
            .height(Length::Fill);

        let content = center_x(content);

        content.into()
    }

    fn video(&self) -> Element<'_, SettingsMessage> {
        let VideoSettings {
            thumbnail_interval,
            volume,
            speed,
            volume_change_amt,
            seek_change_amt,
            seek_shift_change_amt,
            speed_change_amt,
            show_subtitles,
            auto_start,
            auto_next,
            completion_point,
            completion_watch_time,
            subtitles,
            // I cannot think of a reason why these should persist here
            // plus I'm lazy
            muted: _mute,
            filters,
            comment_span,
        } = &self.config.video;

        let playback = draw_playback(
            *volume,
            *volume_change_amt,
            *speed,
            *speed_change_amt,
            *auto_start,
            *auto_next,
            *completion_point,
            *completion_watch_time,
        )
        .map(SettingsMessage::Playback);

        let seeking = draw_seeking(
            *thumbnail_interval,
            *seek_change_amt,
            *seek_shift_change_amt,
            *comment_span,
        )
        .map(SettingsMessage::Seeking);

        let filters = draw_filters(filters).map(SettingsMessage::VideoFilters);

        let subtitles = draw_subtitles(
            *show_subtitles,
            &self.subtitle_state,
            subtitles,
            &self.subtitle_dummy,
            &self.text_color,
            &self.background_color,
        )
        .map(SettingsMessage::Subtitles);

        let content = column!(playback, seeking, filters, subtitles,).spacing(24);

        let content = center_x(content);

        content.into()
    }

    fn keybinds(&self) -> Element<'_, SettingsMessage> {
        let spacing = 10.0;

        let home = {
            let names = table::column(
                table_header("NAME"),
                |(action, _): (&HomeAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("DESCRIPTION"),
                |(action, _): (&HomeAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("KEYBINDING"),
                |(_, keys): (&HomeAction, &Vec<KeyPress>)| table_keys(Page::General, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.home());

            let title = section_label("GENERAL");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::General(None)))
                .style(styles::button::text_primary);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(spacing)
        };

        let player = {
            let names = table::column(
                table_header("NAME"),
                |(action, _): (&PlayerAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("DESCRIPTION"),
                |(action, _): (&PlayerAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("KEYBINDING"),
                |(_, keys): (&PlayerAction, &Vec<KeyPress>)| table_keys(Page::Video, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.player());

            let title = section_label("PLAYBACK");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::Video(None)))
                .style(styles::button::text_primary);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(spacing)
        };

        let settings = {
            let names = table::column(
                table_header("NAME"),
                |(action, _): (&SettingsAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("DESCRIPTION"),
                |(action, _): (&SettingsAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("KEYBINDING"),
                |(_, keys): (&SettingsAction, &Vec<KeyPress>)| table_keys(Page::Keybinds, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.settings());

            let title = section_label("SETTINGS");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::Settings(None)))
                .style(styles::button::text_primary);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(spacing)
        };

        let content = column!(home, player, settings)
            .spacing(48.0)
            .height(Length::Fill);

        content.into()
    }

    fn update_scroll(&mut self) -> Task<Message> {
        match self.page {
            Page::General => operation::scroll_to(
                self.scroll_state.general.id.clone(),
                self.scroll_state.general.offset,
            ),
            Page::Video => operation::scroll_to(
                self.scroll_state.video.id.clone(),
                self.scroll_state.video.offset,
            ),
            Page::Keybinds => operation::scroll_to(
                self.scroll_state.keybinds.id.clone(),
                self.scroll_state.keybinds.offset,
            ),
        }
    }

    fn cancel(&mut self) -> Task<Message> {
        match self.view.take() {
            None => Task::done(Message::Back),
            Some(View::FolderSelection { .. }) => self.update_scroll(),
            Some(View::CaptureKey { .. }) => Task::batch([
                Task::done(Message::CaptureKeys(false)),
                self.update_scroll(),
            ]),
        }
    }

    fn goto(&mut self, page: Page) -> Task<Message> {
        self.page = page;

        self.update_scroll()
    }

    fn walk_up(&mut self) -> Task<Message> {
        let new = match self.page {
            Page::General => Page::Keybinds,
            Page::Video => Page::General,
            Page::Keybinds => Page::Video,
        };

        self.goto(new)
    }

    fn walk_down(&mut self) -> Task<Message> {
        let new = match self.page {
            Page::General => Page::Video,
            Page::Video => Page::Keybinds,
            Page::Keybinds => Page::General,
        };

        self.goto(new)
    }

    pub fn captured_key(&mut self, key: KeyPress) -> Task<Message> {
        if let Some(View::CaptureKey {
            key: old,
            action,
            conflict,
        }) = self.view.as_mut()
        {
            match action {
                KeyAction::Video(new) => {
                    let new = new.as_ref();
                    let current = self.config.keystore.get_player(&key);

                    *conflict = if current.is_some() && current != new {
                        KeyAction::Video(current.cloned())
                    } else {
                        action.none()
                    };
                }
                KeyAction::General(new) => {
                    let new = new.as_ref();
                    let current = self.config.keystore.get_home(&key);

                    *conflict = if current.is_some() && current != new {
                        KeyAction::General(current.cloned())
                    } else {
                        action.none()
                    };
                }
                KeyAction::Settings(new) => {
                    let new = new.as_ref();
                    let current = self.config.keystore.get_settings(&key);

                    *conflict = if current.is_some() && current != new {
                        KeyAction::Settings(current.cloned())
                    } else {
                        action.none()
                    };
                }
            };

            *old = Some(key);
        }

        Task::none()
    }

    pub fn action(&mut self, action: SettingsAction) -> Task<Message> {
        match action {
            SettingsAction::Cancel => self.cancel(),
            SettingsAction::Up => self.walk_up(),
            SettingsAction::Down => self.walk_down(),
        }
    }

    pub fn fetched_directories(&mut self, dirs: Vec<Directory>) {
        self.directories.extend(dirs.into_iter().map(Dir::fetched));
    }

    pub fn save(
        self,
    ) -> (
        Config,
        impl Iterator<Item = (Directory, Option<Operation>, bool, bool)>,
    ) {
        let directories = self.directories.into_iter().filter_map(|dir| dir.save());
        let config = self.config;

        (config, directories)
    }
}

fn side_button<'a>(
    value: &'a str,
    message: SettingsMessage,
    current: bool,
) -> Element<'a, SettingsMessage> {
    let text = bold(value);

    container(
        button(text)
            .width(Length::Fill)
            .style(move |theme, status| {
                if current {
                    styles::button::background_primary(theme, status)
                } else {
                    styles::button::subtlest(theme, status)
                }
            })
            .on_press(message),
    )
    .clip(true)
    .max_height(48.0)
    .into()
}

fn help<'a, Message: 'a>(label: &'a str) -> Tooltip<'a, Message> {
    use iced::widget::tooltip::Position;

    tooltip(
        icons::icon(icons::HELP).size(TEXT_SIZE / RATIO),
        label,
        Position::Right,
    )
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}

fn pick_task() -> Task<Message> {
    Task::perform(pick_folder(), |folder| {
        Message::Settings(SettingsMessage::FolderSelected(folder))
    })
}

fn draw_folder_selection<'a>(path: &'a Path, kind: &'a MediaType) -> Element<'a, SettingsMessage> {
    let size = TEXT_SIZE;

    let folder = {
        let path = trim_path(path, 3);

        let path = marquee(path).font(mono_font()).size(size / RATIO);

        let label = sized_bold("Folder: ", size).width(100.0);

        let folder = row!(label, path).align_y(Vertical::Center).spacing(8);

        let reselect = sized_button(icons::REPLAY, size / RATIO)
            .on_press(SettingsMessage::FolderSelection(
                FolderSelectionMessage::Reselect,
            ))
            .style(styles::button::subtle);

        row!(folder, reselect).align_y(Vertical::Center).spacing(12)
    };

    let kind = {
        let handle = picklist_handle(size);
        let label = sized_bold("Media type: ", size).width(100.0);

        let lst = pick_list(Some(*kind), MediaType::ALL, ToString::to_string)
            .style(styles::pick_list::default)
            .font(regular_font())
            .on_select(|kind| SettingsMessage::FolderSelection(FolderSelectionMessage::Kind(kind)))
            .padding([2, 5])
            .handle(handle)
            .text_size(size);

        row!(label, lst).align_y(Vertical::Center).spacing(12)
    };

    let actions = {
        let submit = save_btn().on_press(SettingsMessage::FolderSelection(
            FolderSelectionMessage::Submit,
        ));

        let cancel = cancel_btn().on_press(SettingsMessage::FolderSelection(
            FolderSelectionMessage::Cancel,
        ));

        center_x(row!(submit, cancel).align_y(Vertical::Center).spacing(36))
    };

    let content = column!(folder, kind, actions).spacing(20);

    modal_container(content)
        .max_width(450)
        .padding([12, 16])
        .into()
}

fn draw_capture_key<'a>(
    action: &'a KeyAction,
    keypress: &'a Option<KeyPress>,
    conflict: &'a KeyAction,
) -> Element<'a, SettingsMessage> {
    let size = TEXT_SIZE / RATIO;
    let key = match keypress {
        Some(keypress) => {
            let keypress = table_key(keypress);
            container(keypress)
        }
        None => container("Press a Key"),
    }
    .center_x(Length::Fill)
    .center_y(40)
    .height(40)
    .width(Length::Fill)
    .padding([2, 4])
    .style(|theme: &Theme| {
        let color = theme.palette().secondary.strong.color;
        let default = styles::container::transparent(theme);
        let border = default.border.rounded(5).color(color).width(1.5);

        container::Style { border, ..default }
    });

    let has_conflict = conflict.is_some();

    let conflict: Element<'_, SettingsMessage> = match conflict {
        KeyAction::General(Some(action)) => medium(format!("Conflicts with {action}"))
            .size(size / RATIO)
            .style(|theme| {
                let color = theme.palette().danger.base.color;

                text::Style { color: Some(color) }
            })
            .into(),
        KeyAction::Video(Some(action)) => medium(format!("Conflicts with {action}"))
            .size(size / RATIO)
            .style(|theme| {
                let color = theme.palette().danger.base.color;

                text::Style { color: Some(color) }
            })
            .into(),
        KeyAction::Settings(Some(action)) => medium(format!("Conflicts with {action}"))
            .size(size / RATIO)
            .style(|theme| {
                let color = theme.palette().danger.base.color;

                text::Style { color: Some(color) }
            })
            .into(),
        _ => empty(),
    };

    let key = column!(label_maker("Key Press").size(size), key).spacing(4.0);

    let key = if has_conflict {
        key.push(conflict)
    } else {
        key
    };

    let (action, set_action) = {
        let label = label_maker("Action").size(size);
        let padding = [5, 5];

        let (lst, set): (Element<'_, SettingsMessage>, bool) = match action {
            KeyAction::General(selected) => (
                pick_list(*selected, HomeAction::VARIANTS, ToString::to_string)
                    .style(styles::pick_list::default)
                    .font(regular_font())
                    .on_select(|action| {
                        SettingsMessage::KeyAction(KeyAction::General(Some(action)))
                    })
                    .handle(picklist_handle(size))
                    .padding(padding)
                    .text_size(size)
                    .into(),
                selected.is_some(),
            ),
            KeyAction::Video(selected) => (
                pick_list(*selected, PlayerAction::VARIANTS, ToString::to_string)
                    .style(styles::pick_list::default)
                    .on_select(|action| SettingsMessage::KeyAction(KeyAction::Video(Some(action))))
                    .font(regular_font())
                    .handle(picklist_handle(size))
                    .padding(padding)
                    .text_size(size)
                    .into(),
                selected.is_some(),
            ),
            KeyAction::Settings(selected) => (
                pick_list(*selected, SettingsAction::VARIANTS, ToString::to_string)
                    .style(styles::pick_list::default)
                    .font(regular_font())
                    .on_select(|action| {
                        SettingsMessage::KeyAction(KeyAction::Settings(Some(action)))
                    })
                    .handle(picklist_handle(size))
                    .padding(padding)
                    .text_size(size)
                    .into(),
                selected.is_some(),
            ),
        };

        (column!(label, lst).spacing(4.0), set)
    };

    let set = set_action && keypress.is_some();

    let btns = {
        let save = save_btn().on_press_maybe(set.then_some(SettingsMessage::SaveKeyBinding));
        let cancel = cancel_btn().on_press(SettingsMessage::Cancel);

        let actions = row!(save, cancel).spacing(80.0).align_y(Vertical::Center);

        container(actions)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
    };

    let content = column!(key, action, btns).spacing(16);

    modal_container(content).width(300).into()
}

fn label_maker<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sized_medium(label, TEXT_SIZE)
}

fn section_label<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sized_bold(label, TEXT_SIZE * RATIO)
}

fn table_header<'a>(label: &'a str) -> text::Text<'a> {
    sized_medium(label, TEXT_SIZE / RATIO)
}

fn table_name<'a>(
    label: impl text::IntoFragment<'a>,
    action: KeyAction,
) -> Element<'a, SettingsMessage> {
    let clear = icons::text_button(icons::CANCEL)
        .padding(0)
        .on_press(SettingsMessage::ClearAllBindings(action));
    let clear = binding_tooltip(clear, "Remove all bindings");
    let label = sized_medium(label, TEXT_SIZE / RATIO);

    row!(label, space::horizontal(), clear)
        .align_y(Vertical::Center)
        .into()
}

fn table_description<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sized_regular(label, TEXT_SIZE)
}

fn table_keys<'a>(page: Page, keys: &[KeyPress]) -> Element<'a, SettingsMessage> {
    let keys = keys.iter().map(|key| {
        let key = button(table_key(key))
            .padding(0)
            .style(styles::button::text)
            .on_press(SettingsMessage::ClearBinding(page, key.clone()));

        binding_tooltip(key, "Remove binding")
    });

    row(keys).spacing(6).width(Length::Fill).wrap().into()
}

fn table_key<'a>(key: &KeyPress) -> Element<'a, SettingsMessage> {
    let content = mono(key.to_string()).size(TEXT_SIZE);

    let content = container(content)
        .padding(5)
        .style(|theme| {
            let default = styles::container::bordered(theme);
            let border = default.border.rounded(5).width(1.5);

            container::Style { border, ..default }
        })
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center);

    content.into()
}

fn binding_tooltip<'a>(
    content: impl Into<Element<'a, SettingsMessage>>,
    label: &'a str,
) -> Element<'a, SettingsMessage> {
    use crate::utils::tooltip;
    use iced::widget::tooltip::Position;

    tooltip(content, label, Position::Top).into()
}

fn draw_subtitles<'a>(
    show_subtitles: bool,
    state: &'a FontState,
    subtitles: &'a SubtitleDescription,
    subtitle_dummy: &'a str,
    text_color: &'a str,
    background_color: &'a str,
) -> Element<'a, SubtitleMessage> {
    let color_width = 150.0;

    let dummy = utils::draw_subtitles(subtitle_dummy, subtitles);

    let subtitles_toggle = {
        let label = label_maker("Show subtitles ");
        let label = button(label)
            .padding(0)
            .on_press(SubtitleMessage::ToggleSubtitles)
            .style(styles::button::text);

        let toggle = toggler(show_subtitles).on_toggle(SubtitleMessage::Subtitles);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let sub_size = {
        let label = label_maker("Size ");

        let amt = format!("{}", subtitles.size);

        let input = text_input("", &amt)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SubtitleMessage::SubSize);

        let actions = input_actions(SubtitleMessage::SubSizeIncr, SubtitleMessage::SubSizeDecr);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let color = {
        let label = label_maker("Text Color (rgba) ");

        let input = text_input("", text_color)
            .width(color_width)
            .size(TEXT_SIZE)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SubtitleMessage::SubColor);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let background = {
        let label = label_maker("Background Color (rgba) ");

        let input = text_input("", background_color)
            .width(color_width)
            .size(TEXT_SIZE)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SubtitleMessage::SubBackground);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let dummy_input = {
        let label = label_maker("Subtitle Example ");

        let input = text_input("", subtitle_dummy)
            .width(256)
            .size(TEXT_SIZE)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SubtitleMessage::Dummy);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let font = {
        let label = label_maker("Font ");

        let selection = combo_box(
            &state.state,
            "",
            state.selected.as_ref(),
            SubtitleMessage::Font,
        )
        .width(256)
        .size(TEXT_SIZE)
        .font(regular_font());

        row!(label, space::horizontal(), selection).align_y(Vertical::Center)
    };

    let content = column!(
        subtitles_toggle,
        horizontal_rule(),
        dummy_input,
        horizontal_rule(),
        font,
        horizontal_rule(),
        sub_size,
        horizontal_rule(),
        color,
        horizontal_rule(),
        background,
        horizontal_rule(),
    )
    .spacing(10.0);

    let content = column!(content, dummy)
        .align_x(Horizontal::Center)
        .spacing(20);

    section("Subtitles", content)
}

fn section<'a, Message: 'a>(
    label: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let header = section_label(label);

    let content = container(content)
        .padding(PADDING)
        .style(styles::container::bw3);

    column!(header, content).spacing(SPACING).into()
}

fn draw_general<'a>(
    refresh_interval: &Duration,
    recents_limit: &Option<i32>,
    search_limit: &Option<i32>,
    show_dirs: bool,
) -> Element<'a, GeneralMessage> {
    let refresh_interval = {
        let label = label_maker("Refresh Interval(seconds) ");
        let icon = help("How often to scan for content changes in seconds");
        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let interval = refresh_interval.as_secs().to_string();
        let input = text_input("Interval in seconds", &interval)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .on_input(GeneralMessage::Refresh)
            .align_x(Horizontal::Right);

        let actions = input_actions(GeneralMessage::IncrRefresh, GeneralMessage::DecrRefresh);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let recents_limit = {
        let label = label_maker("Recents Limit ");
        let icon = help("Number of recent media items to display");

        let label = row!(label, icon).spacing(3).align_y(Vertical::Center);

        let recents = recents_limit
            .map(|limit| limit.to_string())
            .unwrap_or_default();
        let input = text_input("", &recents)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .align_x(Horizontal::Right)
            .on_input(GeneralMessage::Recents);

        let actions = input_actions(GeneralMessage::IncrRecents, GeneralMessage::DecrRecents);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let search_limit = {
        let label = label_maker("Search results limit ");
        let icon = help("Number of search results to display");

        let label = row!(label, icon).spacing(3).align_y(Vertical::Center);

        let searches = search_limit
            .map(|limit| limit.to_string())
            .unwrap_or_default();
        let input = text_input("", &searches)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .align_x(Horizontal::Right)
            .on_input(GeneralMessage::Search);

        let actions = input_actions(GeneralMessage::IncrSearch, GeneralMessage::DecrSearch);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let show_dirs = {
        let label = label_maker("Show Directories on sidebar ");

        let label = button(label)
            .padding(0)
            .on_press(GeneralMessage::ToggleShowDirs)
            .style(styles::button::text);

        let toggle = toggler(show_dirs).on_toggle(GeneralMessage::ShowDirs);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let content = column!(
        refresh_interval,
        horizontal_rule(),
        recents_limit,
        horizontal_rule(),
        search_limit,
        horizontal_rule(),
        show_dirs,
    )
    .spacing(SECTION_SPACING);

    section("General", content)
}

fn draw_appearance<'a>(layout: &'a Layout, theme: &'a AppTheme) -> Element<'a, AppearanceMessage> {
    let layouts = {
        let handle = picklist_handle(TEXT_SIZE);
        let label = label_maker("Content layout ");

        let layouts = pick_list(Some(*layout), Layout::VARIANTS, ToString::to_string)
            .font(regular_font())
            .on_select(AppearanceMessage::Layout)
            .handle(handle.clone())
            .padding(LIST_PADDING)
            .text_size(TEXT_SIZE)
            .style(styles::pick_list::default);

        row!(label, space::horizontal(), layouts).align_y(Vertical::Center)
    };

    let theme = {
        let handle = picklist_handle(TEXT_SIZE);

        let label = label_maker("Theme ");

        let theme = pick_list(Some(*theme), AppTheme::VARIANTS, ToString::to_string)
            .font(regular_font())
            .on_select(AppearanceMessage::Theme)
            .handle(handle.clone())
            .padding(LIST_PADDING)
            .text_size(TEXT_SIZE)
            .style(styles::pick_list::default);

        row!(label, space::horizontal(), theme).align_y(Vertical::Center)
    };

    let content = column!(layouts, horizontal_rule(), theme,).spacing(SECTION_SPACING);

    section("Appearance", content)
}

fn draw_media<'a>(
    directories: &'a [Dir],
    directories_shown: bool,
    scan_discoverer: bool,
    restore_deleted: bool,
    movie_depth: u8,
    preferred_sub: &Option<String>,
    preferred_audio: &Option<String>,
) -> Element<'a, MediaMessage> {
    let dirs = {
        let top = {
            let label = label_maker("Media Directories");

            let size = TEXT_SIZE / RATIO;

            let add = button(
                row!(
                    icons::icon(icons::FOLDER_ADD).size(size * RATIO),
                    sized_medium("Add Folder", size)
                )
                .spacing(4.0)
                .align_y(Vertical::Center),
            )
            .padding([3, 6])
            .style(styles::button::text_primary)
            .on_press(MediaMessage::AddFolder);

            let scan = button(
                row!(
                    icons::icon(icons::REFRESH).size(size * RATIO),
                    sized_medium("Scan All", size)
                )
                .spacing(4.0)
                .align_y(Vertical::Center),
            )
            .padding([3, 6])
            .style(styles::button::text_primary)
            .on_press(MediaMessage::ScanAll);

            let icon = if directories_shown {
                icons::CHEV_UP
            } else {
                icons::CHEV_DOWN
            };
            let icon = icons::icon(icon).size(TEXT_SIZE);

            let right = row!(add, scan, icon).spacing(6.0).align_y(Vertical::Center);

            row!(label, space::horizontal(), right).align_y(Vertical::Center)
        };

        let dirs = {
            let size = TEXT_SIZE;
            let header_size = size / RATIO;

            let kind = table::column(table_header("Media").size(header_size), |dir: &Dir| {
                let tag = regular(dir.dir.media_type.to_string()).size(size / (RATIO * RATIO));

                button(tag)
                    .padding([2, 5])
                    .style(|theme, status| {
                        let default = styles::button::text_primary(theme, status);
                        let border = default
                            .border
                            .rounded(3.0)
                            .color(default.text_color)
                            .width(0.75);

                        button::Style { border, ..default }
                    })
                    .on_press(MediaMessage::ToggleDirKind(dir.dir.id))
            })
            .align_y(Vertical::Center);

            let source = table::column(table_header("Source").size(header_size), |dir: &Dir| {
                let handle = picklist_handle(size);
                let source = SourceSet::from_str(&dir.dir.source);
                let id = dir.dir.id;

                pick_list(Some(source), SourceSet::VARIANTS, |source| {
                    source.to_str().to_owned()
                })
                .font(regular_font())
                .on_select(move |source| MediaMessage::DirSource(id, source))
                .handle(handle)
                .padding(LIST_PADDING)
                .text_size(TEXT_SIZE)
                .style(styles::pick_list::default)
            })
            .align_y(Vertical::Center);

            let path = table::column(table_header("Path").size(header_size), |dir: &Dir| {
                let path = trim_path(Path::new(&dir.dir.path), 4);

                let path = span(path)
                    .strikethrough(matches!(dir.operation, Operation::Delete))
                    .font(mono_font())
                    .size(size / RATIO);

                container(rich_text([path]).on_link_click(|_: ()| MediaMessage::None)).clip(true)
            })
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let last = table::column(
                table_header("Last scanned").size(header_size),
                |dir: &Dir| {
                    let last = if dir.toggled.is_some() {
                        humanize_datetime(dir.dir.last_scan, chrono::Local::now())
                    } else {
                        "--:--".to_owned()
                    };

                    sized_italic(last, size / RATIO)
                },
            )
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

            let scan = table::column(table_header("Scan").size(header_size), |dir: &Dir| {
                checkbox(dir.scan).on_toggle(|scan| MediaMessage::Scan(dir.dir.id, scan))
            })
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

            let add = table::column(table_header("Add/Remove").size(header_size), |dir: &Dir| {
                let deleted = matches!(dir.operation, Operation::Delete);
                let icon = if deleted { icons::ADD } else { icons::DELETE };

                let icon = icons::icon(icon).size(size);

                button(icon)
                    .padding([6, 6])
                    .on_press(MediaMessage::ToggleDirectoryAdd(dir.dir.id))
                    .style(move |theme, status| {
                        if deleted {
                            styles::button::text_primary(theme, status)
                        } else {
                            styles::button::text_danger(theme, status)
                        }
                    })
            })
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

            table([kind, scan, source, path, last, add], directories)
        };

        expandable(top, dirs)
            .expanded(directories_shown)
            .on_expand(MediaMessage::ToggleDirShow)
            .spacing(6.0)
    };

    let discoverer = {
        let label = label_maker("Video Discovery on Scan ");
        let icon = help(
            "Whether more information is collected on videos while scanning directories. Scanning is slower as a result ",
        );
        let label = button(label)
            .padding(0)
            .on_press(MediaMessage::ToggleScanDiscover)
            .style(styles::button::text);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let toggle = toggler(scan_discoverer).on_toggle(MediaMessage::ScanDiscoverer);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let restore_deletes = {
        let label = label_maker("Restore deleted media on scan ");
        let icon = help("Whether deleted media is restored when its directory is scanned");

        let label = button(label)
            .padding(0)
            .on_press(MediaMessage::ToggleRestore)
            .style(styles::button::text);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let toggle = toggler(restore_deleted).on_toggle(MediaMessage::RestoreDeleted);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let movie_depth = {
        let label = label_maker("Movie Directory Depth ");
        let icon = help("How deep scans for videos in movie directories should be.");

        let depth = movie_depth.to_string();
        let input = text_input("Depth", &depth)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .on_input(MediaMessage::MovieDepth)
            .align_x(Horizontal::Right);

        let actions = input_actions(MediaMessage::IncrMovieDepth, MediaMessage::DecrMovieDepth);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let preferred_sub = {
        let label = label_maker("Preferred Subtitle Language ");
        let icon = help(
            "Specify a subtitle language code to prefer for embedded subtitle tracks (eg 'en', 'fr'). If not set or unavailable, a neighboring subtitle file with the same name will be used instead.",
        );

        let preferred = preferred_sub.as_deref().unwrap_or_default();
        let input = text_input("", preferred)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .on_input(MediaMessage::PreferredSub)
            .align_x(Horizontal::Right);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let preferred_audio = {
        let label = label_maker("Preferred Audio Language ");
        let icon = help(
            "Specify an audio language code to prefer for embedded audio tracks (eg 'en', 'fr').",
        );

        let preferred = preferred_audio.as_deref().unwrap_or_default();
        let input = text_input("", preferred)
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .font(regular_font())
            .padding(INPUT_PADDING)
            .on_input(MediaMessage::PreferredAudio)
            .align_x(Horizontal::Right);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let content = column!(
        dirs,
        horizontal_rule(),
        movie_depth,
        horizontal_rule(),
        discoverer,
        horizontal_rule(),
        restore_deletes,
        horizontal_rule(),
        preferred_audio,
        horizontal_rule(),
        preferred_sub
    )
    .spacing(SECTION_SPACING);

    section("Media & Scanning", content)
}

fn draw_metadata<'a>(
    auth_token: &str,
    default_source: SourceSet,
    fetching_interval: &Duration,
    tmdb_rating: bool,
) -> Element<'a, MetadataMessage> {
    let fetching_interval = {
        let label = label_maker("TMDB fetch Interval(seconds) ");
        let icon = help(
            "How often to scrape TMDB© for new media data in seconds. Note this only takes effect on restart.",
        );

        let interval = fetching_interval.as_secs().to_string();
        let input = text_input("Interval in seconds", &interval)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .padding(INPUT_PADDING)
            .on_input(MetadataMessage::Fetch)
            .align_x(Horizontal::Right);

        let actions = input_actions(MetadataMessage::IncrFetch, MetadataMessage::DecrFetch);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let tmdb_rating = {
        let label = label_maker("Use TMDB ratings ");
        let icon = help("Use TMDB ratings as a default when fetching media metadata");
        let label = button(label)
            .padding(0)
            .on_press(MetadataMessage::ToggleTMDBRating)
            .style(styles::button::text);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let toggle = toggler(tmdb_rating).on_toggle(MetadataMessage::TMDBRating);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let auth = {
        let label = label_maker("TMDB API Token ");
        let icon = help("The TMDB© API Read Access Token used for fetching media metadata");

        let input = text_input("Token", auth_token)
            .font(mono_font())
            .width(475)
            .size(TEXT_SIZE)
            .padding(INPUT_PADDING)
            .on_input(MetadataMessage::Auth);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let source = {
        let handle = picklist_handle(TEXT_SIZE);
        let label = label_maker("Default Source ");

        let sources = pick_list(Some(default_source), SourceSet::VARIANTS, |source| {
            source.to_str().to_owned()
        })
        .font(regular_font())
        .on_select(MetadataMessage::Source)
        .handle(handle.clone())
        .padding(LIST_PADDING)
        .text_size(TEXT_SIZE)
        .style(styles::pick_list::default);

        row!(label, space::horizontal(), sources).align_y(Vertical::Center)
    };

    let content = column!(
        auth,
        horizontal_rule(),
        source,
        horizontal_rule(),
        fetching_interval,
        horizontal_rule(),
        tmdb_rating,
    )
    .spacing(SECTION_SPACING);

    section("Metadata", content)
}

#[allow(clippy::too_many_arguments)]
fn draw_playback<'a>(
    volume: f64,
    volume_change_amt: f64,
    speed: f64,
    speed_change_amt: f64,
    auto_start: bool,
    auto_next: bool,
    completion_point: f64,
    completion_watch_time: f64,
) -> Element<'a, PlaybackMessage> {
    let volume = {
        let label = label_maker("Default Volume ");

        let value = sized_regular(format!("{volume:.2}"), TEXT_SIZE / RATIO);
        let volume = slider(0.0..=1.0, volume, PlaybackMessage::Volume)
            .step(0.05)
            .shift_step(0.1)
            .width(SLIDER_WIDTH);

        let volume = row!(value, volume)
            .align_y(Vertical::Center)
            .spacing(SLIDER_SPACING);

        row!(label, space::horizontal(), volume).align_y(Vertical::Center)
    };

    let volume_amt = {
        let label = label_maker("Volume amount ");
        let icon = help("Amount the volume changes by");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", volume_change_amt);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(PlaybackMessage::VolumeAmt);

        let actions = input_actions(PlaybackMessage::IncrVolAmt, PlaybackMessage::DecrVolAmt);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let speed = {
        let label = label_maker("Default Speed ");

        let value = sized_regular(format!("{speed:.2}"), TEXT_SIZE / RATIO);
        let speed = slider(0.5..=2.5, speed, PlaybackMessage::Speed)
            .step(0.1)
            .shift_step(0.2)
            .width(SLIDER_WIDTH);

        let speed = row!(value, speed)
            .align_y(Vertical::Center)
            .spacing(SLIDER_SPACING);

        row!(label, space::horizontal(), speed).align_y(Vertical::Center)
    };

    let speed_amt = {
        let label = label_maker("Speed amount ");
        let icon = help("Amount the playback speed changes by");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", speed_change_amt);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(PlaybackMessage::SpeedAmt);

        let actions = input_actions(PlaybackMessage::IncrSpeedAmt, PlaybackMessage::DecrSpeedAmt);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let auto_start = {
        let label = label_maker("Auto Start ");
        let icon = help("Whether a loaded video automatically starts playing");
        let label = button(label)
            .padding(0)
            .on_press(PlaybackMessage::ToggleAutoStart)
            .style(styles::button::text);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let toggle = toggler(auto_start).on_toggle(PlaybackMessage::AutoStart);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let auto_next = {
        let label = label_maker("Autoplay ");
        let icon = help("Whether the next video in a playlist is automatically loaded and played.");
        let label = button(label)
            .padding(0)
            .on_press(PlaybackMessage::ToggleAutoNext)
            .style(styles::button::text);

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let toggle = toggler(auto_next).on_toggle(PlaybackMessage::AutoNext);

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let completion_point = {
        let label = label_maker("Completion point(%) ");
        let icon = help("The percentage progress at which a video is considered as 'watched'");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", completion_point);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(PlaybackMessage::CompletionPoint);

        let actions = input_actions(
            PlaybackMessage::IncrComplPoint,
            PlaybackMessage::DecrComplPoint,
        );

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let completion_time = {
        let label = label_maker("Completion Watch time(%) ");
        let icon = help("The percentage watch time at which a video is considered as 'watched'");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", completion_watch_time);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(PlaybackMessage::CompletionTime);

        let actions = input_actions(
            PlaybackMessage::IncrComplTime,
            PlaybackMessage::DecrComplTime,
        );

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let content = column!(
        auto_start,
        horizontal_rule(),
        auto_next,
        horizontal_rule(),
        volume,
        horizontal_rule(),
        volume_amt,
        horizontal_rule(),
        speed,
        horizontal_rule(),
        speed_amt,
        horizontal_rule(),
        completion_point,
        horizontal_rule(),
        completion_time,
    )
    .spacing(SECTION_SPACING);

    section("Playback", content)
}

fn draw_seeking<'a>(
    thumbnail_interval: u32,
    seek_change_amt: f64,
    seek_shift_change_amt: f64,
    comment_span: u64,
) -> Element<'a, SeekingMessage> {
    let thumbnail = {
        let label = label_maker("Thumbnail Interval(seconds) ");

        let interval = thumbnail_interval.to_string();
        let input = text_input("", &interval)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .on_input(SeekingMessage::ThumbnailInterval)
            .padding(INPUT_PADDING);

        let actions = input_actions(
            SeekingMessage::IncrThumbInterval,
            SeekingMessage::DecrThumbInterval,
        );

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let seek_amt = {
        let label = label_maker("Seek amount(seconds) ");
        let icon = help("Seconds to skip");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", seek_change_amt);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SeekingMessage::Seek);

        let actions = input_actions(SeekingMessage::IncrSeek, SeekingMessage::DecrSeek);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);
        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let seek_amt_shift = {
        let label = label_maker("Seek Shift amount(seconds) ");
        let icon = help("Seconds to skip while holding down the Shift key");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{:.02}", seek_shift_change_amt);
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SeekingMessage::SeekShift);

        let actions = input_actions(SeekingMessage::IncrSeekShift, SeekingMessage::DecrSeekShift);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let cspan = {
        let label = label_maker("Comment span(seconds) ");
        let icon = help("Show comments within ± seconds of the current playback position");

        let label = row!(label, icon).spacing(2).align_y(Vertical::Center);

        let amt = format!("{comment_span}");
        let input = text_input("", &amt)
            .font(regular_font())
            .width(INPUT_WIDTH)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Right)
            .padding(INPUT_PADDING)
            .on_input(SeekingMessage::Span);

        let actions = input_actions(SeekingMessage::IncrSpan, SeekingMessage::DecrSpan);

        let input = row!(input, actions)
            .spacing(ACTIONS_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let content = column!(
        thumbnail,
        horizontal_rule(),
        seek_amt,
        horizontal_rule(),
        seek_amt_shift,
        horizontal_rule(),
        cspan,
    )
    .spacing(SECTION_SPACING);

    section("Seeking & Navigation", content)
}

fn draw_filters<'a>(filters: &VideoFilters) -> Element<'a, VideoFilterMessage> {
    let gamma = {
        let label = label_maker("Gamma ");

        let slider = slider(1.0..=3.0, filters.gamma, VideoFilterMessage::Gamma)
            .step(0.05)
            .shift_step(0.1)
            .width(SLIDER_WIDTH);

        let gamma = sized_regular(format!("{:.01}", filters.gamma), TEXT_SIZE);
        let slider = row!(gamma, slider)
            .spacing(SLIDER_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let brightness = {
        let label = label_maker("Brightness ");

        let slider = slider(
            -1.0..=1.0,
            filters.brightness,
            VideoFilterMessage::Brightness,
        )
        .step(0.05)
        .shift_step(0.1)
        .width(SLIDER_WIDTH);

        let brightness = sized_regular(format!("{:.01}", filters.brightness), TEXT_SIZE);
        let slider = row!(brightness, slider)
            .spacing(SLIDER_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let contrast = {
        let label = label_maker("Contrast ");

        let slider = slider(0.0..=2.0, filters.contrast, VideoFilterMessage::Contrast)
            .step(0.05)
            .shift_step(0.1)
            .width(SLIDER_WIDTH);

        let contrast = sized_regular(format!("{:.01}", filters.contrast), TEXT_SIZE);
        let slider = row!(contrast, slider)
            .spacing(SLIDER_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let hue = {
        let label = label_maker("Hue ");

        let slider = slider(-1.0..=1.0, filters.hue, VideoFilterMessage::Hue)
            .step(0.05)
            .shift_step(0.1)
            .width(SLIDER_WIDTH);

        let hue = sized_regular(format!("{:.01}", filters.hue), TEXT_SIZE);
        let slider = row!(hue, slider)
            .spacing(SLIDER_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let saturation = {
        let label = label_maker("Saturation ");

        let slider = slider(
            0.0..=2.0,
            filters.saturation,
            VideoFilterMessage::Saturation,
        )
        .step(0.05)
        .shift_step(0.1)
        .width(SLIDER_WIDTH);

        let saturation = sized_regular(format!("{:.01}", filters.saturation), TEXT_SIZE);
        let slider = row!(saturation, slider)
            .spacing(SLIDER_SPACING)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let content = column!(
        gamma,
        horizontal_rule(),
        brightness,
        horizontal_rule(),
        contrast,
        horizontal_rule(),
        hue,
        horizontal_rule(),
        saturation,
    )
    .spacing(SECTION_SPACING);

    section("Video Quality", content)
}

fn horizontal_rule<'a, Message: 'a>() -> Element<'a, Message> {
    rule::horizontal(1.0).into()
}

fn input_actions<'a, Message: 'a + Clone>(
    increase: Message,
    decrease: Message,
) -> Element<'a, Message> {
    let incr = button(icons::icon(icons::CHEV_UP).size(ACTIONS_SIZE))
        .padding(ACTIONS_PADDING)
        .style(styles::button::subtler)
        .on_press(increase);
    let decr = button(icons::icon(icons::CHEV_DOWN).size(ACTIONS_SIZE))
        .padding(ACTIONS_PADDING)
        .style(styles::button::subtler)
        .on_press(decrease);

    column!(incr, decr).spacing(2.0).into()
}
