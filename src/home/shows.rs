use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::models::{ItemId, Show, ShowId};
use crate::utils::filter::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll, Sort};
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
    scroll: Scroll,
}

impl TvShows {
    pub fn boot() -> (Self, Task<TvShowsMessage>) {
        let (new, id) = Self::new();
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new() -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { scroll }, id)
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

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn list<'a>(
        &self,
        now: Instant,
        filters: &Filter,
        sort: &Sort,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        let content = filter_sort(thumbnails, filters, sort).map(|thumbnail| {
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

    fn compact<'a>(
        &self,
        filters: &Filter,
        sort: &Sort,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        let content = filter_sort(thumbnails, filters, sort).map(|thumbnail| {
            thumbnail.compact(
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Play,
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
        filters: &Filter,
        sort: &Sort,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        let content = filter_sort(thumbnails, filters, sort).map(|thumbnail| {
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
        filters: Filter,
        sort: Sort,
        layout: Layout,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Show>>,
    ) -> Element<'a, TvShowsMessage> {
        match layout {
            Layout::Grid => self.grid(now, &filters, &sort, thumbnails),
            Layout::List => self.list(now, &filters, &sort, thumbnails),
            Layout::Compact => self.compact(&filters, &sort, thumbnails),
        }
    }
}

pub fn unique<'a, Message: 'a>(show: &Show) -> Element<'a, Message> {
    use iced::font::{Font, Weight};

    let seasons = show.seasons;

    let seasons = format!("{} season{}", seasons, if seasons > 1 { "s" } else { "" });

    text(seasons)
        .size(H8)
        .font(Font {
            weight: Weight::Semibold,
            ..Default::default()
        })
        .into()
}
