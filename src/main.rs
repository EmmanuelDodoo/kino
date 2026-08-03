#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use iced::{Size, Task, application::BootFn, window};

mod app;
mod config;
mod home;
mod player;
mod playground;
mod settings;
pub mod theme;
pub mod utils;

use app::App;
use config::Config;
#[allow(unused_imports)]
use playground::Playground;
use registry::db;
pub use theme::Theme;
use utils::icons;
use utils::typo::*;

pub type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;

// fn test_main() -> iced::Result {
#[rustfmt::skip]
fn main() -> iced::Result {
    use std::env;
    use tracing::{Level, span};
    use velopack::VelopackApp;

    VelopackApp::build().run();

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
