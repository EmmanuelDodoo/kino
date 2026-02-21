#![allow(dead_code, unused_imports)]
use iced::{
    Color, ContentFit, Element, Event, Font, Length, Padding, Point, Radians, Rectangle, Rotation,
    Shadow, Size, Subscription, Task, Theme, Vector,
    advanced::{self, Widget, layout, mouse, overlay, widget::tree},
    alignment::{Horizontal, Vertical},
    animation::{self, Animation, Easing},
    application::BootFn,
    border::{self, Border, Radius},
    color, font, keyboard, padding,
    time::{Duration, Instant, every, milliseconds},
    widget::{
        self, Space, bottom, bottom_center, button, center, center_x, center_y, column, container,
        float, grid, markdown, mouse_area, operation, pick_list, rich_text, row, scrollable,
        sensor, slider, space, span, stack, text, text_editor, text_input, tooltip as tp,
        vertical_slider,
    },
    window,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

mod app;
mod db;
mod error;
mod fetch;
mod home;
mod models;
mod player;
mod scan;
mod settings;
pub mod utils;

use app::App;
use models::{Directory, ItemId, Media, MediaType, Movie, PlayId, SearchItem, Show, collection};
use std::sync::LazyLock;
use utils::config::Config;
use utils::filter;
use utils::filter::*;
use utils::icons;
use utils::icons::*;
use utils::sort;
use utils::sort::*;
use utils::typo;
use utils::typo::*;
use utils::{Layout, Sort, SortKind, cancel_btn, empty, save_btn, styles, tooltip};

const DUMMY: &str = "@0:08 something _long_ __here__ @7:26 and **after**. testing 31:43 and also @me but what about @ 5:13. how about an @2:06:30 \n\n![dragonfly](https://plus.unsplash.com/premium_photo-1710760668546-f241a1899c29?q=80&w=1057&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D)";

// fn _test_main() {
// fn main() {
// }

// fn test_main() -> iced::Result {
#[rustfmt::skip]
fn main() -> iced::Result {
    use std::env;
    use tracing::{Level, span};

    let span = span!(Level::DEBUG, "Kino");
    let _guard = span.enter();

    let mut args = env::args();
    let _ = args.next();

    let mode = args.next();

    let icon = {
        let data = include_bytes!("../resources/logo/logo.png");

        let format = None;

        iced::window::icon::from_file_data(data, format).unwrap()
    };

    let mut fonts = typo_fonts();
    fonts.push(icons::ICONS.into());

    #[allow(unused_variables)]
    let mode = match mode.as_deref() {
        Some("dev") => BootMode::Dev,
        Some("dummies") => BootMode::Dummies,
        _ => BootMode::Prod,
    };

    iced::application::timed(
        mode,
        App::update,
        App::subscription,
        App::view
    )
        .theme(App::theme)

    // iced::application::timed(
    //     Playground::boot,
    //     Playground::update,
    //     Playground::subscription,
    //     Playground::view,
    // )
    //     .theme(Playground::theme)

        .settings(iced::Settings {
            default_font: regular_font(),
            fonts,
            ..Default::default()

        })
        .title("kino")
        .window(window::Settings {
            icon: Some(icon),
            size: Size::new(1280.0, 800.0),
            exit_on_close_request: false,
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Clone, Copy)]
pub enum BootMode {
    Dev,
    Dummies,
    Prod,
}

impl BootFn<App, app::Message> for BootMode {
    fn boot(&self) -> (App, Task<app::Message>) {
        match self {
            Self::Dummies => {
                let config = Config::dev();
                let dummies = "dummy.txt";
                let db = db::Database::open_with_dummies(config.db_path(), dummies)
                    .expect("Failed to open dummy database");

                tracing::debug!("Starting up Dummies instance");
                App::boot(config, db, std::iter::empty(), false)
            }
            Self::Dev => {
                let config = Config::dev();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open dev database");

                tracing::debug!("Starting up Dev instance");
                App::boot(config, db, std::iter::empty(), false)
            }
            Self::Prod => {
                let (config, errors) = Config::load();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open Database");

                tracing::debug!("Starting up Production instance");
                App::boot(config, db, errors, true)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    None,
}

struct Playground {
    now: Instant,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let now = Instant::now();

        let new = Self { now };

        (new, Task::none())
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = medium("Playground....");

        let content = center(content);

        content.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Dracula)
    }
}
