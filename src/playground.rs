#![allow(dead_code, unused_imports)]
use iced::{
    Length, Subscription, Task,
    alignment::{Horizontal, Vertical},
    font,
    time::Instant,
    widget::{self, button, center, column, container, pick_list, row},
};

use crate::Element;
use crate::theme::Theme;
use crate::utils;
use widgets::font_selection;
use widgets::pagination;

#[derive(Debug, Clone)]
pub enum Message {
    Family(font::Family),
    Fonts(Result<Vec<font::Family>, font::Error>),
    Page(pagination::Page),
    Temp(bool),
    Theme(Theme),
    None,
}

pub struct Playground {
    now: Instant,
    selected: Option<font::Family>,
    state: font_selection::State,
    current: usize,
    temp: bool,
    theme: Option<Theme>,
}

impl Playground {
    pub fn boot() -> (Self, Task<Message>) {
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

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
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

    pub fn view(&self) -> Element<'_, Message> {
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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}
