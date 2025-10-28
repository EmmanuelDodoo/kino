use super::{PageUpdate, shared::*};
use crate::models::{Media, shows::*};
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
        self, bottom_center, button, center_x, column, container, grid, image, operation, row,
        rule, scrollable, space, stack, text,
    },
    window,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EpisodePreview {
    pub tab: Tab,
    pub name: String,
    pub id: EpisodeId,
}

impl EpisodePreview {
    pub fn new(id: EpisodeId, name: String) -> Self {
        Self {
            id,
            name,
            tab: Tab::Items,
        }
    }

    fn overlay<'a, Message>(
        &self,
        thumbnail: &'a Thumbnail<Episode>,
        on_play: impl Fn(EpisodeId) -> Message,
        on_tab: impl Fn(Tab) -> Message,
        on_collection: impl Fn(EpisodeId) -> Message,
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

            column!(title, details, rating)
        };

        let item = "Overview";
        let tabs = Tab::ALL.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .on_press((on_tab)(tab))
                        .style(|theme, status| {
                            let default = button::text(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new().width(68).height(4)).style(if is_selected {
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
                .on_press((on_collection)(self.id))
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

    pub fn view<'a, Message>(
        &self,
        thumbnail: &'a Thumbnail<Episode>,
        on_play: impl Fn(EpisodeId) -> Message,
        on_tab: impl Fn(Tab) -> Message,
        on_collection: impl Fn(EpisodeId) -> Message,
    ) -> Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        let overlay = bottom_center(self.overlay(thumbnail, on_play, on_tab, on_collection));

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
pub enum TvSeasonMessage {
    Thumbnails(Vec<Thumbnail<Episode>>),
    Animate,
    AddCollectionSelf,
    AddCollection(EpisodeId),
    Hovered(EpisodeId, bool),
    Selected(EpisodeId),
    Resume,
    Play(EpisodeId),
    Tab(Tab),
    PreviewTab(Tab),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub struct TvSeason {
    season: Season,
    poster: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    now: Instant,
    layout: Layout,
    thumbnails: HashMap<EpisodeId, Thumbnail<Episode>>,
    focused: Option<EpisodeId>,
    sort: Sort,
    filters: Filter,
    tab: Tab,
    selected: Option<EpisodePreview>,
    selected_prev: Option<EpisodePreview>,
    scroll: Scroll,
}

impl TvSeason {
    pub fn boot(
        season: Season,
        sort: Sort,
        filters: Filter,
        layout: Layout,
    ) -> (Self, Task<TvSeasonMessage>) {
        let thumbnails = Task::perform(
            async {
                let alt = (0..6).map(|_| Episode::testing());
                (6..12)
                    .map(|_| Episode::testing())
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            TvSeasonMessage::Thumbnails,
        );
        let (new, id) = Self::new(season, sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());
        let tasks = thumbnails.chain(scroll);

        (new, tasks)
    }

    fn new(season: Season, sort: Sort, filters: Filter, layout: Layout) -> (Self, widget::Id) {
        let poster = season.poster().and_then(round_image);
        let backdrop = season.backdrop().map(image::Handle::from_path);
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                poster,
                backdrop,
                season,
                layout,
                sort,
                filters,
                focused: None,
                thumbnails: HashMap::default(),
                tab: Tab::Items,
                selected: None,
                selected_prev: None,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: TvSeasonMessage, now: Instant) -> Task<TvSeasonMessage> {
        self.now = now;

        match message {
            TvSeasonMessage::Animate => Task::none(),
            TvSeasonMessage::Thumbnails(thumbnails) => {
                for thumbnail in thumbnails {
                    self.thumbnails.insert(thumbnail.id(), thumbnail);
                }

                Task::none()
            }
            TvSeasonMessage::Hovered(id, is_hovered) => {
                let Some(thumbnail) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };

                thumbnail.zoom.go_mut(is_hovered, self.now);
                self.focused = Some(id);
                Task::none()
            }
            TvSeasonMessage::AddCollectionSelf => {
                println!("Add show to collection pressed");
                Task::none()
            }
            TvSeasonMessage::AddCollection(id) => {
                println!("Add {id:?} to collection pressed");
                Task::none()
            }
            TvSeasonMessage::Selected(id) => self.preview(id).unwrap_or(Task::none()),
            TvSeasonMessage::Tab(tab) => {
                self.tab = tab;
                Task::none()
            }
            TvSeasonMessage::Resume => {
                println!("Resume season playback");
                Task::none()
            }
            TvSeasonMessage::Play(episode) => {
                println!("Resume episode {episode:?} playback");
                Task::none()
            }
            TvSeasonMessage::PreviewTab(tab) => {
                if let Some(preview) = self.selected.as_mut() {
                    preview.tab = tab;
                }

                Task::none()
            }
            TvSeasonMessage::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                Task::none()
            }
        }
    }

    pub fn preview(&mut self, id: EpisodeId) -> Option<Task<TvSeasonMessage>> {
        self.focused = None;

        match self.thumbnails.get_mut(&id) {
            Some(thumbnail) => {
                self.selected = Some(EpisodePreview::new(id, thumbnail.media.name().to_owned()));
                thumbnail.zoom.go_mut(false, self.now);
                self.selected_prev = None;
                None
            }
            None => todo!("Fetch episode here?"),
        }
    }

    pub fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
            thumbnail.zoom.go_mut(false, self.now);
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
        self.layout = layout;
        self.filters = filters;
    }

    pub fn name(&self) -> String {
        let name = self.selected.as_ref().map(|preview| &preview.name);

        match name {
            Some(selected) => format!("{} - {selected}", self.season.name()),
            None => self.season.name().to_owned(),
        }
    }

    pub fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    pub fn can_forward(&self) -> bool {
        self.selected_prev.is_some()
    }

    pub fn show_tools(&self) -> bool {
        self.selected.is_none()
    }

    pub fn rand(&mut self) -> Task<TvSeasonMessage> {
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        let temp = self.thumbnails.keys().collect::<Vec<_>>();

        let Some(rand) = temp.choose(&mut rng).copied().copied() else {
            return Task::none();
        };

        self.preview(rand).unwrap_or(Task::none())
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let selected = self.selected.take()?;

        self.selected_prev = Some(selected);

        Some(self.update_scroll())
    }

    pub fn forward(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let Some(prev) = self.selected_prev.take() else {
            return None;
        };

        self.selected = Some(prev);

        Some(self.update_scroll())
    }

    fn list(&self) -> Element<'_, TvSeasonMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.list(
                    self.now,
                    TvSeasonMessage::AddCollection,
                    TvSeasonMessage::Selected,
                    TvSeasonMessage::Hovered,
                    TvSeasonMessage::Play,
                    |_| empty(),
                )
            });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(TvSeasonMessage::Scroll),
        );

        content.into()
    }

    fn grid(&self) -> Element<'_, TvSeasonMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.card(
                    self.now,
                    TvSeasonMessage::AddCollection,
                    TvSeasonMessage::Selected,
                    TvSeasonMessage::Hovered,
                    TvSeasonMessage::Play,
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
                .on_scroll(TvSeasonMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn top(&self) -> Element<'_, TvSeasonMessage> {
        let img_height = CARD_HEIGHT * 0.85;
        let img: Element<'_, TvSeasonMessage> = {
            let ratio = 2.0 / 3.0;
            match &self.poster {
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
            let separator = || Element::from(text("•").line_height(0.9).size(H4));

            let title = text(self.season.name()).size(H2);
            let duration = duration(&self.season);
            let rating = ratings(&self.season);
            let release = text(self.season.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let synapsis = container(text(self.season.synapsis()))
                .max_width(750)
                .height(Length::Fill);

            let actions = row!(
                button(
                    row!(icon(PLAY).size(P), text("Resume").size(H7))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(TvSeasonMessage::Resume)
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(P),
                        text("Add to Collection").size(H7)
                    )
                    .spacing(10.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(TvSeasonMessage::AddCollectionSelf)
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
            )
            .align_y(Vertical::Center)
            .spacing(16.0);

            let details = column!(details, rating).spacing(8.0);

            column!(
                title,
                details,
                space::vertical().height(3),
                synapsis,
                actions
            )
            .height(img_height)
            .spacing(10.0)
        };

        let backdrop: Element<'_, TvSeasonMessage> = {
            let height = img_height + 68.5;

            match &self.backdrop {
                Some(handle) => image(handle)
                    .height(height)
                    .width(Length::Fill)
                    .content_fit(ContentFit::Cover)
                    .into(),
                None => container(empty())
                    .height(height)
                    .width(Length::Fill)
                    .style(container::dark)
                    .into(),
            }
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let item = "Episodes";
        let tabs = Tab::ALL.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .padding([3, 6])
                        .on_press(TvSeasonMessage::Tab(tab))
                        .style(|theme, status| {
                            let default = button::text(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new().width(68).height(4)).style(if is_selected {
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

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, rule::horizontal(2.0)).spacing(4.0);

        let content = container(column!(content, tabs).spacing(24))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([4, 6])
            .style(|theme| {
                let default = container::dark(theme);

                container::Style {
                    background: default
                        .background
                        .map(|background| background.scale_alpha(0.85)),
                    ..default
                }
            });

        let content = stack![backdrop, content];

        content.into()
    }

    pub fn view(&self) -> Element<'_, TvSeasonMessage> {
        match &self.selected {
            Some(episode) => {
                let thumbnail = self
                    .thumbnails
                    .get(&episode.id)
                    .expect("Episode Preview missing");

                episode.view(
                    thumbnail,
                    TvSeasonMessage::Play,
                    TvSeasonMessage::PreviewTab,
                    TvSeasonMessage::AddCollection,
                )
            }
            None => {
                let content = {
                    let width = 750.0;

                    match self.tab {
                        Tab::Items => match self.layout {
                            Layout::Grid => self.grid(),
                            Layout::List => self.list(),
                        },
                        Tab::Data => data_tab(&self.season, width),
                        Tab::Comments => {
                            // todo
                            let comments = ["Some comment here: "; 7]
                                .into_iter()
                                .enumerate()
                                .map(|(i, comment)| Element::from(text(format!("{comment}{i}"))));

                            let comments =
                                scrollable(column(comments).spacing(4.0).width(Length::Fill))
                                    .spacing(4.0);

                            column!(comments).spacing(8.0).width(width).into()
                        }
                        Tab::Collections => {
                            // todo
                            let collections = ["Some Collection here: "; 7]
                                .into_iter()
                                .enumerate()
                                .map(|(i, collection)| {
                                    Element::from(text(format!("{collection}{i}")))
                                });

                            let collections =
                                scrollable(column(collections).spacing(4.0).width(Length::Fill))
                                    .spacing(4.0);

                            column!(collections).spacing(8.0).width(width).into()
                        }
                    }
                };

                let content = column!(self.top(), content).spacing(20.0).padding(10);

                content.into()
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.focused
            .as_ref()
            .and_then(|id| self.thumbnails.get(id))
            .map(|thumbnail| thumbnail.is_animating(self.now))
            .unwrap_or_default()
    }

    pub fn subscription(&self) -> Subscription<TvSeasonMessage> {
        if self.is_animating() {
            window::frames().map(|_| TvSeasonMessage::Animate)
        } else {
            Subscription::none()
        }
    }
}

#[derive(Debug, Clone)]
pub enum SeriesMessage {
    Animate,
    Hovered(SeasonId, bool),
    Thumbnails(Vec<Thumbnail<Season>>),
    AddCollection(SeasonId),
    AddCollectionSelf,
    Selected(SeasonId),
    Resume,
    SeasonMessage(TvSeasonMessage),
    ResumeSeason(SeasonId),
    Tab(Tab),
    Scroll(scrollable::Viewport),
    Random,
}

#[derive(Debug, Clone)]
pub struct Series {
    show: Show,
    poster: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    now: Instant,
    layout: Layout,
    thumbnails: HashMap<SeasonId, Thumbnail<Season>>,
    focused: Option<SeasonId>,
    sort: Sort,
    filters: Filter,
    tab: Tab,
    selected: Option<TvSeason>,
    selected_prev: Option<TvSeason>,
    scroll: Scroll,
}

impl Series {
    pub fn boot(
        show: Show,
        sort: Sort,
        filters: Filter,
        layout: Layout,
    ) -> (Self, Task<SeriesMessage>) {
        let task = Task::perform(
            async {
                let alt = (0..6).map(|_| Season::testing2());
                (6..12)
                    .map(|_| Season::testing())
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            SeriesMessage::Thumbnails,
        );
        let (new, id) = Self::new(show, sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());
        let tasks = task.chain(scroll);

        (new, tasks)
    }

    fn new(show: Show, sort: Sort, filters: Filter, layout: Layout) -> (Self, widget::Id) {
        let poster = show.poster().and_then(round_image);
        let backdrop = show.backdrop().map(image::Handle::from_path);
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                poster,
                backdrop,
                show,
                layout,
                sort,
                filters,
                focused: None,
                thumbnails: HashMap::default(),
                tab: Tab::Items,
                selected: None,
                selected_prev: None,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: SeriesMessage, now: Instant) -> Task<SeriesMessage> {
        self.now = now;
        match message {
            SeriesMessage::Animate => Task::none(),
            SeriesMessage::Hovered(id, is_hovered) => {
                let Some(thumbnail) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };

                thumbnail.zoom.go_mut(is_hovered, self.now);
                self.focused = Some(id);
                Task::none()
            }
            SeriesMessage::Thumbnails(thumbnails) => {
                for thumbnail in thumbnails {
                    self.thumbnails.insert(thumbnail.id(), thumbnail);
                }

                Task::none()
            }
            SeriesMessage::AddCollectionSelf => {
                println!("Add show to collection pressed");
                Task::none()
            }
            SeriesMessage::AddCollection(id) => {
                println!("Add {id:?} to collection pressed");
                Task::none()
            }
            SeriesMessage::Selected(id) => self.preview(id),
            SeriesMessage::Resume => {
                println!("Resume series playback");
                Task::none()
            }
            SeriesMessage::Tab(tab) => {
                self.tab = tab;
                Task::none()
            }
            SeriesMessage::ResumeSeason(season) => {
                println!("Resume season {season:?} playback");
                Task::none()
            }
            SeriesMessage::SeasonMessage(message) => {
                let Some(season) = self.selected.as_mut() else {
                    return Task::none();
                };

                season
                    .update(message, now)
                    .map(SeriesMessage::SeasonMessage)
            }
            SeriesMessage::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                Task::none()
            }
            SeriesMessage::Random => {
                self.selected.as_mut().map(|season| season.rand());
                Task::none()
            }
        }
    }

    pub fn preview(&mut self, id: SeasonId) -> Task<SeriesMessage> {
        let Some(season) = self.thumbnails.get_mut(&id) else {
            todo!("Season not found");
        };

        season.zoom.go_mut(false, self.now);

        let (season, tasks) =
            TvSeason::boot(season.media.clone(), self.sort, self.filters, self.layout);

        self.selected = Some(season);
        self.selected_prev = None;
        self.focused = None;

        tasks.map(SeriesMessage::SeasonMessage)
    }

    pub fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        self.focused = None;

        if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
            thumbnail.zoom.go_mut(false, self.now);
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.layout = layout;
        self.filters = filters;

        if let Some(season) = self.selected.as_mut() {
            season.page_update(update, now);
        }
    }

    pub fn name(&self) -> String {
        match self.selected.as_ref().map(|season| season.name()) {
            Some(selected) => format!("{}: {selected}", self.show.name()),
            None => self.show.name().to_owned(),
        }
    }

    pub fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    pub fn can_forward(&self) -> bool {
        self.selected
            .as_ref()
            .map(|selected| selected.can_forward())
            .unwrap_or_default()
            || self.selected_prev.is_some()
    }

    pub fn show_tools(&self) -> bool {
        let Some(season) = &self.selected else {
            return true;
        };

        season.show_tools()
    }

    pub fn rand(&mut self) -> Task<SeriesMessage> {
        match self.selected.as_mut() {
            Some(season) => season.rand().map(SeriesMessage::SeasonMessage),
            None => {
                use rand::seq::SliceRandom;

                let mut rng = rand::thread_rng();
                let temp = self.thumbnails.keys().collect::<Vec<_>>();

                let Some(rand) = temp.choose(&mut rng).copied().copied() else {
                    return Task::none();
                };

                self.preview(rand).chain(Task::done(SeriesMessage::Random))
            }
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self.selected.as_mut() {
            Some(selected) => selected.update_scroll(),
            None => operation::scroll_to(self.scroll.id.clone(), self.scroll.offset),
        }
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let mut season = self.selected.take()?;

        if season.can_back() {
            let task = season.back();
            self.selected = Some(season);
            task
        } else {
            self.selected_prev = Some(season);
            Some(self.update_scroll())
        }
    }

    pub fn forward(&mut self) -> Option<Task<()>> {
        self.unfocus();
        match self.selected.as_mut() {
            Some(season) if season.can_forward() => season.forward(),
            Some(_) => None,
            None => {
                let mut prev = self.selected_prev.take()?;

                let task = prev.update_scroll();
                self.selected = Some(prev);
                Some(task)
            }
        }
    }

    fn list(&self) -> Element<'_, SeriesMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.list(
                    self.now,
                    SeriesMessage::AddCollection,
                    SeriesMessage::Selected,
                    SeriesMessage::Hovered,
                    SeriesMessage::ResumeSeason,
                    |season| {
                        let episodes = season.episodes;
                        let episodes = format!(
                            "{} episodes{}",
                            episodes,
                            if episodes > 1 { "s" } else { "" }
                        );
                        text(episodes).size(H7).into()
                    },
                )
            });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(SeriesMessage::Scroll),
        );

        content.into()
    }

    fn grid(&self) -> Element<'_, SeriesMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.card(
                    self.now,
                    SeriesMessage::AddCollection,
                    SeriesMessage::Selected,
                    SeriesMessage::Hovered,
                    SeriesMessage::ResumeSeason,
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
                .on_scroll(SeriesMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn top(&self) -> Element<'_, SeriesMessage> {
        let img_height = CARD_HEIGHT * 0.85;
        let img: Element<'_, SeriesMessage> = {
            let ratio = 2.0 / 3.0;
            match &self.poster {
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
            let separator = || Element::from(text("•").line_height(0.9).size(H4));

            let title = text(self.show.name()).size(H2);
            let duration = duration(&self.show);
            let rating = ratings(&self.show);
            let release = text(self.show.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let tags = {
                let mut tags = vec![];
                let tag_len = self.show.tags.len();

                for (i, tag) in self.show.tags.iter().enumerate() {
                    tags.push(Element::from(text(tag).size(H7)));

                    if i < tag_len - 1 {
                        tags.push(separator())
                    }
                }

                row(tags).spacing(6).align_y(Vertical::Center)
            };

            let synapsis = container(text(self.show.synapsis()))
                .max_width(750)
                .height(Length::Fill);

            let actions = row!(
                button(
                    row!(icon(PLAY).size(P), text("Resume").size(H7))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(SeriesMessage::Resume)
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(P),
                        text("Add to Collection").size(H7)
                    )
                    .spacing(10.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(SeriesMessage::AddCollectionSelf)
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                })
            )
            .align_y(Vertical::Center)
            .spacing(16.0);

            let details = column!(tags, details, rating).spacing(8.0);

            column!(
                title,
                details,
                space::vertical().height(3),
                synapsis,
                actions
            )
            .height(img_height)
            .spacing(10.0)
        };

        let backdrop: Element<'_, SeriesMessage> = {
            let height = img_height + 68.5;

            match &self.backdrop {
                Some(handle) => image(handle)
                    .height(height)
                    .width(Length::Fill)
                    .content_fit(ContentFit::Cover)
                    .into(),
                None => container(empty())
                    .height(height)
                    .width(Length::Fill)
                    .style(container::dark)
                    .into(),
            }
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let item = "Seasons";
        let tabs = Tab::ALL.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .padding([3, 6])
                        .on_press(SeriesMessage::Tab(tab))
                        .style(|theme, status| {
                            let default = button::text(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new().width(68).height(4)).style(if is_selected {
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

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, rule::horizontal(2.0)).spacing(4.0);

        let content = container(column!(content, tabs).spacing(24))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([4, 6])
            .style(|theme| {
                let default = container::dark(theme);

                container::Style {
                    background: default
                        .background
                        .map(|background| background.scale_alpha(0.85)),
                    ..default
                }
            });

        let content = stack![backdrop, content];

        content.into()
    }

    pub fn view(&self) -> Element<'_, SeriesMessage> {
        match &self.selected {
            Some(season) => season.view().map(SeriesMessage::SeasonMessage),
            None => {
                let content = {
                    let width = 750.0;

                    match self.tab {
                        Tab::Items => match self.layout {
                            Layout::Grid => self.grid(),
                            Layout::List => self.list(),
                        },
                        Tab::Data => data_tab(&self.show, width),
                        Tab::Comments => {
                            // todo
                            let comments = ["Some comment here: "; 7]
                                .into_iter()
                                .enumerate()
                                .map(|(i, comment)| Element::from(text(format!("{comment}{i}"))));

                            let comments =
                                scrollable(column(comments).spacing(4.0).width(Length::Fill))
                                    .spacing(4.0);

                            column!(comments).spacing(8.0).width(width).into()
                        }
                        Tab::Collections => {
                            // todo
                            let collections = ["Some Collection here: "; 7]
                                .into_iter()
                                .enumerate()
                                .map(|(i, collection)| {
                                    Element::from(text(format!("{collection}{i}")))
                                });

                            let collections =
                                scrollable(column(collections).spacing(4.0).width(Length::Fill))
                                    .spacing(4.0);

                            column!(collections).spacing(8.0).width(width).into()
                        }
                    }
                };

                let content = column!(self.top(), content).spacing(20.0).padding(10);

                content.into()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<SeriesMessage> {
        match &self.selected {
            Some(season) => season.subscription().map(SeriesMessage::SeasonMessage),
            None => {
                if self
                    .focused
                    .as_ref()
                    .and_then(|id| self.thumbnails.get(id))
                    .map(|thumbnail| thumbnail.is_animating(self.now))
                    .unwrap_or_default()
                {
                    window::frames().map(|_| SeriesMessage::Animate)
                } else {
                    Subscription::none()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TvShowsMessage {
    Hovered(ShowId, bool),
    Thumbnails(Vec<Thumbnail<Show>>),
    AddCollection(ShowId),
    Selected(ShowId),
    ResumeShow(ShowId),
    SeriesMessage(SeriesMessage),
    Scroll(scrollable::Viewport),
    Random,
    Animate,
}

#[derive(Debug, Clone)]
pub struct TvShows {
    now: Instant,
    thumbnails: HashMap<ShowId, Thumbnail<Show>>,
    layout: Layout,
    focused: Option<ShowId>,
    sort: Sort,
    filters: Filter,
    selected: Option<Series>,
    selected_prev: Option<Series>,
    scroll: Scroll,
}

impl TvShows {
    pub fn boot(sort: Sort, filters: Filter, layout: Layout) -> (Self, Task<TvShowsMessage>) {
        let thumbnails = Task::perform(
            async {
                let alt = (0..6).map(|_| Show::testing());
                (6..12)
                    .map(|_| Show::testing())
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            TvShowsMessage::Thumbnails,
        );

        let tasks = Task::batch([thumbnails]);
        let (new, id) = Self::new(sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, tasks.chain(scroll))
    }

    pub fn dummies(
        sort: Sort,
        filters: Filter,
        layout: Layout,
        shows: Vec<Show>,
    ) -> (Self, Task<TvShowsMessage>) {
        let task = Task::perform(async move { shows }, |shows| {
            TvShowsMessage::Thumbnails(shows.into_iter().map(Thumbnail::new).collect())
        });

        let (new, id) = Self::new(sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, task.chain(scroll))
    }

    fn new(sort: Sort, filters: Filter, layout: Layout) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                sort,
                filters,
                layout,
                thumbnails: HashMap::default(),
                focused: None,
                selected: None,
                selected_prev: None,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: TvShowsMessage, now: Instant) -> Task<TvShowsMessage> {
        self.now = now;

        match message {
            TvShowsMessage::Animate => Task::none(),
            TvShowsMessage::Hovered(id, is_hovered) => {
                let Some(thumbnail) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };

                thumbnail.zoom.go_mut(is_hovered, self.now);
                self.focused = Some(id);
                Task::none()
            }
            TvShowsMessage::Thumbnails(thumbnails) => {
                for thumbnail in thumbnails {
                    self.thumbnails.insert(thumbnail.id(), thumbnail);
                }

                Task::none()
            }
            TvShowsMessage::AddCollection(id) => {
                println!("Add {id:?} to collection pressed");
                Task::none()
            }
            TvShowsMessage::Selected(id) => match self.preview(id) {
                Ok(task) => task,
                Err(task) => task,
            },
            TvShowsMessage::ResumeShow(show) => {
                println!("Resume show {show:?} playback");
                Task::none()
            }
            TvShowsMessage::SeriesMessage(message) => {
                let Some(series) = self.selected.as_mut() else {
                    return Task::none();
                };

                series
                    .update(message, now)
                    .map(TvShowsMessage::SeriesMessage)
            }
            TvShowsMessage::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                Task::none()
            }
            TvShowsMessage::Random => self
                .selected
                .as_mut()
                .map(|series| series.rand().map(TvShowsMessage::SeriesMessage))
                .unwrap_or_default(),
        }
    }

    pub fn preview(&mut self, id: ShowId) -> Result<Task<TvShowsMessage>, Task<TvShowsMessage>> {
        let Some(show) = self.thumbnails.get_mut(&id) else {
            // Err variant
            todo!("Should fetch missing show")
        };

        let (show, tasks) = Series::boot(show.media.clone(), self.sort, self.filters, self.layout);

        self.selected = Some(show);
        self.selected_prev = None;
        self.focused = None;

        Ok(tasks.map(TvShowsMessage::SeriesMessage))
    }

    fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
            thumbnail.zoom.go_mut(false, self.now);
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.layout = layout;
        self.filters = filters;

        if let Some(show) = self.selected.as_mut() {
            show.page_update(update, now);
        }
    }

    pub fn name(&self) -> String {
        self.selected
            .as_ref()
            .map(|show| show.name())
            .unwrap_or(String::from("TV Shows"))
    }

    pub fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    pub fn can_forward(&self) -> bool {
        self.selected
            .as_ref()
            .map(|selected| selected.can_forward())
            .unwrap_or_default()
            || self.selected_prev.is_some()
    }

    pub fn show_tools(&self) -> bool {
        let Some(show) = &self.selected else {
            return true;
        };

        show.show_tools()
    }

    pub fn rand(&mut self) -> Task<TvShowsMessage> {
        match self.selected.as_mut() {
            Some(show) => show.rand().map(TvShowsMessage::SeriesMessage),
            None => {
                todo!()
                // use rand::seq::SliceRandom;
                //
                // let mut rng = rand::thread_rng();
                // let temp = self.thumbnails.keys().collect::<Vec<_>>();
                //
                // let Some(rand) = temp.choose(&mut rng).copied().copied() else {
                //     return Task::none();
                // };
                //
                // self.preview(rand).chain(Task::done(TvShowsMessage::Random))
            }
        }
    }

    pub fn refresh(&mut self) -> Task<TvShowsMessage> {
        todo!()
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self.selected.as_mut() {
            Some(series) => series.update_scroll(),
            None => operation::scroll_to(self.scroll.id.clone(), self.scroll.offset),
        }
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let mut show = self.selected.take()?;

        if show.can_back() {
            let task = show.back();
            self.selected = Some(show);
            task
        } else {
            self.selected_prev = Some(show);
            Some(self.update_scroll())
        }
    }

    pub fn forward(&mut self) -> Option<Task<()>> {
        self.unfocus();
        match self.selected.as_mut() {
            // Some(show) if show.can_forward() => show.forward(),
            Some(show) if show.can_forward() => show.forward(),
            // Some(_) => false,
            Some(_) => None,
            None => {
                let mut prev = self.selected_prev.take()?;

                let task = prev.update_scroll();
                self.selected = Some(prev);
                Some(task)
            }
        }
    }

    fn list(&self) -> Element<'_, TvShowsMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.list(
                    self.now,
                    TvShowsMessage::AddCollection,
                    TvShowsMessage::Selected,
                    TvShowsMessage::Hovered,
                    TvShowsMessage::ResumeShow,
                    unique,
                )
            });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(TvShowsMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    fn grid(&self) -> Element<'_, TvShowsMessage> {
        let content =
            filter_sort(self.thumbnails.values(), &self.filters, &self.sort).map(|thumbnail| {
                thumbnail.card(
                    self.now,
                    TvShowsMessage::AddCollection,
                    TvShowsMessage::Selected,
                    TvShowsMessage::Hovered,
                    TvShowsMessage::ResumeShow,
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
                .on_scroll(TvShowsMessage::Scroll),
        )
        .padding(10);

        content.into()
    }

    pub fn view(&self) -> Element<'_, TvShowsMessage> {
        match &self.selected {
            Some(series) => series.view().map(TvShowsMessage::SeriesMessage),
            None => match self.layout {
                Layout::Grid => self.grid(),
                Layout::List => self.list(),
            },
        }
    }

    pub fn subscription(&self) -> Subscription<TvShowsMessage> {
        match &self.selected {
            Some(show) => show.subscription().map(TvShowsMessage::SeriesMessage),
            None => {
                if self
                    .focused
                    .as_ref()
                    .and_then(|id| self.thumbnails.get(id))
                    .map(|thumbnail| thumbnail.is_animating(self.now))
                    .unwrap_or_default()
                {
                    window::frames().map(|_| TvShowsMessage::Animate)
                } else {
                    Subscription::none()
                }
            }
        }
    }
}

pub fn unique<'a, Message: 'a>(show: &Show) -> Element<'a, Message> {
    let seasons = show.seasons;

    let seasons = format!("{} season{}", seasons, if seasons > 1 { "s" } else { "" });

    text(seasons).size(H7).into()
}
