use super::{HomeMessage, PageKind, ShowItem, ViewMessage, shared::*};
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll};
use iced::{
    Element, Length, Padding, Task,
    time::Instant,
    widget::{self, column, container, grid, operation, scrollable, text},
};
use registry::models::{ItemId, Show, ShowId};

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
        shows: impl Iterator<Item = &'a ShowItem>,
    ) -> Element<'a, TvShowsMessage> {
        let content = shows.map(|show| {
            show.list(
                now,
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Hovered,
                TvShowsMessage::Play,
            )
        });

        let content = column(content)
            .spacing(16)
            .padding(Padding::new(10.0).bottom(0));

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(TvShowsMessage::Scroll);

        content.into()
    }

    fn compact<'a>(
        &self,
        now: Instant,
        shows: impl Iterator<Item = &'a ShowItem>,
    ) -> Element<'a, TvShowsMessage> {
        let content = shows.map(|show| {
            show.compact(
                now,
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Hovered,
                TvShowsMessage::Play,
            )
        });

        let content = column(content)
            .spacing(16)
            .padding(Padding::new(10.0).bottom(0));

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(TvShowsMessage::Scroll);

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        shows: impl Iterator<Item = &'a ShowItem>,
    ) -> Element<'a, TvShowsMessage> {
        let content = shows.map(|show| {
            show.card(
                now,
                TvShowsMessage::Add,
                TvShowsMessage::Details,
                TvShowsMessage::Hovered,
                TvShowsMessage::Play,
            )
        });

        let content = grid(content)
            .spacing(16)
            .fluid(ShowItem::WIDTH)
            .height(grid::aspect_ratio(ShowItem::WIDTH, ShowItem::HEIGHT));

        let content = scrollable(container(content).padding(Padding::new(16.0)))
            .auto_scroll(true)
            .height(Length::Fill)
            .id(self.scroll.id.clone())
            .on_scroll(TvShowsMessage::Scroll);

        content.into()
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        layout: Layout,
        shows: impl Iterator<Item = &'a ShowItem>,
    ) -> Element<'a, TvShowsMessage> {
        match layout {
            Layout::Grid => self.grid(now, shows),
            Layout::List => self.list(now, shows),
            Layout::Compact => self.compact(now, shows),
        }
    }
}
