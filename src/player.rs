use iced::{
    Element, Font, Length, Size, Subscription, Task,
    alignment::{Horizontal, Vertical},
    font,
    widget::{
        column, container, horizontal_space, image, mouse_area, row, slider, stack, text,
        vertical_space,
    },
    window,
};
use iced_video_player::{Button, Icon, KeyPress, Kind, MouseClick, Video, VideoPlayer, key};
use std::path::PathBuf;
use std::time::Duration;

use crate::utils::{
    self,
    icons::{self, text_button},
};
use crate::widgets;

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    WindowId(Option<window::Id>),
    FontLoad(Result<(), font::Error>),
    SeekRelease,
    Seek(f64),
    Resize((window::Id, Size)),
    ThumbnailsReady(Result<Vec<image::Handle>, ()>),
    ChangeVolume(f64),
    IncrVolume,
    DecrVolume,
    IncrSpeed,
    DecrSpeed,
    ResetSpeed,
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
    ExitFullscreen,
    EndOfStream,
    NewFrame,
    None,
}

#[derive(Debug)]
pub struct Player {
    position: f64,
    show_controls: bool,
    maximised: bool,
    is_fullscreen: bool,
    volume: f64,
    speed: f64,
    has_previous: bool,
    has_next: bool,
    path: PathBuf,
    video: Video,
    thumbnails: Vec<image::Handle>,
    is_dragging: bool,
    window_id: Option<window::Id>,
}

impl Player {
    const WIDTH: f32 = 150.0;

    pub fn boot() -> (Self, Task<PlayerMessage>) {
        let path = PathBuf::from("assets/test.mkv");
        // A better approach would carry only the name and some way to identify the video.
        let path_ref = path.clone();
        let mut video =
            Video::new(&url::Url::from_file_path(path.canonicalize().unwrap()).unwrap()).unwrap();
        video.set_gamma(1.5);

        let thumbnails_task = {
            let duration = video.duration().as_secs_f64();
            let interval = 100u32;
            let num = duration as u32 / interval;
            let (width, height) = video.size();
            Task::perform(
                tokio::task::spawn_blocking(move || {
                    let generator = utils::ThumbnailGenerator::new(&path_ref, width, height, 8);

                    Ok((1..=num)
                        .map(|i| {
                            generator.generate(gstreamer::ClockTime::from_seconds_f64(
                                duration * (i as f64 / num as f64),
                            ))
                        })
                        .collect())
                }),
                |res| PlayerMessage::ThumbnailsReady(res.unwrap()),
            )
        };
        // todo: belongs in main app
        let load_font = font::load(icons::LUCIDE_BYTES).map(PlayerMessage::FontLoad);
        let load_id = window::get_oldest().map(PlayerMessage::WindowId);

        let tasks = Task::batch(vec![thumbnails_task, load_font, load_id]);

        (Self::new(video, path), tasks)
    }

    fn new(video: Video, path: PathBuf) -> Self {
        let is_fullscreen = false;
        Self {
            position: video.position().as_secs_f64(),
            volume: video.volume(),
            speed: video.speed(),
            show_controls: !is_fullscreen,
            maximised: false,
            is_fullscreen,
            has_previous: false,
            has_next: false,
            is_dragging: false,
            video,
            path,
            thumbnails: vec![],
            window_id: None,
        }
    }

    pub fn update(&mut self, message: PlayerMessage) -> Task<PlayerMessage> {
        match message {
            PlayerMessage::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
            PlayerMessage::Resize((id, size)) => {
                if Some(id) == self.window_id {
                    let maximised = Size::new(1000., 1000.);
                    self.maximised =
                        size.width >= maximised.width && size.height >= maximised.height;
                    self.show_controls = !self.maximised;
                }
                Task::none()
            }
            PlayerMessage::None => Task::none(),
            PlayerMessage::FontLoad(Err(error)) => {
                eprintln!("{error:?}");
                Task::none()
            }
            PlayerMessage::FontLoad(_) => Task::none(),
            PlayerMessage::EndOfStream => {
                todo!("Send message to main with some video stats");
            }
            PlayerMessage::NewFrame => {
                if !self.is_dragging {
                    self.position = self.video.position().as_secs_f64();
                }
                Task::none()
            }
            PlayerMessage::SeekRelease => {
                self.is_dragging = false;
                self.video
                    .seek(Duration::from_secs_f64(self.position.max(0.0)), false)
                    .unwrap();

                self.video.set_paused(false);
                Task::none()
            }
            PlayerMessage::Seek(pos) => {
                self.position = pos.max(0.0);
                self.is_dragging = true;
                self.video.set_paused(true);
                Task::none()
            }
            PlayerMessage::TogglePlay => {
                self.video.set_paused(!self.video.paused());
                Task::none()
            }
            PlayerMessage::ThumbnailsReady(Err(_err)) => {
                todo!("Set up error toasts here");
            }
            PlayerMessage::ThumbnailsReady(Ok(thumbnails)) => {
                self.thumbnails = thumbnails;
                Task::none()
            }
            PlayerMessage::ChangeVolume(volume) => {
                self.volume = volume;
                self.video.set_volume(volume);
                Task::none()
            }
            PlayerMessage::ToggleMute => {
                let mute = !self.video.muted();
                self.video.set_muted(mute);

                if mute {
                    self.volume = 0.0
                } else {
                    self.volume = self.video.volume()
                }
                Task::none()
            }
            PlayerMessage::SeekBack(shift) => {
                self.is_dragging = false;
                let mult = if shift { 2.0 } else { 1.0 };
                self.position = (self.position - (self.seek_amount() * mult)).max(0.0);
                self.video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .unwrap();
                Task::none()
            }
            PlayerMessage::SeekFront(shift) => {
                self.is_dragging = false;
                let duration = self.video.duration().as_secs_f64();
                let mult = if shift { 2.0 } else { 1.0 };
                self.position = (self.position + (self.seek_amount() * mult)).min(duration);
                self.video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .unwrap();
                Task::none()
            }
            PlayerMessage::IncrVolume => {
                self.volume = (self.volume + self.volume_amount()).min(1.0);
                self.video.set_volume(self.volume);
                Task::none()
            }
            PlayerMessage::DecrVolume => {
                self.volume = (self.volume - self.volume_amount()).max(0.0);
                self.video.set_volume(self.volume);
                Task::none()
            }
            PlayerMessage::IncrSpeed => {
                self.speed += self.speed_amount();
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PlayerMessage::DecrSpeed => {
                self.speed -= self.speed_amount();
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PlayerMessage::ResetSpeed => {
                self.speed = 1.0;
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PlayerMessage::CursorExit => {
                if self.is_fullscreen || self.maximised {
                    self.show_controls = false;
                }
                Task::none()
            }
            PlayerMessage::CursorEnter => {
                self.show_controls = true;
                Task::none()
            }
            PlayerMessage::ToggleFullscreen => {
                self.show_controls = self.is_fullscreen;
                self.is_fullscreen = !self.is_fullscreen;
                let fullscreen = self.is_fullscreen;
                iced::window::get_latest()
                    .and_then(move |id| {
                        iced::window::set_mode::<()>(
                            id,
                            if fullscreen {
                                iced::window::Mode::Fullscreen
                            } else {
                                iced::window::Mode::Windowed
                            },
                        )
                    })
                    .discard()
            }
            PlayerMessage::ExitFullscreen => {
                self.show_controls = true;
                self.is_fullscreen = false;
                let fullscreen = self.is_fullscreen;
                iced::window::get_latest()
                    .and_then(move |id| {
                        iced::window::set_mode::<()>(
                            id,
                            if fullscreen {
                                iced::window::Mode::Fullscreen
                            } else {
                                iced::window::Mode::Windowed
                            },
                        )
                    })
                    .discard()
            }
            PlayerMessage::PreviousScreen => {
                todo!("send message to main with video stats");
            }
            PlayerMessage::AddCollection => Task::none(),
            PlayerMessage::Config => Task::none(),
            PlayerMessage::ToggleSubtitles => Task::none(),
            PlayerMessage::PlayNext => Task::none(),
            PlayerMessage::PlayPrevious => Task::none(),
            PlayerMessage::Favorite => Task::none(),
            PlayerMessage::Comment => Task::none(),
        }
    }

    pub fn subscriptions(&self) -> Subscription<PlayerMessage> {
        window::resize_events().map(PlayerMessage::Resize)
    }

    fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    fn volume_amount(&self) -> f64 {
        0.05
    }

    fn seek_amount(&self) -> f64 {
        10.0
    }

    fn speed_amount(&self) -> f64 {
        0.1
    }

    fn play_btn(&self) -> (char, PlayerMessage) {
        if self.video.paused() {
            (icons::PLAY, PlayerMessage::TogglePlay)
        } else if self.video.eos() {
            (icons::REPLAY, PlayerMessage::TogglePlay)
        } else {
            (icons::PAUSE, PlayerMessage::TogglePlay)
        }
    }

    fn top(&self) -> Element<'_, PlayerMessage> {
        let title = text(self.name());
        let options = column!(
            row!(
                text_button(icons::ADD_COLLECTION).on_press(PlayerMessage::AddCollection),
                text_button(icons::VIDEO_CONFIG).on_press(PlayerMessage::Config)
            )
            .spacing(6.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);
        let back = container(text_button(icons::BACK).on_press(PlayerMessage::PreviousScreen))
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .width(Self::WIDTH);

        let content = row!(back, horizontal_space(), title, horizontal_space(), options)
            .width(Length::Fill)
            .align_y(Vertical::Center);

        let content: Element<'_, PlayerMessage> = if self.show_controls {
            content.into()
        } else {
            horizontal_space().height(35).into()
        };

        let content = mouse_area(content)
            .on_exit(PlayerMessage::CursorExit)
            .on_enter(PlayerMessage::CursorEnter);

        content.into()
    }

    fn media_controls(&self) -> Element<'_, PlayerMessage> {
        let left = {
            let volume = slider(0.0..=1.0, self.volume, PlayerMessage::ChangeVolume)
                .step(0.05)
                .shift_step(0.1)
                .width(125.0);
            row!(
                text_button(icons::SUBTITLES).on_press(PlayerMessage::ToggleSubtitles),
                text_button(if self.video.muted() {
                    icons::MUTE
                } else {
                    icons::VOLUME
                })
                .on_press(PlayerMessage::ToggleMute),
                volume
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        }
        .width(Self::WIDTH);

        let middle = {
            let (play, message) = self.play_btn();

            row!(
                text_button(icons::PREVIOUS_VIDEO)
                    .on_press_maybe(self.has_previous.then_some(PlayerMessage::PlayPrevious)),
                text_button(icons::SEEK_BACK).on_press(PlayerMessage::SeekBack(false)),
                text_button(play).on_press(message),
                text_button(icons::SEEK_FRONT).on_press(PlayerMessage::SeekFront(false)),
                text_button(icons::NEXT_VIDEO)
                    .on_press_maybe(self.has_next.then_some(PlayerMessage::PlayNext))
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        };

        let right = column!(
            row!(
                text_button(icons::FAVORITE).on_press(PlayerMessage::Favorite),
                text_button(icons::COMMENT).on_press(PlayerMessage::Comment),
                text_button(if self.is_fullscreen {
                    icons::MINIMIZE
                } else {
                    icons::MAXIMIZE
                })
                .on_press(PlayerMessage::ToggleFullscreen)
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);

        let content = row!(left, horizontal_space(), middle, horizontal_space(), right)
            .width(Length::Fill)
            .align_y(Vertical::Center);

        let timeline = {
            let duration = self.video.duration();
            let spent = format!(
                "{:02}:{:02}:{:02}",
                self.position as u64 / 3600,
                self.position as u64 / 60,
                self.position as u64 % 60,
            );
            let total = format!(
                "{:02}:{:02}:{:02}",
                duration.as_secs() as u64 / 3600,
                duration.as_secs() as u64 / 60,
                duration.as_secs() as u64 % 60,
            );

            let slider = widgets::slider::VideoSlider::new(
                0.0..=duration.as_secs_f64(),
                self.position,
                PlayerMessage::Seek,
                self.thumbnails.clone(),
                Font::default(),
                duration,
            )
            .step(0.1)
            .on_release(PlayerMessage::SeekRelease);

            row!(text(spent), slider, text(total))
                .spacing(20.0)
                .align_y(Vertical::Center)
                .width(Length::Fill)
        };

        let content = column!(timeline, content, vertical_space().height(8.0))
            .spacing(8)
            .width(Length::Fill);

        let content: Element<'_, PlayerMessage> = if self.show_controls {
            content.into()
        } else {
            horizontal_space().height(75).into()
        };

        let content = mouse_area(content)
            .on_exit(PlayerMessage::CursorExit)
            .on_enter(PlayerMessage::CursorEnter);

        content.into()
    }

    fn video_elem(&self) -> Element<'_, PlayerMessage> {
        let play = self.play_btn();
        let fullscreen = video_icon(if self.is_fullscreen {
            icons::MINIMIZE
        } else {
            icons::MAXIMIZE
        });
        let fullscreen = Icon {
            size: Some(24.0.into()),
            ..fullscreen
        };
        let video = container(
            VideoPlayer::new(&self.video)
                .width(Length::Fill)
                .height(Length::Fill)
                .play_icon(video_icon(play.0), play.1)
                .next_icon(video_icon(icons::NEXT_VIDEO), PlayerMessage::PlayNext)
                .previous_icon(
                    video_icon(icons::PREVIOUS_VIDEO),
                    PlayerMessage::PlayPrevious,
                )
                .fullscreen_icon(fullscreen, PlayerMessage::ToggleFullscreen)
                .on_keypress(handle_keypress)
                .on_click(handle_clicks)
                .enable_overlay(self.is_fullscreen && !self.show_controls)
                .content_fit(iced::ContentFit::Contain)
                .on_end_of_stream(PlayerMessage::EndOfStream)
                .on_new_frame(PlayerMessage::NewFrame),
        )
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

        video.into()
    }

    pub fn view(&self) -> Element<'_, PlayerMessage> {
        let content = stack!(
            self.video_elem(),
            column!(self.top(), vertical_space(), self.media_controls())
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
}

fn video_icon(unicode: char) -> Icon<iced::Font> {
    Icon {
        code_point: unicode,
        font: icons::FONT,
        size: Some(52.0.into()),
        color: None,
    }
}

fn handle_keypress(keypress: KeyPress) -> PlayerMessage {
    use key::{Key, Named};

    match keypress.key {
        Key::Named(Named::Space) => PlayerMessage::TogglePlay,
        Key::Named(Named::Enter) => PlayerMessage::ToggleFullscreen,
        Key::Named(Named::Escape) => PlayerMessage::ExitFullscreen,
        Key::Named(Named::ArrowLeft) => PlayerMessage::SeekBack(keypress.modifiers.shift()),
        Key::Named(Named::ArrowRight) => PlayerMessage::SeekFront(keypress.modifiers.shift()),
        Key::Named(Named::ArrowUp) => PlayerMessage::IncrVolume,
        Key::Named(Named::ArrowDown) => PlayerMessage::DecrVolume,
        Key::Character(char) if char.as_str() == "f" => PlayerMessage::ToggleFullscreen,
        Key::Character(char) if char.as_str() == "c" => PlayerMessage::IncrSpeed,
        Key::Character(char) if char.as_str() == "x" => PlayerMessage::DecrSpeed,
        Key::Character(char) if char.as_str() == "z" => PlayerMessage::ResetSpeed,
        _ => PlayerMessage::None,
    }
}

fn handle_clicks(click: MouseClick) -> PlayerMessage {
    match click.button {
        Button::Left if matches!(click.kind, Kind::Single) => PlayerMessage::TogglePlay,
        Button::Left if matches!(click.kind, Kind::Double) => PlayerMessage::ToggleFullscreen,
        Button::Right => PlayerMessage::Config,
        _ => PlayerMessage::None,
    }
}
