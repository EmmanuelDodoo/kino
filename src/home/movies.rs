// #![allow(dead_code)]
use super::PageUpdate;
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Sort, SortKind, ViewType, empty};
use crate::video::{Video, VideoId};
use iced::{
    Color, ContentFit, Element, Length, Shadow, Subscription, Task,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    mouse,
    time::Instant,
    widget::{
        bottom_center, button, center_x, column, container, float, grid, horizontal_space, image,
        mouse_area, row, scrollable, stack, text, vertical_space,
    },
};
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum View {
    #[default]
    /// Video overview
    Info,
    /// Comments on Video
    Comments,
    /// Video details: duration, comments no, release, ratings, added, watch progress, watch count, recent,
    Data,
    /// Collection memberships
    Collections,
}

impl View {
    pub const ALL: [Self; 4] = [Self::Info, Self::Data, Self::Comments, Self::Collections];
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Info => "Overview",
                Self::Comments => "Comments",
                Self::Data => "Data",
                Self::Collections => "Collections",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Thumbnail {
    handle: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    zoom: Animation<bool>,
    video: Video,
}

impl Thumbnail {
    const CARD_WIDTH: f32 = 275.0;
    const CARD_HEIGHT: f32 = 275.0;
    const LIST_HEIGHT: f32 = 160.0;
    const LIST_WIDTH: f32 = Self::LIST_HEIGHT * 1.5 / 1.0;

    pub fn new(video: Video) -> Self {
        let handle = video.poster.as_ref().map(image::Handle::from_path);
        let backdrop = video.backdrop.as_ref().map(image::Handle::from_path);

        Self {
            zoom: Animation::new(false).very_quick().easing(Easing::EaseInOut),
            handle,
            backdrop,
            video,
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.zoom.is_animating(now)
    }

    fn image(&self) -> Element<'_, MoviesMessage> {
        match &self.handle {
            Some(handle) => image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Cover)
                .into(),
            None => container(empty()).style(container::dark).into(),
        }
    }

    fn duration(&self) -> text::Text<'_, iced::Theme, iced::Renderer> {
        text(self.duration_full()).size(H7)
    }

    fn ratings<'a, Message: 'a>(&self) -> Element<'a, Message> {
        let unstars = (5 - self.video.rating).clamp(0, 5);
        let stars = (0..self.video.rating).map(|_| Element::from(icon(STAR).size(H7)));
        let unstars = (0..unstars).map(|_| Element::from(icon(UNSTAR).size(H7)));
        let ratings = row(stars.chain(unstars))
            .spacing(2.0)
            .align_y(Vertical::Center);

        ratings.into()
    }

    fn progress(&self) -> Element<'_, MoviesMessage> {
        let progress = match self.video.progress {
            ..0.15 => PROGRESS_10,
            0.15..0.3 => PROGRESS_20,
            0.3..0.5 => PROGRESS_40,
            0.5..0.7 => PROGRESS_60,
            0.7..0.85 => PROGRESS_80,
            x if x < 1.0 => PROGRESS_90,
            _ => PROGRESS_100,
        };

        let text = text(format!("{}%", self.video.progress * 100.0)).size(H7);

        let icon = icon(progress).size(H4);

        row!(icon, text)
            .spacing(3.0)
            .align_y(Vertical::Center)
            .into()
    }

    pub fn list(&self, now: Instant) -> Element<'_, MoviesMessage> {
        let title = text(&self.video.name).size(H6);

        let ratings = self.ratings();

        let add = mouse_area(icon(BOOKMARK).size(H3))
            .interaction(mouse::Interaction::Pointer)
            .on_press(MoviesMessage::AddCollection(self.video.id));

        let synapsis = text(&self.synapsis).size(H7).height(52.0);

        let bottom = row!(self.progress(), self.duration(), horizontal_space(), add)
            .spacing(24)
            .align_y(Vertical::Center)
            .width(Length::Fill);

        let details = column!(title, ratings, synapsis,vertical_space(), bottom).spacing(8);

        let details = mouse_area(
            container(details)
                .style(container::dark)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([5, 10]),
        )
        .interaction(mouse::Interaction::Pointer)
        .on_press(MoviesMessage::Details(self.video.id));

        let img = container(self.image()).width(Self::LIST_WIDTH);

        let overlay = {
            let size = H2;
            let play = mouse_area(
                icon(PLAY)
                    .size(size)
                    .align_x(Horizontal::Center)
                    .height(size * self.zoom.interpolate(0.0, 1.0, now)),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press(MoviesMessage::Play(self.video.id));

            row!(horizontal_space(), play, horizontal_space())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let img = stack![img, overlay];

        let content = mouse_area(
            row!(img, details)
                .align_y(Vertical::Center)
                .height(Self::LIST_HEIGHT),
        )
        .on_exit(MoviesMessage::Hovered(self.video.id, false))
        .on_enter(MoviesMessage::Hovered(self.video.id, true));

        let content = float(content)
            .scale(self.zoom.interpolate(1.0, 1.025, now))
            .translate(move |bounds, viewport| {
                bounds.zoom(1.025).offset(&viewport.shrink(5))
                    * self.zoom.interpolate(0.0, 1.0, now)
            })
            .style(move |_theme| float::Style {
                shadow: Shadow {
                    color: Color::BLACK.scale_alpha(self.zoom.interpolate(0.0, 1.0, now)),
                    blur_radius: self.zoom.interpolate(0.0, 20.0, now),
                    ..Shadow::default()
                },
                ..float::Style::default()
            });

        content.into()
    }

    pub fn card(&self, now: Instant) -> Element<'_, MoviesMessage> {
        let padding = [3, 6];

        let top = {
            let progress = self.progress();
            let add = mouse_area(icon(BOOKMARK).size(H4))
                .interaction(mouse::Interaction::Pointer)
                .on_press(MoviesMessage::AddCollection(self.video.id));

            container(
                row!(progress, horizontal_space(), add)
                    .padding(padding)
                    .width(Length::Fill)
                    .align_y(Vertical::Center),
            )
        };

        let details = {
            let title = text(&self.video.name).size(H7);
            let ratings = self.ratings();

            let details = row!(ratings, horizontal_space(), self.duration())
                .width(Length::Fill)
                .align_y(Vertical::Center);

            mouse_area(
                container(column!(title, details).width(Length::Fill).spacing(10.0))
                    .padding(padding)
                    .style(container::dark),
            )
            .interaction(mouse::Interaction::Pointer)
            .on_press(MoviesMessage::Details(self.video.id))
        };

        let play = {
            let size = H2 * 1.75;

            let play = mouse_area(
                icon(PLAY)
                    .size(size)
                    .align_x(Horizontal::Center)
                    .height(size * self.zoom.interpolate(0.0, 1.0, now)),
            )
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press(MoviesMessage::Play(self.video.id));

            row!(horizontal_space(), play, horizontal_space())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Vertical::Center)
        };

        let overlay = column!(top, vertical_space(), play, vertical_space())
            .width(Length::Fill)
            .height(Length::Fill);

        let img: Element<'_, MoviesMessage> = self.image();

        let content = stack![img, overlay].width(Thumbnail::CARD_WIDTH);

        let content = column!(content, details);

        let content = mouse_area(content)
            .on_exit(MoviesMessage::Hovered(self.video.id, false))
            .on_enter(MoviesMessage::Hovered(self.video.id, true));

        let content = float(content)
            .scale(self.zoom.interpolate(1.0, 1.025, now))
            .translate(move |bounds, viewport| {
                bounds.zoom(1.025).offset(&viewport.shrink(5))
                    * self.zoom.interpolate(0.0, 1.0, now)
            })
            .style(move |_theme| float::Style {
                shadow: Shadow {
                    color: Color::BLACK.scale_alpha(self.zoom.interpolate(0.0, 1.0, now)),
                    blur_radius: self.zoom.interpolate(0.0, 20.0, now),
                    ..Shadow::default()
                },
                ..float::Style::default()
            });

        content.into()
    }
}

impl Deref for Thumbnail {
    type Target = Video;

    fn deref(&self) -> &Self::Target {
        &self.video
    }
}

fn sort(x: &Video, y: &Video, sorts: &[SortKind]) -> std::cmp::Ordering {
    for kind in sorts.iter() {
        let ord = match kind {
            SortKind::Name => x.name.cmp(&y.name),
            SortKind::Duration => x.duration.cmp(&y.duration),
            SortKind::Added => x.added.cmp(&y.added),
            SortKind::Rating => x.rating.cmp(&y.rating),
            SortKind::Recent => x.recent.cmp(&y.recent),
            SortKind::Release => x.release.cmp(&y.release),
            SortKind::Progress => x.progress.total_cmp(&y.progress),
            SortKind::Comments => x.comments.cmp(&y.comments),
        };

        if !matches!(ord, std::cmp::Ordering::Equal) {
            return ord;
        }
    }

    std::cmp::Ordering::Equal
}

fn filter(video: &Video, filter: Filter) -> bool {
    let progress = filter.progress.compare(video.progress);
    let rating = filter.rating.compare(video.rating);
    let comments = filter
        .comments
        .map(|comments| comments.compare(video.comments))
        .unwrap_or_else(|| matches!(filter.mode, FilterMode::And));
    let release = filter
        .release
        .map(|release| release.compare(video.release))
        .unwrap_or_else(|| matches!(filter.mode, FilterMode::And));
    let duration = filter
        .duration
        .map(|duration| duration.compare(video.duration))
        .unwrap_or_else(|| matches!(filter.mode, FilterMode::And));

    filter
        .mode
        .compare_many(&[progress, rating, comments, release, duration])
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Preview {
    view: View,
    id: VideoId,
}

impl Preview {
    pub fn new(id: VideoId) -> Self {
        Self {
            view: View::Info,
            id,
        }
    }

    pub fn overlay<'a, Message>(
        &self,
        thumbnail: &'a Thumbnail,
        on_play: impl Fn(VideoId) -> Message,
        on_view: impl Fn(View) -> Message,
    ) -> Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        let img: Element<'_, Message> = {
            let img_height = 300.0;
            let ratio = 1.0;
            match &thumbnail.poster {
                Some(handle) => image(handle)
                    .height(img_height)
                    .width(img_height * ratio)
                    .content_fit(ContentFit::Cover)
                    .into(),
                None => container(empty())
                    .height(img_height)
                    .width(img_height * ratio)
                    .style(container::dark)
                    .into(),
            }
        };

        let header = {
            let separator = || Element::from(text("•").size(H3));

            let title = text(&thumbnail.name).size(H4);
            let duration = thumbnail.duration();
            let rating = thumbnail.ratings();
            let release = text(thumbnail.release).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let mut tags = vec![];
            let tag_len = thumbnail.tags.len();

            for (i, tag) in thumbnail.tags.iter().enumerate() {
                tags.push(Element::from(text(tag).size(H7)));

                if i < tag_len - 1 {
                    tags.push(separator())
                }
            }

            let tags = row(tags).spacing(6).align_y(Vertical::Center);
            column!(title, tags, details, rating)
        };

        let tabs = View::ALL.into_iter().map(|view| {
            let is_selected = self.view == view;

            Element::from(
                button(text(view.to_string()).size(H7))
                    .padding([3, 6])
                    .on_press((on_view)(view))
                    .style(move |theme, status| {
                        use button::{Status, Style};
                        let default = button::text(theme, status);
                        let border = default.border.rounded(5.0);
                        let default = Style { border, ..default };

                        match status {
                            Status::Active if is_selected => {
                                let background = theme.extended_palette().background.neutral;
                                Style {
                                    background: Some(background.color.into()),
                                    text_color: background.text,
                                    ..default
                                }
                            }
                            _ => default,
                        }
                    }),
            )
        });

        let tabs = row(tabs).spacing(8.0);

        let view: Element<'_, Message> = {
            let width = 750;

            match self.view {
                View::Info => {
                    let synapsis = text(&thumbnail.synapsis);

                    scrollable(column!(synapsis).spacing(4.0).width(width))
                        .spacing(4.0)
                        .into()
                }
                View::Comments => {
                    // todo
                    let comments = ["Some comment here: "; 7]
                        .into_iter()
                        .enumerate()
                        .map(|(i, comment)| Element::from(text(format!("{comment}{i}"))));

                    let comments =
                        scrollable(column(comments).spacing(4.0).width(Length::Fill)).spacing(4.0);

                    column!(comments).spacing(8.0).width(width).into()
                }
                View::Data => {
                    fn data<'a, Message: 'a>(
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

                    let duration = data("Duration", thumbnail.duration_short(), CLOCK);

                    let rating = data("Rating", format!("{}/5", thumbnail.rating), STAR);

                    let comments = data("Comments", thumbnail.comments, NUMBER);

                    let release = data("Release Date", thumbnail.release_short(), CALENDAR);

                    let added = data("Date Added", thumbnail.added_short(), CALENDAR);

                    let count = data("Watch Count", thumbnail.watch_count, EYE);

                    let progress = data(
                        "Watch Progress",
                        format!("{}%", thumbnail.progress * 100.0),
                        HOURGLASS,
                    );

                    let recent = data("Recent Watch", thumbnail.release_short(), CALENDAR);

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
                View::Collections => {
                    // todo
                    let collections = ["Some Collection here: "; 7]
                        .into_iter()
                        .enumerate()
                        .map(|(i, collection)| Element::from(text(format!("{collection}{i}"))));

                    let collections =
                        scrollable(column(collections).spacing(4.0).width(Length::Fill))
                            .spacing(4.0);

                    column!(collections).spacing(8.0).width(width).into()
                }
            }
        };

        let play = center_x(
            button(
                row!(icon(PLAY).size(H5), text("Play").size(H5))
                    .spacing(16.0)
                    .align_y(Vertical::Center),
            )
            .padding([6, 12])
            .on_press((on_play)(self.id))
            .style(|theme, status| {
                let default = button::background(theme, status);
                let border = default.border.rounded(5);

                button::Style { border, ..default }
            }),
        );

        let tabs = column!(tabs, view).height(Length::Fill).spacing(16.0);

        let content = column!(header, tabs).spacing(24.0).width(675.0);

        let content = row!(img, content).spacing(20.0);

        container(column!(content,  play))
            .padding([20, 28])
            .max_height(465.0)
            .align_x(Horizontal::Center)
            .width(Length::Fill)
            .style(|theme| {
                let default = container::dark(theme);
                let background = default
                    .background
                    .map(|background| background.scale_alpha(0.75));

                let shadow = default.shadow;
                let shadow = Shadow {
                    color: Color::BLACK.scale_alpha(0.75),
                    blur_radius: 20.0,
                    ..shadow
                };

                container::Style {
                    background,
                    shadow,
                    ..default
                }
            })
            .into()
    }

    fn view<'a, Message>(
        &self,
        thumbnail: &'a Thumbnail,
        on_play: impl Fn(VideoId) -> Message,
        on_view: impl Fn(View) -> Message,
    ) -> Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        let overlay = bottom_center(self.overlay(thumbnail, on_play, on_view));

        let img: Element<'_, Message> = match &thumbnail.backdrop {
            Some(handle) => image(handle)
                .width(Length::Fill)
                .height(Length::FillPortion(3))
                .content_fit(ContentFit::Cover)
                .into(),
            None => container(empty())
                .width(Length::Fill)
                .height(Length::FillPortion(3))
                .style(container::dark)
                .into(),
        };

        let content = container(column!(img,)).style(container::dark);

        let content = stack![content, overlay];

        content.into()
    }
}

#[derive(Debug, Clone)]
pub enum MoviesMessage {
    Hovered(VideoId, bool),
    Thumbnails(Vec<Thumbnail>),
    Play(VideoId),
    AddCollection(VideoId),
    Details(VideoId),
    View(View),
    Animate,
    None,
}

#[derive(Debug, Clone)]
pub struct Movies {
    now: Instant,
    thumbnails: HashMap<VideoId, Thumbnail>,
    grid: bool,
    focused: Option<VideoId>,
    sort: Sort,
    filter: Filter,
    preview: Option<Preview>,
    preview_back: Option<Preview>,
}

impl Movies {
    pub fn boot(sort: Sort, filters: Filter, grid: bool) -> (Self, Task<MoviesMessage>) {
        let load_thumbnails = Task::perform(
            async {
                let alt = (6..12).map(Video::testing2);
                (0..6).map(Video::testing).chain(alt).collect::<Vec<_>>()
            },
            |videos| MoviesMessage::Thumbnails(videos.into_iter().map(Thumbnail::new).collect()),
        );

        (
            Self::new(sort, grid, filters),
            Task::batch([load_thumbnails]),
        )
    }

    fn new(sort: Sort, grid: bool, filter: Filter) -> Self {
        let now = Instant::now();
        Self {
            now,
            thumbnails: HashMap::default(),
            focused: None,
            grid,
            sort,
            filter,
            preview: None,
            preview_back: None,
        }
    }

    pub fn update(&mut self, message: MoviesMessage, now: Instant) -> Task<MoviesMessage> {
        self.now = now;

        match message {
            MoviesMessage::None => Task::none(),
            MoviesMessage::Animate => Task::none(),
            MoviesMessage::Hovered(id, is_hovered) => {
                let Some(thumbnail) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };

                thumbnail.zoom.go_mut(is_hovered, self.now);
                self.focused = Some(id);
                Task::none()
            }
            MoviesMessage::Play(id) => {
                println!("Play {id:?} pressed");
                Task::none()
            }
            MoviesMessage::Details(id) => {
                self.preview = Some(Preview {
                    id,
                    view: View::Info,
                });
                self.preview_back = None;
                self.focused = None;
                Task::none()
            }
            MoviesMessage::AddCollection(id) => {
                println!("Add {id:?} to collection pressed");
                Task::none()
            }
            MoviesMessage::Thumbnails(thumbnails) => {
                for thumbnail in thumbnails {
                    self.thumbnails.insert(thumbnail.video.id, thumbnail);
                }

                Task::none()
            }
            MoviesMessage::View(view) => {
                if let Some(preview) = self.preview.as_mut() {
                    preview.view = view;
                }
                Task::none()
            }
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        match update {
            PageUpdate::Sort(sort) => self.sort = sort,
            PageUpdate::Layout(kind) => self.grid = matches!(kind, ViewType::Grid),
            PageUpdate::Filters(filters) => self.filter = filters,
        }
    }

    pub fn name(&self) -> &str {
        self.preview
            .and_then(|preview| {
                self.thumbnails
                    .get(&preview.id)
                    .map(|thumbnail| thumbnail.name.as_str())
            })
            .unwrap_or("Movies")
    }

    pub fn can_back(&self) -> bool {
        self.preview.is_some()
    }

    pub fn can_forward(&self) -> bool {
        self.preview_back.is_some()
    }

    pub fn show_tools(&self) -> bool {
        self.preview.is_none()
    }

    fn thumbnails(&self) -> impl Iterator<Item = &Thumbnail> {
        let mut temp = self
            .thumbnails
            .values()
            .filter(|thumbnail| filter(&thumbnail.video, self.filter))
            .collect::<Vec<_>>();

        temp.sort_by(|x, y| sort(&x.video, &y.video, &self.sort.kinds));

        if self.sort.reverse {
            temp.reverse();
        }

        temp.into_iter()
    }

    fn grid(&self) -> Element<'_, MoviesMessage> {
        let content = self.thumbnails().map(|thumbnail| thumbnail.card(self.now));

        let content = grid(content)
            .spacing(16)
            .fluid(Thumbnail::CARD_WIDTH)
            .height(grid::aspect_ratio(
                Thumbnail::CARD_WIDTH,
                Thumbnail::CARD_HEIGHT,
            ));

        let content = container(scrollable(content).spacing(20.0)).padding(10);

        content.into()
    }

    fn list(&self) -> Element<'_, MoviesMessage> {
        let content = self.thumbnails().map(|thumbnail| thumbnail.list(self.now));

        let content = column(content).spacing(16);

        let content = container(scrollable(content).spacing(20.0)).padding(10);

        content.into()
    }

    pub fn view(&self) -> Element<'_, MoviesMessage> {
        match self.preview {
            Some(preview) => {
                let thumbnail = self
                    .thumbnails
                    .get(&preview.id)
                    .expect("Preview Id missing");

                preview.view(thumbnail, MoviesMessage::Play, MoviesMessage::View)
            }
            None if self.grid => self.grid(),
            None => self.list(),
        }
    }

    fn is_animating(&self) -> bool {
        self.focused
            .as_ref()
            .and_then(|id| self.thumbnails.get(id))
            .map(|thumbnail| thumbnail.is_animating(self.now))
            .unwrap_or_default()
    }

    pub fn back(&mut self) -> bool {
        let Some(preview) = self.preview.take() else {
            return false;
        };

        self.preview_back = Some(preview);
        true
    }

    pub fn forward(&mut self) -> bool {
        match self.preview_back.take() {
            Some(preview) => {
                self.preview = Some(preview);
                false
            }
            None => true,
        }
    }

    pub fn subscription(&self) -> Subscription<MoviesMessage> {
        if self.is_animating() {
            iced::window::frames().map(|_| MoviesMessage::Animate)
        } else {
            Subscription::none()
        }
    }
}
