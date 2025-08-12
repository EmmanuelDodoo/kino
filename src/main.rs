#![allow(dead_code, unused_imports)]
use iced::{
    Element, Font, Length, Point, Task,
    alignment::{Horizontal, Vertical},
    font,
    widget::{
        Space, center, column, container, horizontal_space, image, mouse_area, row, slider, stack,
        text, vertical_space,
    },
    window,
};
use iced_video_player::{
    Button, Icon, KeyPress, Kind, Modifiers, MouseClick, Position, Video, VideoPlayer, key,
};
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod app;
mod error;
mod utils;
mod widgets;
use utils::icons::{self, text_button};

fn _test_main() {
    // fn main() {
    let temp = utils::ThumbnailGenerator::new("assets/test1.mp4", 500, 31, 8);

    let total = temp.duration;
    dbg!(total);
    let unit = (total * 25) / 100;

    for i in 1..4 {
        let time = unit * i;
        temp.generate(time);
        dbg!(i);
    }
}

// fn test_main() -> iced::Result {
fn main() -> iced::Result {
    // iced::run(app::App::update, app::App::view)
    iced::application(PlayGround::boot, PlayGround::update, PlayGround::view).run()
}

#[derive(Debug, Clone)]
enum PGMessage {
    WindowId(Option<window::Id>),
    FontLoad(Result<(), font::Error>),
    SeekRelease,
    Seek(f64),
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
struct PlayGround {
    position: f64,
    show_controls: bool,
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

impl PlayGround {
    const WIDTH: f32 = 150.0;

    fn boot() -> (Self, Task<PGMessage>) {
        let path = PathBuf::from("assets/test3.mp4");
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
                |res| PGMessage::ThumbnailsReady(res.unwrap()),
            )
        };
        let load_font = font::load(icons::LUCIDE_BYTES).map(PGMessage::FontLoad);
        let load_id = window::get_oldest().map(PGMessage::WindowId);

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

    fn update(&mut self, message: PGMessage) -> Task<PGMessage> {
        match message {
            PGMessage::WindowId(id) => {
                self.window_id = id;
                Task::none()
            }
            PGMessage::None => Task::none(),
            PGMessage::FontLoad(Err(error)) => {
                eprintln!("{error:?}");
                Task::none()
            }
            PGMessage::FontLoad(_) => Task::none(),
            PGMessage::EndOfStream => Task::none(),
            PGMessage::NewFrame => {
                if !self.is_dragging {
                    self.position = self.video.position().as_secs_f64();
                }
                Task::none()
            }
            PGMessage::SeekRelease => {
                self.is_dragging = false;
                self.video
                    .seek(Duration::from_secs_f64(self.position.max(0.0)), false)
                    .unwrap();

                self.video.set_paused(false);
                Task::none()
            }
            PGMessage::Seek(pos) => {
                self.position = pos.max(0.0);
                self.is_dragging = true;
                self.video.set_paused(true);
                Task::none()
            }
            PGMessage::TogglePlay => {
                self.video.set_paused(!self.video.paused());
                Task::none()
            }
            PGMessage::ThumbnailsReady(Err(_err)) => {
                todo!("Set up error toasts here");
            }
            PGMessage::ThumbnailsReady(Ok(thumbnails)) => {
                self.thumbnails = thumbnails;
                Task::none()
            }
            PGMessage::ChangeVolume(volume) => {
                self.volume = volume;
                self.video.set_volume(volume);
                Task::none()
            }
            PGMessage::ToggleMute => {
                let mute = !self.video.muted();
                self.video.set_muted(mute);

                if mute {
                    self.volume = 0.0
                } else {
                    self.volume = self.video.volume()
                }
                Task::none()
            }
            PGMessage::SeekBack(shift) => {
                self.is_dragging = false;
                let mult = if shift { 2.0 } else { 1.0 };
                self.position = (self.position - (self.seek_amount() * mult)).max(0.0);
                self.video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .unwrap();
                Task::none()
            }
            PGMessage::SeekFront(shift) => {
                self.is_dragging = false;
                let duration = self.video.duration().as_secs_f64();
                let mult = if shift { 2.0 } else { 1.0 };
                self.position = (self.position + (self.seek_amount() * mult)).min(duration);
                self.video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .unwrap();
                Task::none()
            }
            PGMessage::IncrVolume => {
                self.volume = (self.volume + self.volume_amount()).min(1.0);
                self.video.set_volume(self.volume);
                Task::none()
            }
            PGMessage::DecrVolume => {
                self.volume = (self.volume - self.volume_amount()).max(0.0);
                self.video.set_volume(self.volume);
                Task::none()
            }
            PGMessage::IncrSpeed => {
                self.speed += self.speed_amount();
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PGMessage::DecrSpeed => {
                self.speed -= self.speed_amount();
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PGMessage::ResetSpeed => {
                self.speed = 1.0;
                self.video.set_speed(self.speed).unwrap();
                Task::none()
            }
            PGMessage::CursorExit => {
                if self.is_fullscreen {
                    self.show_controls = false;
                }
                Task::none()
            }
            PGMessage::CursorEnter => {
                self.show_controls = true;
                Task::none()
            }
            PGMessage::ToggleFullscreen => {
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
            PGMessage::ExitFullscreen => {
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
            PGMessage::PreviousScreen => Task::none(),
            PGMessage::AddCollection => Task::none(),
            PGMessage::Config => Task::none(),
            PGMessage::ToggleSubtitles => Task::none(),
            PGMessage::PlayNext => Task::none(),
            PGMessage::PlayPrevious => Task::none(),
            PGMessage::Favorite => Task::none(),
            PGMessage::Comment => Task::none(),
        }
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

    fn play_btn(&self) -> (char, PGMessage) {
        if self.video.paused() {
            (icons::PLAY, PGMessage::TogglePlay)
        } else if self.video.eos() {
            (icons::REPLAY, PGMessage::TogglePlay)
        } else {
            (icons::PAUSE, PGMessage::TogglePlay)
        }
    }

    fn top(&self) -> Element<'_, PGMessage> {
        let title = text(self.name());
        let options = column!(
            row!(
                text_button(icons::ADD_COLLECTION).on_press(PGMessage::AddCollection),
                text_button(icons::VIDEO_CONFIG).on_press(PGMessage::Config)
            )
            .spacing(6.0)
            .align_y(Vertical::Center)
        )
        .align_x(Horizontal::Right)
        .width(Self::WIDTH);
        let back = container(text_button(icons::BACK).on_press(PGMessage::PreviousScreen))
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .width(Self::WIDTH);

        let content = row!(back, horizontal_space(), title, horizontal_space(), options)
            .width(Length::Fill)
            .align_y(Vertical::Center);

        let content: Element<'_, PGMessage> = if self.show_controls {
            content.into()
        } else {
            horizontal_space().height(35).into()
        };

        let content = mouse_area(content)
            .on_exit(PGMessage::CursorExit)
            .on_enter(PGMessage::CursorEnter);

        content.into()
    }

    fn media_controls(&self) -> Element<'_, PGMessage> {
        let left = {
            let volume = slider(0.0..=1.0, self.volume, PGMessage::ChangeVolume)
                .step(0.05)
                .shift_step(0.1)
                .width(125.0);
            row!(
                text_button(icons::SUBTITLES).on_press(PGMessage::ToggleSubtitles),
                text_button(if self.video.muted() {
                    icons::MUTE
                } else {
                    icons::VOLUME
                })
                .on_press(PGMessage::ToggleMute),
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
                    .on_press_maybe(self.has_previous.then_some(PGMessage::PlayPrevious)),
                text_button(icons::SEEK_BACK).on_press(PGMessage::SeekBack(false)),
                text_button(play).on_press(message),
                text_button(icons::SEEK_FRONT).on_press(PGMessage::SeekFront(false)),
                text_button(icons::NEXT_VIDEO)
                    .on_press_maybe(self.has_next.then_some(PGMessage::PlayNext))
            )
            .spacing(2.0)
            .align_y(Vertical::Center)
        };

        let right = column!(
            row!(
                text_button(icons::FAVORITE).on_press(PGMessage::Favorite),
                text_button(icons::COMMENT).on_press(PGMessage::Comment),
                text_button(if self.is_fullscreen {
                    icons::MINIMIZE
                } else {
                    icons::MAXIMIZE
                })
                .on_press(PGMessage::ToggleFullscreen)
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
                PGMessage::Seek,
                self.thumbnails.clone(),
                Font::default(),
                duration,
            )
            .step(0.1)
            .on_release(PGMessage::SeekRelease);

            row!(text(spent), slider, text(total))
                .spacing(20.0)
                .align_y(Vertical::Center)
                .width(Length::Fill)
        };

        let content = column!(timeline, content, vertical_space().height(8.0))
            .spacing(8)
            .width(Length::Fill);

        let content: Element<'_, PGMessage> = if self.show_controls {
            content.into()
        } else {
            horizontal_space().height(75).into()
        };

        let content = mouse_area(content)
            .on_exit(PGMessage::CursorExit)
            .on_enter(PGMessage::CursorEnter);

        content.into()
    }

    fn video_elem(&self) -> Element<'_, PGMessage> {
        let play = self.play_btn();
        let video = container(
            VideoPlayer::new(&self.video)
                .width(Length::Fill)
                .height(Length::Fill)
                .play_icon(video_icon(play.0), play.1)
                .next_icon(video_icon(icons::NEXT_VIDEO), PGMessage::PlayNext)
                .previous_icon(video_icon(icons::PREVIOUS_VIDEO), PGMessage::PlayPrevious)
                .fullscreen_icon(
                    video_icon(if self.is_fullscreen {
                        icons::MINIMIZE
                    } else {
                        icons::MAXIMIZE
                    }),
                    PGMessage::ToggleFullscreen,
                )
                .on_keypress(handle_keypress)
                .on_click(handle_clicks)
                .enable_overlay(self.is_fullscreen && !self.show_controls)
                .content_fit(iced::ContentFit::Contain)
                .on_end_of_stream(PGMessage::EndOfStream)
                .on_new_frame(PGMessage::NewFrame),
        )
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

        video.into()
    }

    fn view(&self) -> Element<'_, PGMessage> {
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
        size: Some(36.0.into()),
        color: None,
    }
}

fn handle_keypress(keypress: KeyPress) -> PGMessage {
    use key::{Key, Named};

    match keypress.key {
        Key::Named(Named::Space) => PGMessage::TogglePlay,
        Key::Named(Named::Enter) => PGMessage::ToggleFullscreen,
        Key::Named(Named::Escape) => PGMessage::ExitFullscreen,
        Key::Named(Named::ArrowLeft) => PGMessage::SeekBack(keypress.modifiers.shift()),
        Key::Named(Named::ArrowRight) => PGMessage::SeekFront(keypress.modifiers.shift()),
        Key::Named(Named::ArrowUp) => PGMessage::IncrVolume,
        Key::Named(Named::ArrowDown) => PGMessage::DecrVolume,
        Key::Character(char) if char.as_str() == "f" => PGMessage::ToggleFullscreen,
        Key::Character(char) if char.as_str() == "c" => PGMessage::IncrSpeed,
        Key::Character(char) if char.as_str() == "x" => PGMessage::DecrSpeed,
        Key::Character(char) if char.as_str() == "z" => PGMessage::ResetSpeed,
        _ => PGMessage::None,
    }
}

fn handle_clicks(click: MouseClick) -> PGMessage {
    match click.button {
        Button::Left if matches!(click.kind, Kind::Single) => PGMessage::TogglePlay,
        Button::Left if matches!(click.kind, Kind::Double) => PGMessage::ToggleFullscreen,
        Button::Right => PGMessage::Config,
        _ => PGMessage::None,
    }
}
