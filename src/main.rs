#![allow(dead_code, unused_imports)]
use iced::{
    Element, Font, Length, Point, Size, Subscription, Task,
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
mod player;
pub mod utils;
pub mod widgets;
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
    iced::run(app::App::update, app::App::view)
    // iced::application(PlayGround::boot, PlayGround::update, PlayGround::view)
    //     .subscription(PlayGround::subscriptions)
    //     .run()
}
