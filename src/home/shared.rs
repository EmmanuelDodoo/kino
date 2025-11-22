use crate::models::{Collection, CollectionId, ItemId, Media, SearchItem, SimpleCollection};
use crate::utils::empty;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Filter, Sort};
use iced::{
    ContentFit, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    font,
    font::{Family, Font, Style, Weight},
    mouse,
    time::Instant,
    widget::{
        self, button, center, column, container, image::Handle, markdown, mouse_area, operation,
        row, rule, space, stack, text,
    },
};
use image::{DynamicImage, GenericImage, ImageBuffer, ImageReader, Rgba, imageops::FilterType};

pub const CARD_HEIGHT: f32 = 375.0;
pub const CARD_WIDTH: f32 = CARD_HEIGHT * 0.7;
pub const LIST_HEIGHT: f32 = 150.0;
pub const LIST_WIDTH: f32 = LIST_HEIGHT * 5.5 / 10.0;
pub const IMAGE_RADIUS: f32 = 7.0;

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
    let duration = text(media.duration_full()).size(H8).font(Font {
        weight: Weight::Semibold,
        ..Default::default()
    });
    let icon = icon(HOURGLASS).size(H8);

    row!(icon, duration)
        .align_y(Vertical::Center)
        .spacing(1.0)
        .into()
}

pub fn ratings<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let size = H7;
    let color = |theme: &Theme| -> text::Style {
        let color = theme.extended_palette().primary.strong.color;
        text::Style { color: Some(color) }
    };

    match media.rating() {
        Some(value) => {
            let rating = (value * 10.0).round() / 10.0;
            let text = text(format!("{rating:.1}")).size(H8).font(Font {
                weight: Weight::Semibold,
                ..Default::default()
            });

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

            let ratings = row!(text, ratings).align_y(Vertical::Center).spacing(6.0);

            ratings.into()
        }
        None => row((0..5).map(|_| Element::from(icon(UNSTAR).size(size).style(color)))).into(),
    }
}

pub fn progress<'a, T: Media, Message: 'a>(media: &T) -> Element<'a, Message> {
    let font = Font {
        family: Family::Monospace,
        weight: Weight::Semibold,
        ..Default::default()
    };
    let progress = (media.progress() * 1000.0).round() / 10.0;
    let text = text(format!("{}%", progress)).size(H8).font(font);

    let progress = media.progress_icon();
    let icon = icon(progress).size(H6);

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

    let label = text("Add to collection").size(H8).font(Font {
        weight: Weight::Semibold,
        ..Default::default()
    });

    mouse_area(row!(icon, label).align_y(Vertical::Center).spacing(6.0))
        .interaction(mouse::Interaction::Pointer)
        .on_press((on_press)(media.id()))
        .into()
}

pub fn synopsis<'a, T: Media, Message: 'a>(media: &'a T) -> Element<'a, Message> {
    container(text(media.synopsis()).size(P).font(Font {
        family: Family::Serif,
        ..Default::default()
    }))
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

    temp.into_iter()
}

pub fn data_tab<'a, Message: 'a, T: Media>(media: &T, width: f32) -> Element<'a, Message> {
    let duration = data("Duration", media.duration_short(), CLOCK);

    let rating = data(
        "Rating",
        format!("{}/5", media.rating().unwrap_or_default()),
        STAR,
    );

    let comments = data("Comments", media.comments(), NUMBER);

    let release = data("Release Date", media.release_my(), CALENDAR);

    let added = data("Date Added", media.added_my(), CALENDAR);

    let count = data("Watch Count", media.watch_count(), EYE);

    let progress = (media.progress() * 1000.0).round() / 10.0;
    let progress = data("Watch Progress", format!("{:.1}%", progress), HOURGLASS);

    let recent = data(
        "Recent Watch",
        media.recent_short().unwrap_or(String::from(" --:--:--")),
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

    let content = column!(r1, r2).spacing(30.0);

    content.width(width).into()
}

pub fn draw_collection_tab<'a, Message: 'a + Clone>(
    collection: &'a SimpleCollection,
    on_press: impl Fn(CollectionId) -> Message + 'a,
) -> Element<'a, Message> {
    let size = P;
    let unicode = Icon::new(collection.icon).unicode();
    let icon = icon(unicode).size(size);
    let text = container(text(&collection.name).size(size))
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
        let default = button::subtle(theme, status);

        let border = default.border.rounded(IMAGE_RADIUS);

        button::Style { border, ..default }
    })
    .into()
}

pub fn collection_collage<'a>(
    paths: impl Iterator<Item = &'a str>,
    width: u32,
    height: u32,
) -> Option<Handle> {
    let imgs: Vec<DynamicImage> = paths
        .filter_map(|p| {
            ImageReader::open(p)
                .and_then(|reader| reader.with_guessed_format())
                .inspect_err(|error| eprintln!("Collage generation error on {p}. Error \n{error}"))
                .ok()
                .and_then(|reader| {
                    reader
                        .decode()
                        .inspect_err(|error| {
                            eprintln!("Collage error decoding {p}. Error \n{error}")
                        })
                        .ok()
                })
        })
        .take(3)
        .collect();

    if imgs.is_empty() {
        return None;
    }

    let len = imgs.len();
    let mut canvas: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::new(width, height);

    let mut flip = true;
    let mut img_width = 0;
    let mut img_height = 0;

    for (i, img) in imgs.into_iter().enumerate() {
        let remaining_height = height.saturating_sub(img_height);
        let remaining_width = width.saturating_sub(img_width);
        let last = i == len - 1;

        if flip {
            let width = if last {
                remaining_width
            } else {
                remaining_width / 2
            };
            let height = remaining_height;
            let img = img.resize_to_fill(width, height, FilterType::Triangle);

            if let Err(error) = canvas.copy_from(&img, img_width, img_height) {
                eprintln!("Collection collage error: Error\n{error}");
                continue;
            };

            img_width += width;
        } else {
            let width = remaining_width;
            let height = if last {
                remaining_height
            } else {
                remaining_height / 2
            };
            let img = img.resize_to_fill(width, height, FilterType::Triangle);

            if let Err(error) = canvas.copy_from(&img, img_width, img_height) {
                eprintln!("Collection collage error: Error\n{error}");
                continue;
            };

            img_height += height;
        }

        flip = !flip;
    }

    Some(Handle::from_rgba(
        canvas.width(),
        canvas.height(),
        bytes::Bytes::from(canvas.into_raw()),
    ))
}

#[derive(Debug, Clone)]
pub struct Thumbnail<T: Media> {
    poster: Option<Handle>,
    backdrop: Option<Handle>,
    pub zoom: Animation<bool>,
    pub media: T,
}

impl<T: Media> Thumbnail<T> {
    pub fn new(media: T) -> Self {
        let poster = media.poster().map(Handle::from_path);
        let backdrop = media.backdrop().map(Handle::from_path);

        Self {
            zoom: Animation::new(false)
                .duration(iced::time::Duration::from_millis(100))
                .easing(Easing::EaseInOut),
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

    pub fn poster<'a, Message: 'a>(
        &'a self,
        width: impl Into<Length>,
        height: impl Into<Length>,
    ) -> Element<'a, Message> {
        match &self.poster {
            Some(handle) => widget::image(handle)
                .border_radius(IMAGE_RADIUS)
                .height(height)
                .width(width)
                .content_fit(ContentFit::Contain)
                .into(),
            None => container(empty())
                .height(height)
                .width(width)
                .style(move |theme| {
                    let default = container::dark(theme);
                    let border = default.border.rounded(IMAGE_RADIUS);

                    container::Style { border, ..default }
                })
                .into(),
        }
    }

    pub fn backdrop<'a, Message: 'a>(
        &'a self,
        width: impl Into<Length>,
        height: impl Into<Length>,
    ) -> Element<'a, Message> {
        match &self.backdrop {
            Some(handle) => widget::image(handle)
                .height(height)
                .width(width)
                .content_fit(ContentFit::Cover)
                .into(),
            None => container(empty())
                .height(height)
                .width(width)
                .style(container::dark)
                .into(),
        }
    }

    fn poster_helper<'a, Message: 'a>(&self) -> Element<'a, Message> {
        match &self.poster {
            Some(handle) => widget::image(handle)
                .border_radius(IMAGE_RADIUS)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Fill)
                .into(),

            None => container(empty()).style(container::dark).into(),
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
        let title = text(self.media.name()).size(H5).font(Font {
            weight: Weight::Semibold,
            ..Default::default()
        });

        let ratings = ratings(&self.media);

        let synopsis = synopsis(&self.media);

        let unique = unique(&self.media);

        let bottom = row!(
            progress(&self.media),
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

        let img = container(self.poster_helper()).width(LIST_WIDTH * 1.75);

        let overlay = {
            let size = H1 * 1.5;
            let play = icon(PLAY)
                .size(size)
                .align_x(Horizontal::Center)
                .height(size * self.zoom.interpolate(0.0, 1.0, now));
            let play = mouse_area(
                center(play)
                    .width(1.00 * size * self.zoom.interpolate(0.0, 1.0, now))
                    .height(1.0 * size * self.zoom.interpolate(0.0, 1.0, now))
                    .style(|theme| {
                        let default = container::dark(theme);
                        let background = default
                            .background
                            .map(|background| background.scale_alpha(0.8));
                        let border = default.border.rounded(IMAGE_RADIUS);

                        container::Style {
                            border,
                            background,
                            ..default
                        }
                    }),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press((on_play)(self.media.id()));

            row!(space::horizontal(), play, space::horizontal())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let img = stack![img, overlay];

        let content = row!(img, details)
            .align_y(Vertical::Center)
            .height(LIST_HEIGHT);

        let background_factor = 1.0 * self.zoom.interpolate(0.25, 1.0, now);
        let content = container(content).padding(8).style(move |theme| {
            let default = container::dark(theme);
            let border = default.border.rounded(IMAGE_RADIUS);
            let background = default
                .background
                .map(|background| background.scale_alpha(background_factor));

            container::Style {
                border,
                background,
                ..default
            }
        });

        let content = mouse_area(content)
            .interaction(mouse::Interaction::Pointer)
            .on_press((on_select)(self.media.id()))
            .on_exit((on_hover)(self.media.id(), false))
            .on_enter((on_hover)(self.media.id(), true));

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

        let top = {
            let progress = progress(&self.media);
            let add = mouse_area(icon(BOOKMARK).size(H4))
                .interaction(mouse::Interaction::Pointer)
                .on_press((on_add)(self.media.id()));

            container(
                row!(progress, space::horizontal(), add)
                    .padding(padding)
                    .width(Length::Fill)
                    .align_y(Vertical::Center),
            )
        };

        let details = {
            let font = Font {
                weight: Weight::Semibold,
                ..Default::default()
            };
            let title = container(text(self.media.name()).font(font).size(H7))
                .max_height(20.0)
                .clip(true);
            let ratings = ratings(&self.media);
            let release = {
                let release = text(self.media.release_year()).size(H8).font(font);
                let icon = icon(CALENDAR).size(H7);

                row!(icon, release).align_y(Vertical::Center).spacing(3.0)
            };

            let details = row!(ratings, space::horizontal(), release)
                .width(Length::Fill)
                .align_y(Vertical::Center);

            container(column!(title, details).width(Length::Fill).spacing(4.0)).padding(padding)
        };

        let bottom = {
            let font = Font {
                family: Family::Serif,
                weight: Weight::Semibold,
                ..Default::default()
            };
            let duration = text(self.media.duration_full()).size(H7).font(font);
            row!(space::horizontal(), duration)
                .align_y(Vertical::Center)
                .padding(padding)
        };

        let play = {
            let size = CARD_HEIGHT * 0.135;

            let play = icon(PLAY)
                .size(size)
                .align_x(Horizontal::Center)
                .height(size * self.zoom.interpolate(0.0, 1.0, now));

            let play = center(play)
                .width(1.25 * size * self.zoom.interpolate(0.0, 1.0, now))
                .height(1.1 * size * self.zoom.interpolate(0.0, 1.0, now))
                .style(|theme| {
                    let default = container::dark(theme);
                    let background = default
                        .background
                        .map(|background| background.scale_alpha(0.8));
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
        .interaction(iced::mouse::Interaction::Pointer)
        .on_press((on_play)(self.media.id()));

        let img = self.poster_helper();

        let content = stack![img, overlay].width(CARD_WIDTH);

        let content = column!(content, details)
            .width(CARD_WIDTH)
            .height(CARD_HEIGHT);

        let background_factor = 1.0 * self.zoom.interpolate(0.25, 1.0, now);
        let content = container(content).padding(5).style(move |theme| {
            let default = container::dark(theme);
            let border = default.border.rounded(IMAGE_RADIUS);
            let background = default
                .background
                .map(|background| background.scale_alpha(background_factor));

            container::Style {
                border,
                background,
                ..default
            }
        });

        let content = mouse_area(content)
            .interaction(mouse::Interaction::Pointer)
            .on_press((on_select)(self.media.id()))
            .on_exit((on_hover)(self.media.id(), false))
            .on_enter((on_hover)(self.media.id(), true));

        content.into()
    }
}

#[derive(Debug, Clone)]
pub struct CollectionThumbnail {
    collage: Option<Handle>,
    pub zoom: Animation<bool>,
    pub collection: Collection,
}

impl CollectionThumbnail {
    pub const HEIGHT: u32 = 200;
    pub const WIDTH: u32 = 200;

    pub const CARD_WIDTH: f32 = CARD_WIDTH;
    pub const CARD_HEIGHT: f32 = CARD_HEIGHT * 0.85;

    pub fn new(collection: Collection) -> Self {
        let paths = collection
            .posters
            .iter()
            .filter_map(|poster| poster.as_deref());

        let collage = collection_collage(paths, Self::WIDTH, Self::HEIGHT);

        Self {
            collage,
            collection,
            zoom: Animation::new(false)
                .duration(iced::time::Duration::from_millis(100))
                .easing(Easing::EaseInOut),
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.zoom.is_animating(now)
    }

    pub fn collage<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        match &self.collage {
            Some(handle) => widget::image(handle)
                .border_radius(IMAGE_RADIUS)
                .height(Self::HEIGHT)
                .width(Self::WIDTH)
                .content_fit(ContentFit::Contain)
                .into(),
            None => {
                let len = self.collection.name.len().min(2);
                let name = self.collection.name.get(..len).unwrap_or_default();
                let font = Font {
                    weight: Weight::Bold,
                    family: Family::Cursive,
                    style: Style::Italic,
                    ..Default::default()
                };

                let text = text(name).size(H1 * 2.75).font(font);

                center(text)
                    .height(Self::HEIGHT)
                    .width(Self::WIDTH)
                    .style(move |theme| {
                        let default = container::dark(theme);
                        let border = default.border.rounded(IMAGE_RADIUS);

                        container::Style { border, ..default }
                    })
                    .into()
            }
        }
    }

    pub fn view<'a, Message: 'a + Clone>(
        &'a self,
        now: Instant,
        on_select: impl Fn(CollectionId) -> Message + 'a,
        on_hover: impl Fn(CollectionId, bool) -> Message + 'a,
    ) -> Element<'a, Message> {
        let width = Self::CARD_WIDTH;
        let height = Self::CARD_HEIGHT;
        let padding = [3, 6];

        let name = {
            let title = text(&self.collection.name).size(H6);

            container(title)
                .padding(padding)
                .max_height(24.0)
                .clip(true)
        };

        let img: Element<'_, Message> = {
            match &self.collage {
                Some(handle) => widget::image(handle)
                    .border_radius(IMAGE_RADIUS)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Fill)
                    .into(),

                None => {
                    let len = self.collection.name.len().min(2);
                    let name = self.collection.name.get(..len).unwrap_or_default();
                    let font = Font {
                        weight: Weight::Bold,
                        family: Family::Cursive,
                        style: Style::Italic,
                        ..Default::default()
                    };

                    let text = text(name).size(H1 * 2.75).font(font);

                    center(text)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(move |theme| {
                            let default = container::dark(theme);
                            let border = default.border.rounded(IMAGE_RADIUS);

                            container::Style { border, ..default }
                        })
                        .into()
                }
            }
        };

        let img = container(img).width(width);

        let content = column!(img, name).width(width).height(height);

        let background_factor = 1.0 * self.zoom.interpolate(0.25, 1.0, now);
        let content = container(content).padding(8).style(move |theme| {
            let default = container::dark(theme);
            let border = default.border.rounded(IMAGE_RADIUS);
            let background = default
                .background
                .map(|background| background.scale_alpha(background_factor));

            container::Style {
                border,
                background,
                ..default
            }
        });

        let content = mouse_area(content)
            .interaction(mouse::Interaction::Pointer)
            .on_press((on_select)(self.collection.id))
            .on_exit((on_hover)(self.collection.id, false))
            .on_enter((on_hover)(self.collection.id, true));

        content.into()
    }
}

#[derive(Debug, Clone)]
pub struct SearchView {
    pub item: SearchItem,
    snippet: Vec<markdown::Item>,
}

impl SearchView {
    pub fn new(item: SearchItem) -> Self {
        Self {
            snippet: markdown::parse(&item.snippet).collect(),
            item,
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
        use iced::theme::palette::Pair;

        fn pair(theme: &Theme) -> Pair {
            theme.extended_palette().background.weak
        }

        let separator = || {
            Element::from(text("•").line_height(0.9).size(H5).style(|theme: &Theme| {
                let color = pair(theme).text;
                text::Style { color: Some(color) }
            }))
        };

        let name = {
            let font = Font {
                weight: font::Weight::Medium,
                family: font::Family::Monospace,
                ..Default::default()
            };

            container(
                text(&self.item.name)
                    .size(H6)
                    .font(font)
                    .style(|theme: &Theme| {
                        let color = theme.extended_palette().background.strong.text;
                        text::Style { color: Some(color) }
                    })
                    .width(Length::Fill),
            )
            .clip(true)
            .max_height(24.0)
        };

        let snippet = {
            let settings = markdown::Settings::with_text_size(H7, theme);

            markdown::view(&self.snippet, settings).map(on_url)
        };

        let top = {
            let size = H8;
            let font = Font {
                family: font::Family::Serif,
                style: font::Style::Italic,
                weight: font::Weight::Semibold,
                ..Default::default()
            };

            let media = {
                let media = match &self.item.id {
                    ItemId::Movie(_) => "#movie",
                    ItemId::Show(_) => "#show",
                    ItemId::Season(_) => "#season",
                    ItemId::Episode(_) => "#episode",
                };

                text(media).size(size).font(font).style(|theme| {
                    let color = pair(theme).text;
                    text::Style { color: Some(color) }
                })
            };

            let tags = {
                let mut tags = vec![];
                let tag_len = self.item.tags.len();

                for (i, tag) in self.item.tags.iter().enumerate() {
                    let text = text(tag).size(size).font(font).style(|theme| {
                        let color = pair(theme).text;
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

            row!(media, vert, tags)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let content = column!(top, name, snippet).spacing(2.0);

        let play: Element<'_, Message> = if set_play {
            let size = H1;
            let play = icon(PLAY).size(size).align_x(Horizontal::Center);

            button(play)
                .on_press((on_play)(self.item.id))
                .style(button::text)
                .into()
        } else {
            empty()
        };

        let content = row!(content, play).align_y(Vertical::Center);

        let content = container(content).width(Length::Fill);

        button(content)
            .style(|theme, status| {
                let default = button::subtle(theme, status);
                let border = default.border.rounded(IMAGE_RADIUS);

                button::Style { border, ..default }
            })
            .padding([4, 8])
            .on_press((on_details)(self.item.id))
            .into()
    }
}

#[derive(Debug, Clone)]
pub struct Scroll {
    pub id: widget::Id,
    pub offset: operation::AbsoluteOffset,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            id: widget::Id::unique(),
            offset: operation::AbsoluteOffset::default(),
        }
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icon {
    id: Icons,
}

impl Icon {
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
        }
    }

    pub fn to_u32(self) -> u32 {
        self.id as u32
    }

    pub fn all() -> [Self; 17] {
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
        ]
    }
}
