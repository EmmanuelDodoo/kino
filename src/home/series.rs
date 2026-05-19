use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll, empty, styles};
use devutils::source::SourceSet;
use iced::widget::Space;
use iced::{
    Animation, Color, ContentFit, Element, Length, Padding, Task,
    alignment::{Horizontal, Vertical},
    task,
    time::Instant,
    widget::{
        self, button, column, container, grid, image, image::Handle, operation, row, rule,
        scrollable, space, stack, text,
    },
};
use registry::models::{
    CollectionId, ItemId, Media, Season, SeasonId, Show, ShowId, SimpleCollection,
};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Message {
    Add(SeasonId),
    AddCollection,
    Hovered(SeasonId, bool),
    Details(SeasonId),
    Edit,
    Resume,
    Play(SeasonId),
    Scroll(scrollable::Viewport),
    Goto(CollectionId),
}

#[derive(Debug, Clone)]
pub struct ShowPageMessage {
    pub id: ShowId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct ShowPage {
    id: ShowId,
    scroll: Scroll,
}

impl ShowPage {
    pub fn boot(show: ShowId) -> (Self, Task<ShowPageMessage>) {
        let (new, id) = Self::new(show);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(show: ShowId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { id: show, scroll }, id)
    }

    pub fn update(&mut self, message: ShowPageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Edit => {
                let msg = HomeMessage::OpenView(ViewMessage::ShowEdit(self.id));

                Some(msg)
            }
            Message::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Season(id), is_hovered);

                Some(msg)
            }
            Message::AddCollection => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(self.id)));

                Some(msg)
            }
            Message::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Season(id)));

                Some(msg)
            }
            Message::Resume => {
                let msg = HomeMessage::Play(ItemId::Show(self.id));

                Some(msg)
            }
            Message::Play(season) => {
                let msg = HomeMessage::Play(ItemId::Season(season));

                Some(msg)
            }
            Message::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Season(id));

                Some(msg)
            }
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
            Message::Goto(id) => {
                let msg = HomeMessage::Goto(PageKind::Collection(id));
                Some(msg)
            }
        }
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn list<'a>(
        &self,
        now: Instant,
        seasons: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, Message> {
        let content = seasons.map(|thumbnail| {
            thumbnail.list(
                now,
                Message::Add,
                Message::Details,
                Message::Hovered,
                Message::Play,
                |season| {
                    let episodes = season.episodes;
                    let episodes = format!(
                        "{} episode{}",
                        episodes,
                        if episodes > 1 { "s" } else { "" }
                    );
                    h7(episodes).into()
                },
            )
        });

        let content = column(content).spacing(16);

        content.into()
    }

    fn compact<'a>(
        &self,
        now: Instant,
        seasons: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, Message> {
        let content = seasons.map(|thumbnail| {
            thumbnail.compact(
                now,
                Message::Add,
                Message::Details,
                Message::Hovered,
                Message::Play,
            )
        });

        let content = column(content).spacing(16);

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        seasons: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, Message> {
        let content = seasons.map(|thumbnail| {
            thumbnail.card(
                now,
                Message::Add,
                Message::Details,
                Message::Hovered,
                Message::Play,
            )
        });

        let content = grid(content)
            .spacing(12)
            .fluid(CARD_WIDTH)
            .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

        content.into()
    }

    pub fn overlay<'a>(
        &self,
        now: Instant,
        layout: Layout,
        show: &'a ShowItem,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
    ) -> Element<'a, ShowPageMessage> {
        let id = self.id;

        let img = page_image(|width, height| show.poster(width, height));

        let header = {
            let seasons = show.item.seasons;
            let details = page_details(
                show.item.rating(),
                show.item.release_year(),
                format!(
                    "{:02} Season{}",
                    seasons,
                    if seasons > 1 { "s" } else { "" }
                ),
            );

            let tags = {
                let values = show.item.tags.iter().map(|tag| (tag, None)).take(4);

                page_tags(values)
            };

            page_title(tags, show.item.name(), details)
        };

        let header = page_header(
            header,
            Message::Resume,
            Message::AddCollection,
            Message::Edit,
        );

        let overview = page_overview(show.item.synopsis());

        let seasons = match layout {
            Layout::Grid => self.grid(now, seasons),
            Layout::List => self.list(now, seasons),
            Layout::Compact => self.compact(now, seasons),
        };

        let collections = page_collections(memberships, Message::Goto);

        let data = page_data(
            show.item.added_humaized(),
            show.item.watch_count(),
            show.item.progress(),
            show.item.recent_humanized(),
            show.item.comments(),
            Some(("Seasons", show.item.seasons, NUMBER)),
        );

        let content = column!(header, overview, seasons, collections, data).spacing(40);

        let content = page_layout(content, img, &self.scroll, Message::Scroll);

        content.map(move |message| ShowPageMessage { id, message })
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        layout: Layout,
        show: &'a ShowItem,
        seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
    ) -> Element<'a, ShowPageMessage> {
        let overlay = self.overlay(now, layout, show, memberships, seasons);
        let top = space::vertical();

        let overlay = column!(top, overlay);

        let content = show.backdrop(Length::Fill, Length::FillPortion(3));

        let content = stack![content, overlay];

        content.into()
    }
}

#[derive(Debug, Clone)]
pub struct ShowItemTask {
    pub id: ShowId,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
pub struct ShowItem {
    backdrop: Option<Handle>,
    sample_text: Option<Color>,
    sample_color: Option<Color>,
    background: Animation<bool>,
    icon: Animation<bool>,
    float: Animation<bool>,
    _tasks: task::Handle,
    hovered: bool,
    poster: Image,
    pub selected: bool,
    pub item: Box<Show>,
}

impl ShowItem {
    pub fn new(show: Show) -> (Self, Task<ShowItemTask>) {
        let id = show.id;

        let (poster, task) = Image::load(show.poster.as_ref());
        let (task, handle) = task.map(move |kind| ShowItemTask { id, kind }).abortable();
        let handle = handle.abort_on_drop();

        //todo: Sample color is not great for current default poster
        let (sample_color, sample_text) = match show.poster() {
            Some(poster) => (
                poster.get_main().map(to_color),
                poster.get_accent().map(to_color),
            ),
            None => (None, None),
        };

        let backdrop = show.backdrop().map(Handle::from_path);

        let new = Self {
            selected: false,
            poster,
            backdrop,
            sample_color,
            sample_text,
            background: background_animation(),
            icon: icon_animation(),
            float: float_animation(),
            hovered: false,
            _tasks: handle,
            item: Box::new(show),
        };

        (new, task)
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let poster = match &self.poster {
            Image::Ready { fade_in, .. } => fade_in.is_animating(now),
            _ => false,
        };

        self.background.is_animating(now)
            || self.icon.is_animating(now)
            || self.float.is_animating(now)
            || poster
    }

    pub fn go_mut(&mut self, new_state: bool, at: Instant) {
        self.hovered = new_state;
        self.background.go_mut(new_state, at);
        self.icon.go_mut(new_state, at);
        self.float.go_mut(new_state, at);
    }

    fn poster_ready(&self) -> bool {
        matches!(&self.poster, Image::Ready { .. })
    }

    pub fn poster<'a, Message: 'a>(
        &'a self,
        width: impl Into<Length>,
        height: impl Into<Length>,
    ) -> Element<'a, Message> {
        let view = move |handle: &Handle| {
            image(handle)
                .border_radius(IMAGE_RADIUS)
                .height(height)
                .width(width)
                .content_fit(ContentFit::Contain)
                .into()
        };

        match &self.poster {
            Image::Ready { allocation, .. } => view(allocation.handle()),
            Image::Loading => empty().into(),
            Image::Default => match DEFAULT_POSTER.as_ref() {
                Some(handle) => view(handle).into(),
                _ => empty().into(),
            },
        }
    }

    pub fn backdrop<'a, Message: 'a>(
        &'a self,
        width: impl Into<Length>,
        height: impl Into<Length>,
    ) -> Element<'a, Message> {
        match &self.backdrop {
            Some(handle) => image(handle)
                .height(height)
                .width(width)
                .content_fit(ContentFit::Cover)
                .into(),
            None => container(empty())
                .height(height)
                .width(width)
                .style(styles::container::dark)
                .into(),
        }
    }

    pub fn task(&mut self, task: ThumbnailTaskKind, now: Instant) {
        match task {
            ThumbnailTaskKind::Samples { main, accent } => {
                self.sample_color = Some(main);
                self.sample_text = Some(accent);
            }
            ThumbnailTaskKind::Image(Ok(allocation)) => {
                self.poster = Image::Ready {
                    allocation,
                    fade_in: fade_in(now),
                };
            }
            ThumbnailTaskKind::Image(Err(error)) => {
                tracing::error!("Show Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn card<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(ShowId) -> Message + 'a,
        on_select: impl Fn(ShowId) -> Message + 'a,
        on_hover: impl Fn(ShowId, bool) -> Message + 'a,
        on_play: impl Fn(ShowId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let background_inter = self.background.interpolate(0.0, 1.0, now);
        let icon_inter = self.icon.interpolate(0.0, 1.0, now);

        let sample = self.sample_text;
        let seasons = self.item.seasons;
        let seasons = sized_medium(
            format!("{} season{}", seasons, if seasons > 1 { "s" } else { "" }),
            H8,
        )
        .style(move |theme: &iced::Theme| {
            if sample.is_some() {
                text::Style { color: sample }
            } else {
                text::Style {
                    color: Some(theme.palette().primary.strong.text),
                }
            }
        });
        let overlay = card_overlay(
            self.item.as_ref(),
            on_add,
            on_play,
            self.sample_text,
            background_inter,
            icon_inter,
            seasons,
        );

        let card = Card {
            sample_color: self.sample_color,
            background_inter,
            selected: self.selected,
            item: self.item.id,
            poster: &self.poster,
            title: card_title(self.item.name(), self.hovered),
            details: Some(card_details(self.item.rating(), self.item.release_year())),
            overlay: Some(overlay),
            float_anim: Some(&self.float),
        };

        let on_select = move |arg: ShowId| Some((on_select)(arg));

        card.view(now, on_select, on_hover)
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(ShowId) -> Message + 'a,
        on_select: impl Fn(ShowId) -> Message + 'a,
        on_hover: impl Fn(ShowId, bool) -> Message + 'a,
        on_play: impl Fn(ShowId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let seasons = self.item.seasons;
        let seasons = format!("{} season{}", seasons, if seasons > 1 { "s" } else { "" });
        let unique = h7(seasons);

        let background_inter = self.background.interpolate(0.0, 1.0, now);
        let icon_inter = self.icon.interpolate(0.0, 1.0, now);
        let list = List {
            selected: self.selected,
            poster: &self.poster,
            item: self.item.id,
            title: list_title(self.item.name(), self.hovered),
            ratings: Some(ratings(self.item.rating(), true)),
            synopsis: Some(synopsis(self.item.synopsis())),
            bottom: Some(list_bottom(
                self.item.id,
                self.item.progress(),
                self.item.duration_full(),
                unique,
                on_add,
            )),
            overlay: Some(list_overlay(icon_inter, background_inter)),
        };

        list.view(now, on_select, on_hover, on_play)
    }

    pub fn compact<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(ShowId) -> Message + 'a,
        on_select: impl Fn(ShowId) -> Message + 'a,
        on_hover: impl Fn(ShowId, bool) -> Message + 'a,
        on_play: impl Fn(ShowId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let id = self.item.id;

        let compact = Compact {
            selected: self.selected,
            poster: &self.poster,
            item: id,
            title: compact_title(self.item.name(), self.hovered),
            ratings: ratings(self.item.rating(), false),
            progress: Some(compact_progress(self.item.progress())),
            duration: Some(compact_duration(self.item.duration_short())),
            recent: Some(compact_recent(self.item.recent_short())),
        };

        compact.view(now, (on_add)(id), (on_select)(id), on_hover, (on_play)(id))
    }
}
