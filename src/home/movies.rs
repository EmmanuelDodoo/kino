// #![allow(dead_code)]
use super::{PageUpdate, shared::*};
use crate::models::{Media, Movie, MovieId};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort, empty};
use iced::widget::Space;
use iced::{
    Color, ContentFit, Element, Length, Shadow, Subscription, Task,
    alignment::{Horizontal, Vertical},
    time::Instant,
    widget::{
        bottom_center, button, center_x, column, container, grid, image, row, scrollable, stack,
        text,
    },
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
struct MoviePreview {
    tab: Tab,
    id: MovieId,
}

impl MoviePreview {
    pub fn new(id: MovieId) -> Self {
        Self {
            tab: Tab::Items,
            id,
        }
    }

    pub fn overlay<'a, Message>(
        &self,
        thumbnail: &'a Thumbnail<Movie>,
        on_play: impl Fn(MovieId) -> Message,
        on_view: impl Fn(Tab) -> Message,
    ) -> Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        let img: Element<'_, Message> = {
            let img_height = 300.0;
            let ratio = 2.0 / 3.0;
            match &thumbnail.poster {
                Some(handle) => image(handle)
                    .height(img_height)
                    .width(img_height * ratio)
                    .content_fit(ContentFit::Contain)
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

            let title = text(thumbnail.media.name()).size(H4);
            let duration = duration(&thumbnail.media);
            let rating = ratings(&thumbnail.media);
            let release = text(thumbnail.media.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let mut tags = vec![];
            let tag_len = thumbnail.media.tags.len();

            for (i, tag) in thumbnail.media.tags.iter().enumerate() {
                tags.push(Element::from(text(tag).size(H7)));

                if i < tag_len - 1 {
                    tags.push(separator())
                }
            }

            let tags = row(tags).spacing(6).align_y(Vertical::Center);
            column!(title, tags, details, rating)
        };

        let item = "Overview";
        let tabs = Tab::ALL.into_iter().map(|view| {
            let is_selected = self.tab == view;

            Element::from(
                column!(
                    button(text(view.to_str(item)).size(H7))
                        .on_press((on_view)(view))
                        .style(|theme, status| {
                            let default = button::text(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new(68, 4)).style(if is_selected {
                        container::primary
                    } else {
                        container::transparent
                    }),
                )
                .align_x(Horizontal::Center)
                .padding([3, 6])
                .spacing(0.0),
            )
        });

        let tabs = row(tabs).spacing(8.0);

        let view: Element<'_, Message> = {
            let width = 750.0;

            match self.tab {
                Tab::Items => {
                    let synapsis = text(thumbnail.media.synapsis());

                    scrollable(column!(synapsis).spacing(4.0).width(width))
                        .spacing(4.0)
                        .into()
                }
                Tab::Comments => {
                    // todo
                    let comments = ["Some comment here: "; 7]
                        .into_iter()
                        .enumerate()
                        .map(|(i, comment)| Element::from(text(format!("{comment}{i}"))));

                    let comments =
                        scrollable(column(comments).spacing(4.0).width(Length::Fill)).spacing(4.0);

                    column!(comments).spacing(8.0).width(width).into()
                }
                Tab::Data => data_tab(&thumbnail.media, width),
                Tab::Collections => {
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

        let actions = center_x(
            row!(
                button(
                    row!(icon(PLAY).size(H5), text("Play").size(H5))
                        .spacing(16.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press((on_play)(self.id))
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(H5),
                        text("Add to Collection").size(H5)
                    )
                    .spacing(16.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press((on_play)(self.id))
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
            )
            .align_y(Vertical::Center)
            .spacing(16.0),
        );

        let tabs = column!(tabs, view).height(Length::Fill).spacing(16.0);

        let content = column!(header, tabs).spacing(24.0).width(675.0);

        let content = center_x(row!(img, content).spacing(20.0));

        container(column!(content, actions))
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
                    color: Color::BLACK.scale_alpha(0.85),
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
        thumbnail: &'a Thumbnail<Movie>,
        on_play: impl Fn(MovieId) -> Message,
        on_view: impl Fn(Tab) -> Message,
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
    Hovered(MovieId, bool),
    Thumbnails(Vec<Thumbnail<Movie>>),
    Play(MovieId),
    AddCollection(MovieId),
    Details(MovieId),
    Scroll(scrollable::Viewport),
    Tab(Tab),
    Animate,
}

#[derive(Debug, Clone)]
pub struct Movies {
    now: Instant,
    thumbnails: HashMap<MovieId, Thumbnail<Movie>>,
    grid: bool,
    focused: Option<MovieId>,
    sort: Sort,
    filter: Filter,
    preview: Option<MoviePreview>,
    preview_back: Option<MoviePreview>,
    scroll: Scroll,
}

impl Movies {
    pub fn boot(sort: Sort, filters: Filter, grid: bool) -> (Self, Task<MoviesMessage>) {
        let load_thumbnails = Task::perform(
            async {
                let alt = (6..12).map(|_| Movie::testing2());
                (0..6)
                    .map(|_| Movie::testing())
                    .chain(alt)
                    .collect::<Vec<_>>()
            },
            |videos| MoviesMessage::Thumbnails(videos.into_iter().map(Thumbnail::new).collect()),
        );

        let (new, id) = Self::new(sort, grid, filters);
        let scroll = scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, load_thumbnails.chain(scroll))
    }

    pub fn dummies(
        sort: Sort,
        filters: Filter,
        grid: bool,
        movies: Vec<Movie>,
    ) -> (Self, Task<MoviesMessage>) {
        let task = Task::perform(async move { movies }, |movies| {
            MoviesMessage::Thumbnails(movies.into_iter().map(Thumbnail::new).collect())
        });

        let (new, id) = Self::new(sort, grid, filters);
        let scroll = scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, task.chain(scroll))
    }

    fn new(sort: Sort, grid: bool, filter: Filter) -> (Self, scrollable::Id) {
        let now = Instant::now();
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now,
                thumbnails: HashMap::default(),
                focused: None,
                grid,
                sort,
                filter,
                preview: None,
                preview_back: None,
                scroll,
            },
            id,
        )
    }

    pub fn preview(&mut self, id: MovieId) -> Option<Task<MoviesMessage>> {
        self.preview = Some(MoviePreview::new(id));
        self.preview_back = None;
        self.focused = None;

        match self.thumbnails.get_mut(&id) {
            Some(thumbnail) => {
                thumbnail.zoom.go_mut(false, self.now);
                None
            }
            None => {
                todo!("Should fetch movie if not already present.")
            }
        }
    }

    pub fn update(&mut self, message: MoviesMessage, now: Instant) -> Task<MoviesMessage> {
        self.now = now;

        match message {
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
                self.preview(id);
                Task::none()
            }
            MoviesMessage::AddCollection(id) => {
                println!("Add {id:?} to collection pressed");
                Task::none()
            }
            MoviesMessage::Thumbnails(thumbnails) => {
                for thumbnail in thumbnails {
                    self.thumbnails.insert(thumbnail.media.id(), thumbnail);
                }

                Task::none()
            }
            MoviesMessage::Tab(view) => {
                if let Some(preview) = self.preview.as_mut() {
                    preview.tab = view;
                }
                Task::none()
            }
            MoviesMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                Task::none()
            }
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        let PageUpdate {
            layout,
            sort,
            filters,
        } = update;

        self.sort = sort;
        self.grid = matches!(layout, Layout::Grid);
        self.filter = filters;
    }

    pub fn name(&self) -> String {
        self.preview
            .and_then(|preview| {
                self.thumbnails
                    .get(&preview.id)
                    .map(|thumbnail| thumbnail.media.name().to_owned())
            })
            .unwrap_or(String::from("Movies"))
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

    pub fn rand(&mut self) -> Task<()> {
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        let temp = self.thumbnails.keys().collect::<Vec<_>>();

        if let Some(rand) = temp.choose(&mut rng).copied() {
            self.preview(*rand);
        }

        Task::none()
    }

    pub fn refresh(&mut self) -> Task<MoviesMessage> {
        todo!()
    }

    fn grid(&self) -> Element<'_, MoviesMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filter, &self.sort).map(|thumbnail| {
                thumbnail.card(
                    self.now,
                    MoviesMessage::AddCollection,
                    MoviesMessage::Details,
                    MoviesMessage::Hovered,
                    MoviesMessage::Play,
                )
            });

        let content = grid(content)
            .spacing(16)
            .fluid(CARD_WIDTH)
            .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn list(&self) -> Element<'_, MoviesMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filter, &self.sort).map(|thumbnail| {
                thumbnail.list(
                    self.now,
                    MoviesMessage::AddCollection,
                    MoviesMessage::Details,
                    MoviesMessage::Hovered,
                    MoviesMessage::Play,
                    unique,
                )
            });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(MoviesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    pub fn view(&self) -> Element<'_, MoviesMessage> {
        match self.preview {
            Some(preview) => {
                let thumbnail = self
                    .thumbnails
                    .get(&preview.id)
                    .expect("Preview Id missing");

                preview.view(thumbnail, MoviesMessage::Play, MoviesMessage::Tab)
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

    pub fn update_scroll(&mut self) -> Task<()> {
        scrollable::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        let preview = self.preview.take()?;

        self.preview_back = Some(preview);

        Some(self.update_scroll())
    }

    pub fn forward(&mut self) -> Option<Task<()>> {
        match self.preview_back.take() {
            Some(preview) => {
                self.preview = Some(preview);
                Some(Task::none())
            }
            None => None,
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

pub fn unique<'a, Message: 'a>(movie: &Movie) -> Element<'a, Message> {
    let release = text(movie.release_year()).size(H7);
    let icon = icon(CALENDAR).size(H7);

    row!(icon, release)
        .align_y(Vertical::Center)
        .spacing(3.0)
        .into()
}
