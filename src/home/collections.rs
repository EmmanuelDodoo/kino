use super::{HomeMessage, PageKind, shared::*};
use crate::Element;
use crate::utils::Scroll;
use iced::{
    Padding, Task,
    time::Instant,
    widget::{self, container, grid, operation, scrollable},
};
use registry::models::CollectionId;

#[derive(Debug, Clone, Copy)]
pub enum CollectionsMessage {
    Details(CollectionId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct Collections {
    scroll: Scroll,
}

impl Collections {
    pub fn boot() -> (Self, Task<CollectionsMessage>) {
        let (new, id) = Self::new();
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new() -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { scroll }, id)
    }

    pub fn update(&mut self, message: CollectionsMessage) -> Option<HomeMessage> {
        match message {
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
        thumbnails: impl Iterator<Item = &'a CollectionThumbnail>,
        now: Instant,
    ) -> Element<'a, CollectionsMessage> {
        self.grid(thumbnails, now)
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn grid<'a>(
        &self,
        thumbnails: impl Iterator<Item = &'a CollectionThumbnail>,
        now: Instant,
    ) -> Element<'a, CollectionsMessage> {
        let content = thumbnails.map(|thumbnail| thumbnail.view(CollectionsMessage::Details, now));

        let content = grid(content)
            .spacing(16)
            .fluid(CollectionThumbnail::CARD_WIDTH)
            .height(grid::aspect_ratio(
                CollectionThumbnail::CARD_WIDTH,
                CollectionThumbnail::CARD_HEIGHT,
            ));

        let content = container(
            scrollable(content)
                .id(self.scroll.id.clone())
                .on_scroll(CollectionsMessage::Scroll),
        )
        .padding(Padding::new(10.0).bottom(0));

        content.into()
    }
}
