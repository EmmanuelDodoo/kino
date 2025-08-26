use crate::media::Media;
use crate::utils::empty;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Filter, Sort};
use iced::{
    Color, ContentFit, Element, Length, Shadow,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    mouse,
    time::Instant,
    widget::{
        self, column, container, horizontal_space, image, mouse_area, row, stack, text,
        vertical_space,
    },
};

pub const CARD_HEIGHT: f32 = 350.0;
pub const CARD_WIDTH: f32 = CARD_HEIGHT * 1.0 / 1.0;
pub const LIST_HEIGHT: f32 = 200.0;
pub const LIST_WIDTH: f32 = LIST_HEIGHT * 2.0 / 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Tab {
    #[default]
    Items,
    Data,
    Comments,
    Collections,
}

impl Tab {
    pub const ALL: [Self; 4] = [Self::Items, Self::Data, Self::Comments, Self::Collections];

    pub const EPISODE: [Self; 3] = [Self::Data, Self::Comments, Self::Collections];

    pub fn to_str(self, item: &str) -> &str {
        match self {
            Self::Items => item,
            Self::Data => "Data",
            Self::Collections => "Collections",
            Self::Comments => "Comments",
        }
    }
}

pub fn duration<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let duration = text(media.duration_full()).size(H7);
    let icon = icon(HOURGLASS).size(H8);

    row!(icon, duration)
        .align_y(Vertical::Center)
        .spacing(1.0)
        .into()
}

pub fn ratings<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let rating = media.rating();

    let unstars = (5 - rating).clamp(0, 5);
    let stars = (0..rating).map(|_| Element::from(icon(STAR).size(H7)));
    let unstars = (0..unstars).map(|_| Element::from(icon(UNSTAR).size(H7)));
    let ratings = row(stars.chain(unstars))
        .spacing(2.0)
        .align_y(Vertical::Center);

    ratings.into()
}

pub fn progress<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let progress = media.progress_icon();

    let text = text(format!("{}%", media.progress() * 100.0)).size(H7);

    let icon = icon(progress).size(H4);

    row!(icon, text)
        .spacing(3.0)
        .align_y(Vertical::Center)
        .into()
}

pub fn add_labelled<'a, T: Media, Message: 'a + Clone>(
    media: &T,
    on_press: impl Fn(T::Id) -> Message + 'a,
) -> Element<'a, Message> {
    let size = H7;
    let icon = icon(BOOKMARK).size(H6);

    let label = text("Add to collection").size(size);

    mouse_area(row!(icon, label).align_y(Vertical::Center).spacing(6.0))
        .interaction(mouse::Interaction::Pointer)
        .on_press((on_press)(media.id()))
        .into()
}

pub fn synapsis<'a, T: Media, Message: 'a>(media: &'a T) -> Element<'a, Message> {
    container(text(media.synapsis()).size(H7))
        .max_height(52.0)
        .into()
}

pub fn poster<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    match media.poster() {
        Some(handle) => container(
            image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain),
        )
        .style(container::dark)
        .into(),
        None => container(empty()).style(container::dark).into(),
    }
}

pub fn float<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    zoom: &'a Animation<bool>,
    now: Instant,
) -> Element<'a, Message> {
    widget::float(content)
        .scale(zoom.interpolate(1.0, 1.025, now))
        .translate(move |bounds, viewport| {
            bounds.zoom(1.025).offset(&viewport.shrink(5)) * zoom.interpolate(0.0, 1.0, now)
        })
        .style(move |_theme| widget::float::Style {
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(zoom.interpolate(0.0, 1.0, now)),
                blur_radius: zoom.interpolate(0.0, 20.0, now),
                ..Shadow::default()
            },
            ..widget::float::Style::default()
        })
        .into()
}

pub fn data<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    value: impl text::IntoFragment<'a>,
    unicode: char,
) -> Element<'a, Message> {
    let size = H7;
    let value = text(value).size(size);
    let value = row!(icon(unicode).size(size), value)
        .spacing(2.0)
        .align_y(Vertical::Center);

    column!(value, text(label).size(size))
        .align_x(Horizontal::Center)
        .spacing(0.0)
        .into()
}

pub fn filter_sort<'a, T: 'a + Media>(
    thumbnails: impl Iterator<Item = &'a Thumbnail<T>>,
    filters: &Filter,
    sort: &Sort,
) -> impl Iterator<Item = &'a Thumbnail<T>> {
    let mut temp = thumbnails
        .filter(|thumbnail| filters.filter(&thumbnail.media))
        .collect::<Vec<_>>();

    temp.sort_by(|x, y| sort.sort(&x.media, &y.media));

    if sort.reverse {
        temp.reverse();
    }

    temp.into_iter()
}

pub fn data_tab<'a, Message: 'a, T: Media>(media: &T, width: f32) -> Element<'a, Message> {
    let duration = data("Duration", media.duration_short(), CLOCK);

    let rating = data("Rating", format!("{}/5", media.rating()), STAR);

    let comments = data("Comments", media.comments(), NUMBER);

    let release = data("Release Date", media.release_my(), CALENDAR);

    let added = data("Date Added", media.added_my(), CALENDAR);

    let count = data("Watch Count", media.watch_count(), EYE);

    let progress = data(
        "Watch Progress",
        format!("{}%", media.progress() * 100.0),
        HOURGLASS,
    );

    let recent = data("Recent Watch", media.recent_short(), CALENDAR);

    let r1 = row!(
        duration,
        horizontal_space(),
        release,
        horizontal_space(),
        count,
        horizontal_space(),
        progress
    )
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let r2 = row!(
        rating,
        horizontal_space(),
        added,
        horizontal_space(),
        comments,
        horizontal_space(),
        recent,
    )
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let content = column!(r1, r2).spacing(30.0);

    content.width(width).into()
}

#[derive(Debug, Clone)]
pub struct Thumbnail<T: Media> {
    pub poster: Option<image::Handle>,
    pub backdrop: Option<image::Handle>,
    pub zoom: Animation<bool>,
    pub media: T,
}

impl<T: Media> Thumbnail<T> {
    pub fn new(media: T) -> Self {
        let poster = media.poster().map(image::Handle::from_path);
        let backdrop = media.backdrop().map(image::Handle::from_path);

        Self {
            zoom: Animation::new(false).very_quick().easing(Easing::EaseInOut),
            poster,
            backdrop,
            media,
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.zoom.is_animating(now)
    }

    pub fn id(&self) -> T::Id {
        self.media.id()
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
        let title = text(self.media.name()).size(H5);

        let ratings = ratings(&self.media);

        let synapsis = synapsis(&self.media);

        let unique = unique(&self.media);

        let bottom = row!(
            progress(&self.media),
            duration(&self.media),
            unique,
            horizontal_space(),
            add_labelled(&self.media, on_add)
        )
        .spacing(24.0)
        .align_y(Vertical::Center)
        .width(Length::Fill);

        let details = row!(column!(title, ratings, synapsis, bottom).spacing(16))
            .height(Length::Fill)
            .align_y(Vertical::Center);

        let details = container(details)
            .style(container::dark)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([5, 10]);

        let img = container(poster(&self.media)).width(LIST_WIDTH);

        let overlay = {
            let size = H1 * 1.95;
            let play = mouse_area(
                icon(PLAY)
                    .size(size)
                    .align_x(Horizontal::Center)
                    .height(size * self.zoom.interpolate(0.0, 1.0, now)),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press((on_play)(self.media.id()));

            row!(horizontal_space(), play, horizontal_space())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let img = stack![img, overlay];

        let content = mouse_area(
            row!(img, details)
                .align_y(Vertical::Center)
                .height(LIST_HEIGHT),
        )
        .interaction(mouse::Interaction::Pointer)
        .on_press((on_select)(self.media.id()))
        .on_exit((on_hover)(self.media.id(), false))
        .on_enter((on_hover)(self.media.id(), true));

        float(content, &self.zoom, now)
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

        let top = {
            let progress = progress(&self.media);
            let add = mouse_area(icon(BOOKMARK).size(H4))
                .interaction(mouse::Interaction::Pointer)
                .on_press((on_add)(self.media.id()));

            container(
                row!(progress, horizontal_space(), add)
                    .padding(padding)
                    .width(Length::Fill)
                    .align_y(Vertical::Center),
            )
        };

        let details = {
            let title = text(self.media.name()).size(H7);
            let ratings = ratings(&self.media);
            let release = {
                let release = text(self.media.release_year()).size(H7);
                let icon = icon(CALENDAR).size(H7);

                row!(icon, release).align_y(Vertical::Center).spacing(3.0)
            };

            let details = row!(ratings, horizontal_space(), release)
                .width(Length::Fill)
                .align_y(Vertical::Center);

            container(column!(title, details).width(Length::Fill).spacing(10.0))
                .padding(padding)
                .style(container::dark)
        };

        let bottom = {
            let duration = text(self.media.duration_full()).size(H7);
            row!(horizontal_space(), duration)
                .align_y(Vertical::Center)
                .padding(padding)
        };

        let play = {
            let size = CARD_HEIGHT * 0.18;

            let play = icon(PLAY)
                .size(size)
                .align_x(Horizontal::Center)
                .height(size * self.zoom.interpolate(0.0, 1.0, now));

            row!(horizontal_space(), play, horizontal_space())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let overlay = mouse_area(
            column!(top, vertical_space(), play, vertical_space(), bottom)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .interaction(iced::mouse::Interaction::Pointer)
        .on_press((on_play)(self.media.id()));

        let img = poster(&self.media);

        let content = stack![img, overlay].width(CARD_WIDTH);

        let content = column!(content, details);

        let content = mouse_area(content)
            .interaction(mouse::Interaction::Pointer)
            .on_press((on_select)(self.media.id()))
            .on_exit((on_hover)(self.media.id(), false))
            .on_enter((on_hover)(self.media.id(), true));

        float(content, &self.zoom, now)
    }
}
