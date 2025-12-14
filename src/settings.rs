use crate::app::Message;
use crate::db::Operation;
use crate::models::{Directory, DirectoryId, MediaType};
use crate::utils::{
    AppTheme, Config, GeneralSettings, HomeAction, KeyPress, Layout, PlayerAction, Scroll,
    SettingsAction, SubtitleDescription, SubtitleFont, VideoSettings, convert_color_str,
    draw_subtitles, icons, modal_container, picklist_handle, sized_button, styles, tooltip,
    trim_path, typo::*,
};
use crate::widgets::{modal, toast, toggler};
use iced::{
    Border, Element, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    font::{self, Font},
    widget::{
        button, center_x, column, container, operation, pick_list, rich_text, row, rule,
        scrollable, slider, space, span, table, text, text_input, tooltip::Tooltip,
    },
};

use std::path::{Path, PathBuf};
use std::time::Duration;

const TEXT_SIZE: f32 = P;
const LABEL_WIDTH: f32 = 300.0;

#[derive(Debug, Clone, Copy)]
pub enum KeyAction {
    General(Option<HomeAction>),
    Video(Option<PlayerAction>),
    Settings(Option<SettingsAction>),
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
pub enum SettingsMessage {
    Goto(Page),
    Scroll(scrollable::Viewport),
    OpenLog,
    OpenConfig,
    Auth(String),
    Refresh(String),
    Fetch(String),
    MovieDepth(String),
    ThumbnailInterval(String),
    Recents(String),
    Search(String),
    Seek(String),
    SeekShift(String),
    Layout(Layout),
    Theme(AppTheme),
    Volume(f64),
    VolumeAmt(String),
    Speed(f64),
    SpeedAmt(String),
    Gamma(f64),
    Subtitles(bool),
    AutoStart(bool),
    AutoNext(bool),
    ScanDiscoverer(bool),
    ToggleScanDiscover,
    ToggleSubtitles,
    ToggleAutoStart,
    ToggleAutoNext,
    CompletionPoint(String),
    CompletionTime(String),
    Save,
    Cancel,
    ToggleDirectory(DirectoryId),
    ToggleDirKind(DirectoryId),
    FolderSelected(Option<PathBuf>),
    FolderSelection(FolderSelectionMessage),
    AddFolder,
    ClearAllBindings(KeyAction),
    ClearBinding(Page, KeyPress),
    NewKeyPress(KeyAction),
    KeyAction(KeyAction),
    SaveKeyBinding,
    SubSizeIncr,
    SubSizeDecr,
    SubSize(String),
    SubColor(String),
    SubBackground(String),
    None,
}

#[derive(Debug)]
pub struct Settings {
    pub config: Config,

    page: Page,
    view: Option<View>,

    scroll_state: ScrollState,

    text_color: String,
    background_color: String,

    pub directories: Vec<(Directory, Option<bool>, Operation)>,
}

impl Settings {
    pub fn boot(config: Config) -> (Self, Task<Message>) {
        let new = Self::new(config);
        let tasks = Task::done(Message::FetchDirectories);

        (new, tasks)
    }

    fn new(config: Config) -> Self {
        let text_color = format!("#{:08x}", config.video.subtitles.color);
        let background_color = format!("#{:08x}", config.video.subtitles.background_color);
        Self {
            config,
            page: Page::default(),
            view: None,
            scroll_state: ScrollState::new(),
            directories: Vec::default(),
            text_color,
            background_color,
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
            SettingsMessage::Auth(auth) => {
                self.config.general.auth_token = auth;
                Task::none()
            }
            SettingsMessage::MovieDepth(depth) => {
                let depth = depth.trim();
                if depth.is_empty() {
                    self.config.general.movie_depth = 0;
                    return Task::none();
                }

                let Ok(depth) = depth.parse::<u8>() else {
                    let msg = Message::error(format!("Invalid input: {depth}"));
                    return Task::done(msg);
                };

                self.config.general.movie_depth = depth;
                Task::none()
            }
            SettingsMessage::Refresh(interval) => {
                let interval = interval.trim();
                if interval.is_empty() {
                    self.config.general.refresh_interval = Duration::ZERO;
                    return Task::none();
                }

                let Ok(interval) = interval.parse::<u64>() else {
                    let msg = Message::error(format!("Invalid input: {interval}"));
                    return Task::done(msg);
                };

                self.config.general.refresh_interval = Duration::from_secs(interval);

                Task::none()
            }
            SettingsMessage::Fetch(interval) => {
                let interval = interval.trim();
                if interval.is_empty() {
                    self.config.general.fetching_interval = Duration::ZERO;
                    return Task::none();
                }

                let Ok(interval) = interval.parse::<u64>() else {
                    let msg = Message::error(format!("Invalid input: {interval}"));
                    return Task::done(msg);
                };

                self.config.general.fetching_interval = Duration::from_secs(interval);

                Task::none()
            }
            SettingsMessage::Recents(recents) => {
                let recents = recents.trim();
                if recents.is_empty() {
                    self.config.general.recents_limit = None;
                    return Task::none();
                }

                let Ok(recents) = recents.parse::<i32>() else {
                    let msg = Message::error(format!("Invalid input: {recents}"));
                    return Task::done(msg);
                };

                self.config.general.recents_limit = Some(recents);

                Task::none()
            }
            SettingsMessage::Search(searches) => {
                let search = searches.trim();
                if search.is_empty() {
                    self.config.general.search_limit = None;
                    return Task::none();
                }

                let Ok(searches) = searches.parse::<i32>() else {
                    let msg = Message::error(format!("Invalid input: {searches}"));
                    return Task::done(msg);
                };

                self.config.general.search_limit = Some(searches);

                Task::none()
            }
            SettingsMessage::Seek(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.seek_change_amt = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.seek_change_amt = amt;

                Task::none()
            }
            SettingsMessage::SeekShift(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.seek_shift_change_amt = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.seek_shift_change_amt = amt;

                Task::none()
            }
            SettingsMessage::VolumeAmt(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.volume_change_amt = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.volume_change_amt = amt.min(1.0);

                Task::none()
            }
            SettingsMessage::SpeedAmt(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.speed_change_amt = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.speed_change_amt = amt;

                Task::none()
            }
            SettingsMessage::CompletionPoint(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.completion_point = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.completion_point = amt.min(1.0);

                Task::none()
            }
            SettingsMessage::CompletionTime(amt) => {
                let amt = amt.trim();
                if amt.is_empty() {
                    self.config.video.completion_watch_time = 0.0;
                    return Task::none();
                }

                let Ok(amt) = amt.parse::<f64>() else {
                    let msg = Message::error(format!("Invalid input: {amt}"));
                    return Task::done(msg);
                };

                self.config.video.completion_watch_time = amt.min(1.0);

                Task::none()
            }
            SettingsMessage::ThumbnailInterval(interval) => {
                let interval = interval.trim();
                if interval.is_empty() {
                    self.config.video.thumbnail_interval = 0;
                    return Task::none();
                }

                let Ok(interval) = interval.parse::<u32>() else {
                    let msg = Message::error(format!("Invalid input: {interval}"));
                    return Task::done(msg);
                };

                self.config.video.thumbnail_interval = interval;

                Task::none()
            }
            SettingsMessage::Layout(layout) => {
                self.config.general.layout = layout;

                Task::none()
            }
            SettingsMessage::Theme(theme) => {
                self.config.general.theme = theme;
                Task::none()
            }
            SettingsMessage::Volume(new) => {
                self.config.video.volume = new;
                Task::none()
            }
            SettingsMessage::Speed(new) => {
                self.config.video.speed = new;
                Task::none()
            }
            SettingsMessage::Gamma(new) => {
                self.config.video.gamma = new;
                Task::none()
            }
            SettingsMessage::Subtitles(show) => {
                self.config.video.show_subtitles = show;
                Task::none()
            }
            SettingsMessage::ToggleSubtitles => {
                self.config.video.show_subtitles = !self.config.video.show_subtitles;
                Task::none()
            }
            SettingsMessage::AutoStart(toggle) => {
                self.config.video.auto_start = toggle;
                Task::none()
            }
            SettingsMessage::ToggleAutoStart => {
                self.config.video.auto_start = !self.config.video.auto_start;
                Task::none()
            }
            SettingsMessage::AutoNext(toggle) => {
                self.config.video.auto_next = toggle;
                Task::none()
            }
            SettingsMessage::ToggleAutoNext => {
                self.config.video.auto_next = !self.config.video.auto_next;
                Task::none()
            }
            SettingsMessage::AddFolder => pick_task(),
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
                        return Task::none();
                    };

                    let path = path.canonicalize().unwrap().display().to_string();
                    let path = path
                        .strip_prefix(r"\\?\")
                        .map(ToOwned::to_owned)
                        .unwrap_or(path);

                    if self.directories.iter().any(|(dir, _, _)| dir.path == path) {
                        return Task::none();
                    }

                    // todo??
                    let (new, _query) = Directory::new(path, kind, true);

                    self.directories.push((new, None, Operation::Insert));

                    self.update_scroll()
                }
            },
            SettingsMessage::ToggleDirectory(id) => {
                if let Some((_, new, operation)) =
                    self.directories.iter_mut().find(|(dir, _, _)| dir.id == id)
                {
                    *operation = match operation {
                        Operation::Update => Operation::Delete,
                        Operation::Insert => Operation::Delete,
                        Operation::Delete if new.is_none() => Operation::Insert,
                        Operation::Delete => Operation::Update,
                    };
                }
                Task::none()
            }
            SettingsMessage::ToggleDirKind(id) => {
                if let Some((dir, new, _)) =
                    self.directories.iter_mut().find(|(dir, _, _)| dir.id == id)
                {
                    dir.media_type = match dir.media_type {
                        MediaType::Shows => MediaType::Movies,
                        MediaType::Movies => MediaType::Shows,
                    };

                    if let Some(toggled) = new {
                        *toggled = !*toggled;
                    }
                }

                Task::none()
            }
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
                self.view = Some(View::CaptureKey { action, key: None });
                Task::done(Message::CaptureKeys(true))
            }
            SettingsMessage::KeyAction(action) => {
                if let Some(View::CaptureKey { action: old, .. }) = self.view.as_mut() {
                    *old = action
                }

                Task::none()
            }
            SettingsMessage::SaveKeyBinding => {
                let Some(View::CaptureKey { action, key }) = self.view.take() else {
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

                Task::done(Message::CaptureKeys(false))
            }
            SettingsMessage::ToggleScanDiscover => {
                self.config.general.scan_discoverer = !self.config.general.scan_discoverer;
                Task::none()
            }
            SettingsMessage::ScanDiscoverer(enable) => {
                self.config.general.scan_discoverer = enable;

                Task::none()
            }
            SettingsMessage::OpenLog => {
                let Some(path) = self.config.log_path() else {
                    return Task::none();
                };

                match open::that(path) {
                    Ok(_) => Task::none(),
                    Err(error) => {
                        println!("{error}");
                        Task::done(Message::error(error))
                    }
                }
            }
            SettingsMessage::OpenConfig => {
                let Some(path) = self.config.config_path() else {
                    return Task::none();
                };

                match open::that(path) {
                    Ok(_) => Task::none(),
                    Err(error) => Task::done(Message::error(error)),
                }
            }
            SettingsMessage::SubSizeIncr => {
                self.config.video.subtitles.size = (self.config.video.subtitles.size + 1).min(60);
                Task::none()
            }
            SettingsMessage::SubSizeDecr => {
                self.config.video.subtitles.size = (self.config.video.subtitles.size - 1).max(5);
                Task::none()
            }
            SettingsMessage::SubSize(size) => {
                let size = size.trim();
                if size.is_empty() {
                    self.config.video.subtitles.size = 5;
                    return Task::none();
                }

                let Ok(size) = size.parse::<u32>() else {
                    let msg = Message::error(format!("Invalid input: {size}"));
                    return Task::done(msg);
                };

                self.config.video.subtitles.size = size.max(5);

                Task::none()
            }
            SettingsMessage::SubColor(color) => {
                if let Some(color) = convert_color_str(&color) {
                    self.config.video.subtitles.color = color;
                };

                self.text_color = color;

                Task::none()
            }
            SettingsMessage::SubBackground(color) => {
                if let Some(color) = convert_color_str(&color) {
                    self.config.video.subtitles.background_color = color;
                };

                self.background_color = color;

                Task::none()
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
            Some(View::CaptureKey { action, key }) => {
                let overlay = draw_capture_key(action, key);

                modal(content, overlay, SettingsMessage::Cancel)
            }
        }
    }

    fn side(&self) -> Element<'_, SettingsMessage> {
        let header = {
            let text = text("Settings").size(H4).font(font::Font {
                weight: font::Weight::Semibold,
                family: font::Family::Serif,
                ..Default::default()
            });

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
            .width(300.0)
            .height(Length::Fill);

        let content = container(content).style(|theme| {
            let default = styles::container::bw(theme);
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

        let title = container(text(title).size(H6).font(font::Font {
            family: font::Family::Serif,
            weight: font::Weight::Semibold,
            ..Default::default()
        }))
        .height(28.0)
        .center_x(Length::Fill);

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

        let top = column!(title, rule::horizontal(1.0), space::vertical().height(10.0));

        let content = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8.0)
            .id(scroll)
            .on_scroll(SettingsMessage::Scroll);

        let actions = {
            let save = button("Save").on_press(SettingsMessage::Save);
            let cancel = button("Cancel").on_press(SettingsMessage::Cancel);

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
        let width = LABEL_WIDTH;
        let size = TEXT_SIZE;
        let spacing = 20;
        let padding = Padding::from([2, 5]);

        let GeneralSettings {
            layout,
            refresh_interval,
            recents_limit,
            search_limit,
            theme,
            scan_discoverer,
            auth_token,
            movie_depth,
            fetching_interval,
        } = &self.config.general;

        let refresh = {
            let label = label_maker("Refresh Interval: ");
            let icon = help(
                "How often to scan for content changes in seconds",
                size / RATIO,
            );

            let interval = refresh_interval.as_secs().to_string();
            let input = text_input("Interval in seconds", &interval)
                .width(64)
                .size(size)
                .padding(padding)
                .on_input(SettingsMessage::Refresh)
                .align_x(Horizontal::Right);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let fetching_interval = {
            let label = label_maker("Fetch Interval: ");
            let icon = help(
                "How often to scrape TMDB© for new media data in seconds. Note this only takes effect on restart.",
                size / RATIO,
            );

            let interval = fetching_interval.as_secs().to_string();
            let input = text_input("Interval in seconds", &interval)
                .width(64)
                .size(size)
                .padding(padding)
                .on_input(SettingsMessage::Fetch)
                .align_x(Horizontal::Right);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let recents_limit = {
            let label = label_maker("Recents Limit: ");
            let icon = help("Number of recent items to display", size / RATIO);

            let recents = recents_limit
                .map(|limit| limit.to_string())
                .unwrap_or_default();
            let input = text_input("", &recents)
                .width(64)
                .size(size)
                .padding(padding)
                .align_x(Horizontal::Right)
                .on_input(SettingsMessage::Recents);

            let label = row!(label, icon)
                .spacing(3)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let search_limit = {
            let label = label_maker("Search Limit: ");
            let icon = help("Number of search items to display", size / RATIO);

            let searches = search_limit
                .map(|limit| limit.to_string())
                .unwrap_or_default();
            let input = text_input("", &searches)
                .width(64)
                .size(size)
                .padding(padding)
                .align_x(Horizontal::Right)
                .on_input(SettingsMessage::Search);

            let label = row!(label, icon)
                .spacing(3)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let layouts = {
            let label = label_maker("Layout: ").width(width);
            let layouts = Layout::ALL.iter().map(|value| {
                let label = value.str();
                let content = text(label).size(size);
                Element::from(
                    button(content)
                        .on_press(SettingsMessage::Layout(*value))
                        .style(move |theme, status| {
                            let default = if value == layout {
                                styles::button::subtle(theme, status)
                            } else {
                                styles::button::subtlest(theme, status)
                            };

                            let border = Border::default().width(2.0).rounded(5.0);

                            button::Style { border, ..default }
                        }),
                )
            });

            row!(label)
                .extend(layouts)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let theme = {
            let handle = picklist_handle(size);

            let label = label_maker("Theme: ").width(width);

            let theme = pick_list(AppTheme::ALL, Some(*theme), SettingsMessage::Theme)
                .handle(handle.clone())
                .padding(padding)
                .text_size(size);

            row!(label, theme)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let dirs = {
            let label = label_maker("Media Directories: ").width(width);

            let add = button(
                row!(
                    icons::icon(icons::FOLDER_ADD).size(size),
                    text("Add").size(size)
                )
                .spacing(8.0)
                .align_y(Vertical::Center),
            )
            .padding([3, 6])
            .style(styles::button::subtle)
            .on_press(SettingsMessage::AddFolder);

            let dirs = column(
                self.directories
                    .iter()
                    .map(|(dir, _, operation)| directory_draw(dir, *operation, size)),
            )
            .spacing(12)
            .push(add);

            row!(label, dirs).spacing(spacing)
        };

        let discoverer = {
            let label = label_maker("Enable Discovery Scan ");
            let icon = help(
                "More information is collected on videos but directory scanning is slower as a result ",
                size,
            );
            let label = button(label)
                .padding(0)
                .on_press(SettingsMessage::ToggleScanDiscover)
                .style(styles::button::text);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let toggle = toggler(*scan_discoverer)
                .on_toggle(SettingsMessage::ScanDiscoverer)
                .size(size);

            row!(label, toggle)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let auth = {
            let label = label_maker("TMDB API Token: ");
            let icon = help(
                "The TMDB© API Read Access Token used for fetching media metadata",
                size / RATIO,
            );

            let input = text_input("Token", auth_token)
                .width(500)
                .size(size)
                .padding(padding)
                .on_input(SettingsMessage::Auth);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let movie_depth = {
            let label = label_maker("Movie Directory Depth: ");
            let icon = help(
                "How often deep movie directory scans for videos should go.",
                size / RATIO,
            );

            let depth = movie_depth.to_string();
            let input = text_input("Depth", &depth)
                .width(64)
                .size(size)
                .padding(padding)
                .on_input(SettingsMessage::MovieDepth)
                .align_x(Horizontal::Right);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            row!(label, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let open = {
            let config = {
                let label = label_maker("Config File");
                let icon = icons::icon(icons::EXTERNAL).size(size / RATIO);

                let label = row!(label, icon).spacing(4).align_y(Vertical::Center);

                button(label)
                    .on_press(SettingsMessage::OpenConfig)
                    .style(styles::button::text)
            };

            let log = {
                let label = label_maker("Log File");
                let icon = icons::icon(icons::EXTERNAL).size(size / RATIO);

                let label = row!(label, icon).spacing(4).align_y(Vertical::Center);

                button(label)
                    .on_press(SettingsMessage::OpenLog)
                    .style(styles::button::text)
            };

            column!(config, log).spacing(8)
        };

        let content = column!(
            refresh,
            recents_limit,
            search_limit,
            layouts,
            theme,
            auth,
            fetching_interval,
            movie_depth,
            discoverer,
            dirs,
            open,
        )
        .spacing(36.0)
        .height(Length::Fill);

        let content = center_x(content);

        content.into()
    }

    fn video(&self) -> Element<'_, SettingsMessage> {
        let size = TEXT_SIZE;
        let width = LABEL_WIDTH;
        let spacing = 20;
        let padding = Padding::from([2, 5]);

        let VideoSettings {
            thumbnail_interval,
            volume,
            speed,
            gamma,
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
            filters: _filters,
        } = &self.config.video;

        let thumbnail = {
            let label = label_maker("Thumbnail Interval: ").width(width);

            let interval = thumbnail_interval.to_string();
            let input = text_input("", &interval)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .on_input(SettingsMessage::ThumbnailInterval)
                .padding(padding);

            let secs = text("seconds").size(size / RATIO);
            let input = row!(input, secs).align_y(Vertical::Center).spacing(4);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let volume = {
            let label = label_maker("Default Volume: ").width(width);

            let value = text(format!("{volume:.2}")).size(size / RATIO);
            let volume = slider(0.0..=1.0, *volume, SettingsMessage::Volume)
                .step(0.05)
                .shift_step(0.1)
                .width(125.0);

            let volume = row!(volume, value).align_y(Vertical::Center).spacing(4);

            row!(label, volume)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let volume_amt = {
            let label = label_maker("Volume amount: ");
            let icon = help("Amount the volume changes by", size / RATIO);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", volume_change_amt);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::VolumeAmt);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let speed = {
            let label = label_maker("Default Speed: ").width(width);

            let value = text(format!("{speed:.2}")).size(size / RATIO);
            let speed = slider(0.5..=2.5, *speed, SettingsMessage::Speed)
                .step(0.1)
                .shift_step(0.2)
                .width(125.0);

            let speed = row!(speed, value).align_y(Vertical::Center).spacing(4);

            row!(label, speed)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let speed_amt = {
            let label = label_maker("Speed amount: ");
            let icon = help("Amount the playback speed changes by", size / RATIO);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", speed_change_amt);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::SpeedAmt);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let gamma = {
            let label = label_maker("Default Gamma: ").width(width);

            let value = text(format!("{gamma:.2}")).size(size / RATIO);
            let gamma = slider(1.0..=3.0, *gamma, SettingsMessage::Gamma)
                .default(1.3)
                .step(0.1)
                .shift_step(0.2)
                .width(125.0);

            let gamma = row!(gamma, value).align_y(Vertical::Center).spacing(4);

            row!(label, gamma)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let seek_amt = {
            let label = label_maker("Seek amount: ");
            let icon = help("Seconds to skip", size / RATIO);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", seek_change_amt);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::Seek);

            let secs = text("seconds").size(size / RATIO);
            let input = row!(input, secs).align_y(Vertical::Center).spacing(4);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let seek_amt_shift = {
            let label = label_maker("Seek Shift amount: ");
            let icon = help(
                "Seconds to skip while holding down the Shift key",
                size / RATIO,
            );

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", seek_shift_change_amt);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::SeekShift);

            let secs = text("seconds").size(size / RATIO);
            let input = row!(input, secs).align_y(Vertical::Center).spacing(4);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let subtitles_toggle = {
            let label = label_maker("Show subtitles: ").width(width);
            let label = button(label)
                .padding(0)
                .on_press(SettingsMessage::ToggleSubtitles)
                .style(styles::button::text);

            let toggle = toggler(*show_subtitles)
                .on_toggle(SettingsMessage::Subtitles)
                .size(size);

            row!(label, toggle)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let auto_start = {
            let label = label_maker("Auto Start: ");
            let icon = help("Whether a loaded video automatically starts playing", size);
            let label = button(label)
                .padding(0)
                .on_press(SettingsMessage::ToggleAutoStart)
                .style(styles::button::text);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let toggle = toggler(*auto_start)
                .on_toggle(SettingsMessage::AutoStart)
                .size(size);

            row!(label, toggle)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let auto_next = {
            let label = label_maker("Autoplay: ");
            let icon = help(
                "Whether the next video in a playlist is automatically loaded and played.",
                size,
            );
            let label = button(label)
                .padding(0)
                .on_press(SettingsMessage::ToggleAutoNext)
                .style(styles::button::text);

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let toggle = toggler(*auto_next)
                .on_toggle(SettingsMessage::AutoNext)
                .size(size);

            row!(label, toggle)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let completion_point = {
            let label = label_maker("Completion point: ");
            let icon = help(
                "The percentage progress at which a video is considered as 'watched'",
                size / RATIO,
            );

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", completion_point);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::CompletionPoint);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let completion_time = {
            let label = label_maker("Completion Watch time: ");
            let icon = help(
                "The percentage watch time at which a video is considered as 'watched'",
                size / RATIO,
            );

            let label = row!(label, icon)
                .spacing(2)
                .align_y(Vertical::Center)
                .width(width);

            let amt = format!("{:.02}", completion_watch_time);
            let input = text_input("", &amt)
                .width(64)
                .size(size)
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SettingsMessage::CompletionTime);

            row!(label, input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let subtitles = subtitle_config(*subtitles, &self.text_color, &self.background_color);

        let content = column!(
            thumbnail,
            completion_point,
            completion_time,
            volume,
            volume_amt,
            speed,
            speed_amt,
            gamma,
            seek_amt,
            seek_amt_shift,
            subtitles_toggle,
            auto_start,
            auto_next,
            subtitles,
        )
        .spacing(24);

        let content = center_x(content);

        content.into()
    }

    fn keybinds(&self) -> Element<'_, SettingsMessage> {
        let home = {
            let names = table::column(
                table_header("Name"),
                |(action, _): (&HomeAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("Description"),
                |(action, _): (&HomeAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("Keybinding"),
                |(_, keys): (&HomeAction, &Vec<KeyPress>)| table_keys(Page::General, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.home());

            let title = label_maker("GENERAL");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::General(None)))
                .style(styles::button::text);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(4)
        };

        let player = {
            let names = table::column(
                table_header("Name"),
                |(action, _): (&PlayerAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("Description"),
                |(action, _): (&PlayerAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("Keybinding"),
                |(_, keys): (&PlayerAction, &Vec<KeyPress>)| table_keys(Page::Video, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.player());

            let title = label_maker("PLAYBACK");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::Video(None)))
                .style(styles::button::text);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(4)
        };

        let settings = {
            let names = table::column(
                table_header("Name"),
                |(action, _): (&SettingsAction, &Vec<KeyPress>)| {
                    table_name(action.to_string(), (*action).into())
                },
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let descriptions = table::column(
                table_header("Description"),
                |(action, _): (&SettingsAction, &Vec<KeyPress>)| table_description(action.descr()),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let keys = table::column(
                table_header("Keybinding"),
                |(_, keys): (&SettingsAction, &Vec<KeyPress>)| table_keys(Page::Keybinds, keys),
            )
            .width(Length::Fill)
            .align_y(Vertical::Center);

            let table = table([names, descriptions, keys], self.config.keystore.settings());

            let title = label_maker("SETTINGS");
            let new = {
                let icon = icons::icon(icons::ADD).size(TEXT_SIZE);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            let new = button(new)
                .on_press(SettingsMessage::NewKeyPress(KeyAction::Video(None)))
                .style(styles::button::text);

            let title = row!(title, space::horizontal(), new).align_y(Vertical::Center);

            column!(title, table).spacing(4)
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
        if let Some(View::CaptureKey { key: old, .. }) = self.view.as_mut() {
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
        self.directories.extend(
            dirs.into_iter()
                .map(|dirs| (dirs, Some(false), Operation::Update)),
        );
    }

    pub fn save(self) -> (Config, Vec<(Directory, Operation)>) {
        let directories = self
            .directories
            .into_iter()
            .filter_map(|(dir, new, op)| {
                match new {
                    Some(false) if matches!(op, Operation::Update) => None,
                    _ => Some((dir, op)),

                }
            })
            .collect();

        let config = self.config;

        (config, directories)
    }
}

fn side_button<'a>(
    value: &'a str,
    message: SettingsMessage,
    current: bool,
) -> Element<'a, SettingsMessage> {
    let size = P;
    let text = text(value).size(size).font(font::Font {
        weight: font::Weight::Semibold,
        ..Default::default()
    });

    container(
        button(text)
            .width(Length::Fill)
            .style(move |theme, status| {
                if current {
                    styles::button::subtle_primary(theme, status)
                } else {
                    styles::button::subtler(theme, status)
                }
            })
            .on_press(message),
    )
    .clip(true)
    .max_height(48.0)
    .into()
}

fn help<'a>(label: &'a str, size: f32) -> Tooltip<'a, SettingsMessage> {
    use iced::widget::tooltip::Position;

    tooltip(icons::icon(icons::HELP).size(size), label, Position::Right)
}

fn directory_draw<'a>(
    directory: &'a Directory,
    operation: Operation,
    size: f32,
) -> Element<'a, SettingsMessage> {
    let icon = if matches!(operation, Operation::Delete) {
        icons::ADD
    } else {
        icons::MINUS
    };
    let icon = icons::icon(icon).size(size / RATIO);

    let icon_btn = button(icon)
        .padding([3, 6])
        .on_press(SettingsMessage::ToggleDirectory(directory.id))
        .style(styles::button::subtler);

    let path = trim_path(Path::new(&directory.path), 5);
    let label = span(path)
        .strikethrough(matches!(operation, Operation::Delete))
        .size(size);
    let label = rich_text([label]).on_link_click(|_: ()| SettingsMessage::None);

    let tag = text(directory.media_type.to_string()).size(size / (RATIO * RATIO));

    let tag = button(tag)
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
        .on_press(SettingsMessage::ToggleDirKind(directory.id));

    row!(icon_btn, label, tag)
        .align_y(Vertical::Center)
        .spacing(12)
        .wrap()
        .into()
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
    let font = label_font();

    let folder = {
        let path = trim_path(path, 3);

        let path = text(path).size(size).font(Font {
            style: font::Style::Italic,
            ..Default::default()
        });

        let label = text("Folder: ").size(size).font(font).width(100.0);

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
        let label = text("Media type: ").size(size).font(font).width(100.0);

        let lst = pick_list(MediaType::ALL, Some(*kind), |kind| {
            SettingsMessage::FolderSelection(FolderSelectionMessage::Kind(kind))
        })
        .padding([2, 5])
        .handle(handle)
        .text_size(size);

        row!(label, lst).align_y(Vertical::Center).spacing(12)
    };

    let actions = {
        let submit = button("Save").on_press(SettingsMessage::FolderSelection(
            FolderSelectionMessage::Submit,
        ));

        let cancel = button("Cancel").on_press(SettingsMessage::FolderSelection(
            FolderSelectionMessage::Cancel,
        ));

        center_x(row!(submit, cancel).align_y(Vertical::Center).spacing(36))
    };

    let content = column!(folder, kind, actions).spacing(20);

    modal_container(content).max_width(450).into()
}

fn draw_capture_key<'a>(
    action: &'a KeyAction,
    keypress: &'a Option<KeyPress>,
) -> Element<'a, SettingsMessage> {
    let size = TEXT_SIZE / RATIO;
    let key = match keypress {
        Some(keypress) => {
            let keypress = table_key(keypress);
            container(keypress)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Center)
        }
        None => container(""),
    }
    .height(40)
    .width(Length::Fill)
    .padding([2, 4])
    .style(|theme: &Theme| {
        let color = theme.extended_palette().secondary.strong.color;
        let default = styles::container::transparent(theme);
        let border = default.border.rounded(5).color(color).width(1.5);

        container::Style { border, ..default }
    });

    let key = column!(label_maker("Key Press").size(size), key).spacing(4.0);

    let (action, set_action) = {
        let label = label_maker("Action").size(size);
        let padding = [5, 5];

        let (lst, set): (Element<'_, SettingsMessage>, bool) = match action {
            KeyAction::General(selected) => (
                pick_list(HomeAction::ALL, *selected, |action| {
                    SettingsMessage::KeyAction(KeyAction::General(Some(action)))
                })
                .handle(picklist_handle(size))
                .padding(padding)
                .text_size(size)
                .into(),
                selected.is_some(),
            ),
            KeyAction::Video(selected) => (
                pick_list(PlayerAction::ALL, *selected, |action| {
                    SettingsMessage::KeyAction(KeyAction::Video(Some(action)))
                })
                .handle(picklist_handle(size))
                .padding(padding)
                .text_size(size)
                .into(),
                selected.is_some(),
            ),
            KeyAction::Settings(selected) => (
                pick_list(SettingsAction::ALL, *selected, |action| {
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
        let save = button("Save").on_press_maybe(set.then_some(SettingsMessage::SaveKeyBinding));
        let cancel = button("Cancel").on_press(SettingsMessage::Cancel);

        let actions = row!(save, cancel).spacing(80.0).align_y(Vertical::Center);

        container(actions)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
    };

    let content = column!(key, action, btns).spacing(16);

    modal_container(content).width(300).into()
}

fn label_font() -> Font {
    Font {
        family: font::Family::Serif,
        weight: font::Weight::Semibold,
        ..Default::default()
    }
}

fn label_maker<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(label).size(TEXT_SIZE).font(label_font())
}

fn table_header<'a>(label: &'a str) -> text::Text<'a> {
    text(label).size(TEXT_SIZE / RATIO).font(label_font())
}

fn table_name<'a>(
    label: impl text::IntoFragment<'a>,
    action: KeyAction,
) -> Element<'a, SettingsMessage> {
    let clear = icons::text_button(icons::CANCEL)
        .padding(0)
        .on_press(SettingsMessage::ClearAllBindings(action));
    let clear = binding_tooltip(clear, "Remove all bindings");
    let label = text(label).size(TEXT_SIZE / RATIO).font(label_font());

    row!(label, space::horizontal(), clear)
        .align_y(Vertical::Center)
        .into()
}

fn table_description<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(label).size(TEXT_SIZE).font(Font {
        family: font::Family::Serif,
        ..Default::default()
    })
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
    let content = text(key.to_string()).size(TEXT_SIZE / RATIO).font(Font {
        family: font::Family::Monospace,
        ..Default::default()
    });

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

fn subtitle_config<'a>(
    subtitles: SubtitleDescription,
    text_color: &'a str,
    background_color: &'a str,
) -> Element<'a, SettingsMessage> {
    let size = H7;
    let input_width = 48.0;
    let color_width = 150.0;
    let padding = [2, 5];
    let spacing = 8;

    let label = label_maker("Subtitle Style").size(H6);
    let dummy = draw_subtitles("An example subtitle", subtitles);

    let sub_size = {
        let label = label_maker("Size: ");

        let amt = format!("{}", subtitles.size);

        let actions = {
            let incr = button(icons::icon(icons::CHEV_UP).size(10))
                .padding([2, 2])
                .style(styles::button::subtler)
                .on_press(SettingsMessage::SubSizeIncr);
            let decr = button(icons::icon(icons::CHEV_DOWN).size(10))
                .padding([2, 2])
                .style(styles::button::subtler)
                .on_press(SettingsMessage::SubSizeDecr);

            column!(incr, decr).spacing(2.0)
        };

        let input = text_input("", &amt)
            .width(input_width)
            .size(size)
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(SettingsMessage::SubSize);

        let input = row!(input, actions).spacing(4.0).align_y(Vertical::Center);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let color = {
        let label = label_maker("Text Color (rgba): ");

        let input = text_input("", text_color)
            .width(color_width)
            .size(size)
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(SettingsMessage::SubColor);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let background = {
        let label = label_maker("Background Color (rgba): ");

        let input = text_input("", background_color)
            .width(color_width)
            .size(size)
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(SettingsMessage::SubBackground);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let content = column!(sub_size, color, background, dummy)
        .align_x(Horizontal::Center)
        .spacing(8.0);

    column!(label, content).spacing(12).into()
}
