use super::{HomeMessage, PageKind, Scroll, ViewMessage, shared::*};
use crate::utils::icons::*;
use crate::utils::styles;
use crate::utils::typo::*;
use crate::utils::{empty, trim_path, typo};
use devutils::source::SourceSet;
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
    Audio, CollectionId, ItemId, Media, Movie, MovieId, SimpleCollection, Subtitle, VideoInfo,
};
use std::iter::Peekable;
use widgets::marquee;

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    AddCollection,
    Edit,
    Goto(CollectionId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct MoviePageMessage {
    pub id: MovieId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct MoviePage {
    pub id: MovieId,
    scroll: Scroll,
}

impl MoviePage {
    pub fn boot(show: MovieId) -> (Self, Task<MoviePageMessage>) {
        let (new, id) = Self::new(show);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(show: MovieId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { id: show, scroll }, id)
    }

    pub fn update(&mut self, message: MoviePageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Play => {
                let msg = HomeMessage::Play(ItemId::Movie(self.id));
                Some(msg)
            }
            Message::AddCollection => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(self.id)));
                Some(msg)
            }
            Message::Goto(id) => {
                let msg = HomeMessage::Goto(PageKind::Collection(id));
                Some(msg)
            }
            Message::Edit => Some(HomeMessage::OpenView(ViewMessage::MovieEdit(self.id))),
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
        }
    }

    pub fn overlay<'a>(
        &self,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        movie: &'a MovieItem,
        video: Option<&'a VideoInfo>,
        audio: Option<&'a Audio>,
        subtitle: Option<&'a Subtitle>,
    ) -> Element<'a, MoviePageMessage> {
        let id = self.id;

        let img = page_image(|width, height| movie.poster(width, height));

        let header = {
            let details = page_details(
                movie.item.rating(),
                movie.item.release_year(),
                movie.item.duration_short(),
            );

            let tags = {
                let values = movie.item.tags.iter().map(|tag| (tag, None)).take(4);

                page_tags(values)
            };

            page_title(tags, movie.item.name(), details)
        };

        let header = page_header(header, Message::Play, Message::AddCollection, Message::Edit);

        let overview = page_overview(movie.item.synopsis());

        let info = page_video(video, audio, subtitle);

        let collections = page_collections(memberships, Message::Goto);

        let data = page_data(
            movie.item.added_humaized(),
            movie.item.watch_count(),
            movie.item.progress(),
            movie.item.recent_humanized(),
            movie.item.comments(),
            Some(("Duration", movie.item.duration_short(), CLOCK)),
        );

        let content = column!(header, overview, info, collections, data).spacing(40);

        let content = page_layout(content, img, &self.scroll, Message::Scroll);

        content.map(move |message| MoviePageMessage { id, message })
    }

    pub fn view<'a>(
        &self,
        movie: &'a MovieItem,
        memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        video: Option<&'a VideoInfo>,
        audio: Option<&'a Audio>,
        subtitle: Option<&'a Subtitle>,
    ) -> Element<'a, MoviePageMessage> {
        let overlay = self.overlay(memberships, movie, video, audio, subtitle);
        let top = space::vertical();

        let overlay = column!(top, overlay);

        let content = movie.backdrop(Length::Fill, Length::FillPortion(3));

        let content = stack![content, overlay];

        content.into()
    }

    pub fn show_tools(&self) -> bool {
        false
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}

#[derive(Debug, Clone)]
pub struct MovieItemTask {
    pub id: MovieId,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
pub struct MovieItem {
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
    pub item: Box<Movie>,
}

impl MovieItem {
    pub const WIDTH: f32 = CARD_WIDTH;
    pub const HEIGHT: f32 = CARD_HEIGHT;

    pub fn new(movie: Movie) -> (Self, Task<MovieItemTask>) {
        let id = movie.id;

        let (poster, task) = Image::load(movie.poster.as_ref());
        let (task, handle) = task.map(move |kind| MovieItemTask { id, kind }).abortable();
        let handle = handle.abort_on_drop();

        //todo: Sample color is not great for current default poster
        let (sample_color, sample_text) = match movie.poster() {
            Some(poster) => (
                poster.get_main().map(to_color),
                poster.get_accent().map(to_color),
            ),
            None => (None, None),
        };

        let backdrop = movie.backdrop().map(Handle::from_path);

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
            item: Box::new(movie),
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
                tracing::error!("Movie Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn card<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(MovieId) -> Message + 'a,
        on_select: impl Fn(MovieId) -> Message + 'a,
        on_hover: impl Fn(MovieId, bool) -> Message + 'a,
        on_play: impl Fn(MovieId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let background_inter = self.background.interpolate(0.0, 1.0, now);

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

        let on_select = move |arg: MovieId| Some((on_select)(arg));

        card.view(now, Self::WIDTH, Self::HEIGHT, on_select, on_hover)
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(MovieId) -> Message + 'a,
        on_select: impl Fn(MovieId) -> Message + 'a,
        on_hover: impl Fn(MovieId, bool) -> Message + 'a,
        on_play: impl Fn(MovieId) -> Message + 'a,
    ) -> Element<'a, Message> {
        let unique = {
            use iced::font::{Font, Weight};

            let release = text(self.item.release_year()).size(H8).font(Font {
                weight: Weight::Semibold,
                ..Default::default()
            });
            let icon = icon(CALENDAR).size(H7);

            row!(icon, release).align_y(Vertical::Center).spacing(3.0)
        };

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
        on_add: impl Fn(MovieId) -> Message + 'a,
        on_select: impl Fn(MovieId) -> Message + 'a,
        on_hover: impl Fn(MovieId, bool) -> Message + 'a,
        on_play: impl Fn(MovieId) -> Message + 'a,
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
