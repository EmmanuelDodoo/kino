use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{empty, styles, tooltip};
use core::variants;
use devutils::image_ops::{collage, sample_complement};
use iced::{
    Color, ContentFit, Element, Length, Task, Theme,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    mouse,
    time::{Duration, Instant},
    widget::{
        self, button, center, column, container, image,
        image::{Allocation, Handle},
        markdown, mouse_area, row, rule, scrollable, space, stack, text, tooltip as tp,
    },
};
use registry::models::{Collection, CollectionId, ItemId, Media, SearchItem, SimpleCollection};
use widgets::marquee;

use std::sync::LazyLock;
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

pub fn duration<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let duration = sized_medium(media.duration_full(), H8);
    let icon = icon(HOURGLASS).size(H8);

    row!(icon, duration)
        .align_y(Vertical::Center)
        .spacing(1.0)
        .into()
}

pub fn ratings<'a, T: Media, Message: 'a>(media: &T, show_text: bool) -> Element<'a, Message> {
    let size = H7;
    let color = |theme: &Theme| -> text::Style {
        let color = theme.palette().primary.strong.color;
        text::Style { color: Some(color) }
    };

    match media.rating() {
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
                let text = sized_medium(format!("{rating:.1}"), H8).style(color);
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

fn progress_icon<T: Media>(media: &T) -> char {
    match media.progress() {
        ..0.15 => PROGRESS_10,
        0.15..0.3 => PROGRESS_20,
        0.3..0.5 => PROGRESS_40,
        0.5..0.7 => PROGRESS_60,
        0.7..0.85 => PROGRESS_80,
        x if x < 1.0 => PROGRESS_90,
        _ => PROGRESS_100,
    }
}

pub fn progress<'a, T: Media, Message: 'a>(
    media: &T,
    color: Option<Color>,
    primary: bool,
) -> Element<'a, Message> {
    let progress = (media.progress() * 1000.0).round() / 10.0;
    let text = mono_bold(format!("{}%", progress))
        .size(H8)
        .style(move |theme: &Theme| {
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

    let progress = progress_icon(media);
    let icon = icon(progress).size(H6).style(move |theme: &Theme| {
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

pub fn add_labelled<'a, T: Media, Message: 'a + Clone>(
    media: &T,
    on_press: impl Fn(T::Id) -> Message + 'a,
) -> Element<'a, Message> {
    let icon = icon(BOOKMARK).size(P);

    let label = sized_medium("Add to collection", H8);

    mouse_area(row!(icon, label).align_y(Vertical::Center).spacing(6.0))
        .interaction(mouse::Interaction::Pointer)
        .on_press((on_press)(media.id()))
        .into()
}

pub fn synopsis<'a, T: Media, Message: 'a>(media: &'a T) -> Element<'a, Message> {
    container(regular(media.synopsis()))
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
        color: Some(theme.palette().primary.weak.color),
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
    let size = P;
    let unicode = Icon::new(collection.icon).unicode();
    let icon = icon(unicode).size(size);
    let text = container(regular(&collection.name))
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
        let default = styles::button::subtle(theme, status);

        let border = default.border.rounded(IMAGE_RADIUS);

        button::Style { border, ..default }
    })
    .into()
}

pub fn float<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    float: &'a Animation<bool>,
    color: Option<Color>,
    now: Instant,
) -> Element<'a, Message> {
    use iced::{Color, Shadow};

    let interpolation = float.interpolate(0.0, 1.0, now);

    let blur_radius = interpolation * 20.0;
    let scale = 1.0 + (0.05 * interpolation);
    let color = color.unwrap_or(Color::BLACK).scale_alpha(interpolation);

    widget::float(content)
        .scale(scale)
        .translate(move |bounds, viewport| {
            bounds.zoom(1.05).offset(&viewport.shrink(5)) * interpolation
        })
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
pub struct ThumbnailTask<T: Media> {
    pub id: T::Id,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
enum Image {
    Ready {
        allocation: Allocation,
        fade_in: Animation<bool>,
    },
    Loading,
    Default,
}

#[derive(Debug, Clone)]
pub struct Thumbnail<T: Media> {
    pub selected: bool,
    poster: Image,
    backdrop: Option<Handle>,
    sample_text: Option<Color>,
    sample_color: Option<Color>,
    background: Animation<bool>,
    icon: Animation<bool>,
    float: Animation<bool>,
    pub media: T,
}

impl<T: Media + 'static> Thumbnail<T> {
    pub fn new(media: T) -> (Self, Task<ThumbnailTask<T>>) {
        let id = media.id();

        let task = match media.poster() {
            Some(poster) => {
                let path = poster.path.display().to_string();

                let sample = if poster.main.is_none() {
                    Task::future(async move {
                        sample_complement(&path).map(|(a, b)| (to_color(a), to_color(b)))
                    })
                    .and_then(move |(main, accent)| {
                        Task::done(ThumbnailTask {
                            id,
                            kind: ThumbnailTaskKind::Samples { main, accent },
                        })
                    })
                } else {
                    Task::none()
                };

                let images =
                    image::allocate(poster.path.clone()).map(move |allocation| ThumbnailTask {
                        id,
                        kind: ThumbnailTaskKind::Image(allocation),
                    });

                Task::batch([sample, images])
            }
            None => Task::none(),
        };

        let mut sample_text = None;
        let mut sample_color = None;

        //todo: Sample color is not great for current default poster
        let poster = match media.poster() {
            Some(poster) => {
                sample_color = poster.get_main().map(to_color);
                sample_text = poster.get_accent().map(to_color);
                Image::Loading
            }
            None => Image::Default,
        };

        let backdrop = media.backdrop().map(Handle::from_path);

        let new = Self {
            selected: false,
            background: Animation::new(false)
                .duration(iced::time::Duration::from_millis(200))
                .easing(Easing::EaseInOut),
            icon: Animation::new(false)
                .duration(iced::time::Duration::from_millis(100))
                .easing(Easing::EaseOut),
            float: Animation::new(false)
                .duration(iced::time::Duration::from_millis(150))
                .easing(Easing::EaseInOut),
            poster,
            sample_text,
            sample_color,
            backdrop,
            media,
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

    fn poster_ready(&self) -> bool {
        matches!(&self.poster, Image::Ready { .. })
    }

    pub fn go_mut(&mut self, new_state: bool, at: Instant) {
        self.background.go_mut(new_state, at);
        self.icon.go_mut(new_state, at);
        self.float.go_mut(new_state, at);
    }

    pub fn sample(&mut self, samples: Option<(Color, Color)>) {
        if let Some((color, text)) = samples {
            self.sample_color = Some(color);
            self.sample_text = Some(text);
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
                tracing::error!("Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    pub fn id(&self) -> T::Id {
        self.media.id()
    }

    pub fn poster<'a, Message: 'a>(
        &'a self,
        width: impl Into<Length>,
        height: impl Into<Length>,
        now: Instant,
    ) -> Element<'a, Message> {
        let width = width.into();
        let height = height.into();

        let view = move |handle: &Handle| {
            image(handle)
                .border_radius(IMAGE_RADIUS)
                .height(height)
                .width(width)
                .content_fit(ContentFit::Cover)
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

        match &self.poster {
            Image::Ready {
                allocation,
                fade_in,
            } => view(allocation.handle())
                .opacity(fade_in.interpolate(0.0, 1.0, now))
                .scale(fade_in.interpolate(1.2, 1.0, now))
                .into(),
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

    fn poster_helper<'a, Message: 'a>(&self, scale: f32, now: Instant) -> Element<'a, Message> {
        let scale = if self.poster_ready() { scale } else { 1.0 };

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

        match &self.poster {
            Image::Ready {
                allocation,
                fade_in,
            } => view(allocation.handle())
                .opacity(fade_in.interpolate(0.0, 1.0, now))
                .scale(scale * fade_in.interpolate(1.15, 1.0, now))
                .into(),
            Image::Loading => empty().into(),
            Image::Default => match DEFAULT_POSTER.as_ref() {
                Some(handle) => view(handle).into(),
                _ => empty().into(),
            },
        }
    }

    pub fn list<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(T::Id) -> Message + 'a,
        on_select: impl Fn(T::Id) -> Message + 'a,
        on_hover: impl Fn(T::Id, bool) -> Message + 'a,
        on_play: impl Fn(T::Id) -> Message + 'a,
        unique: impl Fn(&T) -> Element<'a, Message>,
    ) -> Element<'a, Message> {
        let title = marquee(self.media.name())
            .size(H6)
            .font(medium_font())
            .height(24.0);

        let ratings = ratings(&self.media, true);

        let synopsis = synopsis(&self.media);

        let unique = unique(&self.media);

        let bottom = row!(
            progress(&self.media, None, false),
            duration(&self.media),
            unique,
            space::horizontal(),
            add_labelled(&self.media, on_add)
        )
        .spacing(20.0)
        .align_y(Vertical::Center)
        .width(Length::Fill);

        let details = row!(column!(title, ratings, synopsis, bottom).spacing(10))
            .height(Length::Fill)
            .align_y(Vertical::Center);

        let details = container(details)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([5, 10]);

        let img = container(self.poster_helper(1.0, now)).width(LIST_WIDTH * 1.75);
        let img = mouse_area(img)
            .interaction(iced::mouse::Interaction::Pointer)
            .on_exit((on_hover)(self.media.id(), false))
            .on_enter((on_hover)(self.media.id(), true))
            .on_press((on_play)(self.media.id()));

        let overlay = {
            let size = H1 * 1.5;

            let factor = self.icon.interpolate(0.0, 1.0, now);
            let play = icon(PLAY)
                .size(size)
                .align_x(Horizontal::Center)
                .height(size)
                .style(move |_| {
                    let color = Color::WHITE.scale_alpha(factor);
                    text::Style { color: Some(color) }
                });

            let factor = self.background.interpolate(0.0, 1.0, now);
            let play = center(play)
                .width(factor * size)
                .height(factor * size)
                .style(move |theme| {
                    let default = styles::container::dark(theme);
                    let background = default
                        .background
                        .map(|background| background.scale_alpha(factor));
                    let border = default.border.rounded(IMAGE_RADIUS);

                    container::Style {
                        border,
                        background,
                        ..default
                    }
                });

            row!(space::horizontal(), play, space::horizontal())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let img = stack![img, overlay];

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
            .on_press((on_select)(self.media.id()));

        content.into()
    }

    pub fn card<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(T::Id) -> Message + 'a,
        on_select: impl Fn(T::Id) -> Message + 'a,
        on_hover: impl Fn(T::Id, bool) -> Message + 'a,
        on_play: impl Fn(T::Id) -> Message + 'a,
    ) -> Element<'a, Message> {
        let padding = [3, 6];
        let sample = self.sample_text;

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
            let progress = progress(&self.media, sample, true);
            let add = mouse_area(icon(BOOKMARK).size(H4).style(color))
                .on_press((on_add)(self.media.id()));

            container(
                row!(progress, space::horizontal(), add)
                    .padding(padding)
                    .width(Length::Fill)
                    .align_y(Vertical::Center),
            )
        };

        let details = {
            let title = marquee(self.media.name()).size(P).font(medium_font());
            let title = container(title).max_height(20.0).clip(true);
            let ratings = ratings(&self.media, true);
            let release = {
                let release = sized_medium(self.media.release_year(), H8);
                let icon = icon(CALENDAR).size(H7);

                row!(icon, release).align_y(Vertical::Center).spacing(3.0)
            };

            let details = row!(ratings, space::horizontal(), release)
                .width(Length::Fill)
                .align_y(Vertical::Center);

            container(column!(title, details).width(Length::Fill).spacing(4.0)).padding(padding)
        };

        let bottom = {
            let duration = sized_medium(self.media.duration_full(), H8).style(color);
            row!(space::horizontal(), duration)
                .align_y(Vertical::Center)
                .padding(padding)
        };

        let background_interpolation = self.background.interpolate(0.0, 1.0, now);
        let play = {
            let size = CARD_HEIGHT * 0.135;

            let icon_interpolation = self.icon.interpolate(0.0, 1.0, now);
            let play = icon(PLAY)
                .size(size)
                .align_x(Horizontal::Center)
                .height(size)
                .style(move |_| {
                    let color = Color::WHITE.scale_alpha(icon_interpolation);
                    text::Style { color: Some(color) }
                });

            let play = center(play)
                .width(size * background_interpolation)
                .height(size * background_interpolation)
                .style(move |theme| {
                    let default = styles::container::dark(theme);
                    let background = default
                        .background
                        .map(|background| background.scale_alpha(background_interpolation));
                    let border = default.border.rounded(IMAGE_RADIUS);

                    container::Style {
                        border,
                        background,
                        ..default
                    }
                });

            row!(space::horizontal(), play, space::horizontal())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let overlay = mouse_area(
            column!(top, space::vertical(), play, space::vertical(), bottom)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press((on_play)(self.media.id()));

        let img = self.poster_helper(1.0 + (background_interpolation * 0.05), now);

        let content = stack![img, overlay].width(CARD_WIDTH).height(Length::Fill);

        let selected = self.selected;
        let content = container(column!(content, details))
            .padding(4)
            .width(CARD_WIDTH)
            .height(CARD_HEIGHT)
            .style(move |theme| {
                let default = styles::container::bw3(theme);
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

        let content = float(content, &self.float, self.sample_color, now);

        let content = mouse_area(content)
            .interaction(mouse::Interaction::Pointer)
            .on_press((on_select)(self.media.id()))
            .on_exit((on_hover)(self.media.id(), false))
            .on_enter((on_hover)(self.media.id(), true));

        content.into()
    }

    pub fn compact<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_add: impl Fn(T::Id) -> Message + 'a,
        on_select: impl Fn(T::Id) -> Message + 'a,
        on_play: impl Fn(T::Id) -> Message + 'a,
    ) -> Element<'a, Message> {
        let width = 56.0;
        let size = H7;

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

            match &self.poster {
                Image::Ready {
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
                Image::Loading => empty().into(),
            }
        };

        let img = button(img)
            .style(styles::button::text)
            .padding(0)
            .on_press((on_play)(self.media.id()));

        let name = marquee(self.media.name())
            .size(P)
            .font(medium_font())
            .width(Length::Fill);

        let name = container(name)
            .clip(true)
            .align_y(Vertical::Center)
            .height(24.0);

        let ratings = ratings(&self.media, false);

        let progress = {
            let progress = (self.media.progress() * 1000.0).round() / 10.0;
            let text = mono_bold(format!("{}%", progress)).size(H7);

            container(text)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Right)
                .width(32.0)
        };

        let duration = container(sized_medium(self.media.duration_short(), H7))
            .width(72.0)
            .height(24.0)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center);

        let recent = self.media.recent_short().unwrap_or(String::from("--:--"));
        let recent = container(sized_medium(recent, H7))
            .height(24.0)
            .width(100.0)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center);

        let add = sized_button(ADD_COLLECTION, size * RATIO)
            .on_press((on_add)(self.media.id()))
            .padding(0);

        let selected = self.selected;
        button(
            row!(img, name, ratings, progress, duration, recent, add)
                .spacing(20.0)
                .align_y(Vertical::Center),
        )
        .padding([6, 6])
        .on_press((on_select)(self.media.id()))
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
        .into()
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

        let collage = Image::Loading;
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
            Image::Ready { fade_in, .. } => fade_in.is_animating(now),
            _ => false,
        }
    }

    pub fn task(&mut self, task: CollectionTaskKind, now: Instant) {
        match task {
            CollectionTaskKind::Image(Ok(allocation)) => {
                self.collage = Image::Ready {
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
            Image::Loading | Image::Default => {
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
            Image::Ready {
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
                Image::Ready {
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

                Image::Loading | Image::Default => {
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

fn to_color(color: devutils::Color) -> Color {
    Color::from_rgba8(color.0, color.1, color.2, color.3)
}

fn to_handle(img: devutils::Image) -> Handle {
    Handle::from_rgba(img.width, img.height, bytes::Bytes::from(img.bytes))
}

fn fade_in(now: Instant) -> Animation<bool> {
    Animation::new(false)
        .duration(Duration::from_millis(500))
        .easing(Easing::EaseInOut)
        .go(true, now)
}
