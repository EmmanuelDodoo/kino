use super::movie::MovieItem;
use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::Element;
use crate::theme::{self, Theme};
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll};
use iced::{
    Length, Padding, Task,
    alignment::Vertical,
    time::Instant,
    widget::{self, column, container, grid, operation, row, scrollable, text},
};
use registry::models::{ItemId, Media, Movie, MovieId};

#[derive(Debug, Clone, Copy)]
pub enum MoviesMessage {
    Hovered(MovieId, bool),
    Shown(MovieId, bool),
    Play(MovieId),
    Add(MovieId),
    Details(MovieId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct Movies {
    scroll: Scroll,
}

impl Movies {
    pub fn boot() -> (Self, Task<MoviesMessage>) {
        let (new, id) = Self::new();
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new() -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { scroll }, id)
    }

    pub fn update(&mut self, message: MoviesMessage) -> Option<HomeMessage> {
        match message {
            MoviesMessage::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Movie(id), is_hovered);

                Some(msg)
            }
            MoviesMessage::Shown(id, shown) => Some(HomeMessage::Shown(id.into(), shown)),
            MoviesMessage::Play(id) => {
                let msg = HomeMessage::Play(ItemId::Movie(id));
                Some(msg)
            }
            MoviesMessage::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Movie(id));
                Some(msg)
            }
            MoviesMessage::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(id)));
                Some(msg)
            }
            MoviesMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                None
            }
        }
    }

    fn grid<'a>(
        &self,
        now: Instant,
        movies: impl Iterator<Item = &'a MovieItem>,
    ) -> Element<'a, MoviesMessage> {
        let content = movies.map(|movie| {
            movie.card(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Shown,
                MoviesMessage::Play,
            )
        });

        let content = grid(content)
            .spacing(16)
            .fluid(MovieItem::WIDTH)
            .height(grid::aspect_ratio(MovieItem::WIDTH, MovieItem::HEIGHT));

        let content = scrollable(container(content).padding(Padding::new(16.0)))
            .auto_scroll(true)
            .height(Length::Fill)
            .id(self.scroll.id.clone())
            .on_scroll(MoviesMessage::Scroll);

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        movies: impl Iterator<Item = &'a MovieItem>,
    ) -> Element<'a, MoviesMessage> {
        let content = movies.map(|movie| {
            movie.list(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Shown,
                MoviesMessage::Play,
            )
        });

        let content = column(content)
            .spacing(16)
            .padding(Padding::new(10.0).bottom(0));

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(MoviesMessage::Scroll);

        content.into()
    }

    fn compact<'a>(
        &self,
        now: Instant,
        movies: impl Iterator<Item = &'a MovieItem>,
    ) -> Element<'a, MoviesMessage> {
        let content = movies.map(|movie| {
            movie.compact(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Shown,
                MoviesMessage::Play,
            )
        });

        let content = column(content)
            .spacing(16)
            .padding(Padding::new(10.0).bottom(0));

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(MoviesMessage::Scroll);

        content.into()
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        layout: Layout,
        movies: impl Iterator<Item = &'a MovieItem>,
    ) -> Element<'a, MoviesMessage> {
        match layout {
            Layout::Grid => self.grid(now, movies),
            Layout::List => self.list(now, movies),
            Layout::Compact => self.compact(now, movies),
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}
