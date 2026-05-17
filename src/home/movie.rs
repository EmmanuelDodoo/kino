use super::{HomeMessage, PageKind, ViewMessage, shared::*};
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
        bottom_center, button, center_x, column, container, image, image::Handle, responsive, row,
        rule, scrollable, space, stack, text,
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
}

#[derive(Debug, Clone)]
pub struct MoviePageMessage {
    pub id: MovieId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct MoviePage {
    pub id: MovieId,
    pub tab: Tab,
}

impl MoviePage {
    pub fn new(id: MovieId) -> Self {
        Self {
            id,
            tab: Tab::Items,
        }
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
        }
    }

    pub fn overlay<'a>(
        &self,
        mut memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
        movie: &'a MovieItem,
        video: Option<&'a VideoInfo>,
        audio: Option<&'a Audio>,
        subtitle: Option<&'a Subtitle>,
    ) -> Element<'a, MoviePageMessage> {
        let id = self.id;
        let size = P;
        let separator = || Element::from(text("•").line_height(1.0).size(H7));

        let img = {
            let img = responsive(move |size| {
                let img_height = size.height * 0.85;
                let ratio = 2.0 / 3.0;
                movie.poster(img_height * ratio, img_height)
            })
            .width(Length::Shrink);

            img
        };

        let header = {
            let vert = || container(rule::vertical(2.0)).height(H8).clip(true);

            let title = sized_bold(movie.item.name(), H3)
                .width(Length::FillPortion(2))
                .height(32);

            let duration = sized_medium(movie.item.duration_short(), H7);
            let rating = ratings_short(movie.item.rating());
            let release = sized_medium(movie.item.release_year(), H7);

            let details = row!(release, vert(), duration, vert(), rating)
                .spacing(8)
                .align_y(Vertical::Center);

            let mut tags = vec![];
            let movie_tags = movie.item.tags.iter().take(4);
            let tag_len = movie_tags.len();

            for (i, tag) in movie_tags.enumerate() {
                let tag = text(tag).size(H8).font(bold_italic_font());

                tags.push(Element::from(tag));

                if i < tag_len - 1 {
                    tags.push(separator())
                }
            }

            let tags = row(tags).spacing(6).align_y(Vertical::Center);
            column!(tags, title, details).spacing(4.0)
        };

        let actions = {
            let size = H2;
            let play = button(icon(PLAY).size(size))
                .style(styles::button::text_primary)
                .padding(0)
                .on_press(MoviePageMessage {
                    id,
                    message: Message::Play,
                });

            let collection = button(icon(ADD_COLLECTION).size(size))
                .style(styles::button::text)
                .padding(0)
                .on_press(MoviePageMessage {
                    id,
                    message: Message::AddCollection,
                });

            let config = button(icon(VIDEO_CONFIG).size(size / typo::RATIO))
                .style(styles::button::text)
                .padding(0)
                .on_press(MoviePageMessage {
                    id,
                    message: Message::Edit,
                });

            row!(play, collection, config)
                .align_y(Vertical::Center)
                .spacing(16)
        };

        let header = row!(
            header,
            container(actions)
                .align_x(Horizontal::Right)
                .width(Length::Fill)
        )
        .align_y(Vertical::Center)
        .spacing(4);

        let overview = {
            let synopsis = regular(movie.item.synopsis());

            container(scrollable(synopsis).spacing(4.0))
                .max_width(750)
                .max_height(500)
        };

        let info = {
            let info = |value: String| sized_medium(value, size / typo::RATIO);

            let video = video.map(|video| {
                let title = sized_medium("Video", size);

                let resolution =
                    (video.height > 0).then(|| info(format!("Resolution: {}", video.resolution())));

                let codec = video
                    .codec
                    .as_deref()
                    .map(|codec| info(format!("Codec: {codec}")));

                let framerate = (video.framerate > 0.0)
                    .then(|| info(format!("Framerate: {:.0}", video.framerate)));

                let info = column!(resolution, codec, framerate)
                    .spacing(4)
                    .padding(Padding::new(0.0).left(12));

                column!(title, info).spacing(8)
            });

            let audio = audio.map(|audio| {
                let title = sized_medium("Audio", size);

                let codec = audio
                    .codec
                    .as_deref()
                    .map(|codec| info(format!("Codec: {codec}")));

                let lang = audio
                    .lang
                    .as_deref()
                    .map(|lang| info(format!("Language: {lang}")));

                let bitrate = (audio.bitrate > 0).then(|| {
                    info(format!(
                        "Bitrate: {:.2} kbps",
                        audio.bitrate as f32 / 1000.0
                    ))
                });

                let info = column!(lang, codec, bitrate)
                    .spacing(4)
                    .padding(Padding::new(0.0).left(12));

                column!(title, info).spacing(8)
            });

            let subtitle = subtitle.map(|sub| {
                let title = sized_medium("Subtitle", size);

                let name = info(format!("Title: {}", sub.title));
                let lang = info(format!("Language: {}", sub.lang));

                let (kind, path) = match &sub.kind {
                    registry::models::SubtitleKind::Embedded => ("Embedded", None),
                    registry::models::SubtitleKind::Loaded { path, .. } => ("Loaded", Some(path)),
                };

                let kind = info(format!("Kind: {kind}"));

                let path = path.map(|path| {
                    let name = info("Path: ".to_string());
                    let path = trim_path(&path, 3);
                    let path = marquee(path).size(size / typo::RATIO);

                    row!(name, path).spacing(2).align_y(Vertical::Center)
                });

                let info = column!(name, lang, kind, path)
                    .spacing(4)
                    .padding(Padding::new(0.0).left(12));

                column!(title, info).spacing(8)
            });

            column!(video, audio, subtitle).spacing(12)
        };

        let collections = if memberships.peek().is_some() {
            let title = sized_medium("Collections", size);
            let collections = memberships.map(|collection| {
                draw_collection_tab(collection, move |collection| MoviePageMessage {
                    id,
                    message: Message::Goto(collection),
                })
            });

            let content = column(collections).spacing(4.0).width(Length::Fill);
            let collections = container(scrollable(content).spacing(4.0)).max_height(300);

            Some(column!(title, collections).spacing(8))
        } else {
            None
        };

        let data = {
            let title = sized_medium("Statistics", size);

            let content = {
                let added = data("Date Added", movie.item.added_humaized(), CALENDAR);

                let count = data("Watch Count", movie.item.watch_count(), EYE);

                let progress = (movie.item.progress() * 1000.0).round() / 10.0;
                let progress = data("Watch Progress", format!("{:.1}%", progress), HOURGLASS);

                let recent = data(
                    "Recent Watch",
                    movie
                        .item
                        .recent_humanized()
                        .unwrap_or(String::from(" --:--:--")),
                    CALENDAR,
                );

                let comments = data("Comments", movie.item.comments(), NUMBER);

                let duration = data("Duration", movie.item.duration_short(), CLOCK);

                let c1 = column!(added, recent)
                    .align_x(Horizontal::Center)
                    .spacing(20.0);
                let c2 = column!(count, comments)
                    .align_x(Horizontal::Center)
                    .spacing(20.0);
                let c3 = column!(progress, duration)
                    .align_x(Horizontal::Center)
                    .spacing(20.0);

                row!(c1, c2, c3).spacing(40).width(500)
            };

            column!(title, content).spacing(12)
        };

        let content = column!(header, overview, info, collections, data)
            .spacing(40)
            .padding(Padding::ZERO.top(20).right(30.0));

        let content = scrollable(content).auto_scroll(true).spacing(8.0);
        let content = row!(img, content).spacing(40);

        container(content)
            .padding(Padding::new(20.0).left(40.0).right(10.0))
            .height(Length::FillPortion(4))
            .width(Length::Fill)
            .style(|theme| {
                let default = styles::container::bb(theme);
                let background = default
                    .background
                    .map(|background| background.scale_alpha(0.85));

                container::Style {
                    background,
                    ..default
                }
            })
            .into()
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
        let icon_inter = self.icon.interpolate(0.0, 1.0, now);

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
            self.sample_text,
            background_inter,
            icon_inter,
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

        card.view(now, on_select, on_hover)
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(MovieId) -> Message + 'a,
        on_select: impl Fn(MovieId) -> Message + 'a,
        on_hover: impl Fn(MovieId, bool) -> Message + 'a,
        on_play: impl Fn(MovieId) -> Message + 'a,
        unique: impl Fn(&Movie) -> Element<'a, Message>,
    ) -> Element<'a, Message> {
        let unique = unique(&self.item);

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
