// #![allow(dead_code, unused_imports)]
use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{column, container, text},
};

mod app;
mod error;
mod utils;
mod widgets;

fn _test_main() {
    // fn main() {
    let temp = utils::ThumbnailGenerator::new(".media/test1.mp4", 500, 31, 8);

    let total = temp.duration;
    dbg!(total);
    let unit = (total * 25) / 100;

    for i in 1..4 {
        let time = unit * i;
        temp.generate(time, &i.to_string());
        dbg!(i);
    }
}

// fn test_main() -> iced::Result {
fn main() -> iced::Result {
    // iced::run(app::App::update, app::App::view)
    iced::run(PlayGround::update, PlayGround::view)
}

#[derive(Debug, Clone)]
enum Message {
    SeekRelease,
    Hover(f64),
    Seek(f64),
}

#[derive(Debug)]
struct PlayGround {
    position: f64,
}

impl PlayGround {
    fn new() -> Self {
        Self { position: 0.0 }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SeekRelease => {
                println!("Released");
            }
            Message::Seek(pos) => {
                self.position = pos;
            }
            Message::Hover(pos) => {
                println!("Hovered at: {pos:.02}");
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let text = text(format!("Position, {:.02}", self.position));

        let content = widgets::slider::VideoSlider::new(0.0..=10.0, self.position, Message::Seek)
            .step(0.1)
            .on_hover(Message::Hover)
            .on_release(Message::SeekRelease);

        let content = column!(text, content).spacing(50.0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

impl Default for PlayGround {
    fn default() -> Self {
        Self::new()
    }
}
