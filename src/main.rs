#![allow(dead_code, unused_imports)]
use iced::{
    Element, Font, Length, Padding, Point, Size, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    border::{Border, Radius},
    font,
    widget::{
        button, center, column, container, horizontal_rule, horizontal_space, image, mouse_area,
        pick_list, row, scrollable, slider, stack, text, text_input, vertical_rule, vertical_space,
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
mod home;
mod player;
mod utils;
mod widgets;

use player::{Player, PlayerMessage};
use utils::empty;
use utils::filter::*;
use utils::icons::{self, text_button};
use utils::typo;
use utils::typo::*;

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
    iced::application(home::Home::boot, home::Home::update, home::Home::view)
        .subscription(home::Home::subscription)
        .run()
}
