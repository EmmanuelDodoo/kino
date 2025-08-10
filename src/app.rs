#![allow(unused_imports, dead_code)]
use iced::{
    Element, Length,
    widget::{Button, Container, Slider, Text, column, image, row, text_input},
};
use iced_video_player::{Video, VideoPlayer};
use std::path::PathBuf;
use std::time::Duration;

use crate::error::*;
use crate::utils;

#[derive(Clone, Debug)]
pub enum Message {
    TogglePause,
    Input(String),
    InputSubmit,
    ToggleLoop,
    Seek(f64),
    SeekRelease,
    EndOfStream,
    NewFrame,
}

pub struct App {
    video: Option<Video>,
    path: PathBuf,
    input: Option<String>,
    position: f64,
    dragging: bool,
    thumbnails: Vec<image::Handle>,
}

impl Default for App {
    fn default() -> Self {
        let path = PathBuf::from(".media/test1.mp4");

        App {
            video: None,
            path,
            input: None,
            position: 0.0,
            thumbnails: vec![],
            dragging: false,
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Input(input) => {
                self.input = Some(input);
            }
            Message::InputSubmit => {
                let Some(input) = self.input.take() else {
                    return;
                };

                self.path = PathBuf::from(input);

                let thumbnails = {
                    let generator = utils::ThumbnailGenerator::new(&self.path, 500, 31, 8);
                    let duration = generator.duration;
                    let fraction = if duration.seconds() < 60 { 10 } else { 1 };
                    let len = 100 / fraction;

                    let unit = (duration * fraction) / 100;

                    (1..len)
                        .map(|i| generator.generate(unit * i, &i.to_string()))
                        .collect()
                };

                let mut video = Video::new(
                    &url::Url::from_file_path(self.path.canonicalize().unwrap()).unwrap(),
                )
                .unwrap();
                video.set_gamma(1.5);
                video.set_paused(true);

                self.thumbnails = thumbnails;
                self.video = Some(video);
                self.position = 0.0;
            }
            Message::TogglePause => {
                if let Some(video) = self.video.as_mut() {
                    video.set_paused(!video.paused());
                }
            }
            Message::ToggleLoop => {
                if let Some(video) = self.video.as_mut() {
                    video.set_looping(!video.looping());
                }
            }
            Message::Seek(secs) => {
                let Some(video) = self.video.as_mut() else {
                    return;
                };
                self.dragging = true;
                self.position = secs;
                video.set_paused(true);
            }
            Message::SeekRelease => {
                let Some(video) = self.video.as_mut() else {
                    return;
                };
                self.dragging = false;
                video
                    .seek(Duration::from_secs_f64(self.position), false)
                    .expect("seek");
                video.set_paused(false);
            }
            Message::EndOfStream => {
                println!("end of stream");
            }
            Message::NewFrame => {
                let Some(video) = self.video.as_mut() else {
                    return;
                };
                if !self.dragging {
                    self.position = video.position().as_secs_f64();
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let path = {
            let path = match self.input.as_ref() {
                Some(input) => input,
                None => self.path.to_str().unwrap_or_default(),
            };

            text_input("Video path", path)
                .on_input(Message::Input)
                .on_submit(Message::InputSubmit)
        };

        match &self.video {
            Some(video) => {
                let slider = Container::new(
                    Slider::new(
                        0.0..=video.duration().as_secs_f64(),
                        self.position,
                        Message::Seek,
                    )
                    .step(0.1)
                    .on_release(Message::SeekRelease),
                )
                .padding(iced::Padding::new(5.0).left(10.0).right(10.0));

                let controls = {
                    let play_btn =
                        Button::new(Text::new(if video.paused() { "Play" } else { "Pause" }))
                            .width(80.0)
                            .on_press(Message::TogglePause);

                    let loop_btn = Button::new(Text::new(if video.looping() {
                        "Disable Loop"
                    } else {
                        "Enable Loop"
                    }))
                    .width(120.0)
                    .on_press(Message::ToggleLoop);

                    let time = Text::new(format!(
                        "{}:{:02}s / {}:{:02}s",
                        self.position as u64 / 60,
                        self.position as u64 % 60,
                        video.duration().as_secs() / 60,
                        video.duration().as_secs() % 60,
                    ))
                    .width(iced::Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right);

                    row!(play_btn, loop_btn, time)
                        .spacing(5)
                        .align_y(iced::alignment::Vertical::Center)
                        .padding(iced::Padding::new(10.0).top(0.0))
                };

                let video = Container::new(
                    VideoPlayer::new(video)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Contain)
                        .on_end_of_stream(Message::EndOfStream)
                        .on_new_frame(Message::NewFrame),
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

                let content = column!(path, video, slider, controls);

                content.into()
            }
            None => path.into(),
        }
    }
}
