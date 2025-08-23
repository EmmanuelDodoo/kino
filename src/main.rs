#![allow(dead_code, unused_imports)]
use iced::{
    Color, ContentFit, Element, Event, Font, Length, Padding, Point, Rectangle, Shadow, Size,
    Subscription, Task, Theme, Vector,
    advanced::{
        self, Widget, layout, mouse, overlay,
        widget::{operation, tree},
    },
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    border::{Border, Radius},
    color, font,
    time::Instant,
    widget::{
        bottom, bottom_center, button, center, center_x, center_y, column, container, float, grid,
        horizontal_rule, horizontal_space, image, mouse_area, pick_list, row, scrollable, slider,
        stack, text, text_input, vertical_rule, vertical_space,
    },
    window,
};

mod app;
mod error;
mod home;
mod player;
pub mod utils;
mod video;
mod widgets;

use player::{Player, PlayerMessage};
use utils::filter;
use utils::filter::*;
use utils::icons::*;
use utils::typo;
use utils::typo::*;
use utils::{Sort, SortKind, empty};
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
fn main() -> iced::Result {
    // iced::run(app::App::update, app::App::view)
    // iced::application(Player::boot, Player::update, Player::view)
    //     .subscription(Player::subscriptions)
    //     .run()
    iced::application::timed(
        home::Home::boot,
        home::Home::update,
        home::Home::subscription,
        home::Home::view,
    )
    .window_size(Size::new(1200.0, 750.0))
    .run()

    // iced::application::timed(
    //     Movies::boot,
    //     Movies::update,
    //     Movies::subscription,
    //     Movies::view,
    // )
    // .run()
}
