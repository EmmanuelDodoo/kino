use super::episode::EpisodeItem;
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
    CollectionId, Episode, EpisodeId, ItemId, Media, Season, SeasonId, ShowId, SimpleCollection,
};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Message {
    AddSelf,
    Add(EpisodeId),
    Hovered(EpisodeId, bool),
    Details(EpisodeId),
    Resume,
    Edit,
    Goto(PageKind),
    GotoCollection(CollectionId),
    Sibbling(ShowId, u16),
    Play(EpisodeId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct SeasonPageMessage {
    pub id: SeasonId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct SeasonPage {
    id: SeasonId,
    tab: Tab,
    scroll: Scroll,
}

impl SeasonPage {
    pub fn boot(season: SeasonId) -> (Self, Task<SeasonPageMessage>) {
        let (new, id) = Self::new(season);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(season: SeasonId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                id: season,
                tab: Tab::Items,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: SeasonPageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Episode(id), is_hovered);

                Some(msg)
            }
            Message::AddSelf => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Season(self.id)));

                Some(msg)
            }
            Message::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Episode(id)));

                Some(msg)
            }
            Message::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Episode(id));

                Some(msg)
            }
            Message::Resume => {
                let msg = HomeMessage::Play(ItemId::Season(self.id));

                Some(msg)
            }
            Message::Play(id) => {
                let msg = HomeMessage::Play(ItemId::Episode(id));

                Some(msg)
            }
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
            Message::Goto(page) => Some(HomeMessage::Goto(page)),
            Message::GotoCollection(id) => Some(HomeMessage::Goto(PageKind::Collection(id))),
            Message::Edit => Some(HomeMessage::OpenView(ViewMessage::SeasonEdit(self.id))),
            Message::Sibbling(show, number) => Some(HomeMessage::GotoSeason(show, number)),
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
        episodes: impl Iterator<Item = &'a EpisodeItem>,
    ) -> Element<'a, Message> {
        let content = episodes.map(|thumbnail| {
            thumbnail.list(
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

    fn compact<'a>(
        &self,
        now: Instant,
        episodes: impl Iterator<Item = &'a EpisodeItem>,
    ) -> Element<'a, Message> {
        let content = episodes.map(|thumbnail| {
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
        episodes: impl Iterator<Item = &'a EpisodeItem>,
    ) -> Element<'a, Message> {
        let content = episodes.map(|episode| {
            episode.card(
                now,
                Message::Add,
                Message::Details,
                Message::Hovered,
                Message::Play,
            )
        });

        let content = grid(content)
            .spacing(12)
            .fluid(EpisodeItem::WIDTH)
            .height(grid::aspect_ratio(EpisodeItem::WIDTH, EpisodeItem::HEIGHT));

        content.into()
    }

    pub fn overlay<'a>(
        &self,
        now: Instant,
        layout: Layout,
        season: &'a SeasonItem,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        episodes: impl Iterator<Item = &'a EpisodeItem>,
    ) -> Element<'a, SeasonPageMessage> {
        let id = self.id;

        let img = page_image(|width, height| season.poster(width, height));

        let header = {
            let episodes = season.item.episodes;
            let details = page_details(
                season.item.rating(),
                season.item.release_year(),
                format!(
                    "{:02} Episode{}",
                    episodes,
                    if episodes > 1 { "s" } else { "" }
                ),
            );

            let tags = {
                let values = [
                    (
                        season.item.show_name.clone(),
                        Some(Message::Goto(PageKind::Show(season.item.show))),
                    ),
                    (format!("Season {:02}", season.item.number), None),
                ];

                page_tags(values)
            };

            page_title(tags, season.item.name(), details)
        };

        let header = page_header(header, Message::Resume, Message::AddSelf, Message::Edit);

        let overview = page_overview(season.item.synopsis());

        let episodes = match layout {
            Layout::Grid => self.grid(now, episodes),
            Layout::List => self.list(now, episodes),
            Layout::Compact => self.compact(now, episodes),
        };

        let collections = page_collections(memberships, Message::GotoCollection);

        let data = page_data(
            season.item.added_humaized(),
            season.item.watch_count(),
            season.item.progress(),
            season.item.recent_humanized(),
            season.item.comments(),
            Some(("Episodes", season.item.episodes, NUMBER)),
        );

        let nav = page_nav(
            (season.item.number > 1).then_some(Message::Sibbling(
                season.item.show,
                season.item.number.saturating_sub(1),
            )),
            Message::Sibbling(season.item.show, season.item.number + 1),
        );

        let content = column!(header, overview, episodes, collections, data, nav).spacing(40);

        let content = page_layout(content, img, &self.scroll, Message::Scroll);

        content.map(move |message| SeasonPageMessage { id, message })
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        layout: Layout,
        season: &'a SeasonItem,
        episodes: impl Iterator<Item = &'a EpisodeItem>,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
    ) -> Element<'a, SeasonPageMessage> {
        let overlay = self.overlay(now, layout, season, memberships, episodes);
        let top = space::vertical();

        let overlay = column!(top, overlay);

        let content = season.backdrop(Length::Fill, Length::FillPortion(3));

        let content = stack![content, overlay];

        content.into()
    }
}

#[derive(Debug, Clone)]
pub struct SeasonItemTask {
    pub id: SeasonId,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
pub struct SeasonItem {
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
    pub item: Box<Season>,
}

impl SeasonItem {
    pub const WIDTH: f32 = CARD_WIDTH;
    pub const HEIGHT: f32 = CARD_HEIGHT;

    pub fn new(season: Season) -> (Self, Task<SeasonItemTask>) {
        let id = season.id;

        let (poster, task) = Image::load(season.poster.as_ref());
        let (task, handle) = task
            .map(move |kind| SeasonItemTask { id, kind })
            .abortable();
        let handle = handle.abort_on_drop();

        //todo: Sample color is not great for current default poster
        let (sample_color, sample_text) = match season.poster() {
            Some(poster) => (
                poster.get_main().map(to_color),
                poster.get_accent().map(to_color),
            ),
            None => (None, None),
        };

        let backdrop = season.backdrop().map(Handle::from_path);

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
            item: Box::new(season),
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
                tracing::error!("Season Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn card<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(SeasonId) -> Message + 'a,
        on_select: impl Fn(SeasonId) -> Message + 'a + Clone,
        on_hover: impl Fn(SeasonId, bool) -> Message + 'a + Clone,
        on_play: impl Fn(SeasonId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let background_inter = self.background.interpolate(0.0, 1.0, now);
        let on_select = move |arg: SeasonId| Some((on_select)(arg));

        let sample = self.sample_text;
        let episodes = self.item.episodes;
        let episodes = sized_medium(
            format!(
                "{} episode{}",
                episodes,
                if episodes > 1 { "s" } else { "" }
            ),
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
            on_select.clone(),
            on_hover.clone(),
            self.sample_color,
            self.sample_text,
            background_inter,
            episodes,
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

        card.view(now, Self::WIDTH, Self::HEIGHT, on_select, on_hover)
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(SeasonId) -> Message + 'a,
        on_select: impl Fn(SeasonId) -> Message + 'a,
        on_hover: impl Fn(SeasonId, bool) -> Message + 'a,
        on_play: impl Fn(SeasonId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let episodes = self.item.episodes;
        let episodes = format!(
            "{} episode{}",
            episodes,
            if episodes > 1 { "s" } else { "" }
        );
        let unique = h7(episodes);

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
        on_add: impl Fn(SeasonId) -> Message + 'a,
        on_select: impl Fn(SeasonId) -> Message + 'a,
        on_hover: impl Fn(SeasonId, bool) -> Message + 'a,
        on_play: impl Fn(SeasonId) -> Message + 'a,
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
