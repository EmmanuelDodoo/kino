#![allow(dead_code, unused_imports)]
use iced::{
    Color, ContentFit, Element, Event, Font, Length, Padding, Point, Radians, Rectangle, Rotation,
    Shadow, Size, Subscription, Task, Theme, Vector,
    advanced::{self, Widget, layout, mouse, overlay, widget::tree},
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    border::{self, Border, Radius},
    color, font, padding,
    time::{Duration, Instant},
    widget::{
        self, Space, bottom, bottom_center, button, center, center_x, center_y, column, container,
        float, grid, image, markdown, mouse_area, operation, pick_list, rich_text, row, scrollable,
        slider, space, span, stack, text, text_input, tooltip,
    },
    window,
};

mod app;
mod db;
mod error;
mod home;
mod models;
mod player;
pub mod utils;
mod widgets;

use app::App;
use models::{ItemId, SearchItem};
use utils::filter;
use utils::filter::*;
use utils::icons;
use utils::icons::*;
use utils::sort;
use utils::sort::*;
use utils::typo;
use utils::typo::*;
use utils::{Layout, SearchFilter, Sort, SortKind, empty};
use widgets::*;

// fn _test_main() {
//     // fn main() {
//     let temp = utils::ThumbnailGenerator::new("assets/test1.mp4", 500, 31, 8);
//
//     let total = temp.duration;
//     dbg!(total);
//     let unit = (total * 25) / 100;
//
//     for i in 1..4 {
//         let time = unit * i;
//         temp.generate(time);
//         dbg!(i);
//     }
// }

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
    db: db::Database,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let fonts = utils::load_fonts().map(Message::FontLoad);
        // let load = Task::done(Message::Load);

        let now = Instant::now();

        let new = Self {
            now,
            db: db::Database::open_test_db().unwrap(),
        };

        (new, Task::batch([fonts]))
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
        let tip = container("tooltip")
            .style(|theme: &Theme| {
                let color = theme.extended_palette().secondary.weak.color;
                let default = container::rounded_box(theme);
                let border = default.border.rounded(5.0).width(1.0).color(color);
                let shadow = Shadow {
                    color,
                    blur_radius: 8.0,
                    offset: [0.0, 0.0].into(),
                };

                container::Style {
                    border,
                    shadow,
                    ..default
                }
            })
            .padding([3, 6]);
        let content = tooltip(
            container("Playground").style(container::dark),
            tip,
            tooltip::Position::Right,
        )
        .gap(2.0);

        let content = center(content);

        content.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Light)
    }
}
