use iced::{
    Element, Length, Padding, Size, Subscription, Task, Theme,
    advanced::graphics::futures::MaybeSend,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    time::Instant,
    widget::{
        self, button, center, checkbox, column, combo_box, container, image, mouse_area, operation,
        pick_list, row, rule, scrollable, slider, space, stack, text, text_editor, text_input,
        tooltip as tp,
    },
    window,
};
use iced_video_player::{AudioTag, Button, Kind, MouseAction, MouseClick, Video, VideoPlayer};
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};
use widgets::{marquee, throbber};

pub mod comment;
pub mod playlist;
use crate::app::Message;
use crate::home::shared::Icon;
use crate::utils::{
    self, FontState, PlayerAction, VideoSettings, cancel_btn, convert_color_str, draw_subtitles,
    duration_string, empty,
    icons::{self, CANCEL, sized_button},
    modal::modal,
    modal_container, picklist_handle, save_btn, styles, toggler, tooltip, trim_path, typo,
};
pub use comment::*;
use core::{Error, variants};
use devutils::thumbnails::{Image, ThumbnailGenerator};
pub use playlist::*;
use registry::models::{
    self, CollectionId, CommentId, SimpleCollection, Subtitle, SubtitleKind, VideoId,
};
use typo::*;

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
struct Config {
    tab: ConfigTab,
    subtitle_uri: Option<PathBuf>,
    selected_text: Option<Subtitle>,
    selected_audio: Option<AudioTag>,
    text_color: String,
    background_color: String,
    subtitle_font: FontState,
}

#[derive(Debug, Clone)]
enum Panel {
    Playlist,
    Comments(Option<(widget::Id, text_editor::Content)>),
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
        Filters,
        Subtitles,
        Audio,
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
    CurrentAudio(AudioTag),
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
    current_audio: Option<AudioTag>,
    embedded_audio: Vec<AudioTag>,
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
enum AutoState {
    Loading,
    Idle,
    Ready {
        player: Box<Player>,
        comments: BTreeMap<u64, Vec<Comment>>,
    },
}

enum State {
    Loading,
    Idle,
    Ready {
        player: Box<Player>,
        comments: BTreeMap<u64, Vec<Comment>>,
        awake: Option<keepawake::KeepAwake>,
    },
}

#[derive(Debug, Clone)]
pub enum ManagerMessage {
    Video(bool, Arc<Player>),
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
    SpeedReset,
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
    Subs(Option<String>),
    Config(ConfigMessage),
    Error(String),
    CommentMessage(CommentMessage),
    None,
}

pub struct Manager {
    window: Option<window::Id>,
    playlist: Playlist,
    show_controls: bool,

    pub settings: VideoSettings,
    fonts: Vec<iced::font::Family>,

    maximised: bool,
    is_fullscreen: bool,
    state: State,
    next: AutoState,

    modal: Option<Modal>,
    panel: Option<Panel>,
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
            Some(item) => load_video(item, |video| ManagerMessage::Video(false, video)),
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
            settings,
            maximised: false,
            is_fullscreen: false,
            state,
            next: AutoState::Idle,
            modal: None,
            panel: None,
        }
    }

    pub fn update(&mut self, message: ManagerMessage, now: Instant) -> Task<Message> {
        match message {
            ManagerMessage::None => Task::none(),
            ManagerMessage::Error(error) => Message::error(error).tasked(),
            ManagerMessage::Video(is_next, player) => {
                let mut player = Arc::try_unwrap(player).unwrap();

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
                        let path = url::Url::from_file_path(path.canonicalize().unwrap()).unwrap();
                        let generator = ThumbnailGenerator::new(path, width, height, 8)?;

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
                                let (img, pst) = match generator.generate_with_poster(position) {
                                    Ok(items) => items,
                                    Err(error) => {
                                        tracing::error!("Thumbnail generation error.\n{error}");
                                        continue;
                                    }
                                };

                                imgs.push(convert(img));
                                poster = Some(pst);
                            } else {
                                match generator.generate(position) {
                                    Ok(img) => imgs.push(convert(img)),
                                    Err(error) => {
                                        tracing::error!("Thumbnail generation error.\n{error}");
                                        continue;
                                    }
                                }
                            }
                        }

                        drop(generator);

                        Ok::<_, Error>((id, imgs, poster))
                    }),
                    move |res| match res {
                        Ok(Ok((id, thumbnails, poster))) => ManagerMessage::Thumbnail {
                            id,
                            thumbnails,
                            poster,
                        },
                        Ok(Err(error)) => {
                            tracing::error!("Thumbnail generation error.\n{error}");
                            ManagerMessage::None
                        }
                        Err(error) => {
                            tracing::error!("Thumbnail generation error.\n{error}");
                            ManagerMessage::None
                        }
                    },
                );

                let last_watched = if !is_next || matches!(&self.state, State::Idle) {
                    let task = Task::done(Message::LastWatched(player.item.id));
                    apply_settings(&self.settings, &mut player);

                    let awake = keep_awake().unwrap();

                    self.state = State::Ready {
                        awake: Some(awake),
                        player: Box::new(player),
                        comments: BTreeMap::default(),
                    };

                    task
                } else {
                    apply_settings(&self.settings, &mut player);
                    player.video.set_paused(true);
                    self.next = AutoState::Ready {
                        player: Box::new(player),
                        comments: BTreeMap::default(),
                    };
                    Task::none()
                };

                Task::batch([load_thumbnails.map(Message::Player), last_watched, comments])
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
                    if let Some(Player {
                        item, thumbnails, ..
                    }) = self.player_mut()
                        && item.id == id
                    {
                        *thumbnails = generated;
                    }
                } else if let AutoState::Ready {
                    player,
                    comments: _comments,
                } = &mut self.next
                    && player.item.id == id
                {
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
                                load_video(item, |video| ManagerMessage::Video(true, video))
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
                    return Task::done(Message::error(msg));
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
            ManagerMessage::TogglePlay => self.play_toggle(),
            ManagerMessage::ChangeVolume(volume) => {
                let volume = volume.clamp(0.0, 1.0);
                self.settings.volume = volume;

                if let Some(Player { video, .. }) = self.player_mut() {
                    video.set_volume(volume);
                }

                Task::none()
            }
            ManagerMessage::ToggleMute => self.mute_toggle(),
            ManagerMessage::SeekBack(shift) => self.seek_back(shift),
            ManagerMessage::SeekFront(shift) => self.seek_front(shift),
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
            ManagerMessage::ToggleFullscreen => self.fullscreen_toggle(),
            ManagerMessage::PreviousScreen => {
                let eos = self
                    .player()
                    .map(|player| player.video.eos())
                    .unwrap_or_default();

                let stats = if eos { Task::none() } else { self.stats() };

                self.fullscreen_exit()
                    .chain(Task::batch([Task::done(Message::Back), stats]))
            }
            ManagerMessage::ToggleSubtitles => self.subtitles_toggle(),
            ManagerMessage::PlayNext => self.play_next(),
            ManagerMessage::PlayPrevious => self.play_previous(),
            ManagerMessage::AddCollection => self.collection_add(),
            ManagerMessage::OpenConfig => self.video_config(),
            ManagerMessage::Comment => self.video_comment(),
            ManagerMessage::SpeedReset => self.speed_reset(),
            ManagerMessage::CloseView => self.close_view(),
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

                        self.modal = Some(Modal::CollectionAdd {
                            item,
                            collections,
                            selected,
                            initial,
                        });
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
                        } = &mut self.state
                        {
                            if awake.is_none() {
                                *awake = Some(keep_awake().unwrap());
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
                        Some(item) => load_video(item, |video| ManagerMessage::Video(false, video)),
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
                                let msg = Message::error(format!("Invalid input: {amt}"));
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
                                let msg = Message::error(format!("Invalid input: {amt}"));
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
                                let msg = Message::error(format!("Invalid input: {amt}"));
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
                                let msg = Message::error(format!("Invalid input: {amt}"));
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
                                let msg = Message::error(format!("Invalid input: {span}"));
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

                            Task::none()
                        }
                        SubtitleConfig::CurrentText(text) => {
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
                                let msg = Message::error(format!("Invalid input: {size}"));
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
                            if let Some(color) = convert_color_str(&color) {
                                self.settings.subtitles.color = color;
                            };

                            config.text_color = color;

                            Task::none()
                        }
                        SubtitleConfig::SubBackground(color) => {
                            if let Some(color) = convert_color_str(&color) {
                                self.settings.subtitles.background_color = color;
                            };

                            config.background_color = color;

                            Task::none()
                        }
                        SubtitleConfig::SubFont(family) => {
                            self.settings.subtitles.font = family.to_string();
                            config.subtitle_font.selected = Some(family);

                            Task::none()
                        }
                        SubtitleConfig::OffsetIncr => {
                            if let Some(selected) = config.selected_text.as_mut() {
                                selected.offset += 0.25;
                            }

                            Task::none()
                        }
                        SubtitleConfig::OffsetDecr => {
                            if let Some(selected) = config.selected_text.as_mut() {
                                selected.offset -= 0.25;
                            }

                            Task::none()
                        }
                        SubtitleConfig::Offset(input) => {
                            let Some(selected) = config.selected_text.as_mut() else {
                                return Task::none();
                            };

                            if input.is_empty() {
                                selected.offset = 0.0;
                                return Task::none();
                            }

                            let Ok(input) = input.trim().parse::<f32>() else {
                                return Message::error("Invalid input").tasked();
                            };

                            selected.offset = input;

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
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Some(Panel::Comments(new)) = self.panel.as_mut() else {
                    return Task::none();
                };
                let nulls_first = self.settings.comments_nulls_first;
                let default = if nulls_first {
                    0
                } else {
                    player.video.duration().as_secs()
                };

                match csg {
                    CommentMessage::New => {
                        let id = widget::Id::unique();

                        *new = Some((id.clone(), text_editor::Content::default()));
                        operation::focus(id)
                    }
                    CommentMessage::NewCancel => {
                        new.take();

                        self.play()
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

                        let batch = comments
                            .entry(comment.inner.timestamp.unwrap_or(default))
                            .or_default();
                        batch.push(comment);

                        self.play()
                    }
                    CommentMessage::NewAction(action) => {
                        if let Some((_, content)) = new {
                            content.perform(action);
                        }

                        self.pause()
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
                                        let msg = Message::error(error);
                                        return msg.tasked();
                                    }
                                };

                                player.position = position as f64;
                                if let Err(msg) = player.seek_release(false) {
                                    return Task::done(Message::error(msg));
                                }
                            }
                            Ok(url) => {
                                if let Err(error) = open::that(url.as_str()) {
                                    return Message::error(error).tasked();
                                }
                            }
                            Err(error) => {
                                return Message::error(error).tasked();
                            }
                        }

                        Task::none()
                    }
                    CommentMessage::Edit { id, timestamp } => {
                        match comments.get_mut(&timestamp.unwrap_or_default()).and_then(
                            |comments| comments.iter_mut().find(|comment| comment.inner.id == id),
                        ) {
                            Some(comment) => comment.edit(),
                            None => Task::none(),
                        }
                    }
                    CommentMessage::Save { id, timestamp } => {
                        let Some(batch) = comments.get_mut(&timestamp.unwrap_or(default)) else {
                            return Task::none();
                        };

                        let Some((idx, _)) = batch
                            .iter()
                            .enumerate()
                            .find(|(_, comment)| comment.inner.id == id)
                        else {
                            return Task::none();
                        };

                        let mut saved = batch.remove(idx);
                        let timestamp = saved.save(player.position as u64).unwrap_or(default);

                        let batch = comments.entry(timestamp).or_default();
                        batch.push(saved);

                        self.play()
                    }
                    CommentMessage::Action {
                        id,
                        timestamp,
                        action,
                    } => {
                        if let Some(comment) = comments
                            .get_mut(&timestamp.unwrap_or(default))
                            .and_then(|comments| {
                                comments.iter_mut().find(|comment| comment.inner.id == id)
                            })
                        {
                            comment.perform_action(action);
                        }

                        self.pause()
                    }
                    CommentMessage::Cancel { id, timestamp } => {
                        if let Some(comment) = comments
                            .get_mut(&timestamp.unwrap_or(default))
                            .and_then(|comments| {
                                comments.iter_mut().find(|comment| comment.inner.id == id)
                            })
                        {
                            comment.cancel();
                        }

                        self.play()
                    }
                    CommentMessage::Delete { id, timestamp } => {
                        if let Some(comment) = comments
                            .get_mut(&timestamp.unwrap_or(default))
                            .and_then(|comments| {
                                comments.iter_mut().find(|comment| comment.inner.id == id)
                            })
                        {
                            comment.inner.removed = true;
                        };

                        Task::none()
                    }
                    CommentMessage::ImageShown { id, timestamp, url } => {
                        let Some(comment) = comments
                            .get_mut(&timestamp.unwrap_or(default))
                            .and_then(|comments| {
                                comments.iter_mut().find(|comment| comment.inner.id == id)
                            })
                        else {
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
                            .get_mut(&timestamp.unwrap_or(default))
                            .and_then(|comments| {
                                comments
                                    .iter_mut()
                                    .find(|comment| comment.inner.id == id)
                                    .map(|comment| &mut comment.images)
                            })
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
                    .style(styles::container::text)
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
                    .style(styles::button::text_slate)
            )
            .spacing(6.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);

        let back = container(tooltip(
            sized_button(icons::BACK, icon_size)
                .on_press(ManagerMessage::PreviousScreen)
                .style(styles::button::text_slate),
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

        let content: Element<'_, ManagerMessage> = if self.show_controls {
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
                    .style(styles::container::text)
                    .width(60.0);

                let remaining = duration.as_secs().saturating_sub(*position as u64);
                let remaining = duration_string(remaining);
                let remaining = container(medium(remaining))
                    .style(styles::container::text)
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

        let left = {
            let volume = slider(
                0.0..=1.0,
                self.settings.volume,
                ManagerMessage::ChangeVolume,
            )
            .step(0.05)
            .shift_step(0.1)
            .width(125.0);

            let volume_text = format!("{:.0}", self.settings.volume * 100.0);

            let volume = tooltip(volume, volume_text, tp);

            let speed = container(
                mono_bold(format!("{:.02}x", self.settings.speed)).size(icon_size / (typo::RATIO)),
            )
            .style(styles::container::text);
            let speed = tooltip(
                button(speed)
                    .padding(0)
                    .style(styles::button::text_slate)
                    .on_press(ManagerMessage::SpeedReset),
                "Playback speed",
                tp,
            );

            let subtitles = if self.settings.show_subtitles {
                tooltip(
                    sized_button(icons::SUBTITLES_OFF, icon_size)
                        .on_press(ManagerMessage::ToggleSubtitles)
                        .style(styles::button::text_slate),
                    "Subtitles off",
                    tp,
                )
            } else {
                tooltip(
                    sized_button(icons::SUBTITLES_ON, icon_size)
                        .on_press(ManagerMessage::ToggleSubtitles)
                        .style(styles::button::text_slate),
                    "Subtitles on",
                    tp,
                )
            };

            let mute = if self.settings.muted {
                tooltip(
                    sized_button(icons::MUTE, icon_size)
                        .on_press(ManagerMessage::ToggleMute)
                        .style(styles::button::text_slate),
                    "Unmute",
                    tp,
                )
            } else {
                tooltip(
                    sized_button(icons::VOLUME, icon_size)
                        .on_press(ManagerMessage::ToggleMute)
                        .style(styles::button::text_slate),
                    "Mute",
                    tp,
                )
            };

            row!(subtitles, speed, mute, volume)
                .spacing(4.0)
                .align_y(Vertical::Center)
        }
        .width(Self::WIDTH);

        let middle = {
            let size = if self.is_fullscreen {
                H2 * typo::RATIO
            } else {
                H2
            };
            let play: Element<'_, ManagerMessage> = match &self.state {
                State::Idle => sized_button(icons::PLAY, size)
                    .style(styles::button::text_slate)
                    .into(),
                State::Loading => container(throbber::circular().bar_height(3.0))
                    .style(styles::container::text)
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
                        .style(styles::button::text_slate)
                        .into()
                }
            };

            let previous: Element<'_, ManagerMessage> = match self.playlist.previous_peek() {
                Some(previous) => tooltip(
                    sized_button(icons::PREVIOUS_VIDEO, size)
                        .style(styles::button::text_slate)
                        .on_press(ManagerMessage::PlayPrevious),
                    &previous.name,
                    tp,
                )
                .into(),
                None => sized_button(icons::PREVIOUS_VIDEO, size)
                    .style(styles::button::text_slate)
                    .into(),
            };

            let next: Element<'_, ManagerMessage> = match self.playlist.next_peek() {
                Some(next) => tooltip(
                    sized_button(icons::NEXT_VIDEO, size)
                        .style(styles::button::text_slate)
                        .on_press(ManagerMessage::PlayNext),
                    &next.name,
                    tp,
                )
                .into(),
                None => sized_button(icons::NEXT_VIDEO, size)
                    .style(styles::button::text_slate)
                    .into(),
            };

            let seek_amt = self.settings.seek_change_amt.trunc() as i16;
            let sb = tooltip(
                sized_button(icons::SEEK_BACK, size)
                    .style(styles::button::text_slate)
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
                    .style(styles::button::text_slate)
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
                    .style(styles::button::text_slate)
                    .on_press(ManagerMessage::ToggleFullscreen),
                "Exit Fullscreen",
                tp,
            )
        } else {
            tooltip(
                sized_button(icons::MAXIMIZE, icon_size)
                    .style(styles::button::text_slate)
                    .on_press(ManagerMessage::ToggleFullscreen),
                "Enter Fullscreen",
                tp,
            )
        };

        let right = column!(
            row!(
                tooltip(
                    sized_button(icons::ADD_COLLECTION, icon_size * typo::RATIO)
                        .style(styles::button::text_slate)
                        .on_press_maybe(self.is_ready(ManagerMessage::AddCollection)),
                    "Add to collection",
                    tp
                ),
                tooltip(
                    sized_button(icons::COMMENT, icon_size)
                        .style(styles::button::text_slate)
                        .on_press_maybe(self.is_ready(ManagerMessage::Comment)),
                    "Comments",
                    tp
                ),
                tooltip(
                    sized_button(icons::PLAYLIST, icon_size)
                        .style(styles::button::text_slate)
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

        let content: Element<'_, ManagerMessage> = if self.show_controls {
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
                let video = container(
                    VideoPlayer::new(&player.video)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .on_click(handle_clicks)
                        .on_error(|error| ManagerMessage::Error(error.to_string()))
                        .content_fit(iced::ContentFit::Contain)
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

    pub fn view(&self, theme: &Theme, now: Instant) -> Element<'_, ManagerMessage> {
        let content = stack!(
            self.video_elem(),
            column!(self.top(), space::vertical(), self.media_controls())
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

        let content: Element<'_, ManagerMessage> = match &self.panel {
            Some(Panel::Playlist) => row!(
                content,
                draw_playlist(&self.playlist, self.settings.auto_next)
            )
            .height(Length::Fill)
            .into(),
            Some(Panel::Comments(new)) => match &self.state {
                State::Ready {
                    player,
                    comments,
                    awake: _awake,
                } => row!(
                    content,
                    draw_comments(
                        new,
                        comments,
                        player.position as u64,
                        self.settings.comment_span,
                        theme,
                        now,
                    )
                )
                .height(Length::Fill)
                .into(),
                _ => content.into(),
            },
            None => content.into(),
        };

        match &self.modal {
            None => content,
            Some(Modal::CollectionAdd {
                collections,
                selected,
                ..
            }) => modal(
                content,
                draw_collection_add(selected, collections.is_empty(), collections.iter()),
                ManagerMessage::CloseView,
            ),
            Some(Modal::Config(config)) => {
                let (subs, audio) = self
                    .player()
                    .map(|player| {
                        (
                            player.item.subtitles.as_slice(),
                            player.embedded_audio.as_slice(),
                        )
                    })
                    .unwrap_or_default();
                modal(
                    content,
                    draw_config(&self.settings, config, subs, audio),
                    ManagerMessage::CloseView,
                )
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        match &self.state {
            State::Ready { comments, .. } => comments
                .iter()
                .any(|(_, comments)| comments.iter().any(|comment| comment.is_animating(now))),
            State::Loading | State::Idle => false,
        }
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

    fn play_toggle(&mut self) -> Task<Message> {
        let State::Ready {
            player,
            awake: _awake,
            comments: _comments,
        } = &mut self.state
        else {
            return Task::none();
        };

        let Player { video, .. } = player.as_mut();

        let is_paused = video.paused();

        if is_paused { self.play() } else { self.pause() }
    }

    fn play(&mut self) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
        } = &mut self.state
        else {
            return Task::none();
        };

        let Player {
            video, position, ..
        } = player.as_mut();

        let is_paused = video.paused();

        if is_paused {
            *awake = Some(keep_awake().unwrap());
        } else {
            return Task::none();
        }

        if video.eos() && is_paused {
            if let Err(error) = video.seek(Duration::from_secs(0), false) {
                return Message::error(error).tasked();
            }

            *position = 0.0;

            video.set_paused(false);
        } else {
            video.set_paused(!video.paused());
        }

        Task::none()
    }

    fn pause(&mut self) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
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

        Task::none()
    }

    fn fullscreen_toggle(&mut self) -> Task<Message> {
        if self.modal.is_some() {
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

    fn fullscreen_exit(&mut self) -> Task<Message> {
        self.show_controls = true;
        self.is_fullscreen = false;

        self.window
            .map(move |id| window::set_mode::<Message>(id, window::Mode::Windowed).discard())
            .unwrap_or_default()
    }

    fn seek_back(&mut self, shift: bool) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            player.is_dragging = false;
            let amt = if shift {
                self.settings.seek_shift_change_amt
            } else {
                self.settings.seek_change_amt
            };

            player.last_frame.take();
            player.position = (player.position - amt).max(0.0);
            player
                .video
                .seek(Duration::from_secs_f64(player.position), false)
                .unwrap();
        }

        Task::none()
    }

    fn seek_front(&mut self, shift: bool) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            player.is_dragging = false;
            let duration = player.video.duration().as_secs_f64();
            let amt = if shift {
                self.settings.seek_shift_change_amt
            } else {
                self.settings.seek_change_amt
            };

            player.last_frame.take();
            player.position = (player.position + amt).min(duration);
            player
                .video
                .seek(Duration::from_secs_f64(player.position), false)
                .unwrap();
        }
        Task::none()
    }

    fn volume_increase(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.volume =
                (self.settings.volume + self.settings.volume_change_amt).min(1.0);
            player.video.set_volume(self.settings.volume);
        }

        Task::none()
    }

    fn volume_decrease(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.volume =
                (self.settings.volume - self.settings.volume_change_amt).max(0.0);
            player.video.set_volume(self.settings.volume);
        }

        Task::none()
    }

    fn mute_toggle(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            let mute = !player.video.muted();
            player.video.set_muted(mute);
            self.settings.muted = mute;

            if mute {
                self.settings.volume = 0.0
            } else {
                self.settings.volume = player.video.volume()
            }
        }

        Task::none()
    }

    fn speed_increase(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.speed += self.settings.speed_change_amt;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
    }

    fn speed_decrease(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state {
            self.settings.speed -= self.settings.speed_change_amt;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
    }

    fn speed_reset(&mut self) -> Task<Message> {
        if let State::Ready { player, .. } = &mut self.state
            && player.video.speed() != 1.0
        {
            self.settings.speed = 1.0;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
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

                let load = load_video(next.clone(), |video| ManagerMessage::Video(false, video))
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
                let (mut player, comments) = match player {
                    AutoState::Ready { player, comments } => (player, comments),
                    _ => unreachable!(),
                };

                let last_watched = Message::LastWatched(player.item.id);

                apply_settings(&self.settings, &mut player);
                player.video.set_paused(false);

                let awake = keep_awake().unwrap();

                self.state = State::Ready {
                    player,
                    awake: Some(awake),
                    comments,
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

        let load = load_video(previous.clone(), |video| {
            ManagerMessage::Video(false, video)
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
        } = &mut self.state
        else {
            return Task::none();
        };

        let Modal::Config(config) = modal else {
            return self.play_toggle();
        };

        let Config {
            subtitle_uri,
            selected_text,
            selected_audio,
            tab: _tab,
            text_color: _text_color,
            background_color: _background,
            subtitle_font: _subtitle,
        } = *config;

        apply_settings(&self.settings, player);

        let set_loaded = |player: &mut Player, url: url::Url| {
            let position = player.position;
            let position = Duration::from_secs_f64(position);

            if let Err(error) = player.video.set_subtitle_url(&url) {
                return Some(Message::error(error));
            };

            std::thread::sleep(std::time::Duration::from_millis(150));
            if let Err(error) = player.video.seek(position, false) {
                return Some(Message::error(error));
            };

            None
        };

        let set_uri = |player: &mut Player, uri: PathBuf| {
            let path = match uri.canonicalize() {
                Ok(subtitles) => subtitles,
                Err(error) => return Some(Message::error(error)),
            };

            let url = match url::Url::from_file_path(path) {
                Ok(url) => url,
                Err(_) => {
                    return Some(Message::error("Cannot generate url from subtitle path"));
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

        if let Some(selected) = selected_text {
            let changed = player.item.subtitle_id != Some(selected.id);

            if changed {
                player.item.subtitle_id = Some(selected.id);
                match &selected.kind {
                    SubtitleKind::Loaded { path, .. } => {
                        let path = path.clone();

                        if let Some(message) = set_uri(player, path) {
                            return Task::done(message);
                        }
                    }
                    SubtitleKind::Embedded => {
                        if let Some(tag) =
                            player.video.available_subtitles().into_iter().find(|tag| {
                                tag.title == selected.title && tag.language_code == selected.lang
                            })
                        {
                            player.video.set_text(tag);
                        };
                    }
                }
            }

            if let Some(save_offset) = player
                .item
                .subtitles
                .iter_mut()
                .find(|sub| selected.id == sub.id && (sub.offset != selected.offset || changed))
            {
                save_offset.offset = selected.offset;

                let offset = (1_000_000_000 as f32 * selected.offset) as i64;

                player.video.set_text_offset(offset);
            };
        }

        if let Some(audio) = selected_audio {
            let changed = player
                .current_audio
                .as_ref()
                .map(|og| og != &audio)
                .unwrap_or(true);
            if changed {
                player.video.set_audio(audio);
            }
        }

        if awake.is_none() {
            *awake = Some(keep_awake().unwrap());
        }
        player.video.set_paused(false);

        Task::none()
    }

    pub fn close_view(&mut self) -> Task<Message> {
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
        self.panel = None;
        Task::none()
    }

    fn collection_add(&mut self) -> Task<Message> {
        let State::Ready {
            player,
            awake,
            comments: _comments,
        } = &mut self.state
        else {
            return Task::none();
        };

        player.video.set_paused(true);
        awake.take();

        let id = player.item.id;
        let view = Modal::CollectionAdd {
            item: id,
            collections: vec![],
            selected: HashSet::default(),
            initial: HashSet::default(),
        };

        self.modal = Some(view);

        let ids = Task::done(Message::FetchMembershipIds(id.into()));
        let cols = Task::done(Message::fetch_simple_collections());

        Task::batch([ids, cols])
    }

    fn video_config(&mut self) -> Task<Message> {
        let (selected_text, selected_audio, subtitle_uri) = if let State::Ready {
            player,
            awake,
            comments: _comments,
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

            (current, player.current_audio.clone(), None)
        } else {
            (None, None, None)
        };

        self.modal = Some(Modal::Config(Box::new(Config {
            tab: ConfigTab::General,
            subtitle_uri,
            selected_text,
            selected_audio,
            text_color: format!("#{:08x}", self.settings.subtitles.color),
            background_color: format!("#{:08x}", self.settings.subtitles.background_color),
            subtitle_font: FontState::new(self.fonts.clone(), &self.settings.subtitles.font),
        })));

        Task::none()
    }

    fn video_comment(&mut self) -> Task<Message> {
        if matches!(self.panel, Some(Panel::Comments(_))) {
            self.close_panel()
        } else {
            self.panel = Some(Panel::Comments(None));
            Task::none()
        }
    }

    pub fn action(&mut self, action: PlayerAction) -> Task<Message> {
        match action {
            PlayerAction::PlayToggle => self.play_toggle(),
            PlayerAction::PlayNext => self.play_next(),
            PlayerAction::PlayPrevious => self.play_previous(),
            PlayerAction::FullscreenToggle => self.fullscreen_toggle(),
            PlayerAction::Exit => {
                if self.modal.is_some() || self.panel.is_some() {
                    self.close_view()
                } else {
                    self.fullscreen_exit()
                }
            }
            PlayerAction::SeekBack => self.seek_back(false),
            PlayerAction::SeekBackShift => self.seek_back(true),
            PlayerAction::SeekFront => self.seek_front(false),
            PlayerAction::SeekFrontShift => self.seek_front(true),
            PlayerAction::VolumeIncrease => self.volume_increase(),
            PlayerAction::VolumeDecrease => self.volume_decrease(),
            PlayerAction::MuteToggle => self.mute_toggle(),
            PlayerAction::SpeedIncrease => self.speed_increase(),
            PlayerAction::SpeedDecrease => self.speed_decrease(),
            PlayerAction::SpeedReset => self.speed_reset(),
            PlayerAction::SubtitlesToggle => self.subtitles_toggle(),
            PlayerAction::Add => self.collection_add(),
            PlayerAction::VideoConfig => self.video_config(),
            PlayerAction::VideoComment => self.video_comment(),
            PlayerAction::CloseView => self.close_view(),
            PlayerAction::Back => Task::done(Message::Back),
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

        let comments = comments
            .values()
            .flat_map(|comments| comments.iter().map(|comment| comment.inner.clone()))
            .collect();
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
            Task::done(Message::VideoStats(player.item.clone())),
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
        let nulls_start = self.settings.comments_nulls_first;

        let update = |player: &mut Player, curr: &mut BTreeMap<u64, Vec<Comment>>| {
            let default = if nulls_start {
                0
            } else {
                player.video.duration().as_secs()
            };

            let comments = comments
                .into_iter()
                .map(|comment| (comment.inner.timestamp.unwrap_or(default), comment));

            for (timestamp, comment) in comments {
                let entry = curr.entry(timestamp).or_default();

                entry.push(comment)
            }
        };

        match &mut self.state {
            State::Ready {
                player,
                comments: curr,
                awake: _awake,
            } if player.item.id == id => {
                update(player, curr);

                Task::none()
            }
            _ => match &mut self.next {
                AutoState::Ready {
                    player,
                    comments: curr,
                } if player.item.id == id => {
                    update(player, curr);

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
    f: impl FnOnce(Arc<Player>) -> Message + 'static + MaybeSend,
) -> Task<Message> {
    Task::perform(
        tokio::task::spawn_blocking(move || {
            let path = url::Url::from_file_path(item.path.canonicalize().unwrap()).unwrap();
            let mut video = Video::new(&path).unwrap();

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

                let new = models::Subtitle::new_embedded(item.id, &em.title, &em.language_code);
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
                            .inspect_err(|error| tracing::error!("Video Subtitle error \n{error}"))
                            .ok()
                            .and_then(|path| {
                                url::Url::from_file_path(path)
                                    .inspect_err(|_| {
                                        tracing::error!(
                                            "Video Subtitle error. Cannot create Url from path"
                                        )
                                    })
                                    .ok()
                            });

                        if let Some(url) = path.as_ref()
                            && let Err(error) = video.set_subtitle_url(url)
                        {
                            tracing::error!("Video Subtitle error \n{error}")
                        }
                    }
                }

                let offset = (1_000_000_000 as f32 * saved_sub.offset) as i64;

                video.set_text_offset(offset);
            }

            std::thread::sleep(std::time::Duration::from_millis(150));

            let progress = if item.progress >= 0.98 {
                0.0
            } else {
                item.progress
            };
            let position = (duration * progress as f64).round().clamp(0.0, duration);

            video.seek(Duration::from_secs_f64(position), true).unwrap();

            // todo: There is a race condition when resuming a video. I can't quite pinpoint where
            // so until I do, this is a temporary fix which seems to work.
            std::thread::sleep(std::time::Duration::from_millis(200));

            video.set_paused(true);

            let curr_audio = video.get_audio();

            let audio = video.available_audio();

            Arc::new(Player {
                item,
                video,
                thumbnails: vec![],
                position,
                is_dragging: false,
                watch_time: Duration::ZERO,
                last_frame: None,
                subtitles: None,
                current_audio: curr_audio,
                embedded_audio: audio,
            })
        }),
        move |res| f(res.unwrap()),
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
        comments_nulls_first: _nulls_first,
        filters:
            utils::VideoFilters {
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
    player.video.set_speed(*speed).unwrap();
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

fn draw_playlist<'a>(playlist: &'a Playlist, auto_next: bool) -> Element<'a, ManagerMessage> {
    let rule_height = 1.0;
    let padding = [6, 12];

    let title = {
        let content = row!(
            sized_bold("Playlist", P),
            space::horizontal(),
            button(icons::icon(CANCEL).size(H6))
                .on_press(ManagerMessage::ClosePanel)
                .style(styles::button::text)
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
            let color = theme.palette().primary.base.color;

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
        .style(styles::button::text)
        .into()
    });

    let items = container(column(items).spacing(8)).padding(padding);

    let actions = {
        let size = H6;
        let position = tp::Position::Top;
        let color = |theme: &Theme, active: bool| {
            let color = theme.palette().primary.base.color;

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
            .style(styles::button::text)
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
            .style(styles::button::text)
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
                .style(styles::button::text)
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
        .spacing(0);

    let content = panel_container(content).padding([3, 0]);

    content.into()
}

fn draw_comments<'a>(
    new: &'a Option<(widget::Id, text_editor::Content)>,
    comments: &'a BTreeMap<u64, Vec<Comment>>,
    position: u64,
    span: u64,
    theme: &Theme,
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
                .style(styles::button::text)
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

            let content = container(content).padding(4).style(move |theme| {
                let default = styles::container::bw(theme);
                let border = default.border.rounded(8);

                container::Style { border, ..default }
            });

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

    let content: Element<'_, CommentMessage> = {
        let timestamp = { position.saturating_sub(span)..=position.saturating_add(span) };

        let comments = comments
            .range(timestamp)
            .flat_map(|(_, comments)| comments.iter().filter(|comment| !comment.inner.removed));

        column(comments.map(|comment| {
            container(comment.view(now, theme))
                .max_height(325)
                .width(Length::Fill)
                .into()
        }))
        .spacing(16)
        .into()
    };

    let content: Element<'_, CommentMessage> =
        scrollable(content).height(Length::Fill).spacing(10).into();

    let content = column!(
        title,
        content.map(ManagerMessage::CommentMessage),
        new.map(ManagerMessage::CommentMessage)
    )
    .spacing(20)
    .align_x(Horizontal::Center);

    let content = panel_container(content)
        .padding([3, 6])
        .align_x(Horizontal::Center);

    content.into()
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
            .max_height(48.0)
            .max_width(275);
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
            let default = if selected {
                styles::button::subtle(theme, status)
            } else {
                styles::button::subtlest(theme, status)
            };

            let border = default.border.rounded(5.0);

            button::Style { border, ..default }
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
            let color = theme.palette().secondary.strong.color;
            let default = styles::container::transparent(theme);
            let border = default.border.rounded(5).color(color).width(1.5);

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

    modal_container(content).max_width(400).into()
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

fn draw_filters<'a>(settings: &'a VideoSettings, size: f32) -> Element<'a, VideoConfig> {
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

    column!(gamma, brightness, contrast, hue, saturation)
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
        .style(styles::button::subtle)
        .on_press(SubtitleConfig::SelectFile);

        let path = config.subtitle_uri.as_ref().map(|path| trim_path(path, 3));
        let path: Element<'_, SubtitleConfig> = match path {
            Some(path) => button(
                row!(
                    container(marquee(path).font(mono_font()).size(size))
                        .max_width(250)
                        .height(20)
                        .clip(true),
                    icons::icon(icons::CANCEL).size(size)
                )
                .spacing(4)
                .align_y(Vertical::Center),
            )
            .padding([2, 5])
            .style(styles::button::text)
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
                .ellipsis(text::Ellipsis::Start)
                .on_select(SubtitleConfig::CurrentText)
                .padding(padding)
                .text_size(size)
                .into()
        };

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    let offset: Element<'_, SubtitleConfig> = match &config.selected_text {
        Some(selected) => {
            let label = label_maker("Subtitle Offset(seconds): ");
            let offset = format!("{:.02}", selected.offset);

            let actions = {
                let incr = button(icons::icon(icons::CHEV_UP).size(10))
                    .padding([2, 2])
                    .style(styles::button::subtler)
                    .on_press(SubtitleConfig::OffsetIncr);
                let decr = button(icons::icon(icons::CHEV_DOWN).size(10))
                    .padding([2, 2])
                    .style(styles::button::subtler)
                    .on_press(SubtitleConfig::OffsetDecr);

                column!(incr, decr).spacing(2.0)
            };

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
        }
        None => empty(),
    };

    let style = {
        let label = label_maker("Subtitle Style").size(P);
        let dummy = draw_subtitles("An example subtitle", &settings.subtitles);

        let sub_size = {
            let label = label_maker("Size: ");

            let amt = format!("{}", settings.subtitles.size);

            let actions = {
                let incr = button(icons::icon(icons::CHEV_UP).size(10))
                    .padding([2, 2])
                    .style(styles::button::subtler)
                    .on_press(SubtitleConfig::SubSizeIncr);
                let decr = button(icons::icon(icons::CHEV_DOWN).size(10))
                    .padding([2, 2])
                    .style(styles::button::subtler)
                    .on_press(SubtitleConfig::SubSizeDecr);

                column!(incr, decr).spacing(2.0)
            };

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

            let selection = combo_box(
                &config.subtitle_font.state,
                "",
                config.subtitle_font.selected.as_ref(),
                SubtitleConfig::SubFont,
            )
            .width(200)
            .size(size)
            .font(regular_font())
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
    audio: &'a [AudioTag],
    size: f32,
    padding: Padding,
    spacing: f32,
) -> Element<'a, AudioConfig> {
    let selection = {
        let label = label_maker("Embedded Audio: ");

        let handle = picklist_handle(size);

        let pick: Element<'_, AudioConfig> = if audio.is_empty() {
            label_maker("None").size(size).into()
        } else {
            pick_list(config.selected_audio.clone(), audio, ToString::to_string)
                .handle(handle)
                .on_select(AudioConfig::CurrentAudio)
                .padding(padding)
                .text_size(size)
                .into()
        };

        row!(label, space::horizontal(), pick)
            .align_y(Vertical::Center)
            .spacing(spacing)
    };

    column!(selection).width(Length::Fill).spacing(12).into()
}

fn draw_config<'a>(
    settings: &'a VideoSettings,
    config: &'a Config,
    subtitles: &'a [Subtitle],
    audio: &'a [AudioTag],
) -> Element<'a, ManagerMessage> {
    // todo: Hardware volume, Aspect Ratio
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
                        styles::button::background_primary(theme, status)
                    } else {
                        styles::button::subtlest(theme, status)
                    }
                })
                .on_press(ManagerMessage::Config(ConfigMessage::Tab(*tab))),
        )
        .clip(true)
        .max_height(48.0)
        .into()
    });

    let content: Element<'_, ConfigMessage> = match curr_tab {
        ConfigTab::General => {
            draw_general(settings, size, padding, spacing).map(ConfigMessage::General)
        }
        ConfigTab::Filters => draw_filters(settings, size).map(ConfigMessage::Video),
        ConfigTab::Subtitles => draw_subs(settings, config, subtitles, size, padding, spacing)
            .map(ConfigMessage::Subtitle),
        ConfigTab::Audio => {
            draw_audio(config, audio, size, padding, spacing).map(ConfigMessage::Audio)
        }
    };

    let content = content.map(ManagerMessage::Config);

    let side = column(tabs)
        .spacing(8)
        .width(125.0)
        .height(Length::Fill)
        .padding(3);

    let side = container(side).style(|theme| {
        let default = styles::container::bw3(theme);
        let border = default.border.rounded(styles::RADIUS);

        container::Style { border, ..default }
    });

    let content = row!(side, content).spacing(20);

    let content = column!(header, content)
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(20);

    modal_container(content)
        .style(|theme| {
            let default = styles::container::bb(theme);
            let border = default.border.rounded(5.0);

            container::Style { border, ..default }
        })
        .padding([16, 16])
        .width(600)
        .height(400)
        .into()
}

fn label_maker<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sized_medium(label, H7)
}

fn panel_container<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content)
        .style(styles::container::bw3)
        .width(Length::FillPortion(2))
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
            // let len = name.len();
            // let end = len.min(36);
            // todo!()

            name.into()
        }
    }
}
