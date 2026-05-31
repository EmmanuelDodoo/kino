use super::{HomeMessage, PageKind, Scroll, ViewMessage, shared::*};
use crate::utils::icons::*;
use crate::utils::styles;
use crate::utils::typo::*;
use crate::utils::{empty, trim_path, typo};
use devutils::source::SourceSet;
use iced::widget::Space;
use iced::{
    Animation, Color, ContentFit, Element, Length, Padding, Shadow, Task,
    alignment::{Horizontal, Vertical},
    task,
    time::Instant,
    widget::{
        self, bottom_center, button, center_x, column, container, image, image::Handle, operation,
        responsive, row, rule, scrollable, space, stack, text,
    },
};
use registry::models::{
    Audio, CollectionId, Episode, EpisodeId, ItemId, Media, SeasonId, ShowId, SimpleCollection,
    Subtitle, VideoInfo,
};
use std::iter::Peekable;
use widgets::marquee;

#[derive(Debug, Clone)]
pub enum Message {
    AddCollection,
    Play,
    Edit,
    Goto(PageKind),
    GotoCollection(CollectionId),
    Sibbling(SeasonId, u16),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct EpisodePageMessage {
    pub id: EpisodeId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct EpisodePage {
    pub id: EpisodeId,
    scroll: Scroll,
}

impl EpisodePage {
    pub fn boot(show: EpisodeId) -> (Self, Task<EpisodePageMessage>) {
        let (new, id) = Self::new(show);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(show: EpisodeId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { id: show, scroll }, id)
    }

    pub fn update(&mut self, message: EpisodePageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Play => {
                let msg = HomeMessage::Play(ItemId::Episode(self.id));
                Some(msg)
            }
            Message::AddCollection => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(self.id.into()));
                Some(msg)
            }
            Message::Edit => {
                let msg = HomeMessage::OpenView(ViewMessage::EpisodeEdit(self.id));
                Some(msg)
            }
            Message::Goto(page) => Some(HomeMessage::Goto(page)),
            Message::GotoCollection(collection) => {
                Some(HomeMessage::Goto(PageKind::Collection(collection)))
            }
            Message::Sibbling(season, number) => Some(HomeMessage::GotoEpisode(season, number)),
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
        }
    }

    fn overlay<'a>(
        &self,
        episode: &'a EpisodeItem,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        video: Option<&'a VideoInfo>,
        audio: Option<&'a Audio>,
        subtitle: Option<&'a Subtitle>,
    ) -> Element<'a, EpisodePageMessage> {
        let id = self.id;
        let img = page_image(|width, height| episode.poster(width, height));

        let header = {
            let details = page_details(
                episode.item.rating(),
                episode.item.release_year(),
                episode.item.duration_short(),
            );

            let top = {
                let values = [
                    (
                        episode.item.show_name.clone(),
                        Some(Message::Goto(PageKind::Show(episode.item.show))),
                    ),
                    (
                        format!("Season {:02}", episode.item.season_number),
                        Some(Message::Goto(PageKind::Season(episode.item.season))),
                    ),
                    (format!("Episode {:02}", episode.item.number), None),
                ];

                page_tags(values)
            };

            page_title(top, episode.item.name(), details, episode.item.status)
        };

        let header = page_header(header, Message::Play, Message::AddCollection, Message::Edit);

        let overview = page_overview(episode.item.synopsis());

        let info = page_video(video, audio, subtitle);

        let collections = page_collections(memberships, Message::GotoCollection);

        let data = page_data(
            episode.item.added_humaized(),
            episode.item.watch_count(),
            episode.item.progress(),
            episode.item.recent_humanized(),
            episode.item.comments(),
            Some(("Duration", episode.item.duration_short(), CLOCK)),
        );

        let nav = page_nav(
            (episode.item.number > 1).then_some(Message::Sibbling(
                episode.item.season,
                episode.item.number.saturating_sub(1),
            )),
            Message::Sibbling(episode.item.season, episode.item.number + 1),
        );

        let content = column!(header, overview, info, collections, data, nav)
            .spacing(40)
            .padding(Padding::ZERO.top(20).right(30.0));

        let content = page_layout(content, img, &self.scroll, Message::Scroll);

        content.map(move |message| EpisodePageMessage { id, message })
    }

    pub fn view<'a>(
        &self,
        episode: &'a EpisodeItem,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        video: Option<&'a VideoInfo>,
        audio: Option<&'a Audio>,
        subtitle: Option<&'a Subtitle>,
    ) -> Element<'a, EpisodePageMessage> {
        let overlay = self.overlay(episode, memberships, video, audio, subtitle);
        let top = space::vertical();

        let overlay = column!(top, overlay);

        let content = episode.backdrop(Length::Fill, Length::FillPortion(3));

        let content = stack![content, overlay];

        content.into()
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}

#[derive(Debug, Clone)]
pub struct EpisodeItemTask {
    pub id: EpisodeId,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
pub struct EpisodeItem {
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
    pub item: Box<Episode>,
}

impl EpisodeItem {
    pub const HEIGHT: f32 = CARD_HEIGHT * 0.75;
    pub const WIDTH: f32 = Self::HEIGHT * 3.0 / 2.0;

    pub fn new(episode: Episode) -> (Self, Task<EpisodeItemTask>) {
        let id = episode.id;

        let (poster, task) = Image::load(episode.poster.as_ref());
        let (task, handle) = task
            .map(move |kind| EpisodeItemTask { id, kind })
            .abortable();
        let handle = handle.abort_on_drop();

        let (sample_color, sample_text) = match episode.poster() {
            Some(poster) => (
                poster.get_main().map(to_color),
                poster.get_accent().map(to_color),
            ),
            None => (None, None),
        };

        let backdrop = episode.backdrop().map(Handle::from_path);

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
            item: Box::new(episode),
        };

        (new, task)
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let poster = match &self.poster {
            Image::Shown { fade_in, .. } => fade_in.is_animating(now),
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
            Image::Shown { allocation, .. } => view(allocation.handle()),
            Image::Ready { allocation, .. } => view(allocation.handle()),
            Image::Loading(_) => empty().into(),
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

    pub fn fade_in(&mut self, shown: bool, now: Instant) {
        self.poster.fade_in(shown, now);
    }

    pub fn task(&mut self, task: ThumbnailTaskKind, now: Instant) {
        match task {
            ThumbnailTaskKind::Samples { main, accent } => {
                self.sample_color = Some(main);
                self.sample_text = Some(accent);
            }
            ThumbnailTaskKind::Image(Ok(allocation)) => {
                let mut poster = Image::Ready { allocation };
                if matches!(&self.poster, Image::Loading(true)) {
                    poster.fade_in(true, now);
                }

                self.poster = poster;
            }
            ThumbnailTaskKind::Image(Err(error)) => {
                tracing::error!("Episode Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn card<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(EpisodeId) -> Message + 'a,
        on_select: impl Fn(EpisodeId) -> Message + 'a + Clone,
        on_hover: impl Fn(EpisodeId, bool) -> Message + 'a + Clone,
        on_show: impl Fn(EpisodeId, bool) -> Message + 'a,
        on_play: impl Fn(EpisodeId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let background_inter = self.background.interpolate(0.0, 1.0, now);
        let on_select = move |arg: EpisodeId| Some((on_select)(arg));

        let sample = self.sample_text;
        let duration =
            sized_medium(self.item.duration_full(), H8).style(move |theme: &iced::Theme| {
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
            duration,
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

        card.view(now, Self::WIDTH, Self::HEIGHT, on_select, on_hover, on_show)
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(EpisodeId) -> Message + 'a,
        on_select: impl Fn(EpisodeId) -> Message + 'a,
        on_hover: impl Fn(EpisodeId, bool) -> Message + 'a,
        on_show: impl Fn(EpisodeId, bool) -> Message + 'a,
        on_play: impl Fn(EpisodeId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let unique = empty();

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
                self.item.status,
                self.item.progress(),
                self.item.duration_full(),
                unique,
                on_add,
            )),
            overlay: Some(list_overlay(icon_inter, background_inter)),
        };

        list.view(now, on_select, on_hover, on_show, on_play)
    }

    pub fn compact<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(EpisodeId) -> Message + 'a,
        on_select: impl Fn(EpisodeId) -> Message + 'a,
        on_hover: impl Fn(EpisodeId, bool) -> Message + 'a,
        on_show: impl Fn(EpisodeId, bool) -> Message + 'a,
        on_play: impl Fn(EpisodeId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let id = self.item.id;

        let compact = Compact {
            selected: self.selected,
            poster: &self.poster,
            item: id,
            title: compact_title(self.item.name(), self.hovered),
            ratings: ratings(self.item.rating(), false),
            progress: Some(compact_progress(self.item.status, self.item.progress())),
            duration: Some(compact_duration(self.item.duration_short())),
            recent: Some(compact_recent(self.item.recent_short())),
        };

        compact.view(
            now,
            on_hover,
            on_show,
            (on_add)(id),
            (on_select)(id),
            (on_play)(id),
        )
    }
}
