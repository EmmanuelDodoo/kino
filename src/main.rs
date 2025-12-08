#![allow(dead_code, unused_imports)]

use iced::{
    Color, ContentFit, Element, Event, Font, Length, Padding, Point, Radians, Rectangle, Rotation,
    Shadow, Size, Subscription, Task, Theme, Vector,
    advanced::{self, Widget, layout, mouse, overlay, widget::tree},
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    application::BootFn,
    border::{self, Border, Radius},
    color, font, keyboard, padding,
    time::{Duration, Instant},
    widget::{
        self, Space, bottom, bottom_center, button, center, center_x, center_y, column, container,
        float, grid, image, markdown, mouse_area, operation, pick_list, rich_text, row, scrollable,
        slider, space, span, stack, text, text_input, tooltip,
    },
    window,
};
use serde::{Deserialize, Serialize};
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
mod widgets;

use app::App;
use models::{Directory, ItemId, Media, MediaType, Movie, SearchItem, Show};
use utils::config::Config;
use utils::filter;
use utils::filter::*;
use utils::icons;
use utils::icons::*;
use utils::sort;
use utils::sort::*;
use utils::typo;
use utils::typo::*;
use utils::{Layout, Sort, SortKind, empty, styles};
use widgets::*;

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
                App::boot(config, db, std::iter::empty())
            }
            Self::Dev => {
                let config = Config::dev();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open dev database");

                App::boot(config, db, std::iter::empty())
            }
            Self::Prod => {
                let (config, errors) = Config::load();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open Database");

                App::boot(config, db, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    FontLoad(Result<(), iced::font::Error>),
    Iced(bool),
    Custom(bool),
    Extra(bool),
    None,
}

struct Playground {
    now: Instant,
    iced: bool,
    custom: bool,
    extra: bool,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let fonts = utils::load_fonts().map(Message::FontLoad);

        let now = Instant::now();

        let new = Self {
            now,
            iced: false,
            custom: false,
            extra: false,
        };

        (new, Task::batch([fonts]))
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => Task::none(),
            Message::FontLoad(Ok(_)) => Task::none(),
            Message::FontLoad(Err(error)) => {
                tracing::error!("{error:?}");
                Task::none()
            }
            Message::Iced(toggle) => {
                self.iced = toggle;
                Task::none()
            }
            Message::Custom(toggle) => {
                self.custom = toggle;
                Task::none()
            }
            Message::Extra(toggle) => {
                self.extra = toggle;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // let iced = widget::toggler(self.iced)
        //     .on_toggle(Message::Iced)
        //     .label("Iced");
        // let custom = widgets::toggler(self.custom)
        //     .on_toggle(Message::Custom)
        //     .duration(Duration::from_millis(200))
        //     .label("Custom");
        // let extra = widgets::toggler(self.extra)
        //     .on_toggle(Message::Extra)
        //     .label("Extra");
        //
        // let content = column!(iced, custom, extra).spacing(20.0);
        let handle = image::Handle::from_path("assets/fantastic.qal.png");

        let content = image(handle.clone())
            .height(400)
            .width(400.0 * 2.0 / 3.0)
            .content_fit(ContentFit::Contain);

        let content = center(content);

        content.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Nord)
    }
}
