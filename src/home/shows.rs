use super::{HomeMessage, PageKind, PageUpdate, ViewMessage, shared::*};
use crate::models::{ItemId, Show, ShowId};
use crate::utils::filter::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort};
use iced::{
    Element, Task,
    time::Instant,
    widget::{self, column, container, grid, operation, scrollable, text},
};

#[derive(Debug, Clone, Copy)]
pub enum TvShowsMessage {
    Hovered(ShowId, bool),
    Add(ShowId),
    Details(ShowId),
    Play(ShowId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct TvShows {
    layout: Layout,
    sort: Sort,
    filters: Filter,
    scroll: Scroll,
}

impl TvShows {
    pub fn boot(sort: Sort, filters: Filter, layout: Layout) -> (Self, Task<TvShowsMessage>) {
        let (new, id) = Self::new(sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, scroll)
    }

    fn new(sort: Sort, filters: Filter, layout: Layout) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                sort,
                filters,
                layout,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: TvShowsMessage) -> Option<HomeMessage> {
        match message {
            TvShowsMessage::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Show(id), is_hovered);

                Some(msg)
            }
            TvShowsMessage::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(id)));

                Some(msg)
            }
            TvShowsMessage::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Show(id));

                Some(msg)
            }
            TvShowsMessage::Play(id) => {
                let msg = HomeMessage::Play(ItemId::Show(id));

                Some(msg)
            }
            TvShowsMessage::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();

                None
            }
        }
    }

    pub fn page_update(&mut self, update: PageUpdate) {
        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.layout = layout;
        self.filters = filters;
    }

    pub fn name(&self) -> &str {
        "Tv Shows"
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn list<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        let content = filter_sort(thumbnails, &self.filters, &self.sort).map(|thumbnail| {
            thumbnail.list(
                now,
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Hovered,
                TvShowsMessage::Play,
                unique,
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(TvShowsMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        let content = filter_sort(thumbnails, &self.filters, &self.sort).map(|thumbnail| {
            thumbnail.card(
                now,
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Hovered,
                TvShowsMessage::Play,
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
                .on_scroll(TvShowsMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        match self.layout {
            Layout::Grid => self.grid(now, thumbnails),
            Layout::List => self.list(now, thumbnails),
        }
    }
}

pub fn unique<'a, Message: 'a>(show: &Show) -> Element<'a, Message> {
    let seasons = show.seasons;

    let seasons = format!("{} season{}", seasons, if seasons > 1 { "s" } else { "" });

    text(seasons).size(H7).into()
}
