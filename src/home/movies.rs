#![allow(dead_code)]
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
        column, container, float, grid, horizontal_space, image, mouse_area, row, scrollable,
        stack, text, vertical_space,
    },
};
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Clone)]
pub struct Thumbnail {
    handle: Option<image::Handle>,
    zoom: Animation<bool>,
    video: Video,
}

impl Thumbnail {
    const CARD_WIDTH: f32 = 275.0;
    const CARD_HEIGHT: f32 = 275.0;
    const LIST_HEIGHT: f32 = 125.0;
    const LIST_WIDTH: f32 = Self::LIST_HEIGHT * 1.5 / 1.0;

    pub fn new(video: Video) -> Self {
        let handle = video.poster.as_ref().map(image::Handle::from_path);

        Self {
            zoom: Animation::new(false).very_quick().easing(Easing::EaseInOut),
            handle,
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
        let hrs = self.video.duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs} hour{}", if hrs > 1 { "s" } else { "" })
        } else {
            String::default()
        };

        let mins = (self.video.duration % 3600) / 60;
        let mins = if mins > 0 {
            format!("{mins} min{}", if mins > 1 { "s" } else { "" })
        } else {
            String::default()
        };

        let duration = format!("{hrs} {mins}",);

        text(duration).size(H7)
    }

    fn ratings(&self) -> Element<'_, MoviesMessage> {
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

        let bottom = row!(self.progress(), self.duration(), horizontal_space(), add)
            .spacing(24)
            .align_y(Vertical::Center)
            .width(Length::Fill);

        let details = column!(title, ratings, vertical_space(), bottom).spacing(8);

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

    return std::cmp::Ordering::Equal;
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

#[derive(Debug, Clone)]
pub enum MoviesMessage {
    Hovered(VideoId, bool),
    Thumbnails(Vec<Thumbnail>),
    Play(VideoId),
    AddCollection(VideoId),
    Details(VideoId),
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
}

impl Movies {
    pub fn boot(sort: Sort, filters: Filter, grid: bool) -> (Self, Task<MoviesMessage>) {
        let load_thumbnails = Task::perform(
            async {
                let alt = (6..12).map(|i| Video::testing2(i));
                (0..6)
                    .map(|i| Video::testing(i))
                    .chain(alt)
                    .collect::<Vec<_>>()
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
                println!("Details {id:?} pressed");
                self.focused = Some(id);
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
        "Movies"
    }

    pub fn can_back(&self) -> bool {
        false
    }

    pub fn can_forward(&self) -> bool {
        false
    }

    fn thumbnails(&self) -> impl Iterator<Item = &Thumbnail> {
        let mut temp = self
            .thumbnails
            .iter()
            .map(|(_, thumnail)| thumnail)
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
        let content = if self.grid { self.grid() } else { self.list() };

        content.into()
    }

    fn is_animating(&self) -> bool {
        self.focused
            .as_ref()
            .and_then(|id| self.thumbnails.get(id))
            .map(|thumbnail| thumbnail.is_animating(self.now))
            .unwrap_or_default()
    }

    pub fn subscription(&self) -> Subscription<MoviesMessage> {
        if self.is_animating() {
            iced::window::frames().map(|_| MoviesMessage::Animate)
        } else {
            Subscription::none()
        }
    }
}
