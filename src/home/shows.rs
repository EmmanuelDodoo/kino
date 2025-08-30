use super::{PageUpdate, shared::*};
use crate::media::{Media, show::*};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort, SortKind, empty};
use iced::widget::Space;
use iced::{
    Color, ContentFit, Element, Length, Shadow, Subscription, Task,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    mouse,
    time::Instant,
    widget::{
        bottom_center, button, center_x, column, container, grid, horizontal_rule,
        horizontal_space, image, mouse_area, row, scrollable, stack, text, vertical_space,
    },
    window,
};
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Clone)]
pub enum TvEpisodeMessage {
    AddCollection,
    Tab(Tab),
    Resume,
}

#[derive(Debug, Clone)]
struct TvEpisode {
    episode: Episode,
    poster: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    tab: Tab,
}

impl TvEpisode {
    fn new(episode: Episode) -> Self {
        let poster = episode.poster.as_ref().map(image::Handle::from_path);
        let backdrop = episode.backdrop.as_ref().map(image::Handle::from_path);

        Self {
            episode,
            poster,
            backdrop,
            tab: Tab::Data,
        }
    }

    fn update(&mut self, message: TvEpisodeMessage) -> Task<TvEpisodeMessage> {
        match message {
            TvEpisodeMessage::Resume => {
                println!("Resume episode {:?} playback", self.episode.id);
                Task::none()
            }
            TvEpisodeMessage::Tab(tab) => {
                self.tab = tab;
                Task::none()
            }
            TvEpisodeMessage::AddCollection => {
                println!("Add episode {:?} to collection", self.episode.id);
                Task::none()
            }
        }
    }

    fn name(&self) -> &str {
        self.episode.name()
    }

    fn top(&self) -> Element<'_, TvEpisodeMessage> {
        let img_height = CARD_HEIGHT * 0.85;
        let img: Element<'_, TvEpisodeMessage> = {
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

            let title = text(self.episode.name()).size(H2);
            let duration = duration(&self.episode);
            let rating = ratings(&self.episode);
            let release = text(self.episode.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let synapsis = container(text(&self.episode.synapsis))
                .max_width(750)
                .height(Length::Fill);

            let actions = row!(
                button(
                    row!(icon(PLAY).size(P), text("Resume").size(H7))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(TvEpisodeMessage::Resume)
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
                .on_press(TvEpisodeMessage::AddCollection)
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
                vertical_space().height(3),
                synapsis,
                actions
            )
            .height(img_height)
            .spacing(10.0)
        };

        let backdrop: Element<'_, TvEpisodeMessage> = {
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

        let item = "";
        let tabs = Tab::EPISODE.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .padding([3, 6])
                        .on_press(TvEpisodeMessage::Tab(tab))
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

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, horizontal_rule(2.0)).spacing(4.0);

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

    fn view(&self) -> Element<'_, TvEpisodeMessage> {
        let content: Element<'_, TvEpisodeMessage> = {
            let width = 750.0;

            match self.tab {
                Tab::Data => data_tab(&self.episode, width),
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
                Tab::Items => unreachable!(),
            }
        };

        let content = column!(self.top(), content).spacing(20.0).padding(10);

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
    EpisodeMessage(TvEpisodeMessage),
    Tab(Tab),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
struct TvSeason {
    season: Season,
    poster: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    now: Instant,
    grid: bool,
    thumbnails: HashMap<EpisodeId, Thumbnail<Episode>>,
    focused: Option<EpisodeId>,
    sort: Sort,
    filters: Filter,
    tab: Tab,
    selected: Option<TvEpisode>,
    selected_prev: Option<TvEpisode>,
    scroll: Scroll,
}

impl TvSeason {
    fn boot(
        season: Season,
        sort: Sort,
        filters: Filter,
        grid: bool,
    ) -> (Self, scrollable::Id, Task<TvSeasonMessage>) {
        let thumbnails = Task::perform(
            async {
                let alt = (0..6).map(Episode::testing2);
                (6..12)
                    .map(Episode::testing)
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            TvSeasonMessage::Thumbnails,
        );
        let tasks = Task::batch([thumbnails]);
        let (new, id) = Self::new(season, sort, filters, grid);

        (new, id, tasks)
    }

    fn new(season: Season, sort: Sort, filters: Filter, grid: bool) -> (Self, scrollable::Id) {
        let poster = season.poster.as_ref().map(image::Handle::from_path);
        let backdrop = season.backdrop.as_ref().map(image::Handle::from_path);
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                poster,
                backdrop,
                season,
                grid,
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

    fn update(&mut self, message: TvSeasonMessage, now: Instant) -> Task<TvSeasonMessage> {
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
            TvSeasonMessage::Selected(id) => {
                let Some(episode) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };
                episode.zoom.go_mut(false, now);
                self.focused = None;

                self.selected = Some(TvEpisode::new(episode.media.clone()));
                self.selected_prev = None;
                Task::none()
            }
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
            TvSeasonMessage::EpisodeMessage(message) => {
                let Some(episode) = self.selected.as_mut() else {
                    return Task::none();
                };

                episode.update(message).map(TvSeasonMessage::EpisodeMessage)
            }
            TvSeasonMessage::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                Task::none()
            }
        }
    }

    fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
            thumbnail.zoom.go_mut(false, self.now);
        }
    }

    fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        let PageUpdate {
            layout,
            sort,
            filters,
        } = update;

        self.sort = sort;
        self.grid = matches!(layout, Layout::Grid);
        self.filters = filters;
    }

    fn name(&self) -> String {
        match self.selected.as_ref().map(|episode| episode.name()) {
            Some(selected) => format!("{} - {selected}", self.season.name()),
            None => self.season.name().to_owned(),
        }
    }

    fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    fn can_forward(&self) -> bool {
        self.selected_prev.is_some()
    }

    fn show_tools(&self) -> bool {
        self.selected.is_none()
    }

    fn rand(&mut self) {
        todo!()
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        scrollable::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let Some(selected) = self.selected.take() else {
            return None;
        };

        self.selected_prev = Some(selected);

        Some(self.update_scroll())
    }

    fn forward(&mut self) -> bool {
        self.unfocus();
        let Some(prev) = self.selected_prev.take() else {
            return false;
        };

        self.selected = Some(prev);

        true
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
        );

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

            let title = text(&self.season.name).size(H2);
            let duration = duration(&self.season);
            let rating = ratings(&self.season);
            let release = text(self.season.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let synapsis = container(text(&self.season.synapsis))
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
                vertical_space().height(3),
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

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, horizontal_rule(2.0)).spacing(4.0);

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

    fn view(&self) -> Element<'_, TvSeasonMessage> {
        match &self.selected {
            Some(episode) => episode.view().map(TvSeasonMessage::EpisodeMessage),
            None => {
                let content = {
                    let width = 750.0;

                    match self.tab {
                        Tab::Items => {
                            if self.grid {
                                self.grid()
                            } else {
                                self.list()
                            }
                        }
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

    fn subscription(&self) -> Subscription<TvSeasonMessage> {
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
}

#[derive(Debug, Clone)]
struct Series {
    show: Show,
    poster: Option<image::Handle>,
    backdrop: Option<image::Handle>,
    now: Instant,
    grid: bool,
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
    fn boot(
        show: Show,
        sort: Sort,
        filters: Filter,
        grid: bool,
    ) -> (Self, scrollable::Id, Task<SeriesMessage>) {
        let thumbnails = Task::perform(
            async {
                let alt = (0..6).map(Season::testing2);
                (6..12)
                    .map(Season::testing)
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            SeriesMessage::Thumbnails,
        );
        let tasks = Task::batch([thumbnails]);
        let (new, id) = Self::new(show, sort, filters, grid);

        (new, id, tasks)
    }

    fn new(show: Show, sort: Sort, filters: Filter, grid: bool) -> (Self, scrollable::Id) {
        let poster = show.poster.as_ref().map(image::Handle::from_path);
        let backdrop = show.backdrop.as_ref().map(image::Handle::from_path);
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                poster,
                backdrop,
                show,
                grid,
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

    fn update(&mut self, message: SeriesMessage, now: Instant) -> Task<SeriesMessage> {
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
            SeriesMessage::Selected(id) => {
                let Some(season) = self.thumbnails.get_mut(&id) else {
                    return Task::none();
                };

                season.zoom.go_mut(false, now);

                let (season, id, tasks) = TvSeason::boot(
                    season.media.clone(),
                    self.sort.clone(),
                    self.filters,
                    self.grid,
                );

                self.selected = Some(season);
                self.selected_prev = None;
                self.focused = None;

                let scroll = scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());
                Task::batch([tasks.map(SeriesMessage::SeasonMessage), scroll])
            }
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
        }
    }

    fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        self.focused = None;

        if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
            thumbnail.zoom.go_mut(false, self.now);
        }
    }

    fn page_update(&mut self, update: PageUpdate, now: Instant) {
        self.now = now;

        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.grid = matches!(layout, Layout::Grid);
        self.filters = filters;

        if let Some(season) = self.selected.as_mut() {
            season.page_update(update, now);
        }
    }

    fn name(&self) -> String {
        match self.selected.as_ref().map(|season| season.name()) {
            Some(selected) => format!("{}: {selected}", self.show.name()),
            None => self.show.name.to_owned(),
        }
    }

    fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    fn can_forward(&self) -> bool {
        self.selected
            .as_ref()
            .map(|selected| selected.can_forward())
            .unwrap_or_default()
            || self.selected_prev.is_some()
    }

    fn show_tools(&self) -> bool {
        let Some(season) = &self.selected else {
            return true;
        };

        season.show_tools()
    }

    fn rand(&mut self) {
        match self.selected.as_mut() {
            Some(season) => season.rand(),
            None => todo!(),
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self.selected.as_mut() {
            Some(selected) => selected.update_scroll(),
            None => scrollable::scroll_to(self.scroll.id.clone(), self.scroll.offset),
        }
    }

    fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let Some(mut season) = self.selected.take() else {
            return None;
        };

        if season.can_back() {
            let task = season.back();
            self.selected = Some(season);
            task
        } else {
            self.selected_prev = Some(season);
            Some(self.update_scroll())
        }
    }

    fn forward(&mut self) -> Option<Task<()>> {
        self.unfocus();
        match self.selected.as_mut() {
            Some(season) if season.can_forward() => {
                season.forward();
                Some(season.update_scroll())
            }
            Some(_) => None,
            None => {
                let Some(mut prev) = self.selected_prev.take() else {
                    return None;
                };

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
        );

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

            let title = text(&self.show.name).size(H2);
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

            let synapsis = container(text(&self.show.synapsis))
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
                vertical_space().height(3),
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

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, horizontal_rule(2.0)).spacing(4.0);

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

    fn view(&self) -> Element<'_, SeriesMessage> {
        match &self.selected {
            Some(season) => season.view().map(SeriesMessage::SeasonMessage),
            None => {
                let content = {
                    let width = 750.0;

                    match self.tab {
                        Tab::Items => {
                            if self.grid {
                                self.grid()
                            } else {
                                self.list()
                            }
                        }
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

    fn subscription(&self) -> Subscription<SeriesMessage> {
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
    Animate,
}

#[derive(Debug, Clone)]
pub struct TvShows {
    now: Instant,
    thumbnails: HashMap<ShowId, Thumbnail<Show>>,
    grid: bool,
    focused: Option<ShowId>,
    sort: Sort,
    filters: Filter,
    selected: Option<Series>,
    selected_prev: Option<Series>,
    scroll: Scroll,
}

impl TvShows {
    pub fn boot(
        sort: Sort,
        filters: Filter,
        grid: bool,
    ) -> (Self, scrollable::Id, Task<TvShowsMessage>) {
        let thumbnails = Task::perform(
            async {
                let alt = (0..6).map(Show::testing2);
                (6..12)
                    .map(Show::testing)
                    .chain(alt)
                    .map(Thumbnail::new)
                    .collect::<Vec<_>>()
            },
            TvShowsMessage::Thumbnails,
        );

        let tasks = Task::batch([thumbnails]);
        let (new, id) = Self::new(sort, filters, grid);

        (new, id, tasks)
    }

    fn new(sort: Sort, filters: Filter, grid: bool) -> (Self, scrollable::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                now: Instant::now(),
                sort,
                filters,
                grid,
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
            TvShowsMessage::Selected(id) => self.preview(id),
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
        }
    }

    pub fn preview(&mut self, id: ShowId) -> Task<TvShowsMessage> {
        let Some(show) = self.thumbnails.get_mut(&id) else {
            return Task::none();
        };

        let (show, id, tasks) = Series::boot(
            show.media.clone(),
            self.sort.clone(),
            self.filters,
            self.grid,
        );

        self.selected = Some(show);
        self.selected_prev = None;
        self.focused = None;

        let scroll = scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());
        Task::batch([tasks.map(TvShowsMessage::SeriesMessage), scroll])
    }

    pub fn contains(&self, id: &ShowId) -> bool {
        self.thumbnails.contains_key(id)
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
        self.grid = matches!(layout, Layout::Grid);
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

    pub fn rand(&mut self) {
        match self.selected.as_mut() {
            Some(show) => show.rand(),
            None => todo!(),
        }
    }

    pub fn refresh(&mut self) -> Task<TvShowsMessage> {
        todo!()
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self.selected.as_mut() {
            Some(series) => series.update_scroll(),
            None => scrollable::scroll_to(self.scroll.id.clone(), self.scroll.offset),
        }
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let Some(mut show) = self.selected.take() else {
            return None;
        };

        if show.can_back() {
            let task = show.back();
            self.selected = Some(show);
            return task;
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
                let Some(mut prev) = self.selected_prev.take() else {
                    return None;
                };

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
            None => {
                if self.grid {
                    self.grid()
                } else {
                    self.list()
                }
            }
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
