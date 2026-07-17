use super::{HomeMessage, Layout, MovieItem, PageKind, ShowItem, ViewMessage};
use crate::Element;
use crate::utils::Scroll;
use iced::{
    Length, Padding, Task,
    time::Instant,
    widget::{
        Column, column, grid,
        operation::{self},
        scrollable,
    },
};

use registry::models::{Directory, DirectoryId, ItemId};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Message {
    Scroll(scrollable::Viewport),
    PlayItem(ItemId),
    HoveredItem(bool, ItemId),
    ShownItem(bool, ItemId),
    DetailsItem(ItemId),
    Add(ItemId),
    ToggleMovies(bool),
    ToggleShows(bool),
}

#[derive(Debug, Clone)]
pub struct DirectoryMessage {
    pub id: DirectoryId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct DirectoryPage {
    id: DirectoryId,
    scroll: Scroll,
    movies_shown: bool,
    shows_shown: bool,
}

impl DirectoryPage {
    pub fn boot(dir: DirectoryId) -> (Self, Task<DirectoryMessage>) {
        let new = Self::new(dir);

        let scroll = operation::scroll_to(
            new.scroll.id.clone(),
            scrollable::AbsoluteOffset::<f32>::default(),
        )
        .map(move |message| DirectoryMessage { id: dir, message });

        (new, scroll)
    }

    fn new(dir: DirectoryId) -> Self {
        Self {
            id: dir,
            scroll: Scroll::new(),
            movies_shown: true,
            shows_shown: true,
        }
    }

    pub fn update(&mut self, message: DirectoryMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                None
            }
            Message::PlayItem(item) => {
                let msg = HomeMessage::Play(item);
                Some(msg)
            }
            Message::HoveredItem(hovered, item) => {
                let msg = HomeMessage::Hovered(item, hovered);
                Some(msg)
            }
            Message::ShownItem(shown, item) => Some(HomeMessage::Shown(item, shown)),
            Message::Add(item) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(item));
                Some(msg)
            }
            Message::ToggleMovies(toggle) => {
                self.movies_shown = toggle;

                None
            }
            Message::ToggleShows(toggle) => {
                self.shows_shown = toggle;

                None
            }
            Message::DetailsItem(item) => {
                let kind = match item {
                    ItemId::Movie(id) => PageKind::Movie(id),
                    ItemId::Show(id) => PageKind::Show(id),
                    ItemId::Season(id) => PageKind::Season(id),
                    ItemId::Episode(id) => PageKind::Episode(id),
                };
                let msg = HomeMessage::Goto(kind);
                Some(msg)
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        now: Instant,
        layout: Layout,
        _dir: &'a Directory,
        movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
    ) -> Element<'a, DirectoryMessage> {
        let content = match layout {
            Layout::List => self.list(now, movies, shows),
            Layout::Grid => self.grid(now, movies, shows),
            Layout::Compact => self.compact(now, movies, shows),
        };

        let content = column!(content).spacing(10).width(Length::Fill);

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
    ) -> Element<'a, DirectoryMessage> {
        let id = self.id;

        let content = Column::new()
            .spacing(40)
            .padding(Padding::new(10.0).bottom(0));

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                

                {
                    let content =
                        movies.map(|movie| movie.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                }
            };

            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                

                {
                    let content =
                        shows.map(|thumbnail| thumbnail.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                }
            };

            content.push(shows)
        };

        let content: Element<'_, Message> = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(Message::Scroll)
            .into();

        content.map(move |message| DirectoryMessage { id, message })
    }

    fn compact<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
    ) -> Element<'a, DirectoryMessage> {
        let id = self.id;

        let content = Column::new()
            .spacing(40)
            .padding(Padding::new(10.0).bottom(0));

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                

                {
                    let content = movies
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                }
            };

            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                
                {
                    let content = shows
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                }
            };

            content.push(shows)
        };

        let content: Element<'_, Message> = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(Message::Scroll)
            .into();

        content.map(move |message| DirectoryMessage { id, message })
    }

    fn grid<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
    ) -> Element<'a, DirectoryMessage> {
        let id = self.id;

        let content = Column::new().spacing(40.0).padding([16, 16]);

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let movies = movies.map(|movie| movie.card(now, add, select, hover, shown, play));

                

                grid(movies)
                        .spacing(16)
                        .fluid(MovieItem::WIDTH)
                        .height(if self.movies_shown {
                            grid::aspect_ratio(MovieItem::WIDTH, MovieItem::HEIGHT)
                        } else {
                            grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                        })
            };
            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows =
                {
                    let shows = shows.map(|show| show.card(now, add, select, hover, shown, play));

                    

                    grid(shows).spacing(16).fluid(ShowItem::WIDTH).height(
                        if self.shows_shown {
                            grid::aspect_ratio(ShowItem::WIDTH, ShowItem::HEIGHT)
                        } else {
                            grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                        },
                    )
                };

            content.push(shows)
        };

        let content: Element<'_, Message> = scrollable(content)
            .auto_scroll(true)
            .id(self.scroll.id.clone())
            .height(Length::Fill)
            .on_scroll(Message::Scroll)
            .into();

        content.map(move |message| DirectoryMessage { id, message })
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}

fn add(id: impl Into<ItemId>) -> Message {
    Message::Add(id.into())
}

fn select(id: impl Into<ItemId>) -> Message {
    Message::DetailsItem(id.into())
}

fn hover(item: impl Into<ItemId>, hovered: bool) -> Message {
    Message::HoveredItem(hovered, item.into())
}

fn shown(item: impl Into<ItemId>, shown: bool) -> Message {
    Message::ShownItem(shown, item.into())
}

fn play(item: impl Into<ItemId>) -> Message {
    Message::PlayItem(item.into())
}
