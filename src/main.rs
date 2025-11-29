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
use std::io;
use std::path::{Path, PathBuf};

mod app;
mod db;
mod error;
mod home;
mod models;
mod player;
mod scan;
mod settings;
pub mod utils;
mod widgets;

use app::App;
use models::{Directory, ItemId, Media, MediaType, SearchItem, Show};
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
    Scan,
    Fetch,
    ScanComplete,
    None,
}

struct Playground {
    now: Instant,
    db: db::Database,
    dir: Directory,
    items: Vec<Show>,
    scanning: bool,
}

impl Playground {
    fn boot() -> (Self, Task<Message>) {
        let fonts = utils::load_fonts().map(Message::FontLoad);

        let now = Instant::now();

        let db = db::Database::open_with_schema("test.db").unwrap();
        let (dir, query) = Directory::new(
            r"C:\Users\edodo\Desktop\Series".into(),
            // r"C:\Users\edodo\Desktop\coding\Projects\kino\assets".into(),
            MediaType::Shows,
            true,
        );
        query.execute(&db).unwrap();

        let new = Self {
            now,
            db,
            dir,
            items: vec![],
            scanning: false,
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
            Message::Scan => {
                self.scanning = true;
                let dir = self.dir.clone();

                Task::perform(
                    // async move { scan::scan_dirs("test.db", vec![dir.clone(), dir]).unwrap() },
                    async move { scan::scan_dir("test.db", dir, true) },
                    |res| {
                        if let Some(res) = res {
                            println!("{}", res.successes.len());
                            println!("{}", res.failures.len());
                        }
                        Message::ScanComplete
                    },
                )
            }
            Message::ScanComplete => {
                self.scanning = false;
                Task::none()
            }
            Message::Fetch => {
                let items = self
                    .db
                    .get_shows(None, None, Filter::none(), Sort::default(), |item| item)
                    .unwrap();

                self.items = items;

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let items = self
            .items
            .iter()
            .map(|item| text(item.name()).size(H7).into());
        let items = column(items).spacing(10);

        let scanning: Element<'_, Message> = if self.scanning {
            text("Scanning.....").into()
        } else {
            empty()
        };

        let dir = text(&self.dir.path);

        let actions = row!(
            button("Scan").on_press(Message::Scan),
            button("Fetch").on_press(Message::Fetch)
        )
        .spacing(50.0);

        let content = column!(dir, actions, scanning, items)
            .spacing(8.0)
            .align_x(Horizontal::Center);

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
