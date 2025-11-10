use super::{HomeMessage, PageKind, PageUpdate, shared::*};
use crate::models::{CollectionId, CollectionView};
use crate::utils::filter::*;
use crate::utils::{Layout, Sort};
use iced::{
    Element, Task,
    time::Instant,
    widget::{self, container, grid, operation, scrollable},
};

#[derive(Debug, Clone, Copy)]
pub enum CollectionsMessage {
    Hovered(CollectionId, bool),
    Details(CollectionId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct Collections {
    layout: Layout,
    sort: Sort,
    filter: Filter,
    scroll: Scroll,
}

impl Collections {
    pub fn boot(sort: Sort, filters: Filter, layout: Layout) -> (Self, Task<CollectionsMessage>) {
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

    pub fn update(&mut self, message: CollectionsMessage) -> Option<HomeMessage> {
        match message {
            CollectionsMessage::Hovered(key, is_hovered) => {
                let msg = HomeMessage::HoveredCollection(key, is_hovered);

                Some(msg)
            }
            CollectionsMessage::Details(key) => {
                let msg = HomeMessage::Goto(PageKind::Collection(key));
                Some(msg)
            }
            CollectionsMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                None
            }
        }
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a CollectionThumbnail>,
    ) -> Element<'a, CollectionsMessage> {
        self.grid(now, thumbnails)
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

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn grid<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a CollectionThumbnail>,
    ) -> Element<'a, CollectionsMessage> {
        let content = thumbnails.map(|thumbnail| {
            thumbnail.view(
                now,
                CollectionsMessage::Details,
                CollectionsMessage::Hovered,
            )
        });

        let content = grid(content)
            .spacing(16)
            .fluid(CollectionThumbnail::CARD_WIDTH)
            .height(grid::aspect_ratio(
                CollectionThumbnail::CARD_WIDTH,
                CollectionThumbnail::CARD_HEIGHT,
            ));

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(CollectionsMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    pub fn name(&self) -> &'static str {
        "Collections"
    }
}
