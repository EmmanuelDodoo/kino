#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code, unused_imports)]
use iced::{
    Color, ContentFit, Event, Font, Length, Padding, Point, Radians, Rectangle, Rotation, Shadow,
    Size, Subscription, Task, Vector,
    advanced::{self, Widget, layout, mouse, overlay, widget::tree},
    alignment::{Horizontal, Vertical},
    animation::{self, Animation, Easing},
    application::BootFn,
    border::{self, Border, Radius},
    color, font, keyboard, padding,
    time::{Duration, Instant, every, milliseconds},
    widget::{
        self, Space, bottom, bottom_center, button, center, center_x, center_y, column, combo_box,
        container, float, grid, markdown, mouse_area, operation, pick_list, rich_text, row,
        scrollable, sensor, slider, space, span, stack, text, text_editor, text_input,
        tooltip as tp, vertical_slider,
    },
    window,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

mod app;
mod config;
mod home;
mod player;
mod settings;
pub mod theme;
pub mod utils;

use app::App;
use config::Config;
use registry::db;
use registry::filter;
use registry::filter::*;
use registry::models::{
    Directory, ItemId, Media, MediaType, Movie, SearchItem, Show, VideoId, collection,
};
use registry::sort;
use registry::sort::*;
use std::sync::LazyLock;
pub use theme::Theme;
use utils::icons;
use utils::icons::*;
use utils::typo;
use utils::typo::*;
use utils::{cancel_btn, empty, save_btn, tooltip};
use widgets::{font_selection, modal, pagination};

pub type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;

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
        .power_preference(iced::backend::PowerPreference::HighPerformance)
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
    Prod,
}

impl BootFn<App, app::Message> for BootMode {
    fn boot(&self) -> (App, Task<app::Message>) {
        match self {
            Self::Dev => {
                let config = Config::dev();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open dev database");

                tracing::debug!("Starting up Dev instance");
                App::boot(config, db, std::iter::empty())
            }
            Self::Prod => {
                let (config, errors) = Config::load();
                let db = db::Database::open_with_schema(config.db_path())
                    .expect("Failed to open Database");

                tracing::debug!("Starting up Production instance");
                App::boot(config, db, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Family(font::Family),
    Fonts(Result<Vec<font::Family>, font::Error>),
    Page(pagination::Page),
    Temp(bool),
    Theme(Theme),
    None,
}

struct Playground {
    now: Instant,
    selected: Option<font::Family>,
    state: font_selection::State,
    current: usize,
    temp: bool,
    theme: Option<Theme>,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let now = Instant::now();
        let task = font::list().map(Message::Fonts);

        let new = Self {
            now,
            selected: Some(font::Family::Name("Copperplate Gothic Bold")),
            state: font_selection::State::new(vec![]),
            current: 15,
            temp: false,
            theme: None,
        };

        (new, task)
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => {
                println!("Received None");
            }
            Message::Fonts(Ok(fams)) => {
                self.state = font_selection::State::new(fams);
            }
            Message::Fonts(Err(error)) => {
                println!("{error:?}");
            }
            Message::Family(family) => {
                self.selected = Some(family);
            }
            Message::Theme(theme) => {
                self.theme = Some(theme);
            }
            Message::Page(page) => {
                dbg!(page);
                match page {
                    pagination::Page::Number(page) => self.current = page,
                    pagination::Page::Ellipsis { left, right } => {
                        self.current = left + (right - left) / 2;
                    }
                }
            }
            Message::Temp(temp) => {
                self.temp = temp;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let themes = pick_list(self.theme.as_ref(), Theme::DEFAULTS, Theme::to_string)
            .placeholder("Select theme")
            .on_select(Message::Theme)
            .padding([5, 10])
            .font(self.selected.unwrap_or_default());

        let families =
            font_selection(&self.state, "Select font", self.selected, Message::Family).width(500);

        let pages = {
            let a = pagination(1, self.current, 1250)
                .on_select(Message::Page)
                .font(self.selected.unwrap_or_default().into());
            let b = pagination(1, self.current, 1250)
                .on_select(Message::Page)
                .font(self.selected.unwrap_or_default().into())
                .buttoned();

            let pages = column!(a, b).spacing(16);

            container(pages).width(Length::Fit.max(750))
        };

        let toggles = {
            let a = utils::toggler(self.temp).on_toggle(Message::Temp);
            let b = widget::checkbox(self.temp).on_toggle(Message::Temp);

            row!(a, b).spacing(16)
        };

        let throbbers = {
            let a = widgets::throbber::linear();
            let b = widgets::throbber::circular();

            row!(a, b).spacing(16)
        };

        let content = column!(families, themes, pages, toggles, throbbers)
            .spacing(40)
            .align_x(Horizontal::Center);
        let content = center(content);

        content.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}
