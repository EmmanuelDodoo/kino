use core::{Context, ContextLog, Error, Log, anyhow, error, variants};
use iced::{
    Color, ContentFit, Length, Padding, Size, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    task,
    time::Instant,
    widget::{
        self, button, center, checkbox, column, container, image, mouse_area, operation, pick_list,
        row, rule, scrollable, slider, space, stack, text, text_editor, text_input, tooltip as tp,
        transition,
    },
    window,
};
use iced_video_player::{Button, Kind, MouseAction, MouseClick, Video, VideoPlayer};
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};
use widgets::{font_selection, marquee, menu, throbber};

pub mod comment;
pub mod playlist;
use crate::app::Message;
use crate::config::{PlayerAction, VideoFilters, VideoSettings};
use crate::home::shared::Icon;
use crate::theme::{self, Theme};
use crate::utils::{
    FontState, InterpolableLength, cancel_btn, draw_subtitles, duration_string, empty,
    icons::{self, CANCEL, sized_button},
    input_actions, modal_container, path_container, picklist_handle, save_btn, toggler, tooltip,
    trim_path, typo,
};
pub use comment::*;
use devutils::thumbnails::{Image, ThumbnailGenerator};
pub use playlist::*;
use registry::models::{
    self, Audio, CollectionId, CommentId, SimpleCollection, Subtitle, SubtitleKind, VideoId,
    VideoInfo,
};
use typo::*;

use crate::Element;
const LIST_PADDING: [f32; 2] = [5.0, 10.0];

#[derive(Debug)]
enum Modal {
    CollectionAdd {
        item: VideoId,
        collections: Vec<SimpleCollection>,
        selected: HashSet<CollectionId>,
        initial: HashSet<CollectionId>,
    },
    Config(Box<Config>),
}

#[derive(Debug)]
pub struct ModalState {
    modal: Option<Modal>,
    open: bool,
}

impl ModalState {
    fn none() -> Self {
        Self {
            modal: None,
            open: false,
        }
    }

    fn open(&mut self, modal: Modal) {
        self.modal = Some(modal);
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false
    }

    fn as_ref(&self) -> Option<&Modal> {
        self.modal.as_ref()
    }

    fn as_mut(&mut self) -> Option<&mut Modal> {
        self.modal.as_mut()
    }

    fn take(&mut self) -> Option<Modal> {
        self.open = false;
        self.modal.take()
    }

    fn view(&self) -> Option<(&Modal, bool)> {
        self.as_ref().map(|modal| (modal, self.open))
    }
}

#[derive(Debug)]
struct Config {
    tab: ConfigTab,
    subtitle_uri: Option<PathBuf>,
    selected_text: Option<Subtitle>,
    selected_audio: Option<Audio>,
    selected_video: Option<VideoInfo>,
    text_color: String,
    background_color: String,
    subtitle_font: FontState,
    subtitle_offset: f32,
    fit: ContentFit,
}

#[derive(Debug, Clone)]
enum Panel {
    Playlist,
    PlaylistClosing,
    Comments(Option<(widget::Id, text_editor::Content)>),
    CommentsClosing,
}

#[derive(Debug, Clone, Copy)]
pub enum PlaylistMessge {
    Toggle,
    ToggleShuffle(bool),
    ToggleRepeat(bool),
    ToggleAutoNext(bool),
    Save,
    PlayItem(usize),
}

#[derive(Debug, Clone)]
pub enum CollectionAddMessage {
    Toggle(bool, CollectionId),
    Save,
}

#[derive(Debug, Clone)]
pub enum CommentMessage {
    New,
    NewSubmit,
    NewCancel,
    NewAction(text_editor::Action),
    Link(String),
    Action {
        id: CommentId,
        timestamp: Option<u64>,
        action: text_editor::Action,
    },
    ImageDownloaded {
        id: CommentId,
        timestamp: Option<u64>,
        url: String,
        image: Result<image::Handle, String>,
    },
    ImageShown {
        id: CommentId,
        timestamp: Option<u64>,
        url: String,
    },
    Edit {
        id: CommentId,
        timestamp: Option<u64>,
    },
    Save {
        id: CommentId,
        timestamp: Option<u64>,
    },
    Cancel {
        id: CommentId,
        timestamp: Option<u64>,
    },
    Delete {
        id: CommentId,
        timestamp: Option<u64>,
    },
}

impl comment::CommentMessage for CommentMessage {
    fn link(url: String) -> Self {
        CommentMessage::Link(url)
    }

    fn save(id: CommentId, timestamp: Option<u64>) -> Self {
        Self::Save { id, timestamp }
    }

    fn edit(id: CommentId, timestamp: Option<u64>) -> Self {
        Self::Edit { id, timestamp }
    }

    fn cancel(id: CommentId, timestamp: Option<u64>) -> Self {
        Self::Cancel { id, timestamp }
    }

    fn delete(id: CommentId, timestamp: Option<u64>) -> Self {
        Self::Delete { id, timestamp }
    }

    fn image_shown(id: CommentId, timestamp: Option<u64>, url: String) -> Self {
        Self::ImageShown { id, timestamp, url }
    }

    fn edit_action(id: CommentId, timestamp: Option<u64>, action: text_editor::Action) -> Self {
        Self::Action {
            id,
            timestamp,
            action,
        }
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ConfigTab {
        General,
        Video,
        Subtitles,
        Audio,
        Info,
    }
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Tab(ConfigTab),
    General(GeneralConfig),
    Video(VideoConfig),
    Subtitle(SubtitleConfig),
    Audio(AudioConfig),
}

#[derive(Debug, Clone)]
pub enum AudioConfig {
    CurrentAudio(Audio),
}

#[derive(Debug, Clone)]
pub enum GeneralConfig {
    VolumeAmt(String),
    SpeedAmt(String),
    SeekAmt(String),
    SeekShiftAmt(String),
    Span(String),
}

#[derive(Debug, Clone)]
pub enum VideoConfig {
    Gamma(f64),
    Brightness(f64),
    Contrast(f64),
    Hue(f64),
    Saturation(f64),
    CurrentVideo(VideoInfo),
    Fit(ContentFit),
}

#[derive(Debug, Clone)]
pub enum SubtitleConfig {
    SubSize(String),
    SubSizeIncr,
    SubSizeDecr,
    SubColor(String),
    SubBackground(String),
    SubFont(iced::font::Family),
    SelectFile,
    Selected(Option<PathBuf>),
    ClearSelected,
    CurrentText(Subtitle),
    Offset(String),
    OffsetIncr,
    OffsetDecr,
}

#[derive(Debug, Clone, Copy)]
enum IndicatorKind {
    VolumeUp,
    VolumeDown,
    Mute,
    Unmute,
    SeekFront(f64),
    SeekBack(f64),
    Play,
    Pause,
}

impl IndicatorKind {
    fn char(&self) -> char {
        match self {
            Self::VolumeUp => icons::VOLUME,
            Self::VolumeDown => icons::VOLUME_DOWN,
            Self::Mute => icons::MUTE,
            Self::Unmute => icons::VOLUME,
            Self::SeekFront(_) => icons::CHEV_LEFT,
            Self::SeekBack(_) => icons::CHEV_RIGHT,
            Self::Play => icons::PLAY,
            Self::Pause => icons::PAUSE,
        }
    }
}

#[derive(Debug, Clone)]
struct Indicator {
    kind: IndicatorKind,
    animation: Animation<bool>,
    _handle: task::Handle,
}

impl Indicator {
    fn new(kind: IndicatorKind, now: Instant) -> (Self, Task<ManagerMessage>) {
        let duration = Duration::from_millis(1000);
        let animation = Animation::new(false)
            .easing(Easing::EaseInOut)
            .duration(duration)
            .go(true, now);

        let (task, handle) =
            Task::perform(async move { tokio::time::sleep(duration).await }, |_| {
                ManagerMessage::ClearIndicator
            })
            .abortable();

        let handle = handle.abort_on_drop();

        (
            Self {
                kind,
                animation,
                _handle: handle,
            },
            task,
        )
    }
}

#[derive(Debug)]
pub struct Player {
    video: Video,
    position: f64,
    is_dragging: bool,
    thumbnails: Vec<image::Handle>,
    item: models::Video,
    watch_time: Duration,
    last_frame: Option<Instant>,
    subtitles: Option<String>,
    file_size: u64,
    fit: ContentFit,
}

impl Player {
    fn seek_release(&mut self, pause: bool) -> Result<(), String> {
        self.is_dragging = false;
        self.last_frame.take();

        self.video
            .seek(Duration::from_secs_f64(self.position.max(0.0)), false)
            .map_err(|error| error.to_string())?;

        self.video.set_paused(pause);

        Ok(())
    }
}

#[derive(Debug)]
struct Comments {
    nulls: Vec<Comment>,
    timestamped: BTreeMap<u64, Vec<Comment>>,
}

impl Comments {
    fn new() -> Self {
        Self {
            nulls: Vec::default(),
            timestamped: BTreeMap::new(),
        }
    }

    fn _get_ref(&self, id: CommentId, timestamp: Option<u64>) -> Option<&Comment> {
        match timestamp {
            Some(timestamp) => self
                .timestamped
                .get(&timestamp)
                .and_then(|comments| comments.iter().find(|comment| comment.inner.id == id)),
            None => self.nulls.iter().find(|comment| comment.inner.id == id),
        }
    }

    fn get_mut(&mut self, id: CommentId, timestamp: Option<u64>) -> Option<&mut Comment> {
        match timestamp {
            Some(timestamp) => self
                .timestamped
                .get_mut(&timestamp)
                .and_then(|comments| comments.iter_mut().find(|comment| comment.inner.id == id)),
            None => self.nulls.iter_mut().find(|comment| comment.inner.id == id),
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        self.nulls.iter().any(|comment| comment.is_animating(now))
            || self
                .timestamped
                .iter()
                .any(|(_, comment)| comment.iter().any(|comment| comment.is_animating(now)))
    }

    fn iter(&self) -> impl Iterator<Item = &models::Comment> {
        let nulls = self.nulls.iter();

        let timestamped = self
            .timestamped
            .values()
            .flat_map(|comments| comments.iter());

        nulls.chain(timestamped).map(|comment| &comment.inner)
    }

    fn update(&mut self, comments: Vec<Comment>) {
        let comments = comments
            .into_iter()
            .map(|comment| (comment.inner.timestamp, comment));

        for (timestamp, comment) in comments {
            match timestamp {
                Some(timestamp) => {
                    let entry = self.timestamped.entry(timestamp).or_default();

                    entry.push(comment);
                }
                None => self.nulls.push(comment),
            }
        }
    }
}

#[derive(Debug)]
enum AutoState {
    Loading,
    Idle,
    Ready {
        player: Box<Player>,
        thumbnails_handle: Option<task::Handle>,
        comments: Comments,
    },
}

enum State {
    Loading,
    Idle,
    Ready {
        player: Box<Player>,
        thumbnails_handle: Option<task::Handle>,
        comments: Comments,
        awake: Option<keepawake::KeepAwake>,
    },
}

#[derive(Debug, Clone)]
pub enum ManagerMessage {
    Video {
        is_next: bool,
        video: Arc<error::Result<Player>>,
    },
    Thumbnail {
        id: VideoId,
        thumbnails: Vec<image::Handle>,
        poster: Option<devutils::Image>,
    },
    Resize((window::Id, Size)),
    SeekRelease,
    Seek(f64),
    ChangeVolume(f64),
    CursorExit,
    CursorEnter,
    PreviousScreen,
    AddCollection,
    OpenConfig,
    ToggleSubtitles,
    ToggleMute,
    PlayPrevious,
    PlayNext,
    SetSpeed(f64),
    SeekFront(bool),
    SeekBack(bool),
    TogglePlay,
    Comment,
    ToggleFullscreen,
    EndOfStream,
    NewFrame,
    CloseView,
    CollectionAddMessage(CollectionAddMessage),
    Playlist(PlaylistMessge),
    ClosePanel,
    PanelClosed,
    Subs(Option<String>),
    Config(ConfigMessage),
    Error(String),
    CommentMessage(CommentMessage),
    ClearIndicator,
    SpeedToggle(bool),
    ModalToggled(bool),
    None,
}

pub struct Manager {
    window: Option<window::Id>,
    playlist: Playlist,
    show_controls: bool,

    pub settings: VideoSettings,
    fonts: Vec<iced::font::Family>,

    indicator: Option<Indicator>,

    maximised: bool,
    is_fullscreen: bool,
    state: State,
    next: AutoState,

    modal: ModalState,
    panel: Option<Panel>,

    speed_toggle: bool,
}

impl Manager {
    const WIDTH: f32 = 250.0;

    pub fn boot(
        window: Option<window::Id>,
        settings: VideoSettings,
        playlist: Playlist,
        fonts: Vec<iced::font::Family>,
    ) -> (Self, Task<ManagerMessage>) {
        let load_video = match playlist.current().cloned() {
            Some(item) => load_video(item, |video| ManagerMessage::Video {
                is_next: false,
                video,
            }),
            None => Task::none(),
        };

        let size = window
            .map(|id| window::size(id).map(move |size| ManagerMessage::Resize((id, size))))
            .unwrap_or_default();

        let tasks = Task::batch([size, load_video]);

        (Self::new(window, settings, playlist, fonts), tasks)
    }

    fn new(
        window: Option<window::Id>,
        settings: VideoSettings,
        playlist: Playlist,
        fonts: Vec<iced::font::Family>,
    ) -> Self {
        let state = if !playlist.is_empty() {
            State::Loading
        } else {
            State::Idle
        };

        Self {
            window,
            playlist,
            fonts,
            show_controls: true,
            indicator: None,
            settings,
            maximised: false,
            is_fullscreen: false,
            state,
            next: AutoState::Idle,
            modal: ModalState::none(),
            panel: None,
            speed_toggle: false,
        }
    }

    pub fn update(&mut self, message: ManagerMessage, now: Instant) -> Task<Message> {
        match message {
            ManagerMessage::None => Task::none(),
            ManagerMessage::Error(error) => Message::error(error, true).tasked(),
            ManagerMessage::Video { is_next, video } => {
                let player = Arc::try_unwrap(video).expect("Second player unwrap");

                let mut player = match player {
                    Ok(player) => player,
                    Err(error) => {
                        let msg = Message::anyhow(error);

                        return Task::done(msg);
                    }
                };

                let comments = Message::FetchComments(player.item.id).tasked();

                let id = player.item.id;
                let path = player.item.path.clone();
                let generate_poster = player.item.generate_poster;

                let interval = self.settings.thumbnail_interval;
                let duration = player.video.duration().as_secs_f64();
                let (width, height) = player.video.size();

                let load_thumbnails = Task::perform(
                    tokio::task::spawn_blocking(move || {
                        use rand::Rng;

                        let convert =
                            |img: Image| image::Handle::from_rgba(img.width, img.height, img.bytes);

                        let num = if duration > (interval as f64) {
                            duration as u32 / interval
                        } else {
                            10
                        };
                        let path = path.canonicalize().with_context(|| {
                            format!("Thumbnail path canonicalize on {}", path.display())
                        })?;

                        let path_ref = path.as_path();
                        let path = url::Url::from_file_path(path_ref).map_err(|_| {
                            anyhow!(
                                "Thumbnail Url from path {} on video {id}",
                                path_ref.display()
                            )
                        })?;
                        let generator = ThumbnailGenerator::new(path, width, height, 8)
                            .with_context(|| format!("New ThumbnailGenerator for {id}"))?;

                        let range = 1..=num;
                        let mut rng = rand::thread_rng();
                        let rng = {
                            let left = num / 4;
                            let right = (num * 3) / 4;

                            rng.gen_range(left..=right).max(*range.start())
                        };

                        let mut poster = None;
                        let mut imgs = vec![];

                        for idx in range {
                            let position = duration * (idx as f64 / num as f64);

                            if generate_poster && idx == rng {
                                let Some((img, pst)) =
                                    generator.generate_with_poster(position).with_ctx_log(|| {
                                        format!("Thumbnail generation with poster at {position}")
                                    })
                                else {
                                    continue;
                                };

                                imgs.push(convert(img));
                                poster = Some(pst);
                            } else {
                                let Some(img) = generator
                                    .generate(position)
                                    .with_ctx_log(|| format!("Thumbnail generation at {position}"))
                                else {
                                    continue;
                                };

                                imgs.push(convert(img));
                            }
                        }

                        drop(generator);

                        Ok::<_, Error>((id, imgs, poster))

                        // temp.log_ctx("Thumbnail generation error")
                    }),
                    move |res| {
                        let res = res
                            .with_context(|| format!("Thumbnail generation join error on {id}"))
                            .flatten()
                            .log_err();
                        match res {
                            Some((id, thumbnails, poster)) => ManagerMessage::Thumbnail {
                                id,
                                thumbnails,
                                poster,
                            },
                            None => ManagerMessage::None,
                        }
                    },
                );

                let (load_thumbnails, handle) = load_thumbnails.map(Message::Player).abortable();
                let handle = handle.abort_on_drop();

                let last_watched = if !is_next || matches!(&self.state, State::Idle) {
                    let task = Task::done(Message::LastWatched(player.item.id));
                    apply_settings(&self.settings, &mut player);

                    let awake =
                        keep_awake().with_ctx_log(|| format!("New KeepAwake on video {id}"));

                    self.state = State::Ready {
                        awake,
                        player: Box::new(player),
                        comments: Comments::new(),
                        thumbnails_handle: Some(handle),
                    };

                    task
                } else {
                    apply_settings(&self.settings, &mut player);
                    player.video.set_paused(true);
                    self.next = AutoState::Ready {
                        player: Box::new(player),
                        comments: Comments::new(),
                        thumbnails_handle: Some(handle),
                    };
                    Task::none()
                };

                Task::batch([load_thumbnails, last_watched, comments])
            }
            ManagerMessage::Thumbnail {
                id,
                thumbnails: generated,
                poster,
            } => {
                let current = self
                    .player()
                    .map(|player| player.item.id == id)
                    .unwrap_or_default();

                if current {
                    if let State::Ready {
                        player,
                        thumbnails_handle,
                        comments: _comments,
                        awake: _awake,
                    } = &mut self.state
                        && player.item.id == id
                    {
                        thumbnails_handle.take();
                        player.thumbnails = generated;
                    }
                } else if let AutoState::Ready {
                    player,
                    thumbnails_handle,
                    comments: _comments,
                } = &mut self.next
                    && player.item.id == id
                {
                    thumbnails_handle.take();
                    player.thumbnails = generated;
                }

                let Some(img) = poster else {
                    return Task::none();
                };

                Task::done(Message::GeneratedPoster { id, img })
            }
            ManagerMessage::Resize((id, size)) => {
                if Some(id) == self.window {
                    let maximised = Size::new(1000., 1000.);
                    self.maximised =
                        size.width >= maximised.width && size.height >= maximised.height;
                    self.show_controls = !self.maximised;
                }
                Task::none()
            }
            ManagerMessage::EndOfStream => {
                if self.settings.auto_next && self.playlist.has_next() {
                    self.play_next()
                } else {
                    if let State::Ready { awake, .. } = &mut self.state {
                        awake.take();
                    }

                    self.stats()
                }
            }
            ManagerMessage::NewFrame => {
                if let State::Ready { player, .. } = &mut self.state
                    && !player.is_dragging
                {
                    player.position = player.video.position().as_secs_f64();
                    player.watch_time += player
                        .last_frame
                        .map(|last| last.elapsed())
                        .unwrap_or_default();
                    player.last_frame = Some(Instant::now());

                    if (player.position) / (player.video.duration().as_secs_f64()) >= 0.9
                        && self.playlist.has_next()
                        && !self.playlist.shuffle
                        && matches!(&self.next, AutoState::Idle)
                        && self.settings.auto_next
                    {
                        self.next = AutoState::Loading;
                        let next = self
                            .playlist
                            .next_peek()
                            .cloned()
                            .map(|item| {
                                load_video(item, |video| ManagerMessage::Video {
                                    is_next: true,
                                    video,
                                })
                            })
                            .unwrap_or_default();

                        return next.map(Message::Player);
                    }
                }
                Task::none()
            }
            ManagerMessage::SeekRelease => {
                let Some(player) = self.player_mut() else {
                    return Task::none();
                };

                if let Err(msg) = player.seek_release(false) {
                    return Task::done(Message::error(msg, true));
                }

                Task::none()
            }
            ManagerMessage::Seek(pos) => {
                if let Some(Player {
                    video,
                    position,
                    is_dragging,
                    last_frame,
                    ..
                }) = self.player_mut()
                {
                    last_frame.take();
                    *position = pos.max(0.0);
                    *is_dragging = true;
                    if !video.paused() {
                        video.set_paused(true);
                    }
                }

                Task::none()
            }
            ManagerMessage::TogglePlay => self.play_toggle(Some(now)),
            ManagerMessage::ChangeVolume(volume) => {
                let volume = volume.clamp(0.0, 1.0);
                self.settings.volume = volume;

                if let Some(Player { video, .. }) = self.player_mut() {
                    video.set_volume(volume);
                }

                Task::none()
            }
            ManagerMessage::ToggleMute => self.mute_toggle(now),
            ManagerMessage::SeekBack(shift) => self.seek_back(shift, now),
            ManagerMessage::SeekFront(shift) => self.seek_front(shift, now),
            ManagerMessage::CursorExit => {
                if self.is_fullscreen || self.maximised {
                    self.show_controls = false;
                }
                Task::none()
            }
            ManagerMessage::CursorEnter => {
                self.show_controls = true;
                Task::none()
            }
            ManagerMessage::ModalToggled(opened) => {
                if opened {
                    Task::none()
                } else {
                    self.close_modal_forced()
                }
            }
            ManagerMessage::ToggleFullscreen => self.fullscreen_toggle(),
            ManagerMessage::PreviousScreen => self.previous_screen(),
            ManagerMessage::ToggleSubtitles => self.subtitles_toggle(),
            ManagerMessage::PlayNext => self.play_next(),
            ManagerMessage::PlayPrevious => self.play_previous(),
            ManagerMessage::AddCollection => self.collection_add(),
            ManagerMessage::OpenConfig => self.video_config(),
            ManagerMessage::Comment => self.video_comment(),
            ManagerMessage::SetSpeed(speed) => self.set_speed(speed),
            ManagerMessage::CloseView => self.close_modal(),
            ManagerMessage::CollectionAddMessage(csg) => {
                let Some(Modal::CollectionAdd {
                    item,
                    collections,
                    mut selected,
                    initial,
                }) = self.modal.take()
                else {
                    return Task::none();
                };

                match csg {
                    CollectionAddMessage::Toggle(toggle, id) => {
                        if toggle {
                            selected.remove(&id);
                        } else {
                            selected.insert(id);
                        }

                        let modal = Modal::CollectionAdd {
                            item,
                            collections,
                            selected,
                            initial,
                        };

                        self.modal.open(modal);
                        Task::none()
                    }
                    CollectionAddMessage::Save => {
                        let mut new = selected
                            .iter()
                            .filter_map(|collection| {
                                (!initial.contains(collection)).then_some((*collection, true))
                            })
                            .collect::<Vec<_>>();

                        let remove = initial.iter().filter_map(|init| {
                            let selected = selected.contains(init);
                            if !selected {
                                Some((*init, false))
                            } else {
                                None
                            }
                        });

                        new.extend(remove);

                        if let State::Ready {
                            player,
                            awake,
                            comments: _comments,
                            thumbnails_handle: _handle,
                        } = &mut self.state
                        {
                            if awake.is_none() {
                                *awake =
                                    keep_awake().ctx_log("New KeepAwake after collection save");
                            }

                            player.video.set_paused(false);
                        }

                        Task::done(Message::ToggleMembership {
                            item: item.into(),
                            collections: new,
                        })
                    }
                }
            }
            ManagerMessage::Playlist(psg) => match psg {
                PlaylistMessge::Toggle => self.toggle_playlist(),
                PlaylistMessge::ToggleAutoNext(play) => {
                    self.settings.auto_next = play;
                    Task::none()
                }
                PlaylistMessge::ToggleShuffle(shuffle) => {
                    self.playlist.shuffle(shuffle);
                    self.next = AutoState::Idle;
                    Task::none()
                }
                PlaylistMessge::ToggleRepeat(repeat) => {
                    self.playlist.repeat(repeat);
                    Task::none()
                }
                PlaylistMessge::PlayItem(item) => {
                    if !self.playlist.set_current(item) {
                        return Task::none();
                    };

                    let load_video = match self.playlist.current().cloned() {
                        Some(item) => load_video(item, |video| ManagerMessage::Video {
                            is_next: false,
                            video,
                        }),
                        None => Task::none(),
                    };

                    load_video.map(Message::Player)
                }
                PlaylistMessge::Save => {
                    if self.playlist.is_empty() {
                        Task::none()
                    } else {
                        Message::PlaylistSave(self.playlist.clone()).tasked()
                    }
                }
            },
            ManagerMessage::ClosePanel => self.close_panel(),
            ManagerMessage::PanelClosed => {
                self.panel.take();
                Task::none()
            }
            ManagerMessage::Subs(subs) => {
                let Some(Player { subtitles, .. }) = self.player_mut() else {
                    return Task::none();
                };

                *subtitles = subs.map(|subs| html_escape::decode_html_entities(&subs).into_owned());

                Task::none()
            }
            ManagerMessage::Config(csg) => {
                let Some(Modal::Config(config)) = self.modal.as_mut() else {
                    return Task::none();
                };

                match csg {
                    ConfigMessage::Tab(new) => {
                        config.tab = new;
                        Task::none()
                    }
                    ConfigMessage::General(gsg) => match gsg {
                        GeneralConfig::VolumeAmt(amt) => {
                            let amt = amt.trim();
                            if amt.is_empty() {
                                self.settings.volume_change_amt = 0.0;
                                return Task::none();
                            }

                            let Ok(amt) = amt.parse::<f64>() else {
                                let msg = Message::error(format!("Invalid input: {amt}"), true);
                                return Task::done(msg);
                            };

                            self.settings.volume_change_amt = amt.min(1.0);

                            Task::none()
                        }
                        GeneralConfig::SpeedAmt(amt) => {
                            let amt = amt.trim();
                            if amt.is_empty() {
                                self.settings.speed_change_amt = 0.0;
                                return Task::none();
                            }

                            let Ok(amt) = amt.parse::<f64>() else {
                                let msg = Message::error(format!("Invalid input: {amt}"), true);
                                return Task::done(msg);
                            };

                            self.settings.speed_change_amt = amt;

                            Task::none()
                        }
                        GeneralConfig::SeekAmt(amt) => {
                            let amt = amt.trim();
                            if amt.is_empty() {
                                self.settings.seek_change_amt = 0.0;
                                return Task::none();
                            }

                            let Ok(amt) = amt.parse::<f64>() else {
                                let msg = Message::error(format!("Invalid input: {amt}"), true);
                                return Task::done(msg);
                            };

                            self.settings.seek_change_amt = amt;

                            Task::none()
                        }
                        GeneralConfig::SeekShiftAmt(amt) => {
                            let amt = amt.trim();
                            if amt.is_empty() {
                                self.settings.seek_shift_change_amt = 0.0;
                                return Task::none();
                            }

                            let Ok(amt) = amt.parse::<f64>() else {
                                let msg = Message::error(format!("Invalid input: {amt}"), true);
                                return Task::done(msg);
                            };

                            self.settings.seek_shift_change_amt = amt;

                            Task::none()
                        }
                        GeneralConfig::Span(span) => {
                            let span = span.trim();
                            if span.is_empty() {
                                self.settings.comment_span = 0;
                                return Task::none();
                            }

                            let Ok(span) = span.parse::<u64>() else {
                                let msg = Message::error(format!("Invalid input: {span}"), true);
                                return Task::done(msg);
                            };

                            self.settings.comment_span = span;

                            Task::none()
                        }
                    },
                    ConfigMessage::Video(vsg) => match vsg {
                        VideoConfig::Gamma(gamma) => {
                            self.settings.filters.gamma = gamma;

                            Task::none()
                        }
                        VideoConfig::Brightness(brightness) => {
                            self.settings.filters.brightness = brightness;

                            Task::none()
                        }
                        VideoConfig::Contrast(contrast) => {
                            self.settings.filters.contrast = contrast;

                            Task::none()
                        }
                        VideoConfig::Hue(hue) => {
                            self.settings.filters.hue = hue;

                            Task::none()
                        }
                        VideoConfig::Saturation(saturation) => {
                            self.settings.filters.saturation = saturation;

                            Task::none()
                        }
                        VideoConfig::CurrentVideo(video) => {
                            config.selected_video = Some(video);

                            Task::none()
                        }
                        VideoConfig::Fit(fit) => {
                            config.fit = fit;
                            Task::none()
                        }
                    },
                    ConfigMessage::Subtitle(ssg) => match ssg {
                        SubtitleConfig::SelectFile => Task::perform(
                            rfd::AsyncFileDialog::new()
                                .add_filter("", devutils::scan::SUB_EXT)
                                .pick_file(),
                            |handle| {
                                ManagerMessage::Config(ConfigMessage::Subtitle(
                                    SubtitleConfig::Selected(
                                        handle.map(|handle| handle.path().to_path_buf()),
                                    ),
                                ))
                            },
                        )
                        .map(Message::Player),
                        SubtitleConfig::Selected(selected) => {
                            config.subtitle_uri = selected;
                            config.selected_text.take();
                            config.subtitle_offset = 0.0;
                            Task::none()
                        }
                        SubtitleConfig::ClearSelected => {
                            config.subtitle_uri.take();

                            config.selected_text = match &mut self.state {
                                State::Ready { player, .. } => player
                                    .item
                                    .subtitle_id
                                    .and_then(|id| {
                                        player.item.subtitles.iter().find(|sub| sub.id == id)
                                    })
                                    .cloned(),
                                _ => None,
                            };

                            config.subtitle_offset = config
                                .selected_text
                                .as_ref()
                                .map(|sub| sub.offset)
                                .unwrap_or_default();

                            Task::none()
                        }
                        SubtitleConfig::CurrentText(text) => {
                            config.subtitle_offset = text.offset;
                            config.selected_text = Some(text);
                            config.subtitle_uri.take();
                            Task::none()
                        }
                        SubtitleConfig::SubSize(size) => {
                            let size = size.trim();
                            if size.is_empty() {
                                self.settings.subtitles.size = 5;
                                return Task::none();
                            }

                            let Ok(size) = size.parse::<u32>() else {
                                let msg = Message::error(format!("Invalid input: {size}"), true);
                                return Task::done(msg);
                            };

                            self.settings.subtitles.size = size.max(5);

                            Task::none()
                        }
                        SubtitleConfig::SubSizeIncr => {
                            self.settings.subtitles.size =
                                (self.settings.subtitles.size + 1).min(60);
                            Task::none()
                        }
                        SubtitleConfig::SubSizeDecr => {
                            self.settings.subtitles.size =
                                (self.settings.subtitles.size - 1).max(5);
                            Task::none()
                        }
                        SubtitleConfig::SubColor(color) => {
                            if let Ok(color) = color.parse::<Color>() {
                                self.settings.subtitles.color = color;
                            }

                            config.text_color = color;

                            Task::none()
                        }
                        SubtitleConfig::SubBackground(color) => {
                            if let Ok(color) = color.parse::<Color>() {
                                self.settings.subtitles.background_color = color;
                            }

                            config.background_color = color;

                            Task::none()
                        }
                        SubtitleConfig::SubFont(family) => {
                            self.settings.subtitles.font = family.to_string();
                            config.subtitle_font.selected = Some(family);

                            Task::none()
                        }
                        SubtitleConfig::OffsetIncr => {
                            config.subtitle_offset += 0.25;

                            Task::none()
                        }
                        SubtitleConfig::OffsetDecr => {
                            config.subtitle_offset -= 0.25;

                            Task::none()
                        }
                        SubtitleConfig::Offset(input) => {
                            if input.is_empty() {
                                config.subtitle_offset = 0.0;
                                return Task::none();
                            }

                            let Ok(input) = input.trim().parse::<f32>() else {
                                return Message::error("Invalid input", true).tasked();
                            };

                            config.subtitle_offset = input;

                            Task::none()
                        }
                    },
                    ConfigMessage::Audio(asg) => match asg {
                        AudioConfig::CurrentAudio(audio) => {
                            config.selected_audio = Some(audio);
                            Task::none()
                        }
                    },
                }
            }
            ManagerMessage::CommentMessage(csg) => {
                let State::Ready {
                    player,
                    comments,
                    awake: _awake,
                    thumbnails_handle: _handle,
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Some(Panel::Comments(new)) = self.panel.as_mut() else {
                    return Task::none();
                };

                match csg {
                    CommentMessage::New => {
                        let id = widget::Id::unique();

                        *new = Some((id.clone(), text_editor::Content::default()));
                        operation::focus(id)
                    }
                    CommentMessage::NewCancel => {
                        new.take();

                        self.play(None)
                    }
                    CommentMessage::NewSubmit => {
                        let Some((editor, comment)) = new.take() else {
                            return Task::none();
                        };

                        let comment = Comment::new(
                            comment.text(),
                            player.item.id,
                            player.position as _,
                            editor,
                        );

                        match comment.inner.timestamp {
                            Some(timestamp) => {
                                let batch = comments.timestamped.entry(timestamp).or_default();
                                batch.push(comment);
                            }
                            None => comments.nulls.push(comment),
                        }

                        self.play(None)
                    }
                    CommentMessage::NewAction(action) => {
                        if let Some((_, content)) = new {
                            content.perform(action);
                        }

                        self.pause(None)
                    }
                    CommentMessage::Link(url) => {
                        match url::Url::parse(&url) {
                            Ok(url) if url.scheme() == "video" => {
                                let url = match url.path().strip_prefix("/") {
                                    Some(url) => url,
                                    None => url.path(),
                                };

                                let position = match url.parse::<u64>() {
                                    Ok(position) => position,
                                    Err(error) => {
                                        let msg = Message::error(error, true);
                                        return msg.tasked();
                                    }
                                };

                                player.position = position as f64;
                                if let Err(msg) = player.seek_release(false) {
                                    return Task::done(Message::error(msg, true));
                                }
                            }
                            Ok(url) => {
                                if let Err(error) = open::that(url.as_str()) {
                                    return Message::error(error, true).tasked();
                                }
                            }
                            Err(error) => {
                                return Message::error(error, true).tasked();
                            }
                        }

                        Task::none()
                    }
                    CommentMessage::Edit { id, timestamp } => {
                        let comment = match timestamp {
                            Some(timestamp) => {
                                match comments.timestamped.get_mut(&timestamp).and_then(
                                    |comments| {
                                        comments.iter_mut().find(|comment| comment.inner.id == id)
                                    },
                                ) {
                                    Some(comment) => comment,
                                    None => return Task::none(),
                                }
                            }
                            None => {
                                match comments
                                    .nulls
                                    .iter_mut()
                                    .find(|comment| comment.inner.id == id)
                                {
                                    Some(comment) => comment,
                                    None => return Task::none(),
                                }
                            }
                        };

                        comment.edit()
                    }
                    CommentMessage::Save { id, timestamp } => {
                        let mut saved = match timestamp {
                            Some(timestamp) => {
                                let Some(batch) = comments.timestamped.get_mut(&timestamp) else {
                                    return Task::none();
                                };

                                let Some((idx, _)) = batch
                                    .iter()
                                    .enumerate()
                                    .find(|(_, comment)| comment.inner.id == id)
                                else {
                                    return Task::none();
                                };

                                batch.remove(idx)
                            }
                            None => {
                                let Some((idx, _)) = comments
                                    .nulls
                                    .iter()
                                    .enumerate()
                                    .find(|(_, comment)| comment.inner.id == id)
                                else {
                                    return Task::none();
                                };

                                comments.nulls.remove(idx)
                            }
                        };

                        let timestamp = saved.save(player.position as u64);

                        match timestamp {
                            Some(timestamp) => {
                                let batch = comments.timestamped.entry(timestamp).or_default();
                                batch.push(saved);
                            }
                            None => {
                                comments.nulls.push(saved);
                            }
                        }

                        self.play(None)
                    }
                    CommentMessage::Action {
                        id,
                        timestamp,
                        action,
                    } => {
                        if let Some(comment) = comments.get_mut(id, timestamp) {
                            comment.perform_action(action);
                        };

                        self.pause(None)
                    }
                    CommentMessage::Cancel { id, timestamp } => {
                        if let Some(comment) = comments.get_mut(id, timestamp) {
                            comment.cancel();
                        }

                        self.play(None)
                    }
                    CommentMessage::Delete { id, timestamp } => {
                        if let Some(comment) = comments.get_mut(id, timestamp) {
                            comment.inner.removed = true;
                        };

                        Task::none()
                    }
                    CommentMessage::ImageShown { id, timestamp, url } => {
                        let Some(comment) = comments.get_mut(id, timestamp) else {
                            return Task::none();
                        };

                        if comment.images.contains_key(&url) {
                            return Task::none();
                        }

                        let _ = comment.images.insert(url.clone(), comment::Image::Loading);

                        Task::perform(comment::download_image(url.clone()), move |res| {
                            Message::Player(ManagerMessage::CommentMessage(
                                CommentMessage::ImageDownloaded {
                                    id,
                                    timestamp,
                                    url,
                                    image: res,
                                },
                            ))
                        })
                    }
                    CommentMessage::ImageDownloaded {
                        id,
                        timestamp,
                        url,
                        image,
                    } => {
                        let Some(images) = comments
                            .get_mut(id, timestamp)
                            .map(|comment| &mut comment.images)
                        else {
                            return Task::none();
                        };

                        let _ = images.insert(
                            url,
                            image
                                .map(|handle| comment::Image::Ready {
                                    handle,
                                    fade_in: Animation::new(false)
                                        .duration(Duration::from_millis(750))
                                        .easing(Easing::EaseInOut)
                                        .go(true, now),
                                })
                                .unwrap_or_else(comment::Image::Errored),
                        );

                        Task::none()
                    }
                }
            }
            ManagerMessage::ClearIndicator => {
                self.indicator.take();
                Task::none()
            }
            ManagerMessage::SpeedToggle(toggle) => {
                self.speed_toggle = toggle;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<ManagerMessage> {
        window::resize_events().map(ManagerMessage::Resize)
    }

    fn top(&self) -> Element<'_, ManagerMessage> {
        let title: Element<'_, ManagerMessage> = match &self.state {
            State::Ready { player, .. } => {
                let title = marquee(&player.item.name).size(H4).font(medium_font());

                container(title)
                    .style(theme::container::text)
                    .center_x(Length::FillPortion(12))
                    .center_y(36)
                    .into()
            }
            State::Loading | State::Idle => empty(),
        };

        let icon_size = if self.is_fullscreen { H4 } else { H5 };

        let options = column!(
            row!(
                sized_button(icons::ELLIPSIS_VER, icon_size)
                    .on_press(ManagerMessage::OpenConfig)
                    .style(theme::button::text_slate)
            )
            .spacing(6.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);

        let back = container(tooltip(
            sized_button(icons::BACK, icon_size)
                .on_press(ManagerMessage::PreviousScreen)
                .style(theme::button::text_slate),
            "Exit Player",
            tp::Position::Bottom,
        ))
        .align_x(Horizontal::Left)
        .align_y(Vertical::Center)
        .width(Self::WIDTH);

        let content = row!(
            back,
            space::horizontal(),
            title,
            space::horizontal(),
            options
        )
        .spacing(5.0)
        .width(Length::Fill)
        .align_y(Vertical::Center);

        let content: Element<'_, ManagerMessage> = if self.show_controls || self.is_eos() {
            content.into()
        } else {
            space::horizontal().height(35).into()
        };

        let content = mouse_area(content)
            .on_exit(ManagerMessage::CursorExit)
            .on_enter(ManagerMessage::CursorEnter);

        content.into()
    }

    fn timeline(&self) -> Element<'_, ManagerMessage> {
        match self.player() {
            Some(Player {
                video,
                position,
                thumbnails,
                ..
            }) => {
                let duration = video.duration();
                let spent = duration_string(*position as u64);
                let spent = container(medium(spent))
                    .style(theme::container::text)
                    .width(60.0);

                let remaining = duration.as_secs().saturating_sub(*position as u64);
                let remaining = duration_string(remaining);
                let remaining = container(medium(remaining))
                    .style(theme::container::text)
                    .width(60.0);

                let slider = widgets::slider::VideoSlider::new(
                    0.0..=duration.as_secs_f64(),
                    *position,
                    ManagerMessage::Seek,
                    thumbnails,
                    mono_font(),
                    duration,
                )
                .step(0.1)
                .on_release(ManagerMessage::SeekRelease);

                row!(spent, slider, remaining)
                    .spacing(20.0)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .into()
            }
            _ => space::horizontal().into(),
        }
    }

    fn is_ready(&self, message: ManagerMessage) -> Option<ManagerMessage> {
        matches!(&self.state, State::Ready { .. }).then_some(message)
    }

    fn media_controls(&self) -> Element<'_, ManagerMessage> {
        let icon_size = if self.is_fullscreen { H4 } else { H5 };
        let tp = tp::Position::Top;

        let volume = {
            let volume = slider(
                0.0..=1.0,
                if self.settings.muted {
                    0.0
                } else {
                    self.settings.volume
                },
                ManagerMessage::ChangeVolume,
            )
            .step(0.05)
            .shift_step(0.1)
            .width(125.0);

            let volume_text = format!("{:.0}", self.settings.volume * 100.0);

            tooltip(volume, volume_text, tp)
        };

        let speed = {
            let current = self.settings.speed;
            let overlay = move |speed: f64| {
                let style = if current == speed {
                    theme::button::text_primary
                } else {
                    theme::button::text
                };

                button(mono_bold(format!("{speed:.2}✕")))
                    .padding(0)
                    .on_press(ManagerMessage::SetSpeed(speed))
                    .style(style)
                    .into()
            };

            let opts = [0.5, 0.75, 1., 1.25, 1.5, 1.75, 2.]
                .into_iter()
                .map(overlay);

            let opts = column(opts).spacing(4).align_x(Horizontal::Right);

            let opts = container(opts).padding([5, 8]).style(|theme| {
                let default = theme::container::bb(theme);
                let background = default.background.map(|bg| bg.scale_alpha(0.9));

                container::Style {
                    background,
                    ..default
                }
            });

            let current = container(
                mono_bold(format!("{:.2}✕", self.settings.speed)).size(icon_size / (typo::RATIO)),
            )
            .style(theme::container::text);

            let speed = menu(current, opts)
                .toggle(self.speed_toggle)
                .top()
                .auto_close(true)
                .on_toggle(ManagerMessage::SpeedToggle);

            tooltip(speed, "Playback speed", tp::Position::Bottom)
        };

        let subtitles = {
            if self.settings.show_subtitles {
                tooltip(
                    sized_button(icons::SUBTITLES_OFF, icon_size)
                        .on_press(ManagerMessage::ToggleSubtitles)
                        .style(theme::button::text_slate),
                    "Subtitles off",
                    tp,
                )
            } else {
                tooltip(
                    sized_button(icons::SUBTITLES_ON, icon_size)
                        .on_press(ManagerMessage::ToggleSubtitles)
                        .style(theme::button::text_slate),
                    "Subtitles on",
                    tp,
                )
            }
        };

        let mute = {
            if self.settings.muted {
                tooltip(
                    sized_button(icons::MUTE, icon_size)
                        .on_press(ManagerMessage::ToggleMute)
                        .style(theme::button::text_slate),
                    "Unmute",
                    tp,
                )
            } else {
                tooltip(
                    sized_button(icons::VOLUME, icon_size)
                        .on_press(ManagerMessage::ToggleMute)
                        .style(theme::button::text_slate),
                    "Mute",
                    tp,
                )
            }
        };

        let left = row!(subtitles, speed, mute, volume)
            .spacing(4.0)
            .align_y(Vertical::Center)
            .width(Self::WIDTH);

        let middle = {
            let size = if self.is_fullscreen {
                H2 * typo::RATIO
            } else {
                H2
            };

            let play: Element<'_, ManagerMessage> = match &self.state {
                State::Idle => sized_button(icons::PLAY, size)
                    .style(theme::button::text_slate)
                    .into(),
                State::Loading => container(throbber::circular().bar_height(3.0))
                    .style(theme::container::text)
                    .width(size)
                    .height(size)
                    .into(),
                State::Ready { player, .. } => {
                    let (icon, message) = if player.video.eos() {
                        (icons::REPLAY, ManagerMessage::TogglePlay)
                    } else if player.video.paused() {
                        (icons::PLAY, ManagerMessage::TogglePlay)
                    } else {
                        (icons::PAUSE, ManagerMessage::TogglePlay)
                    };

                    sized_button(icon, size)
                        .on_press(message)
                        .style(theme::button::text_slate)
                        .into()
                }
            };

            let previous: Element<'_, ManagerMessage> = match self.playlist.previous_peek() {
                Some(previous) => tooltip(
                    sized_button(icons::PREVIOUS_VIDEO, size)
                        .style(theme::button::text_slate)
                        .on_press(ManagerMessage::PlayPrevious),
                    &previous.name,
                    tp,
                )
                .into(),
                None => sized_button(icons::PREVIOUS_VIDEO, size)
                    .style(theme::button::text_slate)
                    .into(),
            };

            let next: Element<'_, ManagerMessage> = match self.playlist.next_peek() {
                Some(next) => tooltip(
                    sized_button(icons::NEXT_VIDEO, size)
                        .style(theme::button::text_slate)
                        .on_press(ManagerMessage::PlayNext),
                    &next.name,
                    tp,
                )
                .into(),
                None => sized_button(icons::NEXT_VIDEO, size)
                    .style(theme::button::text_slate)
                    .into(),
            };

            let seek_amt = self.settings.seek_change_amt.trunc() as i16;
            let sb = tooltip(
                sized_button(icons::SEEK_BACK, size)
                    .style(theme::button::text_slate)
                    .on_press_maybe(self.is_ready(ManagerMessage::SeekBack(false))),
                format!(
                    "Backward {} sec{}",
                    seek_amt,
                    if seek_amt.abs() > 1 { "s" } else { "" }
                ),
                tp,
            );

            let sf = tooltip(
                sized_button(icons::SEEK_FRONT, size)
                    .style(theme::button::text_slate)
                    .on_press_maybe(self.is_ready(ManagerMessage::SeekFront(false))),
                format!(
                    "Forward {} sec{}",
                    seek_amt,
                    if seek_amt.abs() > 1 { "s" } else { "" }
                ),
                tp,
            );

            row!(previous, sb, play, sf, next,)
                .spacing(2.0)
                .align_y(Vertical::Center)
        };

        let full = if self.is_fullscreen {
            tooltip(
                sized_button(icons::MINIMIZE, icon_size)
                    .style(theme::button::text_slate)
                    .on_press(ManagerMessage::ToggleFullscreen),
                "Exit Fullscreen",
                tp,
            )
        } else {
            tooltip(
                sized_button(icons::MAXIMIZE, icon_size)
                    .style(theme::button::text_slate)
                    .on_press(ManagerMessage::ToggleFullscreen),
                "Enter Fullscreen",
                tp,
            )
        };

        let right = column!(
            row!(
                tooltip(
                    sized_button(icons::ADD_COLLECTION, icon_size * typo::RATIO)
                        .style(theme::button::text_slate)
                        .on_press_maybe(self.is_ready(ManagerMessage::AddCollection)),
                    "Add to collection",
                    tp
                ),
                tooltip(
                    sized_button(icons::COMMENT, icon_size)
                        .style(theme::button::text_slate)
                        .on_press_maybe(self.is_ready(ManagerMessage::Comment)),
                    "Comments",
                    tp
                ),
                tooltip(
                    sized_button(icons::PLAYLIST, icon_size)
                        .style(theme::button::text_slate)
                        .on_press_maybe(
                            self.is_ready(ManagerMessage::Playlist(PlaylistMessge::Toggle))
                        ),
                    "Playlist",
                    tp
                ),
                full
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);

        let content = row!(
            left,
            space::horizontal(),
            middle,
            space::horizontal(),
            right
        )
        .width(Length::Fill)
        .align_y(Vertical::Center);

        let content = column!(self.timeline(), content, space::vertical().height(8.0))
            .align_x(Horizontal::Center)
            .spacing(8)
            .width(Length::Fill);

        let content: Element<'_, ManagerMessage> =
            if self.show_controls || self.is_eos() || self.speed_toggle {
                content.into()
            } else {
                space::horizontal().height(75).into()
            };

        let content = mouse_area(content)
            .on_exit(ManagerMessage::CursorExit)
            .on_enter(ManagerMessage::CursorEnter);

        let content = column!(self.subtitle_draw(), content).align_x(Horizontal::Center);

        content.into()
    }

    fn video_elem(&self) -> Element<'_, ManagerMessage> {
        match &self.state {
            State::Ready { player, .. } => {
                let fit = match self.modal.as_ref() {
                    Some(Modal::Config(config)) => Some(config.fit),
                    _ => None,
                }
                .unwrap_or(player.fit);

                let video = container(
                    VideoPlayer::new(&player.video)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .on_click(handle_clicks)
                        .on_error(|error| ManagerMessage::Error(error.to_string()))
                        .content_fit(fit)
                        .on_end_of_stream(ManagerMessage::EndOfStream)
                        .on_new_frame(ManagerMessage::NewFrame)
                        .on_subtitle_text(ManagerMessage::Subs),
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

                video.into()
            }
            State::Loading => center(throbber::circular().radius(60.0).bar_height(5.0)).into(),
            State::Idle => center("No video loaded").into(),
        }
    }

    fn subtitle_draw(&self) -> Element<'_, ManagerMessage> {
        let Some(Player {
            item,
            subtitles: Some(subtitles),
            ..
        }) = self.player()
        else {
            return empty();
        };

        if !self.settings.show_subtitles || item.subtitle_id.is_none() {
            return empty();
        }

        let subtitles = draw_subtitles(subtitles, &self.settings.subtitles);

        let content = row!(space::horizontal(), subtitles, space::horizontal())
            .width(Length::Fill)
            .align_y(Vertical::Center);

        let content = column!(content, space::vertical().height(8));

        content.into()
    }

    pub fn indicator(&self, now: Instant) -> Element<'_, ManagerMessage> {
        let Some(indicator) = &self.indicator else {
            return empty();
        };

        let alpha = indicator.animation.interpolate(1.0, 0.0, now);

        let color = move |theme: &Theme| {
            let default = theme::text::primary(theme);

            text::Style {
                color: default
                    .color
                    .map(|color| iced::Color::from_rgba(color.r, color.g, color.b, alpha)),
            }
        };

        let size = H1 * typo::RATIO;

        let padding = if matches!(indicator.kind, IndicatorKind::Pause | IndicatorKind::Play) {
            Padding::new(24.0).horizontal(32.0)
        } else {
            Padding::new(24.0)
        };

        let icon = icons::icon(indicator.kind.char()).size(size).style(color);

        match indicator.kind {
            IndicatorKind::SeekBack(amt) => {
                let amt = sized_medium(amt, H6).style(color);

                row!(icon, amt, space::horizontal())
                    .padding(padding)
                    .align_y(Vertical::Center)
                    .into()
            }
            IndicatorKind::SeekFront(amt) => {
                let amt = sized_medium(format!("+{amt}"), H6).style(color);

                row!(space::horizontal(), amt, icon)
                    .padding(padding)
                    .align_y(Vertical::Center)
                    .into()
            }
            _ => {
                let content = container(icon).padding(padding).style(move |theme| {
                    let default = theme::container::dark(theme);

                    let border = default.border.rounded(100.0);

                    container::Style {
                        border,
                        background: default
                            .background
                            .map(|background| background.scale_alpha(alpha)),
                        ..default
                    }
                });

                row!(space::horizontal(), content, space::horizontal()).into()
            }
        }
    }

    pub fn view<'a>(&'a self, theme: &'a Theme, now: Instant) -> Element<'a, ManagerMessage> {
        let content = stack!(
            self.video_elem(),
            column!(
                self.top(),
                space::vertical(),
                self.indicator(now),
                space::vertical(),
                self.media_controls()
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(3.0).left(8).right(16))
        )
        .height(Length::Fill)
        .width(Length::Fill);

        let content = container(content)
            .width(Length::FillPortion(9))
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                ..Default::default()
            });

        let content: Element<'_, ManagerMessage> = {
            fn trans<'a>(
                init: bool,
                view: impl Fn(Length) -> Element<'a, ManagerMessage> + 'a,
            ) -> transition::Transition<'a, ManagerMessage, Theme, iced::Renderer, Animation<bool>>
            {
                let length = Length::Fixed(350.0);

                transition(
                    init,
                    move || Animation::new(!init),
                    move |animation, now| {
                        let length = InterpolableLength::from(length);
                        let length =
                            animation.interpolate(InterpolableLength::FIXED_ZERO, length, now);

                        view(length.0)
                    },
                )
            }

            match &self.panel {
                Some(panel) => 'panel: {
                    let trans = match panel {
                        Panel::Playlist => trans(true, |width| {
                            draw_playlist(&self.playlist, self.settings.auto_next, width)
                        }),
                        Panel::PlaylistClosing => trans(false, |width| {
                            draw_playlist(&self.playlist, self.settings.auto_next, width)
                        })
                        .on_finish(ManagerMessage::PanelClosed),
                        Panel::Comments(new) => match &self.state {
                            State::Ready {
                                player,
                                comments,
                                awake: _awake,
                                thumbnails_handle: _handle,
                            } => trans(true, move |width| {
                                draw_comments(
                                    new,
                                    comments,
                                    player.position as u64,
                                    self.settings.comment_span,
                                    theme,
                                    width,
                                    now,
                                )
                            }),
                            _ => break 'panel content.into(),
                        },
                        Panel::CommentsClosing => match &self.state {
                            State::Ready {
                                player,
                                comments,
                                awake: _awake,
                                thumbnails_handle: _handle,
                            } => trans(false, move |width| {
                                draw_comments(
                                    &None,
                                    comments,
                                    player.position as u64,
                                    self.settings.comment_span,
                                    theme,
                                    width,
                                    now,
                                )
                            })
                            .on_finish(ManagerMessage::PanelClosed),
                            _ => break 'panel content.into(),
                        },
                    };

                    row!(content, trans).height(Length::Fill).into()
                }
                None => content.into(),
            }
        };

        match self.modal.view() {
            None => content,
            Some((view, open)) => {
                let modal = |overlay| {
                    widgets::modal(content, overlay)
                        .on_blur(ManagerMessage::CloseView)
                        .on_complete(ManagerMessage::ModalToggled)
                        .toggle(open)
                        .center()
                        .into()
                };

                match view {
                    Modal::CollectionAdd {
                        collections,
                        selected,
                        ..
                    } => {
                        let overlay = draw_collection_add(
                            selected,
                            collections.is_empty(),
                            collections.iter(),
                        );

                        modal(overlay)
                    }
                    Modal::Config(config) => {
                        let Some(player) = self.player() else {
                            return empty();
                        };

                        let item = &player.item;
                        let subs = item.subtitles.as_slice();
                        let audio = item.audios.as_slice();
                        let videos = item.videos.as_slice();

                        let overlay = draw_config(
                            &self.settings,
                            config,
                            subs,
                            audio,
                            videos,
                            item,
                            player.file_size,
                        );

                        modal(overlay)
                    }
                }
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let state = match &self.state {
            State::Ready { comments, .. } => comments.is_animating(now),
            State::Loading | State::Idle => false,
        };

        let indicator = self
            .indicator
            .as_ref()
            .map(|indicator| indicator.animation.is_animating(now))
            .unwrap_or_default();

        state || indicator
    }

    fn player(&self) -> Option<&Player> {
        match &self.state {
            State::Ready { player, .. } => Some(player),
            _ => None,
        }
    }

    fn player_mut(&mut self) -> Option<&mut Player> {
        match &mut self.state {
            State::Ready { player, .. } => Some(player),
            _ => None,
        }
    }

    fn is_eos(&self) -> bool {
        self.player()
            .map(|player| player.video.eos())
            .unwrap_or_default()
    }

    fn play_toggle(&mut self, now: Option<Instant>) -> Task<Message> {
        let State::Ready {
            player,
            awake: _awake,
            comments: _comments,
            thumbnails_handle: _handle,
        } = &mut self.state
        else {
            return Task::none();
        };

        let Player { video, .. } = player.as_mut();

        let is_paused = video.paused();

        if is_paused {
            self.play(now)
        } else {
            self.pause(now)
        }
    }

    fn play(&mut self, now: Option<Instant>) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
            thumbnails_handle: _handle,
        } = &mut self.state
        else {
            return Task::none();
        };

        let Player {
            video, position, ..
        } = player.as_mut();

        let is_paused = video.paused();

        if is_paused {
            *awake = keep_awake().ctx_log("New KeepAwake after play");
        } else {
            return Task::none();
        }

        if video.eos() && is_paused {
            if let Err(error) = video.seek(Duration::from_secs(0), false) {
                return Message::error(error, true).tasked();
            }

            *position = 0.0;

            video.set_paused(false);
        } else {
            video.set_paused(!video.paused());
        }

        match now {
            Some(now) => {
                let (indicator, task) = Indicator::new(IndicatorKind::Play, now);
                self.indicator = Some(indicator);

                task.map(Message::Player)
            }
            None => Task::none(),
        }
    }

    fn pause(&mut self, now: Option<Instant>) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
            thumbnails_handle: _handle,
        } = &mut self.state
        else {
            return Task::none();
        };

        *awake = None;

        let Player { video, .. } = player.as_mut();

        if video.paused() || video.eos() {
            return Task::none();
        }

        video.set_paused(true);

        match now {
            Some(now) => {
                let (indicator, task) = Indicator::new(IndicatorKind::Pause, now);
                self.indicator = Some(indicator);

                task.map(Message::Player)
            }
            None => Task::none(),
        }
    }

    fn fullscreen_toggle(&mut self) -> Task<Message> {
        if self.modal.as_ref().is_some() {
            return Task::none();
        }

        self.show_controls = self.is_fullscreen;
        self.is_fullscreen = !self.is_fullscreen;
        let fullscreen = self.is_fullscreen;

        self.window
            .map(move |id| {
                window::set_mode::<Message>(
                    id,
                    if fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    },
                )
                .discard()
            })
            .unwrap_or_default()
    }

    pub fn fullscreen_exit(&mut self) -> Task<Message> {
        self.show_controls = true;
        self.is_fullscreen = false;

        self.window
            .map(move |id| window::set_mode::<Message>(id, window::Mode::Windowed).discard())
            .unwrap_or_default()
    }

    fn previous_screen(&mut self) -> Task<Message> {
        Task::done(Message::Back)
    }

    pub fn back(&mut self) -> Task<Message> {
        let eos = self
            .player()
            .map(|player| player.video.eos())
            .unwrap_or_default();

        let stats = if eos { Task::none() } else { self.stats() };

        self.fullscreen_exit().chain(Task::batch([stats]))
    }

    fn seek_back(&mut self, shift: bool, now: Instant) -> Task<Message> {
        let shift_amt = if shift {
            self.settings.seek_shift_change_amt
        } else {
            self.settings.seek_change_amt
        };

        let seek = match &mut self.state {
            State::Ready { player, .. } => {
                player.is_dragging = false;

                player.last_frame.take();
                player.position = (player.position - shift_amt).max(0.0);

                match player
                    .video
                    .seek(Duration::from_secs_f64(player.position), false)
                    .context("Seeking back failed")
                {
                    Ok(_) => Task::none(),
                    Err(error) => Message::anyhow(error).tasked(),
                }
            }
            _ => Task::none(),
        };

        let indicator_amt = match self.indicator.as_ref().map(|ind| ind.kind) {
            Some(IndicatorKind::SeekBack(amt)) => amt - shift_amt,
            _ => -shift_amt,
        };

        let (indicator, task) = Indicator::new(IndicatorKind::SeekBack(indicator_amt), now);
        self.indicator = Some(indicator);

        Task::batch([task.map(Message::Player), seek])
    }

    fn seek_front(&mut self, shift: bool, now: Instant) -> Task<Message> {
        let shift_amt = if shift {
            self.settings.seek_shift_change_amt
        } else {
            self.settings.seek_change_amt
        };

        let seek = match &mut self.state {
            State::Ready { player, .. } => {
                player.is_dragging = false;
                let duration = player.video.duration().as_secs_f64();

                player.last_frame.take();
                player.position = (player.position + shift_amt).min(duration);

                match player
                    .video
                    .seek(Duration::from_secs_f64(player.position), false)
                    .context("Seeking forward failed")
                {
                    Ok(_) => Task::none(),
                    Err(error) => Message::anyhow(error).tasked(),
                }
            }
            _ => Task::none(),
        };

        let indicator_amt = match self.indicator.as_ref().map(|ind| ind.kind) {
            Some(IndicatorKind::SeekFront(amt)) => amt + shift_amt,
            _ => shift_amt,
        };

        let (indicator, task) = Indicator::new(IndicatorKind::SeekFront(indicator_amt), now);
        self.indicator = Some(indicator);

        Task::batch([task.map(Message::Player), seek])
    }

    fn volume_increase(&mut self, now: Instant) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.volume =
                (self.settings.volume + self.settings.volume_change_amt).min(1.0);
            player.video.set_volume(self.settings.volume);
        }

        let (indicator, task) = Indicator::new(IndicatorKind::VolumeUp, now);
        self.indicator = Some(indicator);

        task.map(Message::Player)
    }

    fn volume_decrease(&mut self, now: Instant) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.volume =
                (self.settings.volume - self.settings.volume_change_amt).max(0.0);
            player.video.set_volume(self.settings.volume);
        }

        let (indicator, task) = Indicator::new(IndicatorKind::VolumeDown, now);
        self.indicator = Some(indicator);

        task.map(Message::Player)
    }

    fn mute_toggle(&mut self, now: Instant) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            let mute = !player.video.muted();
            player.video.set_muted(mute);
            self.settings.muted = mute;
        }

        let kind = if self.settings.muted {
            IndicatorKind::Mute
        } else {
            IndicatorKind::Unmute
        };
        let (indicator, task) = Indicator::new(kind, now);
        self.indicator = Some(indicator);

        task.map(Message::Player)
    }

    fn set_speed(&mut self, speed: f64) -> Task<Message> {
        let speed = speed.clamp(0.1, 3.0);
        self.speed_toggle = false;

        if self.settings.speed == speed {
            return Task::none();
        }

        self.settings.speed = speed;

        match &mut self.state {
            State::Ready { player, .. } => {
                match player
                    .video
                    .set_speed(self.settings.speed)
                    .with_context(|| format!("Set speed {speed:.2} Failed"))
                {
                    Ok(_) => Task::none(),
                    Err(error) => Message::anyhow(error).tasked(),
                }
            }
            _ => Task::none(),
        }
    }

    fn speed_increase(&mut self) -> Task<Message> {
        let speed = self.settings.speed + self.settings.speed_change_amt;
        self.set_speed(speed)
    }

    fn speed_decrease(&mut self) -> Task<Message> {
        let speed = self.settings.speed - self.settings.speed_change_amt;
        self.set_speed(speed)
    }

    fn speed_reset(&mut self) -> Task<Message> {
        self.set_speed(1.0)
    }

    fn subtitles_toggle(&mut self) -> Task<Message> {
        self.settings.show_subtitles = !self.settings.show_subtitles;
        Task::none()
    }

    fn play_next(&mut self) -> Task<Message> {
        if !self.playlist.has_next() {
            return Task::none();
        }

        let stats = self.stats();

        let Some(next) = self.playlist.next() else {
            return Task::none();
        };

        match &mut self.next {
            AutoState::Idle => {
                self.state = State::Loading;

                let load = load_video(next.clone(), |video| ManagerMessage::Video {
                    is_next: false,
                    video,
                })
                .map(Message::Player);

                Task::batch([stats, load])
            }
            AutoState::Loading => {
                // todo: I probably want to inform the user the next video is still loading
                // Idle doesn't do that. Idle is needed so the loaded video is played immediately
                // after loading though.
                self.state = State::Idle;
                stats
            }
            ready => {
                let player = std::mem::replace(ready, AutoState::Idle);
                let (mut player, comments, thumbnails_handle) = match player {
                    AutoState::Ready {
                        player,
                        comments,
                        thumbnails_handle,
                    } => (player, comments, thumbnails_handle),
                    _ => unreachable!(),
                };

                let last_watched = Message::LastWatched(player.item.id);

                apply_settings(&self.settings, &mut player);
                player.video.set_paused(false);

                let awake = keep_awake().ctx_log("New KeepAwake after play next");

                self.state = State::Ready {
                    player,
                    awake,
                    comments,
                    thumbnails_handle,
                };

                Task::batch([Task::done(last_watched), stats])
            }
        }
    }

    fn play_previous(&mut self) -> Task<Message> {
        if !self.playlist.has_previous() {
            return Task::none();
        }

        let stats = self.stats();

        let Some(previous) = self.playlist.previous() else {
            return Task::none();
        };

        // Intentionally discarding the current video. Don't want to hold on to
        // some arbitarily sized memory for who knows how long
        self.state = State::Loading;
        self.next = AutoState::Idle;

        let load = load_video(previous.clone(), |video| ManagerMessage::Video {
            is_next: false,
            video,
        })
        .map(Message::Player);

        Task::batch([load, stats])
    }

    pub fn fetched_collections(&mut self, collections: Vec<SimpleCollection>) -> Task<Message> {
        let Some(Modal::CollectionAdd {
            collections: view, ..
        }) = self.modal.as_mut()
        else {
            return Task::none();
        };

        *view = collections;

        Task::none()
    }

    pub fn fetched_membership_ids(&mut self, collections: Vec<CollectionId>) -> Task<Message> {
        let Some(Modal::CollectionAdd {
            selected, initial, ..
        }) = self.modal.as_mut()
        else {
            return Task::none();
        };

        selected.extend(collections.clone());
        initial.extend(collections);

        Task::none()
    }

    fn save_config(&mut self, modal: Modal) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comment,
            thumbnails_handle: _handle,
        } = &mut self.state
        else {
            return Task::none();
        };

        let Modal::Config(config) = modal else {
            return self.play_toggle(None);
        };

        let Config {
            subtitle_uri,
            selected_text,
            selected_audio,
            selected_video,
            tab: _tab,
            text_color: _text_color,
            background_color: _background,
            subtitle_font: _subtitle,
            subtitle_offset,
            fit,
        } = *config;

        player.fit = fit;
        apply_settings(&self.settings, player);

        let set_loaded = |player: &mut Player, url: url::Url| {
            let position = player.position;
            let position = Duration::from_secs_f64(position);

            if let Err(error) = player.video.set_subtitle_url(&url) {
                return Some(Message::error(error, true));
            };

            std::thread::sleep(std::time::Duration::from_millis(150));
            if let Err(error) = player.video.seek(position, false) {
                return Some(Message::error(error, true));
            };

            None
        };

        let set_uri = |player: &mut Player, uri: PathBuf| {
            let path = match uri.canonicalize() {
                Ok(subtitles) => subtitles,
                Err(error) => return Some(Message::error(error, true)),
            };

            let url = match url::Url::from_file_path(path) {
                Ok(url) => url,
                Err(_) => {
                    return Some(Message::error(
                        "Cannot generate url from subtitle path",
                        true,
                    ));
                }
            };

            set_loaded(player, url)
        };

        if let Some(selected) = subtitle_uri {
            let existing = player
                .item
                .subtitles
                .iter()
                .find_map(|sub| match &sub.kind {
                    SubtitleKind::Loaded { path, .. } => {
                        let path = path.clone();
                        if selected == path {
                            Some((sub.id, path))
                        } else {
                            None
                        }
                    }
                    SubtitleKind::Embedded => None,
                });

            match existing {
                Some((id, path)) => {
                    if player.item.subtitle_id != Some(id) {
                        player.item.subtitle_id = Some(id);

                        if let Some(message) = set_uri(player, path) {
                            return Task::done(message);
                        }
                    }
                }
                None => {
                    let path = selected.display().to_string();
                    let new = Subtitle::new_loaded(player.item.id, path);

                    player.item.subtitle_id = Some(new.id);
                    player.item.subtitles.insert(0, new);

                    if let Some(message) = set_uri(player, selected) {
                        return Task::done(message);
                    }
                }
            }
        }

        if let Some(selected) = selected_text
            && player.item.subtitle_id != Some(selected.id)
        {
            player.item.subtitle_id = Some(selected.id);
            match &selected.kind {
                SubtitleKind::Loaded { path, .. } => {
                    let path = path.clone();

                    if let Some(message) = set_uri(player, path) {
                        return Task::done(message);
                    }
                }
                SubtitleKind::Embedded => {
                    if let Some(tag) = player.video.available_subtitles().into_iter().find(|tag| {
                        tag.title == selected.title && tag.language_code == selected.lang
                    }) {
                        player.video.set_text(tag);
                    };
                }
            }
        }

        if let Some(subtitle) = player.item.subtitle_id.and_then(|id| {
            player
                .item
                .subtitles
                .iter_mut()
                .find(|sub| sub.id == id && (sub.offset != subtitle_offset))
        }) {
            subtitle.offset = subtitle_offset;
            let offset = (1_000_000_000 as f32 * subtitle_offset) as i64;

            player.video.set_text_offset(offset);
        }

        if let Some(audio) = selected_audio
            && player.item.audio_id != Some(audio.id)
        {
            player.item.audio_id = Some(audio.id);

            if let Some(tag) = player.video.available_audio().into_iter().find(|tag| {
                Some(&tag.codec) == audio.codec.as_ref()
                    && audio.lang.as_ref() == Some(&tag.language_code)
            }) {
                player.video.set_audio(tag);
            }
        }

        if let Some(video) = selected_video
            && player.item.video_id != Some(video.id)
        {
            player.item.video_id = Some(video.id);

            // todo: ivp_fork doesn't support changing video yet
        }

        if awake.is_none() {
            *awake = keep_awake().ctx_log("New KeepAwake after save config");
        }

        player.video.set_paused(false);

        Task::none()
    }

    fn open_modal(&mut self, modal: Modal) -> Task<Message> {
        self.modal.open(modal);
        self.close_panel()
    }

    fn close_modal(&mut self) -> Task<Message> {
        if self.modal.as_ref().is_some() {
            self.modal.close();
            Task::none()
        } else {
            self.close_modal_forced()
        }
    }

    fn close_modal_forced(&mut self) -> Task<Message> {
        let previous = match self.modal.take() {
            Some(view) => self.save_config(view),
            None => {
                if self.panel.is_some() {
                    self.close_panel()
                } else {
                    Task::none()
                }
            }
        };

        let controls = Task::done(Message::Player(ManagerMessage::CursorExit));

        Task::batch([previous, controls])
    }

    pub fn close_panel(&mut self) -> Task<Message> {
        match self.panel {
            Some(Panel::Playlist) => {
                self.panel = Some(Panel::PlaylistClosing);
            }
            Some(Panel::Comments(_)) => {
                self.panel = Some(Panel::CommentsClosing);
            }
            Some(Panel::CommentsClosing) | Some(Panel::PlaylistClosing) | None => {}
        }

        Task::none()
    }

    fn collection_add(&mut self) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
            thumbnails_handle: _handle,
        } = &mut self.state
        else {
            return Task::none();
        };

        player.video.set_paused(true);
        awake.take();

        let id = player.item.id;
        let modal = Modal::CollectionAdd {
            item: id,
            collections: vec![],
            selected: HashSet::default(),
            initial: HashSet::default(),
        };

        let close_panel = self.open_modal(modal);

        let ids = Task::done(Message::FetchMembershipIds(id.into()));
        let cols = Task::done(Message::fetch_simple_collections());

        Task::batch([ids, cols, close_panel])
    }

    fn video_config(&mut self) -> Task<Message> {
        let (selected_text, selected_audio, selected_video, subtitle_uri, fit) =
            if let State::Ready {
                player,
                awake,
                comments: _comments,
                thumbnails_handle: _handle,
            } = &mut self.state
            {
                awake.take();
                player.video.set_paused(true);
                let current = player.item.subtitle_id.and_then(|id| {
                    player
                        .item
                        .subtitles
                        .iter()
                        .find(|sub| sub.id == id)
                        .cloned()
                });

                let audio = player.item.audio_id.and_then(|id| {
                    player
                        .item
                        .audios
                        .iter()
                        .find(|audio| audio.id == id)
                        .cloned()
                });

                let video = player.item.video_id.and_then(|id| {
                    player
                        .item
                        .videos
                        .iter()
                        .find(|video| video.id == id)
                        .cloned()
                });

                (current, audio, video, None, Some(player.fit))
            } else {
                (None, None, None, None, None)
            };

        let subtitle_offset = selected_text
            .as_ref()
            .map(|sub| sub.offset)
            .unwrap_or_default();

        let modal = Modal::Config(Box::new(Config {
            tab: ConfigTab::General,
            subtitle_uri,
            selected_text,
            selected_audio,
            selected_video,
            text_color: self.settings.subtitles.color.to_string(),
            background_color: self.settings.subtitles.background_color.to_string(),
            subtitle_font: FontState::new(self.fonts.clone(), &self.settings.subtitles.font),
            subtitle_offset,
            fit: fit.unwrap_or(ContentFit::Contain),
        }));

        self.open_modal(modal)
    }

    fn video_comment(&mut self) -> Task<Message> {
        if matches!(self.panel, Some(Panel::Comments(_))) {
            self.close_panel()
        } else {
            self.panel = Some(Panel::Comments(None));
            Task::none()
        }
    }

    pub fn action(&mut self, action: PlayerAction, now: Instant) -> Task<Message> {
        match action {
            PlayerAction::PlayToggle => self.play_toggle(Some(now)),
            PlayerAction::PlayNext => self.play_next(),
            PlayerAction::PlayPrevious => self.play_previous(),
            PlayerAction::FullscreenToggle => self.fullscreen_toggle(),
            PlayerAction::Exit => {
                if self.modal.as_ref().is_some() || self.panel.is_some() {
                    self.close_modal_forced()
                } else {
                    self.fullscreen_exit()
                }
            }
            PlayerAction::SeekBack => self.seek_back(false, now),
            PlayerAction::SeekBackShift => self.seek_back(true, now),
            PlayerAction::SeekFront => self.seek_front(false, now),
            PlayerAction::SeekFrontShift => self.seek_front(true, now),
            PlayerAction::VolumeIncrease => self.volume_increase(now),
            PlayerAction::VolumeDecrease => self.volume_decrease(now),
            PlayerAction::MuteToggle => self.mute_toggle(now),
            PlayerAction::SpeedIncrease => self.speed_increase(),
            PlayerAction::SpeedDecrease => self.speed_decrease(),
            PlayerAction::SpeedReset => self.speed_reset(),
            PlayerAction::SubtitlesToggle => self.subtitles_toggle(),
            PlayerAction::Add => self.collection_add(),
            PlayerAction::VideoConfig => self.video_config(),
            PlayerAction::VideoComment => self.video_comment(),
            PlayerAction::CloseView => self.close_modal(),
            PlayerAction::Back => self.previous_screen(),
            PlayerAction::PlaylistToggle => self.toggle_playlist(),
            PlayerAction::VideoCommentNew => self.new_comment(),
        }
    }

    pub fn stats(&mut self) -> Task<Message> {
        let State::Ready {
            player, comments, ..
        } = &mut self.state
        else {
            return Task::none();
        };

        let comments = comments.iter().cloned().collect();
        let comments = Message::SaveComments(comments).tasked();

        let duration = player.video.duration().as_secs_f64();
        let progress = player.position / duration;
        let watch_time = player.watch_time.as_secs_f64();

        let watch_count = if progress >= self.settings.completion_point
            && (watch_time / duration >= self.settings.completion_watch_time)
        {
            player.item.watch_count + 1
        } else {
            player.item.watch_count
        };

        let progress = (progress * 1000.0).round() / 1000.0;
        let progress = progress.clamp(0.0, 1.0);
        player.item.progress = progress as f32;
        player.item.watch_count = watch_count;
        player.item.duration = duration as u64;

        self.playlist.update_current(&player.item);

        Task::batch([
            Task::done(Message::VideoStats(Box::new(player.item.clone()))),
            comments,
        ])
    }

    pub fn toggle_playlist(&mut self) -> Task<Message> {
        if matches!(self.panel, Some(Panel::Playlist)) {
            self.close_panel()
        } else {
            self.panel = Some(Panel::Playlist);
            Task::none()
        }
    }

    pub fn fetched_comments(&mut self, id: VideoId, comments: Vec<Comment>) -> Task<Message> {
        match &mut self.state {
            State::Ready {
                player,
                comments: curr,
                awake: _awake,
                thumbnails_handle: _handle,
            } if player.item.id == id => {
                curr.update(comments);

                Task::none()
            }
            _ => match &mut self.next {
                AutoState::Ready {
                    player,
                    comments: curr,
                    thumbnails_handle: _handle,
                } if player.item.id == id => {
                    curr.update(comments);

                    Task::none()
                }
                _ => Task::none(),
            },
        }
    }

    fn new_comment(&mut self) -> Task<Message> {
        let editor = widget::Id::unique();
        self.panel = Some(Panel::Comments(Some((
            editor.clone(),
            text_editor::Content::default(),
        ))));

        operation::focus(editor)
    }
}

fn load_video<Message: 'static + MaybeSend>(
    mut item: models::Video,
    f: impl FnOnce(Arc<error::Result<Player>>) -> Message + 'static + MaybeSend,
) -> Task<Message> {
    let id = item.id;

    Task::perform(
        tokio::task::spawn_blocking(move || {
            let path_ref = item.path.as_path();

            let path = item
                .path
                .canonicalize()
                .with_context(|| format!("canonicalize video {id} path {}", path_ref.display()))?;

            let file_size = std::fs::metadata(&path)
                .map(|metadata| metadata.len())
                .unwrap_or_default();

            let path_ref = item.path.as_path();
            let path = url::Url::from_file_path(path_ref)
                .map_err(|_| anyhow!("Creating video {id} url from path {}", path_ref.display()))?;

            let mut video =
                Video::new(&path).with_context(|| format!("Creating Video player on {id}"))?;

            let duration = video.duration().as_secs_f64();
            let embedded = video.available_subtitles();

            for em in &embedded {
                let exists = item.subtitles.iter().any(|sub| {
                    matches!(sub.kind, models::SubtitleKind::Embedded)
                        && em.title == sub.title
                        && em.language_code == sub.lang
                });

                if exists {
                    continue;
                }

                let new = Subtitle::new_embedded(item.id, &em.title, &em.language_code);
                item.subtitles.push(new);
            }

            if let Some(saved_sub) = item
                .subtitle_id
                .and_then(|id| item.subtitles.iter().find(|sub| sub.id == id))
            {
                match &saved_sub.kind {
                    models::SubtitleKind::Embedded => {
                        if let Some(em) = embedded.iter().find(|em| {
                            em.title == saved_sub.title && em.language_code == saved_sub.lang
                        }) {
                            video.set_text(em.clone());
                        }
                    }
                    models::SubtitleKind::Loaded { path, .. } => {
                        let path: &std::path::Path = path.as_ref();
                        let path = path
                            .canonicalize()
                            .with_context(|| {
                                format!("Loading saved subtitle path at {}", path.display())
                            })
                            .and_then(|path| {
                                url::Url::from_file_path(&path).map_err(|_| {
                                    anyhow!(
                                        "Cannot create url from subtitle path at {}",
                                        path.display()
                                    )
                                })
                            })
                            .log_err();

                        if let Some(url) = path.as_ref() {
                            video
                                .set_subtitle_url(url)
                                .ctx_log("Setting video subtitle url");
                        }
                    }
                }

                let offset = (1_000_000_000 as f32 * saved_sub.offset) as i64;

                video.set_text_offset(offset);
            }

            if let Some(saved_audio) = item
                .audio_id
                .and_then(|id| item.audios.iter().find(|audio| audio.id == id))
                && let Some(audio) = video.available_audio().iter().find(|audio| {
                    audio.id == saved_audio.stream as i32
                        && Some(&audio.language_code) == saved_audio.lang.as_ref()
                })
            {
                video.set_audio(audio.clone());
            }

            std::thread::sleep(std::time::Duration::from_millis(150));

            let progress = if item.progress >= 0.98 {
                0.0
            } else {
                item.progress
            };
            let position = (duration * progress as f64).round().clamp(0.0, duration);

            video
                .seek(Duration::from_secs_f64(position), true)
                .with_context(|| format!("Resuming at position {position} on {id}"))?;

            // todo: There is a race condition when resuming a video. I can't quite pinpoint where
            // so until I do, this is a temporary fix which seems to work.
            std::thread::sleep(std::time::Duration::from_millis(200));

            video.set_paused(true);

            Ok(Arc::new(Player {
                item,
                video,
                file_size,
                thumbnails: vec![],
                position,
                is_dragging: false,
                watch_time: Duration::ZERO,
                last_frame: None,
                subtitles: None,
                fit: ContentFit::Contain,
            }))
        }),
        move |res| {
            let res = res
                .with_context(|| format!("Joining video {id}"))
                .flatten()
                .map(|player| Arc::try_unwrap(player).expect("First player unwrap"));

            f(Arc::new(res))
        },
    )
}

fn apply_settings(settings: &VideoSettings, player: &mut Player) {
    let VideoSettings {
        thumbnail_interval: _thumbnails,
        volume,
        speed,
        seek_change_amt: _seek_amt,
        seek_shift_change_amt: _seek_shift_amt,
        volume_change_amt: _volume,
        speed_change_amt: _speed,
        show_subtitles: _show_subtitles,
        muted,
        auto_start,
        auto_next: _autoplay,
        completion_point: _completion,
        completion_watch_time: _completion_watch,
        subtitles: _subtitles,
        comment_span: _comment_span,
        filters:
            VideoFilters {
                contrast,
                brightness,
                hue,
                saturation,
                gamma,
            },
    } = settings;

    {
        player.video.set_contrast(*contrast);
        player.video.set_brightness(*brightness);
        player.video.set_hue(*hue);
        player.video.set_saturation(*saturation);
    }

    player.video.set_volume(*volume);
    let id = player.item.id;
    player
        .video
        .set_speed(*speed)
        .with_ctx_log(|| format!("Applying Video settings on {id}"));
    player.video.set_paused(!auto_start);
    player.video.set_gamma(*gamma);
    player.video.set_muted(*muted);
}

fn handle_clicks(click: MouseClick) -> Option<ManagerMessage> {
    let msg = match click.action {
        MouseAction::Button { button, kind } => match button {
            Button::Left if matches!(kind, Kind::Single) => ManagerMessage::TogglePlay,
            Button::Left if matches!(kind, Kind::Double) => ManagerMessage::ToggleFullscreen,
            Button::Right => ManagerMessage::OpenConfig,
            _ => return None,
        },
        MouseAction::Scroll(_) => return None,
    };

    Some(msg)
}

fn draw_playlist<'a>(
    playlist: &'a Playlist,
    auto_next: bool,
    width: impl Into<Length>,
) -> Element<'a, ManagerMessage> {
    let rule_height = 1.0;
    let padding = [6, 12];

    let title = {
        let content = row!(
            sized_bold("Playlist", P),
            space::horizontal(),
            button(icons::icon(CANCEL).size(H6))
                .on_press(ManagerMessage::ClosePanel)
                .style(theme::button::text)
                .padding(0),
        )
        .padding(padding)
        .align_y(Vertical::Center);

        column!(content, rule::horizontal(rule_height))
    };

    let items = playlist.items().map(|(idx, item, current)| {
        let size = H7;
        let height = 20;

        let duration = item.duration;
        let hrs = duration / 3600;

        let mins = (duration % 3600) / 60;

        let secs = duration % 60;

        let color = move |theme: &Theme| {
            let color = theme.schema().primary.base.color;

            text::Style {
                color: current.then_some(color),
            }
        };

        let name = if current {
            marquee(&item.name).font(medium_font())
        } else {
            marquee(&item.name)
        }
        .size(size);

        let name = container(name.height(height).style(color)).width(Length::FillPortion(10));

        let duration = format!("{hrs:02}:{mins:02}:{secs:02}");

        let duration = if current {
            medium(duration)
        } else {
            regular(duration)
        }
        .size(size)
        .height(height)
        .style(color);

        button(
            row!(name, space::horizontal(), duration)
                .spacing(4)
                .clip(true)
                .align_y(Vertical::Center),
        )
        .on_press(ManagerMessage::Playlist(PlaylistMessge::PlayItem(idx)))
        .padding(0)
        .style(theme::button::text)
        .into()
    });

    let items = container(column(items).spacing(8)).padding(padding);

    let actions = {
        let size = H6;
        let position = tp::Position::Top;
        let color = |theme: &Theme, active: bool| {
            let color = theme.schema().primary.base.color;

            text::Style {
                color: active.then_some(color),
            }
        };

        let repeat = tooltip(
            button(
                icons::icon(icons::LOOP)
                    .size(size)
                    .style(move |theme| color(theme, playlist.repeat)),
            )
            .padding(0)
            .style(theme::button::text)
            .on_press(ManagerMessage::Playlist(PlaylistMessge::ToggleRepeat(
                !playlist.repeat,
            ))),
            "Loop",
            position,
        );

        let shuffle = tooltip(
            button(
                icons::icon(icons::SHUFFLE)
                    .size(size)
                    .style(move |theme| color(theme, playlist.shuffle)),
            )
            .padding(0)
            .style(theme::button::text)
            .on_press(ManagerMessage::Playlist(PlaylistMessge::ToggleShuffle(
                !playlist.shuffle,
            ))),
            "Shuffle",
            position,
        );

        let auto_next = tooltip(
            toggler(auto_next).on_toggle(|toggle| {
                ManagerMessage::Playlist(PlaylistMessge::ToggleAutoNext(toggle))
            }),
            "Play next media",
            position,
        );

        let save = tooltip(
            button(icons::icon(icons::SAVE).size(size))
                .padding(0)
                .style(theme::button::text)
                .on_press(ManagerMessage::Playlist(PlaylistMessge::Save)),
            "Save playlist",
            position,
        );

        let center = row!(repeat, shuffle)
            .spacing(20.0)
            .align_y(Vertical::Center);

        let content = row!(
            auto_next,
            space::horizontal(),
            center,
            space::horizontal(),
            save,
        )
        .align_y(Vertical::Center)
        .spacing(8)
        .width(Length::Fill)
        .padding(padding);

        column!(rule::horizontal(rule_height), content)
    };

    let content = scrollable(items)
        .height(Length::Fill)
        .spacing(6)
        .auto_scroll(true);

    let content = column!(title, content, actions)
        .height(Length::Fill)
        .padding([3, 0])
        .spacing(0);

    panel_container(content, width)
}

fn draw_comments<'a>(
    new: &'a Option<(widget::Id, text_editor::Content)>,
    comments: &'a Comments,
    position: u64,
    span: u64,
    theme: &Theme,
    width: impl Into<Length>,
    now: Instant,
) -> Element<'a, ManagerMessage> {
    let rule_height = 1.0;
    let padding = [6, 12];

    let title = {
        let content = row!(
            sized_bold("Comments", P),
            space::horizontal(),
            button(icons::icon(CANCEL).size(H6))
                .on_press(ManagerMessage::ClosePanel)
                .style(theme::button::text)
                .padding(0),
        )
        .padding(padding)
        .align_y(Vertical::Center);

        column!(content, rule::horizontal(rule_height))
    };

    let new: Element<'_, CommentMessage> = match new {
        Some((editor, comment)) => {
            let editor = text_editor(comment)
                .id(editor.clone())
                .on_action(CommentMessage::NewAction)
                .wrapping(text::Wrapping::WordOrGlyph)
                .key_binding(move |press| {
                    use iced::keyboard::{Key, key::Named};
                    use text_editor::Binding;
                    match press.key {
                        Key::Named(Named::Enter) if press.modifiers.command() => {
                            Some(Binding::Custom(CommentMessage::NewSubmit))
                        }
                        _ => Binding::from_key_press(press),
                    }
                })
                .padding(6)
                .highlight("markdown", iced::highlighter::Theme::Base16Ocean);
            let cancel = cancel_btn().on_press(CommentMessage::NewCancel);
            let save = save_btn().on_press(CommentMessage::NewSubmit);

            let btns = row!(save, cancel).spacing(40).align_y(Vertical::Center);

            let content = column!(editor, btns)
                .align_x(Horizontal::Center)
                .spacing(10);

            let content = container(content).padding(4).style(theme::container::bw);

            content.into()
        }
        None => button(medium("New"))
            .padding([5, 5])
            .on_press(CommentMessage::New)
            .into(),
    };

    let new: Element<'_, CommentMessage> = column!(rule::horizontal(rule_height), new)
        .spacing(6.0)
        .align_x(Horizontal::Center)
        .padding(padding)
        .into();

    let draw_comment = |comment: &'a Comment| {
        container(comment.view(now, theme))
            .height(Length::Fit.max(325.0))
            .width(Length::Fill)
            .into()
    };

    let nulls = {
        let nulls = comments.nulls.iter().filter_map(|comment| {
            if !comment.inner.removed {
                Some(draw_comment(comment))
            } else {
                None
            }
        });

        let nulls = container(column(nulls)).height(Length::Fit.max(500.0));

        scrollable(nulls).spacing(10)
    };

    let comments = {
        let timestamp = { position.saturating_sub(span)..=position.saturating_add(span) };

        let comments = comments
            .timestamped
            .range(timestamp)
            .flat_map(|(_, comments)| comments.iter().filter(|comment| !comment.inner.removed));

        column(comments.map(draw_comment)).spacing(16)
    };

    let content = scrollable(comments).height(Length::Fill).spacing(10);

    let content: Element<'_, CommentMessage> = column!(nulls, content).spacing(10).into();

    let content = column!(
        title,
        content.map(ManagerMessage::CommentMessage),
        new.map(ManagerMessage::CommentMessage)
    )
    .spacing(20)
    .padding([3, 6])
    .align_x(Horizontal::Center);

    panel_container(content, width)
}

fn draw_collection_add<'a>(
    selected: &'a HashSet<CollectionId>,
    is_empty: bool,
    collections: impl Iterator<Item = &'a SimpleCollection>,
) -> Element<'a, ManagerMessage> {
    let title = h6("Collections");

    fn btn(collection: &SimpleCollection, selected: bool) -> Element<'_, ManagerMessage> {
        let size = P;
        let unicode = Icon::new(collection.icon).unicode();
        let icon = icons::icon(unicode).size(size);
        let text = container(regular(&collection.name))
            .width(Length::Fit.max(275.0))
            .height(Length::Fit.max(48.0));
        let check = checkbox(selected).on_toggle(|value| {
            ManagerMessage::CollectionAddMessage(CollectionAddMessage::Toggle(
                !value,
                collection.id,
            ))
        });

        button(
            row!(icon, text, space::horizontal(), check)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(8.0),
        )
        .padding([8, 12])
        .on_press(ManagerMessage::CollectionAddMessage(
            CollectionAddMessage::Toggle(selected, collection.id),
        ))
        .style(move |theme, status| {
            if selected {
                theme::button::subtle(theme, status)
            } else {
                theme::button::text(theme, status)
            }
        })
        .into()
    }

    let collections =
        column(collections.map(|collection| btn(collection, selected.contains(&collection.id))))
            .spacing(8.0);

    let collections = scrollable(collections).spacing(16.0);

    let collections = container(collections)
        .padding(if is_empty { [0, 0] } else { [6, 8] })
        .style(|theme: &Theme| {
            let color = theme.schema().secondary.strong.color;
            let default = theme::container::transparent(theme);
            let border = default.border.color(color).width(1.5);

            container::Style { border, ..default }
        });

    let actions = {
        let save = save_btn().on_press(ManagerMessage::CollectionAddMessage(
            CollectionAddMessage::Save,
        ));

        let cancel = cancel_btn().on_press(ManagerMessage::CloseView);

        row!(save, cancel).spacing(100)
    };

    let content = column!(title, collections, actions)
        .spacing(24)
        .align_x(Horizontal::Center);

    modal_container(content).width(Length::Fit.max(400)).into()
}

fn draw_general<'a>(
    settings: &'a VideoSettings,
    size: f32,
    padding: Padding,
    spacing: f32,
) -> Element<'a, GeneralConfig> {
    let input_width = 48;
    let volume_amt = {
        let label = label_maker("Volume amount: ");

        let amt = format!("{:.02}", settings.volume_change_amt);
        let input = text_input("", &amt)
            .width(input_width)
            .size(size)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(GeneralConfig::VolumeAmt);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let speed_amt = {
        let label = label_maker("Speed amount: ");

        let amt = format!("{:.02}", settings.speed_change_amt);
        let input = text_input("", &amt)
            .width(input_width)
            .size(size)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(GeneralConfig::SpeedAmt);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let seek_amt = {
        let label = label_maker("Seek amount: ");

        let amt = format!("{:.02}", settings.seek_change_amt);
        let input = text_input("", &amt)
            .width(input_width)
            .size(size)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(GeneralConfig::SeekAmt);

        let input = row!(input).align_y(Vertical::Center).spacing(4);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let seek_amt_shift = {
        let label = label_maker("Seek Shift amount: ");

        let amt = format!("{:.02}", settings.seek_shift_change_amt);
        let input = text_input("", &amt)
            .width(input_width)
            .size(size)
            .font(regular_font())
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(GeneralConfig::SeekShiftAmt);

        let input = row!(input).align_y(Vertical::Center).spacing(4);

        row!(label, space::horizontal(), input)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let cspan = {
        let label = label_maker("Comment span(seconds) ");

        let amt = settings.comment_span.to_string();
        let input = text_input("", &amt)
            .font(regular_font())
            .width(input_width)
            .size(size)
            .align_x(Horizontal::Right)
            .padding(padding)
            .on_input(GeneralConfig::Span);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    column!(volume_amt, speed_amt, seek_amt, seek_amt_shift, cspan)
        .spacing(16)
        .into()
}

fn draw_video<'a>(
    settings: &'a VideoSettings,
    config: &'a Config,
    videos: &'a [VideoInfo],
    size: f32,
    spacing: f32,
) -> Element<'a, VideoConfig> {
    let width = 200;
    let slider_width = 200;

    let gamma = {
        let label = label_maker("Gamma: ").width(width);

        let slider = slider(1.0..=3.0, settings.filters.gamma, VideoConfig::Gamma)
            .step(0.05)
            .shift_step(0.1)
            .width(slider_width);

        let gamma = sized_regular(format!("{:.01}", settings.filters.gamma), size);
        let slider = row!(gamma, slider).spacing(4.0).align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let brightness = {
        let label = label_maker("Brightness: ").width(width);

        let slider = slider(
            -1.0..=1.0,
            settings.filters.brightness,
            VideoConfig::Brightness,
        )
        .step(0.05)
        .shift_step(0.1)
        .width(slider_width);

        let brightness = sized_regular(format!("{:.01}", settings.filters.brightness), size);
        let slider = row!(brightness, slider)
            .spacing(4.0)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let contrast = {
        let label = label_maker("Contrast: ").width(width);

        let slider = slider(0.0..=2.0, settings.filters.contrast, VideoConfig::Contrast)
            .step(0.05)
            .shift_step(0.1)
            .width(slider_width);

        let contrast = sized_regular(format!("{:.01}", settings.filters.contrast), size);
        let slider = row!(contrast, slider)
            .spacing(4.0)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let hue = {
        let label = label_maker("Hue: ").width(width);

        let slider = slider(-1.0..=1.0, settings.filters.hue, VideoConfig::Hue)
            .step(0.05)
            .shift_step(0.1)
            .width(slider_width);

        let hue = sized_regular(format!("{:.01}", settings.filters.hue), size);
        let slider = row!(hue, slider).spacing(4.0).align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let saturation = {
        let label = label_maker("Saturation: ").width(width);

        let slider = slider(
            0.0..=2.0,
            settings.filters.saturation,
            VideoConfig::Saturation,
        )
        .step(0.05)
        .shift_step(0.1)
        .width(slider_width);

        let saturation = sized_regular(format!("{:.01}", settings.filters.saturation), size);
        let slider = row!(saturation, slider)
            .spacing(4.0)
            .align_y(Vertical::Center);

        row!(label, space::horizontal(), slider).align_y(Vertical::Center)
    };

    let video = {
        let label = label_maker("Video: ");

        let handle = picklist_handle(size);

        let pick: Element<'_, VideoConfig> = if videos.is_empty() {
            label_maker("None").size(size).into()
        } else {
            pick_list(config.selected_video.clone(), videos, video_info_to_string)
                .handle(handle)
                .on_select(VideoConfig::CurrentVideo)
                .font(regular_font())
                .padding(LIST_PADDING)
                .text_size(size)
                .into()
        };

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let fit = {
        let label = label_maker("Video Content Fit: ");

        let handle = picklist_handle(size);

        let pick = pick_list(
            Some(config.fit),
            [
                ContentFit::Contain,
                ContentFit::Cover,
                ContentFit::Fill,
                ContentFit::ScaleDown,
                ContentFit::None,
            ],
            ToString::to_string,
        )
        .handle(handle)
        .ellipsis(text::Ellipsis::End)
        .on_select(VideoConfig::Fit)
        .font(regular_font())
        .padding(LIST_PADDING)
        .text_size(size);

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    column!(video, fit, gamma, brightness, contrast, hue, saturation)
        .spacing(16)
        .into()
}

fn draw_subs<'a>(
    settings: &'a VideoSettings,
    config: &'a Config,
    subtitles: &'a [Subtitle],
    size: f32,
    padding: Padding,
    spacing: f32,
) -> Element<'a, SubtitleConfig> {
    let input_width = 48.0;
    let color_width = 150.0;
    let file = {
        let add = button(
            row!(
                icons::icon(icons::FILE_UP).size(size),
                sized_medium("Load Subtitles", size)
            )
            .spacing(8.0)
            .align_y(Vertical::Center),
        )
        .padding([3, 6])
        .style(theme::button::subtle)
        .on_press(SubtitleConfig::SelectFile);

        let path = config.subtitle_uri.as_ref().map(|path| trim_path(path, 3));
        let path: Element<'_, SubtitleConfig> = match path {
            Some(path) => button(
                row!(
                    path_container(path, size, false),
                    icons::icon(icons::CANCEL).size(size)
                )
                .spacing(8)
                .align_y(Vertical::Center),
            )
            .padding([2, 5])
            .style(theme::button::text)
            .on_press(SubtitleConfig::ClearSelected)
            .into(),
            None => empty(),
        };

        row!(add, space::horizontal(), path)
            .align_y(Vertical::Center)
            .spacing(12)
    };

    let selection = {
        let label = label_maker("Available Subtitles: ");

        let handle = picklist_handle(size);

        let pick: Element<'_, SubtitleConfig> = if subtitles.is_empty() {
            label_maker("None").size(size).into()
        } else {
            pick_list(config.selected_text.clone(), subtitles, subtitle_to_string)
                .handle(handle)
                .ellipsis(text::Ellipsis::End)
                .on_select(SubtitleConfig::CurrentText)
                .font(regular_font())
                .padding(LIST_PADDING)
                .text_size(size)
                .into()
        };

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let offset: Element<'_, SubtitleConfig> =
        if config.selected_text.is_some() || config.subtitle_uri.is_some() {
            let label = label_maker("Subtitle Offset(seconds): ");
            let offset = format!("{:.02}", config.subtitle_offset);

            let actions = input_actions(SubtitleConfig::OffsetIncr, SubtitleConfig::OffsetDecr);

            let input = text_input("", &offset)
                .width(input_width)
                .size(size)
                .font(regular_font())
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SubtitleConfig::Offset);

            let input = row!(input, actions).spacing(4.0).align_y(Vertical::Center);

            row!(label, space::horizontal(), input)
                .align_y(Vertical::Center)
                .spacing(spacing)
                .into()
        } else {
            empty()
        };

    let style = {
        let label = label_maker("Subtitle Style").size(P);
        let dummy = draw_subtitles("An example subtitle", &settings.subtitles);

        let sub_size = {
            let label = label_maker("Size: ");

            let amt = format!("{}", settings.subtitles.size);

            let actions = input_actions(SubtitleConfig::SubSizeIncr, SubtitleConfig::SubSizeDecr);

            let input = text_input("", &amt)
                .width(input_width)
                .size(size)
                .font(regular_font())
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SubtitleConfig::SubSize);

            let input = row!(input, actions).spacing(4.0).align_y(Vertical::Center);

            row!(label, space::horizontal(), input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let color = {
            let label = label_maker("Text Color (rgba): ");

            let input = text_input("", &config.text_color)
                .width(color_width)
                .size(size)
                .font(regular_font())
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SubtitleConfig::SubColor);

            row!(label, space::horizontal(), input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let background = {
            let label = label_maker("Background Color (rgba): ");

            let input = text_input("", &config.background_color)
                .width(color_width)
                .size(size)
                .font(regular_font())
                .align_x(Horizontal::Right)
                .padding(padding)
                .on_input(SubtitleConfig::SubBackground);

            row!(label, space::horizontal(), input)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let font = {
            let label = label_maker("Font: ");

            let selection = font_selection(
                &config.subtitle_font.state,
                "",
                config.subtitle_font.selected,
                SubtitleConfig::SubFont,
            )
            .width(200)
            .size(size)
            .padding(padding);

            row!(label, space::horizontal(), selection)
                .align_y(Vertical::Center)
                .spacing(spacing)
        };

        let content = column!(font, sub_size, color, background).spacing(8.0);

        let content = column!(content, dummy)
            .spacing(16)
            .align_x(Horizontal::Center);

        column!(label, content).spacing(12)
    };

    column!(file, selection, offset, style)
        .width(Length::Fill)
        .spacing(12)
        .into()
}

fn draw_audio<'a>(
    config: &'a Config,
    audio: &'a [Audio],
    size: f32,
    spacing: f32,
) -> Element<'a, AudioConfig> {
    let selection = {
        let label = label_maker("Audio: ");

        let handle = picklist_handle(size);

        let pick: Element<'_, AudioConfig> = if audio.is_empty() {
            label_maker("None").size(size).into()
        } else {
            pick_list(config.selected_audio.clone(), audio, audio_to_string)
                .handle(handle)
                .on_select(AudioConfig::CurrentAudio)
                .font(regular_font())
                .padding(LIST_PADDING)
                .text_size(size)
                .into()
        };

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    column!(selection).width(Length::Fill).spacing(12).into()
}

fn draw_info<'a>(
    config: &'a Config,
    item: &'a models::Video,
    file_size: u64,
    size: f32,
) -> Element<'a, ConfigMessage> {
    let info = |value: String| sized_regular(value, size / typo::RATIO);

    let general = {
        let title = sized_medium("General", size);

        let kind = match item.id {
            VideoId::Movie(_) => "movie",
            VideoId::Episode(_) => "episode",
        };

        let kind = info(format!("Media type: {kind}"));

        let file_size = {
            let size = file_size as f64 / 1024.0f64.powi(3);
            info(format!("Size: {size:.2} GB"))
        };

        let path = {
            let path = trim_path(&item.path, 5);
            let name = info("Path: ".to_string());
            let path = marquee(path).size(size / typo::RATIO);

            row!(name, path).spacing(2).align_y(Vertical::Center)
        };

        let info = column!(kind, path, file_size)
            .spacing(4)
            .padding(Padding::new(0.0).left(12));

        column!(title, info).spacing(8)
    };

    let video = config.selected_video.as_ref().map(|video| {
        let title = sized_medium("Video", size);

        let codec = video
            .codec
            .as_deref()
            .map(|codec| info(format!("Codec: {codec}")));

        let bitrate = if video.bitrate > 0 {
            Some(info(format!(
                "Bitrate: {:.2} Mbps",
                video.bitrate as f32 / 1_000_000.0
            )))
        } else {
            None
        };

        let framerate = info(format!("Framerate: {:.1}", video.framerate));
        let dimensions = info(format!("Resolution: {}", video.resolution()));

        let info = column!(codec, bitrate, framerate, dimensions)
            .spacing(4)
            .padding(Padding::new(0.0).left(12));

        column!(title, info).spacing(8)
    });

    let audio = config.selected_audio.as_ref().map(|audio| {
        let title = sized_medium("Audio", size);

        let codec = audio
            .codec
            .as_deref()
            .map(|codec| info(format!("Codec: {codec}")));

        let lang = audio
            .lang
            .as_deref()
            .map(|lang| info(format!("Language: {lang}")));

        let channels = if audio.channels > 0 {
            Some(info(format!("Channels: {}", audio.channels)))
        } else {
            None
        };

        let sample = if audio.sample_rate > 0 {
            Some(info(format!("Sample Rate: {} Hz", audio.sample_rate)))
        } else {
            None
        };

        let bitrate = if audio.bitrate > 0 {
            Some(info(format!(
                "Bitrate: {:.2} kbps",
                audio.bitrate as f32 / 1000.0
            )))
        } else {
            None
        };

        let info = column!(lang, codec, channels, sample, bitrate)
            .spacing(4)
            .padding(Padding::new(0.0).left(12));

        column!(title, info).spacing(8)
    });

    let subtitle = config.selected_text.as_ref().map(|sub| {
        let title = sized_medium("Subtitle", size);

        let name = info(format!("Title: {}", sub.title));
        let lang = info(format!("Language: {}", sub.lang));

        let (kind, path) = match &sub.kind {
            SubtitleKind::Embedded => ("Embedded", None),
            SubtitleKind::Loaded { path, .. } => ("Loaded", Some(path)),
        };

        let kind = info(format!("Kind: {kind}"));

        let path = path.map(|path| {
            let name = info("Path: ".to_string());
            let path = trim_path(path, 3);
            let path = marquee(path).size(size / typo::RATIO);

            row!(name, path).spacing(2).align_y(Vertical::Center)
        });

        let info = column!(name, lang, kind, path)
            .spacing(4)
            .padding(Padding::new(0.0).left(12));

        column!(title, info).spacing(8)
    });

    let content = column!(general, video, audio, subtitle).spacing(20);

    let content = scrollable(content).spacing(5.0).width(Length::Fill);

    content.into()
}

fn draw_config<'a>(
    settings: &'a VideoSettings,
    config: &'a Config,
    subtitles: &'a [Subtitle],
    audio: &'a [Audio],
    videos: &'a [VideoInfo],
    item: &'a models::Video,
    file_size: u64,
) -> Element<'a, ManagerMessage> {
    // todo: Hardware volume
    let size = H7;
    let padding = Padding::new(2.0).horizontal(5.0);
    let spacing = 8.0;

    let curr_tab = config.tab;
    let header = h6("Video Config").center().width(Length::Fill);

    let tabs = ConfigTab::VARIANTS.iter().map(|tab| {
        let current = *tab == curr_tab;
        let text = h7(format!("{tab:?}"));
        container(
            button(text)
                .width(Length::Fill)
                .style(move |theme, status| {
                    if current {
                        theme::button::neutral(theme, status)
                    } else {
                        theme::button::text_hover(theme, status)
                    }
                })
                .on_press(ManagerMessage::Config(ConfigMessage::Tab(*tab))),
        )
        .clip(true)
        .height(Length::Fit.max(48.0))
        .into()
    });

    let content = match curr_tab {
        ConfigTab::General => {
            draw_general(settings, size, padding, spacing).map(ConfigMessage::General)
        }
        ConfigTab::Video => {
            draw_video(settings, config, videos, size, spacing).map(ConfigMessage::Video)
        }
        ConfigTab::Subtitles => draw_subs(settings, config, subtitles, size, padding, spacing)
            .map(ConfigMessage::Subtitle),
        ConfigTab::Audio => draw_audio(config, audio, size, spacing).map(ConfigMessage::Audio),
        ConfigTab::Info => draw_info(config, item, file_size, size),
    };

    let content = content.map(ManagerMessage::Config);

    let side = column(tabs)
        .spacing(8)
        .width(125.0)
        .height(Length::Fill)
        .padding(3);

    let content = row!(side, content).spacing(20);

    let content = column!(header, content)
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(20);

    modal_container(content)
        .padding([16, 16])
        .width(600)
        .height(400)
        .into()
}

fn label_maker<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a, Theme> {
    sized_medium(label, H7)
}

fn panel_container<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: impl Into<Length>,
) -> Element<'a, Message> {
    container(content)
        .style(theme::container::bb)
        .width(width)
        .into()
}

fn keep_awake() -> Result<keepawake::KeepAwake, keepawake::Error> {
    keepawake::Builder::default()
        .display(true)
        .app_name("kino")
        .reason("kino video playback")
        .create()
}

fn subtitle_to_string(sub: &Subtitle) -> String {
    match &sub.kind {
        SubtitleKind::Embedded => {
            format!("{}, {}", sub.title, sub.lang)
        }
        SubtitleKind::Loaded { path, .. } => {
            let file = path
                .file_name()
                .expect("Cannot have a non-file subtitles file");

            let name = file.to_string_lossy();

            name.into()
        }
    }
}

fn audio_to_string(audio: &Audio) -> String {
    format!(
        "{} - {}",
        audio.lang.as_deref().unwrap_or("Unk. language"),
        audio.codec.as_deref().unwrap_or("Unk. codec"),
    )
}

fn video_info_to_string(video: &VideoInfo) -> String {
    if video.bitrate > 0 {
        let bitrate = video.bitrate as f32 / 1_000_000.0;
        format!(
            "{}, {bitrate:.1} Mbps, {:.1}fps",
            video.resolution(),
            video.framerate
        )
    } else {
        format!("{}, {:.1}fps", video.resolution(), video.framerate)
    }
}
