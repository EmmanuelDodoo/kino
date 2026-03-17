use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll};
use iced::{
    Element, Length, Padding, Task,
    alignment::Vertical,
    time::Instant,
    widget::{self, column, container, grid, operation, row, scrollable, text},
};
use registry::models::{ItemId, Media, Movie, MovieId};

#[derive(Debug, Clone, Copy)]
pub enum MoviesMessage {
    Hovered(MovieId, bool),
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
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = thumbnails.map(|thumbnail| {
            thumbnail.card(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Play,
            )
        });

        let content = grid(content)
            .spacing(16)
            .fluid(CARD_WIDTH)
            .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

        let content =
            scrollable(container(content).padding(Padding::new(10.0).right(16).bottom(0)))
                .auto_scroll(true)
                .height(Length::Fill)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll);

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = thumbnails.map(|thumbnail| {
            thumbnail.list(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Play,
                unique,
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
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = thumbnails.map(|thumbnail| {
            thumbnail.compact(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
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
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        match layout {
            Layout::Grid => self.grid(now, thumbnails),
            Layout::List => self.list(now, thumbnails),
            Layout::Compact => self.compact(now, thumbnails),
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}

pub fn unique<'a, Message: 'a>(movie: &Movie) -> Element<'a, Message> {
    use iced::font::{Font, Weight};

    let release = text(movie.release_year()).size(H8).font(Font {
        weight: Weight::Semibold,
        ..Default::default()
    });
    let icon = icon(CALENDAR).size(H7);

    row!(icon, release)
        .align_y(Vertical::Center)
        .spacing(3.0)
        .into()
}
