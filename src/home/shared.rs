use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Scroll, empty, styles, tooltip, trim_path, typo};
use core::variants;
use devutils::image_ops::{collage, sample_complement};
use iced::{
    Color, ContentFit, Element, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    mouse, task,
    time::{Duration, Instant},
    widget::{
        self, button, center, column, container, image,
        image::{Allocation, Handle},
        markdown, mouse_area, responsive, row, rule, scrollable, sensor, space, stack, text,
        tooltip as tp,
    },
};
use registry::models::{
    self, Audio, Collection, CollectionId, ItemId, Media, SearchItem, SimpleCollection, Subtitle,
    VideoInfo,
};
use std::iter::Peekable;
use std::sync::LazyLock;
use widgets::marquee;
pub static DEFAULT_POSTER: LazyLock<Option<Handle>> =
    LazyLock::new(|| devutils::image_ops::default_poster().map(to_handle));

pub const CARD_HEIGHT: f32 = 450.0;
pub const CARD_WIDTH: f32 = CARD_HEIGHT * 2.0 / 3.0;
pub const LIST_HEIGHT: f32 = 150.0;
pub const LIST_WIDTH: f32 = LIST_HEIGHT * 5.5 / 10.0;
pub const IMAGE_RADIUS: f32 = 7.0;
const SELECTED_WIDTH: f32 = 2.0;

variants! {
#[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub enum Tab {
        #[default]
        Items,
        Data,
        // Comments,
        Collections,
    }
}

impl Tab {
    pub fn to_str(self, item: &str) -> &str {
        match self {
            Self::Items => item,
            Self::Data => "Data",
            Self::Collections => "Collections",
            // Self::Comments => "Comments",
        }
    }
}

pub fn title<'a, Message: 'a + Clone>(name: &'a str) -> Element<'a, Message> {
    sized_medium(name, H4).into()
}

pub fn tab_synopsis<'a, Message: 'a + Clone>(synopsis: &'a str) -> Element<'a, Message> {
    let synopsis = regular(synopsis);

    scrollable(column!(synopsis).spacing(8.0))
        .spacing(4.0)
        .into()
}

pub fn duration<'a, Message: 'a>(duration: String) -> Element<'a, Message> {
    let duration = sized_medium(duration, H8);
    let icon = icon(HOURGLASS).size(H8);

    row!(icon, duration)
        .align_y(Vertical::Center)
        .spacing(1.0)
        .into()
}

pub fn ratings<'a, Message: 'a>(rating: Option<f32>, show_text: bool) -> Element<'a, Message> {
    let size = H7;
    let color = |theme: &Theme| -> text::Style {
        let color = theme.palette().primary.strong.color;
        text::Style { color: Some(color) }
    };

    match rating {
        Some(value) => {
            let rating = (value * 10.0).round() / 10.0;

            let stars = (rating.trunc() as u8).clamp(0, 5);
            let rem = 5 - stars;
            let frac = rating.fract() >= 0.5;
            let unstars = if frac { rem.saturating_sub(1) } else { rem };
            let frac = rem - unstars;

            let stars = (0..stars).map(|_| Element::from(icon(STAR).size(size).style(color)));
            let frac = (0..frac).map(|_| Element::from(icon(HALF_STAR).size(size).style(color)));
            let unstars = (0..unstars).map(|_| Element::from(icon(UNSTAR).size(size).style(color)));

            let ratings = row(stars.chain(frac).chain(unstars))
                .spacing(2.0)
                .align_y(Vertical::Center);

            let ratings = if show_text {
                let text = sized_medium(format!("{rating:.1}"), H8)
                    .style(color)
                    .line_height(1.0);
                row!(text, ratings)
            } else {
                row!(ratings)
            }
            .align_y(Vertical::Center)
            .spacing(6.0);

            ratings.into()
        }
        None => row((0..5).map(|_| Element::from(icon(UNSTAR).size(size).style(color))))
            .align_y(Vertical::Center)
            .spacing(2.0)
            .into(),
    }
}

pub fn ratings_short<'a, Message: 'a>(rating: Option<f32>) -> Element<'a, Message> {
    let size = H7;
    let color = |theme: &Theme| -> text::Style {
        let color = theme.palette().primary.strong.color;
        text::Style { color: Some(color) }
    };

    let temp = |rating: f32, star: widget::Text<'a>| {
        row!(
            sized_medium(format!("{rating:.1}"), H8),
            star.size(size).style(color)
        )
        .align_y(Vertical::Center)
        .spacing(2.0)
    };

    match rating {
        Some(rating) => {
            let rating = (rating * 10.0).round() / 10.0;
            let star = if rating >= 4.5 {
                icon(STAR)
            } else {
                icon(HALF_STAR)
            };

            temp(rating, star)
        }
        None => temp(0.0, icon(UNSTAR)),
    }
    .into()
}

fn progress_icon(progress: f32) -> char {
    match progress {
        ..0.15 => PROGRESS_10,
        0.15..0.3 => PROGRESS_20,
        0.3..0.5 => PROGRESS_40,
        0.5..0.7 => PROGRESS_60,
        0.7..0.85 => PROGRESS_80,
        x if x < 1.0 => PROGRESS_90,
        _ => PROGRESS_100,
    }
}

fn status_progress<'a, Message: 'a>(
    status: models::media::Status,
    progress: f32,
    color: Option<Color>,
    primary: bool,
) -> Element<'a, Message> {
    let (value_icon, value) = match status {
        models::media::Status::Archived => (ARCHIVED, medium("Archived")),
        _ => {
            let value = (progress * 1000.0).round() / 10.0;
            (progress_icon(progress), mono_bold(format!("{value}%")))
        }
    };

    let text = value.size(H8).style(move |theme: &Theme| {
        if color.is_some() {
            text::Style { color }
        } else {
            text::Style {
                color: if primary {
                    Some(theme.palette().primary.strong.text)
                } else {
                    None
                },
            }
        }
    });

    let icon = icon(value_icon).size(H6).style(move |theme: &Theme| {
        if color.is_some() {
            text::Style { color }
        } else {
            text::Style {
                color: if primary {
                    Some(theme.palette().primary.strong.text)
                } else {
                    None
                },
            }
        }
    });

    row!(icon, text)
        .spacing(3.0)
        .align_y(Vertical::Center)
        .into()
}

pub fn add_labelled<'a, T, Message: 'a + Clone>(
    id: T,
    on_press: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message> {
    let icon = icon(BOOKMARK).size(P);

    let label = sized_medium("Add to collection", H8);

    mouse_area(row!(icon, label).align_y(Vertical::Center).spacing(6.0))
        .interaction(mouse::Interaction::Pointer)
        .on_press((on_press)(id))
        .into()
}

pub fn synopsis<'a, Message: 'a>(synopsis: &'a str) -> Element<'a, Message> {
    container(regular(synopsis).ellipsis(text::Ellipsis::End))
        .clip(true)
        .max_height(44.0)
        .into()
}

pub fn data<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    value: impl text::IntoFragment<'a>,
    unicode: char,
) -> Element<'a, Message> {
    let size = H7;
    let color = |theme: &Theme| text::Style {
        color: Some(theme.palette().primary.base.color),
    };

    let value = h7(value).style(color);
    let icon = icon(unicode).size(size).style(color);

    let value = row!(icon, value).spacing(2.0).align_y(Vertical::Center);
    let label = sized_medium(label, H8);

    column!(value, label)
        .align_x(Horizontal::Center)
        .spacing(0.0)
        .into()
}

pub fn data_tab<'a, Message: 'a + Clone, T: Media>(
    media: &T,
    width: f32,
    on_rename: Message,
    on_refetch: Message,
    on_remove: Message,
    on_synopsis: Message,
    on_tmdb: (Message, bool),
) -> Element<'a, Message> {
    let sts = {
        let icon = icon(STATS).size(P);
        let label = medium("Statistics");

        row!(icon, label).spacing(4.0).align_y(Vertical::Center)
    };

    let duration = data("Duration", media.duration_short(), CLOCK);

    let rating = {
        let rating = media.rating().unwrap_or_default();

        (rating * 100.0).round() / 100.0
    };
    let rating = data("Rating", format!("{}", rating), STAR);

    let comments = data("Comments", media.comments(), NUMBER);

    let release = data("Release Date", media.release_my(), CALENDAR);

    let added = data("Date Added", media.added_humaized(), CALENDAR);

    let count = data("Watch Count", media.watch_count(), EYE);

    let progress = (media.progress() * 1000.0).round() / 10.0;
    let progress = data("Watch Progress", format!("{:.1}%", progress), HOURGLASS);

    let recent = data(
        "Recent Watch",
        media
            .recent_humanized()
            .unwrap_or(String::from(" --:--:--")),
        CALENDAR,
    );

    let r1 = row!(
        duration,
        space::horizontal(),
        release,
        space::horizontal(),
        count,
        space::horizontal(),
        progress
    )
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let r2 = row!(
        rating,
        space::horizontal(),
        added,
        space::horizontal(),
        comments,
        space::horizontal(),
        recent,
    )
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let data = column!(r1, r2).spacing(20.0);

    let data = column!(sts, data).spacing(12.0);

    let ops = {
        let label = {
            let icon = icon(EDIT).size(P);
            let label = medium("Edit");

            row!(icon, label).spacing(4.0).align_y(Vertical::Center)
        };

        let ops = {
            let size = H7;
            let spacing = 4.0;
            let tp = tp::Position::Top;

            let rename = row!(icon(RENAME).size(size), sized_medium("Name", size))
                .spacing(spacing)
                .align_y(Vertical::Center);
            let rename = button(rename)
                .style(styles::button::subtler)
                .on_press(on_rename);

            let synopsis = row!(icon(RENAME).size(size), sized_medium("Overview", size))
                .spacing(spacing)
                .align_y(Vertical::Center);
            let synopsis = button(synopsis)
                .style(styles::button::subtler)
                .on_press(on_synopsis);

            let tmdb: Element<'_, Message> = if on_tmdb.1 {
                let tmdb = sized_medium("TMDB ID", size);

                tooltip(
                        button(tmdb)
                            .style(styles::button::subtler)
                            .on_press(on_tmdb.0),
                        "TMDB id can easily be located as part of the movie/show url. Eg `1233413` from https://www.themoviedb.org/movie/1233413-sinners",
                        tp,
                    )
                    .into()
            } else {
                let tmdb = sized_medium("Number", size);

                tooltip(
                    button(tmdb)
                        .style(styles::button::subtler)
                        .on_press(on_tmdb.0),
                    "Manually set the season/episode number",
                    tp,
                )
                .into()
            };

            let refetch = row!(icon(REFRESH).size(size), sized_medium("Refetch", size))
                .spacing(spacing)
                .align_y(Vertical::Center);
            let refetch = button(refetch)
                .style(styles::button::subtler)
                .on_press(on_refetch);
            let refetch = tooltip(refetch, "Refetch from TMDB", tp);

            let remove = row!(icon(DELETE).size(size), sized_medium("Delete", size))
                .spacing(spacing)
                .align_y(Vertical::Center);
            let remove = button(remove)
                .style(styles::button::danger)
                .on_press(on_remove);

            row!(rename, synopsis, tmdb, refetch, remove).spacing(8.0)
        };

        column!(label, ops).spacing(10.0)
    };

    let content = column!(data, ops).spacing(40.0);

    content.width(width).into()
}

pub fn draw_collection_tab<'a, Message: 'a + Clone>(
    collection: &'a SimpleCollection,
    on_press: impl Fn(CollectionId) -> Message + 'a,
) -> Element<'a, Message> {
    let size = H7;
    let unicode = Icon::new(collection.icon).unicode();
    let icon = icon(unicode).size(size);
    let text = container(medium(&collection.name).size(size))
        .max_height(48.0)
        .max_width(275);

    button(
        row!(icon, text)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .spacing(8.0),
    )
    .padding([8, 12])
    .on_press((on_press)(collection.id))
    .style(move |theme, status| {
        let default = styles::button::text(theme, status);

        let border = default.border.rounded(IMAGE_RADIUS);

        button::Style { border, ..default }
    })
    .into()
}

pub fn float<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    interpolation: f32,
    color: Option<Color>,
) -> Element<'a, Message> {
    use iced::{Color, Shadow};

    let blur_radius = interpolation * 20.0;
    let scale = 1.0 + (0.05 * interpolation);
    let color = color.unwrap_or(Color::BLACK).scale_alpha(interpolation);

    widget::float(content)
        .scale(scale)
        // .translate(move |bounds, viewport| {
        //     bounds.zoom(1.05).offset(&viewport.shrink(5)) * interpolation
        // })
        .style(move |_theme| widget::float::Style {
            shadow: Shadow {
                color,
                blur_radius,
                ..Shadow::default()
            },
            shadow_border_radius: IMAGE_RADIUS.into(),
        })
        .into()
}

#[derive(Debug, Clone)]
pub enum ThumbnailTaskKind {
    Samples { main: Color, accent: Color },
    Image(Result<Allocation, image::Error>),
}

#[derive(Debug, Clone)]
pub enum Image {
    Ready {
        allocation: Allocation,
    },
    Shown {
        allocation: Allocation,
        fade_in: Animation<bool>,
    },
    Loading(bool),
    Default,
}

impl Image {
    pub fn load(image: Option<&models::image::Image>) -> (Self, Task<ThumbnailTaskKind>) {
        match image {
            Some(poster) => {
                let path = poster.path.display().to_string();

                let sample = if poster.main.is_none() {
                    Task::future(async move {
                        sample_complement(&path).map(|(a, b)| (to_color(a), to_color(b)))
                    })
                    .and_then(move |(main, accent)| {
                        Task::done(ThumbnailTaskKind::Samples { main, accent })
                    })
                } else {
                    Task::none()
                };

                let images = image::allocate(poster.path.clone()).map(ThumbnailTaskKind::Image);

                (Self::Loading(false), Task::batch([sample, images]))
            }
            None => (Self::Default, Task::none()),
        }
    }

    pub fn fade_in(&mut self, shown: bool, now: Instant) {
        match std::mem::replace(self, Self::Loading(shown)) {
            Image::Shown { allocation, .. } if !shown => {
                //
                *self = Image::Ready { allocation }
            }
            Image::Ready { allocation } if shown => {
                *self = Image::Shown {
                    allocation,
                    fade_in: fade_in(now),
                }
            }
            Image::Loading(_) => *self = Image::Loading(shown),
            img => {
                *self = img;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionTask {
    pub id: CollectionId,
    pub kind: CollectionTaskKind,
}

#[derive(Debug, Clone)]
pub enum CollectionTaskKind {
    Image(Result<Allocation, image::Error>),
}

#[derive(Debug, Clone)]
pub struct CollectionThumbnail {
    collage: Image,
    pub collection: Collection,
}

impl CollectionThumbnail {
    pub const HEIGHT: u32 = 250;
    pub const WIDTH: u32 = 250;

    pub const CARD_HEIGHT: f32 = CARD_HEIGHT * 0.85;
    pub const CARD_WIDTH: f32 = Self::CARD_HEIGHT * 0.85;

    pub fn new(collection: Collection) -> (Self, Task<CollectionTask>) {
        let id = collection.id;
        let paths = collection.posters.clone().into_iter().flatten();

        let task = Task::future(async move { collage(paths, Self::WIDTH, Self::HEIGHT) }).and_then(
            move |collage| {
                image::allocate(to_handle(collage)).map(move |res| CollectionTask {
                    id,
                    kind: CollectionTaskKind::Image(res),
                })
            },
        );

        let collage = Image::Loading(false);
        (
            Self {
                collage,
                collection,
            },
            task,
        )
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        match &self.collage {
            Image::Shown { fade_in, .. } => fade_in.is_animating(now),
            _ => false,
        }
    }

    pub fn task(&mut self, task: CollectionTaskKind, now: Instant) {
        match task {
            CollectionTaskKind::Image(Ok(allocation)) => {
                self.collage = Image::Shown {
                    allocation,
                    fade_in: fade_in(now),
                };
            }
            CollectionTaskKind::Image(Err(error)) => {
                tracing::error!("Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn collage<'a, Message: 'a>(&'a self, now: Instant) -> Element<'a, Message> {
        match &self.collage {
            Image::Loading(_) | Image::Default => {
                let len = self.collection.name.len().min(2);
                let name = self.collection.name.get(..len).unwrap_or_default();

                let font = display_font();
                let text = text(name).size(H1 * 2.75).font(font);

                center(text)
                    .height(Self::HEIGHT)
                    .width(Self::WIDTH)
                    .style(move |theme| {
                        let default = styles::container::dark(theme);
                        let border = default.border.rounded(IMAGE_RADIUS);

                        container::Style { border, ..default }
                    })
                    .into()
            }
            Image::Ready { allocation } => image(allocation.handle())
                .border_radius(IMAGE_RADIUS)
                .height(Self::HEIGHT)
                .width(Self::WIDTH)
                .content_fit(ContentFit::Contain)
                .into(),
            Image::Shown {
                allocation,
                fade_in,
            } => image(allocation.handle())
                .border_radius(IMAGE_RADIUS)
                .height(Self::HEIGHT)
                .width(Self::WIDTH)
                .content_fit(ContentFit::Contain)
                .opacity(fade_in.interpolate(0.0, 1.0, now))
                .scale(fade_in.interpolate(1.2, 1.0, now))
                .into(),
        }
    }

    pub fn view<'a, Message: 'a + Clone>(
        &'a self,
        on_select: impl Fn(CollectionId) -> Message + 'a,
        now: Instant,
    ) -> Element<'a, Message> {
        let width = Self::CARD_WIDTH;
        let height = Self::CARD_HEIGHT;
        let padding = [3, 6];

        let name = {
            let title = marquee(&self.collection.name).size(P).font(medium_font());

            container(title)
                .padding(padding)
                .max_height(35.0)
                .clip(true)
        };

        let img: Element<'_, Message> = {
            match &self.collage {
                Image::Ready { allocation } => image(allocation.handle())
                    .border_radius(IMAGE_RADIUS)
                    .width(Self::CARD_WIDTH)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Fill)
                    .into(),
                Image::Shown {
                    allocation,
                    fade_in,
                } => image(allocation.handle())
                    .border_radius(IMAGE_RADIUS)
                    .width(Self::CARD_WIDTH)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Fill)
                    .opacity(fade_in.interpolate(0.0, 1.0, now))
                    .scale(fade_in.interpolate(1.2, 1.0, now))
                    .into(),

                Image::Loading(_) | Image::Default => {
                    let len = self.collection.name.len().min(2);
                    let name = self.collection.name.get(..len).unwrap_or_default();

                    let font = display_font();
                    let text = text(name).size(H1 * 2.60).font(font);

                    center(text)
                        .clip(true)
                        .width(Self::CARD_WIDTH)
                        .height(Length::Fill)
                        .style(move |theme| {
                            let default = styles::container::dark(theme);
                            let border = default.border.rounded(IMAGE_RADIUS);

                            container::Style { border, ..default }
                        })
                        .into()
                }
            }
        };

        let img = container(img).width(width);

        let content = column!(img, name).width(width).height(height).spacing(8);

        let content = button(content)
            .padding(10)
            .style(|theme, status| {
                let default = styles::button::subtlest(theme, status);
                let border = default.border.rounded(IMAGE_RADIUS);

                button::Style { border, ..default }
            })
            .on_press((on_select)(self.collection.id));

        content.into()
    }
}

#[derive(Debug)]
pub struct SearchView {
    pub item: SearchItem,
    snippet: markdown::Content,
    poster: Option<Handle>,
}

impl SearchView {
    pub fn new(item: SearchItem) -> Self {
        let poster = match &item.poster {
            Some(poster) => Some(Handle::from_path(poster)),
            None => DEFAULT_POSTER.clone(),
        };
        Self {
            snippet: markdown::Content::parse(&item.snippet),
            item,
            poster,
        }
    }

    fn poster<'a, Message: 'a>(&self) -> Element<'a, Message> {
        match &self.poster {
            Some(handle) => image(handle)
                .border_radius(IMAGE_RADIUS)
                .width(56)
                .height(56)
                .content_fit(ContentFit::Fill)
                .into(),

            None => container(empty()).style(styles::container::dark).into(),
        }
    }

    pub fn view<'a, Message: 'a + Clone>(
        &'a self,
        theme: &Theme,
        on_play: impl Fn(ItemId) -> Message,
        on_details: impl Fn(ItemId) -> Message,
        on_url: impl Fn(String) -> Message + 'a,
        set_play: bool,
    ) -> Element<'a, Message> {
        fn pair(theme: &Theme) -> Color {
            theme.palette().primary.strong.color
        }

        let separator = || {
            Element::from(text("•").line_height(0.9).size(H5).style(|theme: &Theme| {
                let color = theme.palette().background.strongest.color;
                text::Style { color: Some(color) }
            }))
        };

        let name = {
            let name = marquee(&self.item.name)
                .size(H6)
                .font(mono_bold_font())
                .style(|theme: &Theme| {
                    let color = theme.palette().background.strong.text;
                    text::Style { color: Some(color) }
                })
                .width(Length::Fill);
            container(name).clip(true).max_height(24.0)
        };

        let snippet = {
            let settings = markdown::Settings::with_text_size(H7, theme);

            markdown::view(self.snippet.items(), settings).map(on_url)
        };

        let top = {
            let size = H8;

            let media = {
                let media = match &self.item.id {
                    ItemId::Movie(_) => "#movie",
                    ItemId::Show(_) => "#show",
                    ItemId::Season(_) => "#season",
                    ItemId::Episode(_) => "#episode",
                };

                sized_regular(media, size)
                    .font(bold_italic_font())
                    .style(|theme| {
                        let color = pair(theme);
                        text::Style { color: Some(color) }
                    })
            };

            let has_tags = !self.item.tags.is_empty();
            let tags = {
                let max = 4;
                let mut tags = vec![];
                let tag_len = self.item.tags.len().min(max);

                for (i, tag) in self.item.tags.iter().enumerate().take(max) {
                    let text = sized_regular(tag, size)
                        .font(bold_italic_font())
                        .style(|theme| {
                            let color = pair(theme);
                            text::Style { color: Some(color) }
                        });

                    tags.push(Element::from(text));

                    if i < tag_len - 1 {
                        tags.push(separator())
                    }
                }

                row(tags).spacing(6).align_y(Vertical::Center)
            };

            let vert: Element<'_, Message> = if self.item.tags.is_empty() {
                empty()
            } else {
                container(rule::vertical(2.0)).height(H8).clip(true).into()
            };

            if has_tags {
                row!(media, vert, tags)
            } else {
                row!(media)
            }
            .spacing(5.0)
            .align_y(Vertical::Center)
        };

        let content = column!(top, name, snippet).spacing(2.0);

        let play: Element<'_, Message> = if set_play {
            let size = H1;
            let play = icon(PLAY).size(size).align_x(Horizontal::Center);

            button(play)
                .on_press((on_play)(self.item.id))
                .style(styles::button::text)
                .into()
        } else {
            empty()
        };

        let content = row!(self.poster(), content, play)
            .align_y(Vertical::Center)
            .spacing(10);

        let content = container(content).width(Length::Fill);

        button(content)
            .style(|theme, status| {
                let default = styles::button::subtlest(theme, status);
                let border = default.border.rounded(IMAGE_RADIUS);

                button::Style { border, ..default }
            })
            .padding([4, 8])
            .on_press((on_details)(self.item.id))
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Icons {
    Default = 0,
    Icon1 = 1,
    Icon2 = 2,
    Icon3 = 3,
    Icon4 = 4,
    Icon5 = 5,
    Icon6 = 6,
    Icon7 = 7,
    Icon8 = 8,
    Icon9 = 9,
    Icon10 = 10,
    Icon11 = 11,
    Icon12 = 12,
    Icon13 = 13,
    Icon14 = 14,
    Icon15 = 15,
    Icon16 = 16,
    Icon17 = 17,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    id: Icons,
}

impl Icon {
    pub fn to_u32(self) -> u32 {
        self.id as u32
    }

    pub fn playlist() -> u32 {
        Icons::Icon17 as u32
    }

    pub fn new(icon: Option<u32>) -> Self {
        match icon {
            Some(1) => Self { id: Icons::Icon1 },
            Some(2) => Self { id: Icons::Icon2 },
            Some(3) => Self { id: Icons::Icon3 },
            Some(4) => Self { id: Icons::Icon4 },
            Some(5) => Self { id: Icons::Icon5 },
            Some(6) => Self { id: Icons::Icon6 },
            Some(7) => Self { id: Icons::Icon7 },
            Some(8) => Self { id: Icons::Icon8 },
            Some(9) => Self { id: Icons::Icon9 },
            Some(10) => Self { id: Icons::Icon10 },
            Some(11) => Self { id: Icons::Icon11 },
            Some(12) => Self { id: Icons::Icon12 },
            Some(13) => Self { id: Icons::Icon13 },
            Some(14) => Self { id: Icons::Icon14 },
            Some(15) => Self { id: Icons::Icon15 },
            Some(16) => Self { id: Icons::Icon16 },
            Some(17) => Self { id: Icons::Icon17 },
            _ => Self { id: Icons::Default },
        }
    }

    pub fn unicode(self) -> char {
        match self.id {
            Icons::Default => COLLECTION_ICON,
            Icons::Icon1 => UNFAVORITE,
            Icons::Icon2 => MOVIE,
            Icons::Icon3 => SHOW,
            Icons::Icon4 => POPCORN,
            Icons::Icon5 => FILM,
            Icons::Icon6 => TODO,
            Icons::Icon7 => SWORD,
            Icons::Icon8 => HISTORY,
            Icons::Icon9 => GHOST,
            Icons::Icon10 => ALIEN,
            Icons::Icon11 => CROWN,
            Icons::Icon12 => MASKS,
            Icons::Icon13 => TELESCOPE,
            Icons::Icon14 => SOUP,
            Icons::Icon15 => SPARKLES,
            Icons::Icon16 => HAMBURGER,
            Icons::Icon17 => PLAYLIST,
        }
    }

    pub fn label(self) -> &'static str {
        match self.id {
            Icons::Default => "Default",
            Icons::Icon1 => "Favourites",
            Icons::Icon2 => "Movies",
            Icons::Icon3 => "Shows",
            Icons::Icon4 => "Popcorn",
            Icons::Icon5 => "Film",
            Icons::Icon6 => "Watchlist",
            Icons::Icon7 => "Anime",
            Icons::Icon8 => "Recent",
            Icons::Icon9 => "Horror",
            Icons::Icon10 => "Sci-Fi",
            Icons::Icon11 => "Top Rated",
            Icons::Icon12 => "Drama",
            Icons::Icon13 => "Discovery",
            Icons::Icon14 => "Comfort",
            Icons::Icon15 => "Magical",
            Icons::Icon16 => "Casual",
            Icons::Icon17 => "Playlist",
        }
    }

    pub fn all() -> [Self; 18] {
        [
            Self { id: Icons::Default },
            Self { id: Icons::Icon1 },
            Self { id: Icons::Icon2 },
            Self { id: Icons::Icon3 },
            Self { id: Icons::Icon4 },
            Self { id: Icons::Icon5 },
            Self { id: Icons::Icon6 },
            Self { id: Icons::Icon7 },
            Self { id: Icons::Icon8 },
            Self { id: Icons::Icon9 },
            Self { id: Icons::Icon10 },
            Self { id: Icons::Icon11 },
            Self { id: Icons::Icon12 },
            Self { id: Icons::Icon13 },
            Self { id: Icons::Icon14 },
            Self { id: Icons::Icon15 },
            Self { id: Icons::Icon16 },
            Self { id: Icons::Icon17 },
        ]
    }
}

pub fn to_color(color: devutils::Color) -> Color {
    Color::from_rgba8(color.0, color.1, color.2, color.3)
}

fn to_handle(img: devutils::Image) -> Handle {
    Handle::from_rgba(img.width, img.height, bytes::Bytes::from(img.bytes))
}

pub fn fade_in(now: Instant) -> Animation<bool> {
    Animation::new(false)
        .duration(Duration::from_millis(250))
        .easing(Easing::EaseInOut)
        .go(true, now)
}

pub struct Card<'a, T, Message> {
    pub sample_color: Option<Color>,
    pub image_zoom: f32,
    pub selected: bool,
    pub item: T,
    pub poster: &'a Image,
    pub title: Element<'a, Message>,
    pub details: Option<Element<'a, Message>>,
    pub overlay: Option<Element<'a, Message>>,
    pub float_anim: Option<&'a Animation<bool>>,
}

impl<'a, T: Copy, Message: 'a + Clone> Card<'a, T, Message> {
    pub fn view(
        self,
        now: Instant,
        width: impl Into<Length>,
        height: impl Into<Length>,
        on_select: impl Fn(T) -> Option<Message> + 'a,
        on_hover: impl Fn(T, bool) -> Message + 'a,
        on_show: impl Fn(T, bool) -> Message + 'a,
    ) -> Element<'a, Message> {
        let width = width.into();
        let height = height.into();

        let overlay = self.overlay.unwrap_or(empty());

        let details = match self.details {
            Some(details) => {
                column!(self.title, details)
            }
            None => {
                column!(self.title)
            }
        }
        .width(Length::Fill)
        .spacing(4.0)
        .padding([4, 6]);

        let img = card_poster_helper(self.poster, 1.0 + (self.image_zoom * 0.05), now);

        let content = stack![img, overlay].width(width).height(Length::Fill);

        let selected = self.selected;
        let content = container(column!(content, details))
            .padding(4)
            .width(width)
            .height(height)
            .style(move |theme| {
                let default = styles::container::bb(theme);
                let border = default.border.rounded(IMAGE_RADIUS);

                let border = if selected {
                    border
                        .width(SELECTED_WIDTH)
                        .color(theme.palette().primary.strong.color)
                } else {
                    border
                };

                container::Style { border, ..default }
            });

        let content: Element<'_, Message> = match self.float_anim {
            Some(float_anim) => {
                let interpolation = float_anim.interpolate(0.0, 1.0, now);
                float(content, interpolation, self.sample_color)
            }
            None => content.into(),
        };

        let content = mouse_area(content)
            .on_exit((on_hover)(self.item, false))
            .on_enter((on_hover)(self.item, true));

        let content = match (on_select)(self.item) {
            Some(message) => content
                .on_press(message)
                .interaction(mouse::Interaction::Pointer),
            None => content,
        };

        let content = item_sensor(
            content,
            (on_show)(self.item, true),
            (on_show)(self.item, false),
        );

        content.into()
    }
}

pub fn card_overlay<'a, Message: 'a + Clone, T: Media>(
    media: &'a T,
    on_add: impl Fn(T::Id) -> Message + 'a,
    on_play: impl Fn(T::Id) -> Message + 'a,
    on_select: impl Fn(T::Id) -> Option<Message> + 'a,
    on_hover: impl Fn(T::Id, bool) -> Message + 'a,
    sample_color: Option<Color>,
    sample_text: Option<Color>,
    icon_inter: f32,
    unique: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let padding = [3, 6];
    let sample = sample_text;

    let color = move |theme: &Theme| {
        if sample.is_some() {
            text::Style { color: sample }
        } else {
            text::Style {
                color: Some(theme.palette().primary.strong.text),
            }
        }
    };

    let top = {
        let progress = status_progress(media.status(), media.progress(), sample, true);
        let add = mouse_area(icon(BOOKMARK).size(H4).style(color)).on_press((on_add)(media.id()));

        container(
            row!(progress, space::horizontal(), add)
                .padding(padding)
                .width(Length::Fill)
                .align_y(Vertical::Center),
        )
    };

    let bottom = {
        row!(space::horizontal(), unique.into())
            .align_y(Vertical::Center)
            .padding(padding)
    };

    let play = {
        responsive(move |parent| {
            let main = parent.width.min(parent.height);
            let size = (main * 0.25).max(45.0);

            let play = icon(PLAY).size(size).style(move |_| {
                let color = sample_text
                    .unwrap_or(Color::WHITE)
                    .scale_alpha(icon_inter);
                text::Style { color: Some(color) }
            });

            let play = container(play)
                .width(size * 1.25)
                .height(size * 1.25)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(move |_| {
                    let default = container::background(sample_color.unwrap_or(Color::BLACK));
                    let background = default
                        .background
                        .map(|background| background.scale_alpha(icon_inter));
                    let border = default.border.rounded(size * 2.0);

                    container::Style {
                        border,
                        background,
                        ..default
                    }
                });

            let play = mouse_area(play)
                .on_enter((on_hover)(media.id(), true))
                .on_press((on_play)(media.id()));

            row!(space::horizontal(), play, space::horizontal())
                .height(Length::FillPortion(2))
                .width(Length::Fill)
                .align_y(Vertical::Center)
        })
    };

    let content = mouse_area(
        column!(top, play, bottom)
            .width(Length::Fill)
            .height(Length::Fill),
    );

    match (on_select)(media.id()) {
        Some(message) => content.on_press(message),
        None => content,
    }
    .into()
}

pub fn card_title<'a, Message: 'a>(title: &'a str, hovered: bool) -> Element<'a, Message> {
    let title = marquee(title).size(P).font(medium_font()).toggle(hovered);
    container(title).max_height(20.0).clip(true).into()
}

pub fn card_details<'a, Message: 'a>(rating: Option<f32>, release: String) -> Element<'a, Message> {
    let ratings = ratings(rating, true);
    let release = {
        let release = sized_medium(release, H8);
        let icon = icon(CALENDAR).size(H7);

        row!(icon, release).align_y(Vertical::Center).spacing(3.0)
    };

    row!(ratings, space::horizontal(), release)
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .into()
}

pub fn card_poster_helper<'a, Message: 'a>(
    poster: &'a Image,
    scale: f32,
    now: Instant,
) -> Element<'a, Message> {
    let scale = if matches!(poster, Image::Ready { .. }) {
        scale
    } else {
        1.0
    };

    let view = move |handle: &Handle| {
        image(handle)
            .border_radius(IMAGE_RADIUS)
            .width(Length::Fill)
            .height(Length::Fill)
            .scale(scale)
            .content_fit(ContentFit::Fill)
    };

    let empty = || {
        container(empty())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::container::dark)
    };

    match poster {
        Image::Ready { allocation } => view(allocation.handle()).into(),
        Image::Shown {
            allocation,
            fade_in,
        } => view(allocation.handle())
            .opacity(fade_in.interpolate(0.0, 1.0, now))
            .scale(scale * fade_in.interpolate(1.15, 1.0, now))
            .into(),
        // todo: Could make this a linear gradient skeleton
        Image::Loading(_) => empty().into(),
        Image::Default => match DEFAULT_POSTER.as_ref() {
            Some(handle) => view(handle).into(),
            _ => empty().into(),
        },
    }
}

pub struct List<'a, T, Message> {
    pub selected: bool,
    pub poster: &'a Image,
    pub item: T,
    pub title: Element<'a, Message>,
    pub ratings: Option<Element<'a, Message>>,
    pub synopsis: Option<Element<'a, Message>>,
    pub bottom: Option<Element<'a, Message>>,
    pub overlay: Option<Element<'a, Message>>,
}

impl<'a, T: Copy, Message: 'a + Clone> List<'a, T, Message> {
    pub fn view(
        self,
        now: Instant,
        on_select: impl Fn(T) -> Message + 'a,
        on_hover: impl Fn(T, bool) -> Message + 'a,
        on_show: impl Fn(T, bool) -> Message + 'a,
        on_play: impl Fn(T) -> Message + 'a,
    ) -> Element<'a, Message> {
        let details = row!(
            column!(
                self.title,
                self.ratings.unwrap_or_else(empty),
                self.synopsis.unwrap_or_else(empty),
                self.bottom.unwrap_or_else(empty)
            )
            .spacing(10)
        )
        .height(Length::Fill)
        .align_y(Vertical::Center);

        let details = container(details)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([5, 10]);

        let img = container(card_poster_helper(self.poster, 1.0, now)).width(LIST_WIDTH * 1.75);
        let img = mouse_area(img)
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press((on_play)(self.item));

        let img: Element<'_, Message> = match self.overlay {
            Some(overlay) => stack![img, overlay].into(),
            None => img.into(),
        };

        let content = row!(img, details)
            .align_y(Vertical::Center)
            .height(LIST_HEIGHT);

        let selected = self.selected;
        let content = button(content)
            .padding(10)
            .style(move |theme, status| {
                let default = styles::button::subtlest(theme, status);
                let border = default.border.rounded(IMAGE_RADIUS);
                let border = if selected {
                    border
                        .width(SELECTED_WIDTH)
                        .color(theme.palette().primary.strong.color)
                } else {
                    border
                };

                button::Style { border, ..default }
            })
            .on_press((on_select)(self.item));

        let content = mouse_area(content)
            .on_exit((on_hover)(self.item, false))
            .on_enter((on_hover)(self.item, true));

        let content = item_sensor(
            content,
            (on_show)(self.item, true),
            (on_show)(self.item, false),
        );

        content.into()
    }
}

pub fn list_overlay<'a, Message: 'a + Clone>(
    sample_color: Option<Color>,
    sample_text: Option<Color>,
    icon_inter: f32,
) -> Element<'a, Message> {
    responsive(move |parent| {
        let main = parent.width.min(parent.height);
        let size = (main * 0.25).max(45.0);

        let play = icon(PLAY).size(size).style(move |_| {
            let color = sample_text.unwrap_or(Color::WHITE).scale_alpha(icon_inter);
            text::Style { color: Some(color) }
        });

        let play = container(play)
            .width(size * 1.25)
            .height(size * 1.25)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(move |_| {
                let default = container::background(sample_color.unwrap_or(Color::BLACK));
                let background = default
                    .background
                    .map(|background| background.scale_alpha(icon_inter));
                let border = default.border.rounded(size * 2.0);

                container::Style {
                    border,
                    background,
                    ..default
                }
            });

        row!(space::horizontal(), play, space::horizontal())
            .height(Length::FillPortion(2))
            .width(Length::Fill)
            .align_y(Vertical::Center)
    })
    .into()

    // let play = icon(PLAY)
    //     .size(size)
    //     .align_x(Horizontal::Center)
    //     .height(size)
    //     .style(move |_| {
    //         let color = Color::WHITE.scale_alpha(icon_inter);
    //         text::Style { color: Some(color) }
    //     });

    // let play = center(play)
    //     .width(icon_inter * size)
    //     .height(icon_inter * size)
    //     .style(move |theme| {
    //         let default = styles::container::dark(theme);
    //         let background = default
    //             .background
    //             .map(|background| background.scale_alpha(icon_inter));
    //         let border = default.border.rounded(IMAGE_RADIUS);
    //
    //         container::Style {
    //             border,
    //             background,
    //             ..default
    //         }
    //     });
    //
    // row!(space::horizontal(), play, space::horizontal())
    //     .height(Length::Fill)
    //     .width(Length::Fill)
    //     .align_y(Vertical::Center)
    //     .into()
}

pub fn list_title<'a, Message: 'a>(title: &'a str, hovered: bool) -> Element<'a, Message> {
    marquee(title)
        .toggle(hovered)
        .size(H6)
        .font(medium_font())
        .height(24.0)
        .into()
}

pub fn list_bottom<'a, T, Message: 'a + Clone>(
    id: T,
    status: models::media::Status,
    progress: f32,
    duration: String,
    unique: impl Into<Element<'a, Message>>,
    on_add: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message> {
    row!(
        self::status_progress(status, progress, None, false),
        self::duration(duration),
        unique.into(),
        space::horizontal(),
        add_labelled(id, on_add)
    )
    .spacing(20.0)
    .align_y(Vertical::Center)
    .width(Length::Fill)
    .into()
}

pub struct Compact<'a, T, Message> {
    pub selected: bool,
    pub poster: &'a Image,
    pub item: T,
    pub title: Element<'a, Message>,
    pub ratings: Element<'a, Message>,
    pub progress: Option<Element<'a, Message>>,
    pub duration: Option<Element<'a, Message>>,
    pub recent: Option<Element<'a, Message>>,
}

impl<'a, T: Copy, Message: 'a + Clone> Compact<'a, T, Message> {
    pub fn view(
        self,
        now: Instant,
        on_hover: impl Fn(T, bool) -> Message + 'a,
        on_show: impl Fn(T, bool) -> Message + 'a,
        on_add: Message,
        on_select: Message,
        on_play: Message,
    ) -> Element<'a, Message> {
        let img = compact_poster(self.poster, now, on_play);
        let name = self.title;
        let ratings = self.ratings;
        let progress = self.progress.unwrap_or_else(empty);
        let duration = self.duration.unwrap_or_else(empty);
        let recent = self.recent.unwrap_or_else(empty);

        let add = sized_button(ADD_COLLECTION, H7 * RATIO)
            .on_press(on_add)
            .padding(0);

        let selected = self.selected;
        let content = button(
            row!(img, name, ratings, progress, duration, recent, add)
                .spacing(20.0)
                .align_y(Vertical::Center),
        )
        .padding([6, 6])
        .on_press(on_select)
        .style(move |theme, status| {
            let default = styles::button::subtlest(theme, status);
            let border = default.border.rounded(IMAGE_RADIUS);
            let border = if selected {
                border
                    .width(SELECTED_WIDTH)
                    .color(theme.palette().primary.strong.color)
            } else {
                border
            };

            button::Style { border, ..default }
        });
        let content = mouse_area(content)
            .on_exit((on_hover)(self.item, false))
            .on_enter((on_hover)(self.item, true));

        let content = item_sensor(
            content,
            (on_show)(self.item, true),
            (on_show)(self.item, false),
        );

        content.into()
    }
}

pub fn compact_poster<'a, Message: 'a + Clone>(
    poster: &'a Image,
    now: Instant,
    on_play: Message,
) -> Element<'a, Message> {
    let width = 56.0;

    let img: Element<'a, Message> = {
        let view = move |handle: &Handle| {
            image(handle)
                .border_radius(IMAGE_RADIUS)
                .width(width)
                .height(width)
                .content_fit(ContentFit::Cover)
        };

        let empty = || {
            container(empty())
                .width(width)
                .height(width)
                .style(styles::container::dark)
        };

        match poster {
            Image::Ready { allocation } => view(allocation.handle()).into(),
            Image::Shown {
                allocation,
                fade_in,
            } => view(allocation.handle())
                .opacity(fade_in.interpolate(0.0, 1.0, now))
                .scale(fade_in.interpolate(1.2, 1.0, now))
                .into(),
            Image::Default => match DEFAULT_POSTER.as_ref() {
                Some(handle) => view(handle).into(),
                None => empty().into(),
            },
            Image::Loading(_) => empty().into(),
        }
    };

    button(img)
        .style(styles::button::text)
        .padding(0)
        .on_press(on_play)
        .into()
}

pub fn compact_title<'a, Message: 'a + Clone>(
    title: &'a str,
    hovered: bool,
) -> Element<'a, Message> {
    let name = marquee(title)
        .toggle(hovered)
        .size(P)
        .font(medium_font())
        .width(Length::Fill);

    container(name)
        .clip(true)
        .align_y(Vertical::Center)
        .height(24.0)
        .into()
}

pub fn compact_progress<'a, Message: 'a>(
    status: models::media::Status,
    progress: f32,
) -> Element<'a, Message> {
    let text = match status {
        models::media::Status::Archived => medium("Archived"),
        _ => {
            let progress = (progress * 1000.0).round() / 10.0;
            mono_bold(format!("{}%", progress))
        }
    }
    .size(H7);

    container(text)
        .align_y(Vertical::Center)
        .align_x(Horizontal::Right)
        .width(32.0)
        .into()
}

pub fn compact_duration<'a, Message: 'a>(duration: String) -> Element<'a, Message> {
    container(sized_medium(duration, H7))
        .width(72.0)
        .height(24.0)
        .align_x(Horizontal::Right)
        .align_y(Vertical::Center)
        .into()
}

pub fn compact_recent<'a, Message: 'a>(recent: Option<String>) -> Element<'a, Message> {
    let recent = recent.unwrap_or(String::from("--:--"));

    container(sized_medium(recent, H7))
        .height(24.0)
        .width(100.0)
        .align_x(Horizontal::Right)
        .align_y(Vertical::Center)
        .into()
}

pub fn background_animation() -> Animation<bool> {
    Animation::new(false)
        .duration(iced::time::Duration::from_millis(200))
        .easing(Easing::EaseInOut)
}

pub fn icon_animation() -> Animation<bool> {
    Animation::new(false)
        .duration(iced::time::Duration::from_millis(100))
        .easing(Easing::EaseInOut)
}

pub fn float_animation() -> Animation<bool> {
    Animation::new(false)
        .duration(iced::time::Duration::from_millis(150))
        .easing(Easing::EaseInOut)
}

pub fn image_poster<'a, Message: 'a>(
    poster: &'a Image,
    width: impl Into<Length>,
    height: impl Into<Length>,
    fit: ContentFit,
    now: Instant,
) -> Element<'a, Message> {
    let width = width.into();
    let height = height.into();

    let view = move |handle: &Handle| {
        image(handle)
            .border_radius(IMAGE_RADIUS)
            .height(height)
            .width(width)
            .content_fit(fit)
    };

    let empty = move || {
        container(empty())
            .height(height)
            .width(width)
            .style(move |theme| {
                let default = styles::container::dark(theme);
                let border = default.border.rounded(IMAGE_RADIUS);

                container::Style { border, ..default }
            })
    };

    match poster {
        Image::Ready { allocation } => view(allocation.handle()).into(),
        Image::Shown {
            allocation,
            fade_in,
        } => view(allocation.handle())
            .opacity(fade_in.interpolate(0.0, 1.0, now))
            .scale(fade_in.interpolate(1.2, 1.0, now))
            .into(),
        Image::Loading(_) => empty().into(),
        Image::Default => match DEFAULT_POSTER.as_ref() {
            Some(handle) => view(handle).into(),
            _ => empty().into(),
        },
    }
}

pub fn page_image<'a, Message: 'a>(
    f: impl Fn(f32, f32) -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    responsive(move |size| {
        let img_height = size.height * 0.85;
        let ratio = 2.0 / 3.0;
        f(img_height * ratio, img_height)
    })
    .width(Length::Shrink)
    .into()
}

pub fn page_tags<'a, Message: 'a + Clone, T: text::IntoFragment<'a>>(
    values: impl IntoIterator<Item = (T, Option<Message>)>,
) -> Element<'a, Message> {
    let separator = || Element::from(text("•").line_height(1.0).size(H7));

    let values = values
        .into_iter()
        .flat_map(|(value, message)| {
            let value = text(value).size(H8).font(bold_italic_font());

            let value = match message {
                Some(message) => Element::from(
                    button(value)
                        .padding(0)
                        .style(styles::button::text)
                        .on_press(message),
                ),
                None => Element::from(value),
            };

            [separator(), value]
        })
        .skip(1);

    row(values).spacing(4).align_y(Vertical::Center).into()
}

pub fn page_header<'a, Message: 'a + Clone>(
    header: impl Into<Element<'a, Message>>,
    on_play: Message,
    on_collection: Message,
    on_edit: Message,
) -> Element<'a, Message> {
    let header = header.into();

    let actions = {
        let size = H2;
        let play = button(icon(PLAY).size(size))
            .style(styles::button::text_primary)
            .padding(0)
            .on_press(on_play);

        let collection = button(icon(ADD_COLLECTION).size(size))
            .style(styles::button::text)
            .padding(0)
            .on_press(on_collection);

        let config = button(icon(VIDEO_CONFIG).size(size / RATIO))
            .style(styles::button::text)
            .padding(0)
            .on_press(on_edit);

        row!(play, collection, config)
            .align_y(Vertical::Center)
            .spacing(16)
    };

    row!(
        header,
        container(actions)
            .align_x(Horizontal::Right)
            .width(Length::Fill)
    )
    .align_y(Vertical::Center)
    .spacing(4)
    .into()
}

pub fn page_details<'a, Message: 'a>(
    rating: Option<f32>,
    release: impl text::IntoFragment<'a>,
    unique: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    let vert = || container(rule::vertical(2.0)).height(H8).clip(true);

    let rating = ratings_short(rating);
    let release = sized_medium(release, H7);
    let duration = sized_medium(unique, H7);

    row!(release, vert(), rating, vert(), duration)
        .spacing(8)
        .align_y(Vertical::Center)
        .into()
}

pub fn page_title<'a, Message: 'a>(
    top: impl Into<Element<'a, Message>>,
    title: impl text::IntoFragment<'a>,
    details: impl Into<Element<'a, Message>>,
    status: models::media::Status,
) -> Element<'a, Message> {
    let title = sized_bold(title, H3)
        .width(Length::FillPortion(2))
        .height(32);

    let status = if matches!(status, models::media::Status::Archived) {
        Some(Element::from(sized_medium("Archived", H8)))
    } else {
        None
    };

    column!(top.into(), title, details.into(), status)
        .spacing(4.0)
        .into()
}

pub fn page_overview<'a, Message: 'a>(
    overview: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    let synopsis = regular(overview);

    container(scrollable(synopsis).spacing(4.0))
        .max_width(750)
        .max_height(500)
        .into()
}

pub fn page_video<'a, Message: 'a>(
    video: Option<&'a VideoInfo>,
    audio: Option<&'a Audio>,
    subtitle: Option<&'a Subtitle>,
) -> Element<'a, Message> {
    if video.is_none() && audio.is_none() && subtitle.is_none() {
        return empty();
    }

    let size = P;

    let info = |value: String| sized_medium(value, size / RATIO);

    let video = video.map(|video| {
        let title = sized_medium("Video", size);

        let resolution =
            (video.height > 0).then(|| info(format!("Resolution: {}", video.resolution())));

        let codec = video
            .codec
            .as_deref()
            .map(|codec| info(format!("Codec: {codec}")));

        let framerate =
            (video.framerate > 0.0).then(|| info(format!("Framerate: {:.0}", video.framerate)));

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

    column!(video, audio, subtitle).spacing(12).into()
}

pub fn page_collections<'a, Message: 'a + Clone>(
    mut memberships: Peekable<impl Iterator<Item = &'a SimpleCollection>>,
    goto: impl Fn(CollectionId) -> Message + 'a + Clone,
) -> Element<'a, Message> {
    let size = P;

    if memberships.peek().is_some() {
        let title = sized_medium("Collections", size);
        let collections =
            memberships.map(|collection| draw_collection_tab(collection, goto.clone()));

        let content = column(collections).spacing(4.0).width(Length::Fill);
        let collections = container(scrollable(content).spacing(4.0)).max_height(300);

        column!(title, collections).spacing(8).into()
    } else {
        empty()
    }
}

pub fn page_nav<'a, Message: 'a + Clone>(
    on_prev: Option<Message>,
    on_next: Message,
) -> Element<'a, Message> {
    let size = H3;
    let color = |theme: &iced::Theme| text::Style {
        color: Some(theme.palette().primary.base.color.into()),
    };

    let prev = button(icon(CHEV_RIGHT).size(size).style(color))
        .style(styles::button::subtlest)
        .on_press_maybe(on_prev);

    let next = button(icon(CHEV_LEFT).size(size).style(color))
        .style(styles::button::subtlest)
        .on_press(on_next);

    row!(space::horizontal(), prev, next, space::horizontal())
        .spacing(40)
        .align_y(Vertical::Center)
        .into()
}

pub fn page_data<'a, Message: 'a>(
    added: String,
    count: u32,
    progress: f32,
    recent: Option<String>,
    comments: u32,
    unique: Option<(&'a str, impl text::IntoFragment<'a>, char)>,
) -> Element<'a, Message> {
    let title = sized_medium("Statistics", P);

    let content = {
        let added = data("Date Added", added, CALENDAR);

        let count = data("Watch Count", count, EYE);

        let progress = (progress * 1000.0).round() / 10.0;
        let progress = data("Watch Progress", format!("{:.1}%", progress), HOURGLASS);

        let recent = data(
            "Recent Watch",
            recent.unwrap_or(String::from(" --:--:--")),
            CALENDAR,
        );

        let comments = data("Comments", comments, NUMBER);

        let unique = unique.map(|(label, value, icon)| data(label, value, icon));

        let c1 = column!(added, recent)
            .align_x(Horizontal::Center)
            .spacing(20.0);
        let c2 = column!(count, comments)
            .align_x(Horizontal::Center)
            .spacing(20.0);
        let c3 = column!(progress, unique)
            .align_x(Horizontal::Center)
            .spacing(20.0);

        row!(c1, c2, c3).spacing(40).width(500)
    };

    column!(title, content).spacing(12).into()
}

pub fn page_layout<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    img: impl Into<Element<'a, Message>>,
    scroll: &Scroll,
    on_scroll: impl Fn(scrollable::Viewport) -> Message + 'a,
) -> Element<'a, Message> {
    let h_padding = 40.0;
    let r_padding_inner = 35.0;

    let content = container(content).padding(Padding::ZERO.left(40.0).right(r_padding_inner));

    let content = scrollable(content)
        .auto_scroll(true)
        .spacing(8.0)
        .id(scroll.id.clone())
        .on_scroll(on_scroll);
    let content = row!(img.into(), content).spacing(0);

    container(content)
        .padding(
            Padding::new(20.0)
                .left(h_padding)
                .right(h_padding - r_padding_inner),
        )
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

pub fn item_sensor<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    show: Message,
    hide: Message,
) -> sensor::Sensor<'a, (), Message> {
    use iced::Size;

    sensor(content)
        // .delay(iced::time::milliseconds(100))
        .on_show(move |_| show.clone())
        .on_hide(hide)
}
