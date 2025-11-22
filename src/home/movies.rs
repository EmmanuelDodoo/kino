use super::{HomeMessage, PageKind, PageUpdate, ViewMessage, shared::*};
use crate::models::{ItemId, Media, Movie, MovieId};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort};
use iced::{
    Element, Task,
    alignment::Vertical,
    time::Instant,
    widget::{self, column, container, grid, operation, row, scrollable, text},
};

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
    layout: Layout,
    sort: Sort,
    filter: Filter,
    scroll: Scroll,
}

impl Movies {
    pub fn boot(sort: Sort, filters: Filter, layout: Layout) -> (Self, Task<MoviesMessage>) {
        let (new, id) = Self::new(sort, layout, filters);
        let scroll = operation::scroll_to(id, operation::AbsoluteOffset::default());

        (new, scroll)
    }

    fn new(sort: Sort, layout: Layout, filter: Filter) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                layout,
                sort,
                filter,
                scroll,
            },
            id,
        )
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

    pub fn page_update(&mut self, update: PageUpdate) {
        let PageUpdate {
            layout,
            sort,
            filters,
        } = update;

        self.sort = sort;
        self.layout = layout;
        self.filter = filters;
    }

    fn grid<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = filter_sort(thumbnails, &self.filter, &self.sort).map(|thumbnail| {
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

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = filter_sort(thumbnails, &self.filter, &self.sort).map(|thumbnail| {
            thumbnail.list(
                now,
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Hovered,
                MoviesMessage::Play,
                unique,
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn compact<'a>(
        &self,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        let content = filter_sort(thumbnails, &self.filter, &self.sort).map(|thumbnail| {
            thumbnail.compact(
                MoviesMessage::Add,
                MoviesMessage::Details,
                MoviesMessage::Play,
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Movie>>,
    ) -> Element<'a, MoviesMessage> {
        match self.layout {
            Layout::Grid => self.grid(now, thumbnails),
            Layout::List => self.list(now, thumbnails),
            Layout::Compact => self.compact(thumbnails),
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
