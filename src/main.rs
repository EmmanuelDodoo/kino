#![allow(dead_code, unused_imports)]
use iced::{
    Color, ContentFit, Element, Event, Font, Length, Padding, Point, Radians, Rectangle, Rotation,
    Shadow, Size, Subscription, Task, Theme, Vector,
    advanced::{
        self, Widget, layout, mouse, overlay,
        widget::{operation, tree},
    },
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    border::{self, Border, Radius},
    color, font, padding,
    time::{Instant, Duration},
    widget::{
        Space, bottom, bottom_center, button, center, center_x, center_y, column, container, float,
        grid, image, mouse_area, pick_list, row, scrollable, slider, stack, text, text_input,
        tooltip,
    },
    window,
};
use std::{collections::HashMap, ops::Deref};

mod app;
mod db;
mod error;
mod home;
mod models;
mod player;
pub mod utils;
mod widgets;

use app::App;
// use player::{Player, PlayerMessage};
use utils::filter;
use utils::filter::*;
use utils::icons::*;
use utils::sort;
use utils::sort::*;
use utils::typo;
use utils::typo::*;
use utils::{Layout, Sort, SortKind, empty};
use widgets::*;

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
#[rustfmt::skip]
fn main() -> iced::Result {
    // iced::run(app::App::update, app::App::view)


    iced::application::timed(
        App::boot, 
        App::update, 
        App::subscription,
        App::view
    )
        .exit_on_close_request(false)
        .theme(App::theme)

    // iced::application::timed(
    //     Playground::boot,
    //     Playground::update,
    //     Playground::subscription,
    //     Playground::view,
    // )
    //     .theme(Playground::theme)

    .window_size(Size::new(1200.0, 750.0))
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    FontLoad(Result<(), iced::font::Error>),
    None,
}

struct Playground {
    now: Instant,
    animation: Animation<bool>,
    state: bool,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let fonts = utils::load_fonts().map(Message::FontLoad);

        let now = Instant::now();
        let mut animation = Animation::new(false)
            .duration(Duration::from_millis(1500))
            .easing(Easing::EaseInOut)
            .repeat_forever();
        animation.go_mut(true, now);

        let new = Self {
            now,
            animation,
            state: false,
        };

        (new, fonts)
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => Task::none(),
            Message::FontLoad(Ok(_)) => Task::none(),
            Message::FontLoad(Err(error)) => {
                eprintln!("{error:?}");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let rotation = self.animation.interpolate(0.2, std::f32::consts::TAU, self.now);
        let rotation = Rotation::Floating(Radians(rotation));

        let svg = utils::loading_svg().rotation(rotation);

        let content = column!(svg).spacing(20);
        let content = center(content);

        content.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(|_| Message::None)
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Nightfly)
    }
}
