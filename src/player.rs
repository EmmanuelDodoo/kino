use iced::{
    Element, Font, Length, Size, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    time::{self, Instant},
    widget::{Svg, center, column, container, image, mouse_area, row, slider, space, stack, text},
    window,
};
use iced_video_player::{Button, Kind, MouseClick, Video, VideoPlayer};
use std::sync::Arc;
use std::time::Duration;

use crate::app::Message;
use crate::utils::{
    self, PlayId, PlayItem, PlayerAction, Playlist, VideoSettings,
    icons::{self, sized_button, text_button},
    typo::*,
};
use crate::widgets;

#[derive(Debug)]
pub struct Player {
    video: Video,
    position: f64,
    is_dragging: bool,
    thumbnails: Vec<image::Handle>,
    item: PlayItem,
    duration: f64,
}

#[derive(Debug)]
enum AutoState {
    Loading,
    Idle,
    Ready(Box<Player>),
}

#[derive(Debug)]
enum State {
    Loading(Animation<bool>),
    Idle,
    Ready(Box<Player>),
}

#[derive(Debug, Clone)]
pub enum ManagerMessage {
    Video(bool, Arc<Player>),
    Thumbnail((PlayId, Vec<image::Handle>)),
    Resize((window::Id, Size)),
    SeekRelease,
    Seek(f64),
    ChangeVolume(f64),
    CursorExit,
    CursorEnter,
    PreviousScreen,
    AddCollection,
    Config,
    ToggleSubtitles,
    ToggleMute,
    PlayPrevious,
    PlayNext,
    SeekFront(bool),
    SeekBack(bool),
    TogglePlay,
    Favorite,
    Comment,
    ToggleFullscreen,
    EndOfStream,
    NewFrame,
    None,
}

#[derive(Debug)]
pub struct Manager {
    window: Option<window::Id>,
    playlist: Playlist,
    show_controls: bool,

    settings: VideoSettings,

    maximised: bool,
    is_fullscreen: bool,
    state: State,
    next: AutoState,
}

impl Manager {
    const WIDTH: f32 = 150.0;

    pub fn boot(
        window: Option<window::Id>,
        settings: VideoSettings,
        playlist: Playlist,
    ) -> (Self, Task<ManagerMessage>) {
        let load_video = match playlist.current().cloned() {
            Some(item) => load_video(item, |video| ManagerMessage::Video(false, video)),
            None => Task::none(),
        };

        (Self::new(window, settings, playlist), load_video)
    }

    fn new(window: Option<window::Id>, settings: VideoSettings, playlist: Playlist) -> Self {
        let state = if !playlist.is_empty() {
            State::Loading(loading_animation(Instant::now()))
        } else {
            State::Idle
        };

        Self {
            window,
            playlist,
            show_controls: true,
            settings,
            // todo Probably best to get the initial value from app
            maximised: false,
            is_fullscreen: false,
            state,
            next: AutoState::Idle,
        }
    }

    fn prep_video(&self, player: &mut Player) {
        let VideoSettings {
            thumbnail_interval: _thumbnails,
            volume,
            speed,
            gamma,
            seek_mult: _seek,
            seek_shift_mult: _seek_shift,
            seek_change_amt: _seek_amt,
            volume_change_amt: _volume,
            speed_change_amt: _speed,
            show_subtitles: _subtitles,
            muted,
            autoplay,
        } = self.settings;

        player.video.set_volume(volume);
        player.video.set_speed(speed).unwrap();
        player.video.set_paused(!autoplay);
        player.video.set_gamma(gamma);
        player.video.set_muted(muted);
    }

    pub fn update(&mut self, message: ManagerMessage, now: Instant) -> Task<Message> {
        match message {
            ManagerMessage::None => Task::none(),
            ManagerMessage::Video(is_next, player) => {
                let mut player = Arc::try_unwrap(player).unwrap();

                let id = player.item.id;
                let path = player.item.path.clone();

                let interval = self.settings.thumbnail_interval;
                let duration = player.duration;
                let (width, height) = player.video.size();

                let load_thumbnails = Task::perform(
                    tokio::task::spawn_blocking(move || {
                        let num = duration as u32 / interval;
                        let generator = utils::ThumbnailGenerator::new(path, width, height, 8);
                        let imgs = (1..=num)
                            .map(|i| {
                                generator.generate(gstreamer::ClockTime::from_seconds_f64(
                                    duration * (i as f64 / num as f64),
                                ))
                            })
                            .collect();

                        drop(generator);

                        (id, imgs)
                    }),
                    move |res| ManagerMessage::Thumbnail(res.unwrap()),
                );

                if !is_next || matches!(&self.state, State::Idle) {
                    self.prep_video(&mut player);
                    self.state = State::Ready(Box::new(player));
                } else {
                    player.video.set_paused(true);
                    self.next = AutoState::Ready(Box::new(player));
                }

                load_thumbnails.map(Message::Player)
            }
            ManagerMessage::Thumbnail((id, generated)) => {
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
                } else if let AutoState::Ready(player) = &mut self.next
                    && player.item.id == id
                {
                    player.thumbnails = generated;
                }

                Task::none()
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
                // update playlist
                todo!("Send message to main with some video stats");
            }
            ManagerMessage::NewFrame => {
                if let State::Ready(player) = &mut self.state
                    && !player.is_dragging
                {
                    player.position = player.video.position().as_secs_f64();

                    if (player.position) / (player.duration) >= 0.9
                        && self.playlist.has_next()
                        && matches!(&self.next, AutoState::Idle)
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
                if let Some(Player {
                    video,
                    position,
                    is_dragging,
                    ..
                }) = self.player_mut()
                {
                    *is_dragging = false;

                    video
                        .seek(Duration::from_secs_f64(position.max(0.0)), false)
                        .unwrap();
                    video.set_paused(false);
                }

                Task::none()
            }
            ManagerMessage::Seek(pos) => {
                if let Some(Player {
                    video,
                    position,
                    is_dragging,
                    ..
                }) = self.player_mut()
                {
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
                use utils::Action;

                Task::done(Message::Action(Action::Back))
            }
            ManagerMessage::ToggleSubtitles => self.subtitles_toggle(),
            ManagerMessage::PlayNext => self.play_next(now),
            ManagerMessage::PlayPrevious => self.play_previous(now),
            ManagerMessage::AddCollection => self.collection_add(),
            ManagerMessage::Favorite => self.collection_favorite(),
            ManagerMessage::Config => self.video_config(),
            ManagerMessage::Comment => self.video_comment(),
        }
    }

    pub fn subscription(&self) -> Subscription<ManagerMessage> {
        window::resize_events().map(ManagerMessage::Resize)
    }

    fn top(&self) -> Element<'_, ManagerMessage> {
        let title = match &self.state {
            State::Ready(player) => text(&player.item.name).size(H6),
            State::Loading(_) | State::Idle => text(""),
        };

        let icon_size = if self.is_fullscreen { H4 } else { H5 };
        let options = column!(
            row!(
                sized_button(icons::ADD_COLLECTION, icon_size)
                    .on_press_maybe(self.is_ready(ManagerMessage::AddCollection)),
                sized_button(icons::ELLIPSIS_VER, icon_size).on_press(ManagerMessage::Config)
            )
            .spacing(6.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);
        let back = container(
            sized_button(icons::BACK, icon_size).on_press(ManagerMessage::PreviousScreen),
        )
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
                let spent = format!(
                    "{:02}:{:02}:{:02}",
                    *position as u64 / 3600,
                    *position as u64 / 60,
                    *position as u64 % 60,
                );

                let remaining = duration.as_secs().saturating_sub(*position as u64);
                let total = format!(
                    "{:02}:{:02}:{:02}",
                    remaining / 3600,
                    remaining / 60,
                    remaining % 60,
                );

                let slider = widgets::slider::VideoSlider::new(
                    0.0..=duration.as_secs_f64(),
                    *position,
                    ManagerMessage::Seek,
                    thumbnails,
                    Font::default(),
                    duration,
                )
                .step(0.1)
                .on_release(ManagerMessage::SeekRelease);

                row!(text(spent), slider, text(total))
                    .spacing(20.0)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .into()
            }
            _ => space::horizontal().into(),
        }
    }

    fn is_ready(&self, message: ManagerMessage) -> Option<ManagerMessage> {
        matches!(&self.state, State::Ready(_)).then_some(message)
    }

    fn media_controls(&self, now: Instant) -> Element<'_, ManagerMessage> {
        let icon_size = if self.is_fullscreen { H4 } else { H5 };
        let left = {
            let volume = slider(
                0.0..=1.0,
                self.settings.volume,
                ManagerMessage::ChangeVolume,
            )
            .step(0.05)
            .shift_step(0.1)
            .width(125.0);
            row!(
                sized_button(
                    if self.settings.show_subtitles {
                        icons::SUBTITLES_OFF
                    } else {
                        icons::SUBTITLES_ON
                    },
                    icon_size
                )
                .on_press(ManagerMessage::ToggleSubtitles),
                sized_button(
                    if self.settings.muted {
                        icons::MUTE
                    } else {
                        icons::VOLUME
                    },
                    icon_size
                )
                .on_press(ManagerMessage::ToggleMute),
                volume
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        }
        .width(Self::WIDTH);

        let middle = {
            let size = if self.is_fullscreen { H1 * 1.125 } else { H1 };
            let play: Element<'_, ManagerMessage> = match &self.state {
                State::Idle => sized_button(icons::PLAY, size).into(),
                State::Loading(animation) => container(loading_svg(animation, now))
                    .width(size)
                    .height(size)
                    .into(),
                State::Ready(player) => {
                    let (icon, message) = if player.video.paused() {
                        (icons::PLAY, ManagerMessage::TogglePlay)
                    } else if player.video.eos() {
                        (icons::REPLAY, ManagerMessage::TogglePlay)
                    } else {
                        (icons::PAUSE, ManagerMessage::TogglePlay)
                    };

                    sized_button(icon, size).on_press(message).into()
                }
            };

            row!(
                sized_button(icons::PREVIOUS_VIDEO, size).on_press_maybe(
                    self.playlist
                        .has_previous()
                        .then_some(ManagerMessage::PlayPrevious)
                ),
                sized_button(icons::SEEK_BACK, size)
                    .on_press_maybe(self.is_ready(ManagerMessage::SeekBack(false))),
                play,
                sized_button(icons::SEEK_FRONT, size)
                    .on_press_maybe(self.is_ready(ManagerMessage::SeekFront(false)),),
                sized_button(icons::NEXT_VIDEO, size)
                    .on_press_maybe(self.playlist.has_next().then_some(ManagerMessage::PlayNext))
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        };

        let right = column!(
            row!(
                sized_button(icons::FAVORITE, icon_size)
                    .on_press_maybe(self.is_ready(ManagerMessage::Favorite)),
                sized_button(icons::COMMENT, icon_size)
                    .on_press_maybe(self.is_ready(ManagerMessage::Comment)),
                sized_button(
                    if self.is_fullscreen {
                        icons::MINIMIZE
                    } else {
                        icons::MAXIMIZE
                    },
                    icon_size
                )
                .on_press(ManagerMessage::ToggleFullscreen)
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

        content.into()
    }

    fn video_elem(&self, now: Instant) -> Element<'_, ManagerMessage> {
        match &self.state {
            State::Ready(player) => {
                let video = container(
                    VideoPlayer::new(&player.video)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .on_click(handle_clicks)
                        .content_fit(iced::ContentFit::Contain)
                        .on_end_of_stream(ManagerMessage::EndOfStream)
                        .on_new_frame(ManagerMessage::NewFrame),
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

                video.into()
            }
            State::Loading(animation) => center(loading_svg(animation, now)).into(),
            State::Idle => center("No video loaded").into(),
        }
    }

    pub fn view(&self, now: Instant) -> Element<'_, ManagerMessage> {
        let content = stack!(
            self.video_elem(now),
            column!(self.top(), space::vertical(), self.media_controls(now))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([3, 6])
        )
        .height(Length::Fill)
        .width(Length::Fill);

        let content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                ..Default::default()
            });

        content.into()
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let State::Loading(animation) = &self.state else {
            return false;
        };

        animation.is_animating(now)
    }

    fn player(&self) -> Option<&Player> {
        match &self.state {
            State::Ready(player) => Some(player),
            _ => None,
        }
    }

    fn player_mut(&mut self) -> Option<&mut Player> {
        match &mut self.state {
            State::Ready(player) => Some(player),
            _ => None,
        }
    }

    fn play_toggle(&mut self) -> Task<Message> {
        if let Some(Player { video, .. }) = self.player_mut() {
            video.set_paused(!video.paused());
        }
        Task::none()
    }

    fn fullscreen_toggle(&mut self) -> Task<Message> {
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
        if let State::Ready(player) = &mut self.state {
            player.is_dragging = false;
            let mult = if shift {
                self.settings.seek_shift_mult
            } else {
                self.settings.seek_mult
            };

            player.position = (player.position - (self.settings.seek_change_amt * mult)).max(0.0);
            player
                .video
                .seek(Duration::from_secs_f64(player.position), false)
                .unwrap();
        }

        Task::none()
    }

    fn seek_front(&mut self, shift: bool) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
            player.is_dragging = false;
            let duration = player.video.duration().as_secs_f64();
            let mult = if shift {
                self.settings.seek_shift_mult
            } else {
                self.settings.seek_mult
            };
            player.position =
                (player.position + (self.settings.seek_change_amt * mult)).min(duration);
            player
                .video
                .seek(Duration::from_secs_f64(player.position), false)
                .unwrap();
        }
        Task::none()
    }

    fn volume_increase(&mut self) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
            self.settings.volume =
                (self.settings.volume + self.settings.volume_change_amt).min(1.0);
            player.video.set_volume(self.settings.volume);
        }

        Task::none()
    }

    fn volume_decrease(&mut self) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
            self.settings.volume =
                (self.settings.volume - self.settings.volume_change_amt).max(0.0);
            player.video.set_volume(self.settings.volume);
        }

        Task::none()
    }

    fn mute_toggle(&mut self) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
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
        if let State::Ready(player) = &mut self.state {
            self.settings.speed += self.settings.speed_change_amt;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
    }

    fn speed_decrease(&mut self) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
            self.settings.speed -= self.settings.speed_change_amt;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
    }

    fn speed_reset(&mut self) -> Task<Message> {
        if let State::Ready(player) = &mut self.state {
            self.settings.speed = 1.0;
            player.video.set_speed(self.settings.speed).unwrap();
        }

        Task::none()
    }

    fn subtitles_toggle(&mut self) -> Task<Message> {
        // todo: Video settings
        if let Some(player) = self.player_mut() {
            player.video.toggle_subtitle();
        }

        Task::none()
    }

    fn play_next(&mut self, now: Instant) -> Task<Message> {
        if !self.playlist.has_next() {
            return Task::none();
        }

        let Some(next) = self.playlist.next() else {
            return Task::none();
        };

        match &mut self.next {
            AutoState::Idle => {
                self.state = State::Loading(loading_animation(now));

                load_video(next.clone(), |video| ManagerMessage::Video(false, video))
                    .map(Message::Player)
            }
            AutoState::Loading => {
                self.state = State::Idle;
                Task::none()
            }
            ready => {
                let player = std::mem::replace(ready, AutoState::Idle);
                let mut player = match player {
                    AutoState::Ready(player) => player,
                    _ => unreachable!(),
                };

                self.prep_video(&mut player);
                player.video.set_paused(false);

                self.state = State::Ready(player);

                Task::none()
            }
        }
    }

    fn play_previous(&mut self, now: Instant) -> Task<Message> {
        if !self.playlist.has_previous() {
            return Task::none();
        }

        let Some(previous) = self.playlist.previous() else {
            return Task::none();
        };

        // Intentionally discarding the current video. Don't want to hold on to
        // some arbitarily sized memory for who knows how long
        self.state = State::Loading(loading_animation(now));
        self.next = AutoState::Idle;

        load_video(previous.clone(), |video| {
            ManagerMessage::Video(false, video)
        })
        .map(Message::Player)
    }

    fn collection_add(&mut self) -> Task<Message> {
        todo!()
    }

    fn collection_favorite(&mut self) -> Task<Message> {
        todo!()
    }

    fn video_config(&mut self) -> Task<Message> {
        todo!()
    }

    fn video_comment(&mut self) -> Task<Message> {
        todo!()
    }

    pub fn action(&mut self, action: PlayerAction, now: Instant) -> Task<Message> {
        match action {
            PlayerAction::PlayToggle => self.play_toggle(),
            PlayerAction::PlayNext => self.play_next(now),
            PlayerAction::PlayPrevious => self.play_previous(now),
            PlayerAction::FullscreenToggle => self.fullscreen_toggle(),
            PlayerAction::FullscreenExit => self.fullscreen_exit(),
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
            PlayerAction::Favorite => self.collection_favorite(),
            PlayerAction::VideoConfig => self.video_config(),
            PlayerAction::VideoComment => self.video_comment(),
        }
    }
}

fn load_video<Message: 'static + MaybeSend>(
    item: PlayItem,
    f: impl FnOnce(Arc<Player>) -> Message + 'static + MaybeSend,
) -> Task<Message> {
    let url = url::Url::from_file_path(item.path.canonicalize().unwrap())
        .expect("File path should be validated");

    Task::perform(
        tokio::task::spawn_blocking(move || {
            let mut video = Video::new(&url).unwrap();
            video.set_paused(true);
            let position = video.position().as_secs_f64();
            let duration = video.duration().as_secs_f64();

            Arc::new(Player {
                item,
                video,
                thumbnails: vec![],
                position,
                is_dragging: false,
                duration,
            })
        }),
        move |res| f(res.unwrap()),
    )
}

fn handle_clicks(click: MouseClick) -> ManagerMessage {
    match click.button {
        Button::Left if matches!(click.kind, Kind::Single) => ManagerMessage::TogglePlay,
        Button::Left if matches!(click.kind, Kind::Double) => ManagerMessage::ToggleFullscreen,
        Button::Right => ManagerMessage::Config,
        _ => ManagerMessage::None,
    }
}

fn loading_animation(now: Instant) -> Animation<bool> {
    Animation::new(false)
        .easing(Easing::EaseInOut)
        .duration(time::Duration::from_millis(1500))
        .repeat_forever()
        .go(true, now)
}

fn loading_svg(animation: &Animation<bool>, now: Instant) -> Svg<'static> {
    use iced::{Radians, Rotation};
    let rotation = animation.interpolate(0.0, std::f32::consts::TAU, now);
    let rotation = Rotation::Floating(Radians(rotation));

    utils::loading_svg().rotation(rotation)
}
