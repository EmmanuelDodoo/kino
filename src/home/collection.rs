use super::episode::EpisodeItem;
use super::movie::MovieItem;
use super::season::SeasonItem;
use super::series::ShowItem;
use super::{
    CollectionThumbnail, HomeMessage, PageKind, ViewMessage, movies, shared::*, shows, view_unicode,
};
use crate::Element;
use crate::theme::{self, Theme};
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{self, Layout, Scroll, empty};
use iced::{
    Border, Length, Padding, Task,
    alignment::Vertical,
    time::Instant,
    widget::{
        Button, Column, button, column, container, grid,
        operation::{self},
        row, rule, scrollable, space, text,
    },
};
use registry::models::{
    CollectionId, CollectionView, Episode, Movie, Season, Show, collection::ItemId,
    collection::Items,
};
use std::iter::Peekable;
use widgets::{expandable, menu};

#[derive(Debug, Clone)]
pub enum Message {
    Scroll(scrollable::Viewport),
    PlayItem(ItemId),
    HoveredItem(bool, ItemId),
    ShownItem(bool, ItemId),
    DetailsItem(ItemId),
    Add(ItemId),
    Play(Items),
    Delete(Items),
    Remove(String),
    OpenConfig,
    AddNewItem,
    Triggers,
    ToggleMovies(bool),
    ToggleShows(bool),
    ToggleSeasons(bool),
    ToggleEpisodes(bool),
    None,
}

#[derive(Debug, Clone)]
pub struct CollectionMessage {
    pub id: CollectionId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct CollectionPage {
    id: CollectionId,
    scroll: Scroll,
    movies_shown: bool,
    shows_shown: bool,
    seasons_shown: bool,
    episodes_shown: bool,
}

impl CollectionPage {
    pub fn boot(collection: CollectionId) -> (Self, Task<CollectionMessage>) {
        let id = collection;

        let new = Self::new(collection);

        let scroll = operation::scroll_to(
            new.scroll.id.clone(),
            scrollable::AbsoluteOffset::<f32>::default(),
        )
        .map(move |message| CollectionMessage { id, message });

        (new, scroll)
    }

    fn new(collection: CollectionId) -> Self {
        Self {
            id: collection,
            scroll: Scroll::new(),
            movies_shown: true,
            shows_shown: true,
            seasons_shown: true,
            episodes_shown: true,
        }
    }

    pub fn update(&mut self, message: CollectionMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::None => None,
            Message::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                None
            }
            Message::PlayItem(item) => {
                let msg = HomeMessage::Play(item);
                Some(msg)
            }
            Message::HoveredItem(hovered, item) => {
                let msg = HomeMessage::Hovered(item, hovered);
                Some(msg)
            }
            Message::ShownItem(shown, item) => Some(HomeMessage::Shown(item, shown)),
            Message::DetailsItem(item) => {
                let kind = match item {
                    ItemId::Movie(id) => PageKind::Movie(id),
                    ItemId::Show(id) => PageKind::Show(id),
                    ItemId::Season(id) => PageKind::Season(id),
                    ItemId::Episode(id) => PageKind::Episode(id),
                };
                let msg = HomeMessage::Goto(kind);
                Some(msg)
            }
            Message::Add(item) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(item));
                Some(msg)
            }
            Message::Play(items) => {
                let msg = HomeMessage::PlayCollection { id: self.id, items };

                Some(msg)
            }
            Message::OpenConfig => {
                let msg = HomeMessage::OpenView(ViewMessage::CollectionConfig);

                Some(msg)
            }
            Message::AddNewItem => {
                let msg = HomeMessage::OpenView(ViewMessage::AddToCollection(self.id));

                Some(msg)
            }
            Message::Remove(name) => {
                let msg =
                    HomeMessage::OpenView(ViewMessage::RemoveCollection { id: self.id, name });

                Some(msg)
            }
            Message::Triggers => {
                let msg = HomeMessage::OpenView(ViewMessage::CollectionTriggers);

                Some(msg)
            }
            Message::Delete(items) => {
                let msg = HomeMessage::RemoveCollectionItems(self.id, items);

                Some(msg)
            }
            Message::ToggleMovies(toggle) => {
                self.movies_shown = toggle;

                None
            }
            Message::ToggleShows(toggle) => {
                self.shows_shown = toggle;

                None
            }
            Message::ToggleSeasons(toggle) => {
                self.seasons_shown = toggle;

                None
            }
            Message::ToggleEpisodes(toggle) => {
                self.episodes_shown = toggle;

                None
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn view<'a>(
        &'a self,
        now: Instant,
        layout: Layout,
        collection: &'a CollectionThumbnail,
        movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
        seasons: Peekable<impl Iterator<Item = &'a SeasonItem>>,
        episodes: Peekable<impl Iterator<Item = &'a EpisodeItem>>,
    ) -> Element<'a, CollectionMessage> {
        let id = self.id;
        let content = match layout {
            Layout::List => self.list(now, movies, shows, seasons, episodes),
            Layout::Grid => self.grid(now, movies, shows, seasons, episodes),
            Layout::Compact => self.compact(now, movies, shows, seasons, episodes),
        }
        .map(move |message| CollectionMessage { id: id, message });

        let content = column!(self.top(collection, now), content).spacing(10);

        content.into()
    }

    fn top<'a>(
        &self,
        collection: &'a CollectionThumbnail,
        now: Instant,
    ) -> Element<'a, CollectionMessage> {
        let id = self.id;

        let img_height = CollectionThumbnail::HEIGHT;

        let img = collection.collage(now);

        let header = {
            let title = container(h4(&collection.collection.name))
                .clip(true)
                .max_height(56);

            let title = row!(title)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(10.0);

            let title = if matches!(collection.collection.view, CollectionView::Hidden) {
                let view = icon(view_unicode(collection.collection.view)).size(H3);

                title.push(view)
            } else {
                title
            };

            let description = collection
                .collection
                .description
                .as_deref()
                .unwrap_or_default();
            let description = container(regular(description))
                .clip(true)
                .max_width(750)
                .height(Length::Fill);

            let play = {
                let play = btn(id, PLAY, "Play", Message::Play(Items::All));

                let base = container(icon(ELLIPSIS_VER).size(H7))
                    .style(|theme: &Theme| {
                        let pair = theme.schema().danger.strong;
                        let text = pair.text;
                        let color = pair.color;

                        container::Style {
                            background: Some(color.into()),
                            text_color: Some(text),
                            ..Default::default()
                        }
                    })
                    .padding([8, 3]);

                let actions = column!(
                    btn(id, PLAY, "Play movies", Message::Play(Items::Movies)),
                    btn(id, PLAY, "Play shows", Message::Play(Items::Shows)),
                    btn(id, PLAY, "Play seasons", Message::Play(Items::Seasons)),
                    btn(id, PLAY, "Play episodes", Message::Play(Items::Episodes)),
                )
                .spacing(8);

                let overlay = container(actions)
                    .padding([8, 12])
                    .style(theme::container::bordered);

                let hidden = menu(base, overlay)
                    .on_toggle(move |_| CollectionMessage {
                        id,
                        message: Message::None,
                    })
                    .position(menu::Position::Right);

                row!(play, hidden).spacing(2.0).align_y(Vertical::Center)
            };

            let delete = {
                let delete = delete_btn(
                    id,
                    "Delete",
                    Message::Remove(collection.collection.name.clone()),
                );

                let base = container(icon(ELLIPSIS_VER).size(H7))
                    .style(|theme: &Theme| {
                        let pair = theme.schema().danger.strong;
                        let text = pair.text;
                        let color = pair.color;

                        container::Style {
                            background: Some(color.into()),
                            text_color: Some(text),
                            ..Default::default()
                        }
                    })
                    .padding([8, 3]);

                let actions = column!(
                    delete_btn(id, "Remove movies", Message::Delete(Items::Movies)),
                    delete_btn(id, "Remove shows", Message::Delete(Items::Shows)),
                    delete_btn(id, "Remove seasons", Message::Delete(Items::Seasons)),
                    delete_btn(id, "Remove episodes", Message::Delete(Items::Episodes)),
                )
                .spacing(8);

                let overlay = container(actions)
                    .padding([8, 12])
                    .style(theme::container::bordered);

                let hidden = menu(base, overlay)
                    .on_toggle(move |_| CollectionMessage {
                        id,
                        message: Message::None,
                    })
                    .position(menu::Position::Right);

                row!(delete, hidden).spacing(2.0).align_y(Vertical::Center)
            };

            let actions = row!(
                btn(id, ADD, "Add", Message::AddNewItem),
                btn(id, EDIT, "Edit", Message::OpenConfig),
                btn(id, VIDEO_CONFIG, "Rules", Message::Triggers),
                play,
                delete
            )
            .align_y(Vertical::Center)
            .spacing(16.0);

            column!(title, description, actions)
                .height(img_height)
                .spacing(10.0)
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let content = container(content)
            .padding(20)
            .width(Length::Fill)
            .style(theme::container::bordered);

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
        mut seasons: Peekable<impl Iterator<Item = &'a SeasonItem>>,
        mut episodes: Peekable<impl Iterator<Item = &'a EpisodeItem>>,
    ) -> Element<'a, Message> {
        let content = Column::new()
            .spacing(40)
            .padding(Padding::new(10.0).bottom(0));

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let movies = {
                    let content = movies
                        .map(|thumbnail| thumbnail.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };

                self.movie_expandable(movies)
            };

            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                let shows = {
                    let content =
                        shows.map(|thumbnail| thumbnail.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };

                self.shows_expandable(shows)
            };

            content.push(shows)
        };

        let content = if seasons.peek().is_none() {
            content
        } else {
            let seasons = {
                let seasons = {
                    let content = seasons
                        .map(|thumbnail| thumbnail.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };

                self.seasons_expandable(seasons)
            };

            content.push(seasons)
        };

        let content = if episodes.peek().is_none() {
            content
        } else {
            let episodes = {
                let episodes = {
                    let content = episodes
                        .map(|thumbnail| thumbnail.list(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };
                self.episodes_expandable(episodes)
            };
            content.push(episodes)
        };

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(Message::Scroll);

        content.into()
    }

    fn compact<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
        mut seasons: Peekable<impl Iterator<Item = &'a SeasonItem>>,
        mut episodes: Peekable<impl Iterator<Item = &'a EpisodeItem>>,
    ) -> Element<'a, Message> {
        let content = Column::new()
            .spacing(40)
            .padding(Padding::new(10.0).bottom(0));

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let movies = {
                    let content = movies
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };

                self.movie_expandable(movies)
            };

            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                let shows = {
                    let content = shows
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };
                self.shows_expandable(shows)
            };

            content.push(shows)
        };

        let content = if seasons.peek().is_none() {
            content
        } else {
            let seasons = {
                let seasons = {
                    let content = seasons
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };
                self.seasons_expandable(seasons)
            };

            content.push(seasons)
        };

        let content = if episodes.peek().is_none() {
            content
        } else {
            let episodes = {
                let episodes = {
                    let content = episodes
                        .map(|thumbnail| thumbnail.compact(now, add, select, hover, shown, play));

                    column(content).spacing(16.0)
                };
                self.episodes_expandable(episodes)
            };
            content.push(episodes)
        };

        let content = scrollable(content)
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(Message::Scroll);

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a MovieItem>>,
        mut shows: Peekable<impl Iterator<Item = &'a ShowItem>>,
        mut seasons: Peekable<impl Iterator<Item = &'a SeasonItem>>,
        mut episodes: Peekable<impl Iterator<Item = &'a EpisodeItem>>,
    ) -> Element<'a, Message> {
        let content = Column::new()
            .spacing(40.0)
            .padding(Padding::ZERO.left(16).right(16).bottom(16));

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let movies = movies.map(|movie| movie.card(now, add, select, hover, shown, play));

                let movies =
                    grid(movies)
                        .spacing(16)
                        .fluid(MovieItem::WIDTH)
                        .height(if self.movies_shown {
                            grid::aspect_ratio(MovieItem::WIDTH, MovieItem::HEIGHT)
                        } else {
                            grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                        });

                self.movie_expandable(movies)
            };
            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows =
                {
                    let shows = shows.map(|show| show.card(now, add, select, hover, shown, play));

                    let shows = grid(shows).spacing(16).fluid(ShowItem::WIDTH).height(
                        if self.shows_shown {
                            grid::aspect_ratio(ShowItem::WIDTH, ShowItem::HEIGHT)
                        } else {
                            grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                        },
                    );

                    self.shows_expandable(shows)
                };

            content.push(shows)
        };

        let content = if seasons.peek().is_none() {
            content
        } else {
            let seasons = {
                let seasons =
                    seasons.map(|season| season.card(now, add, select, hover, shown, play));

                let seasons = grid(seasons).spacing(16).fluid(SeasonItem::WIDTH).height(
                    if self.seasons_shown {
                        grid::aspect_ratio(SeasonItem::WIDTH, SeasonItem::HEIGHT)
                    } else {
                        grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                    },
                );

                self.seasons_expandable(seasons)
            };

            content.push(seasons)
        };

        let content = if episodes.peek().is_none() {
            content
        } else {
            let episodes = {
                let episodes =
                    episodes.map(|episode| episode.card(now, add, select, hover, shown, play));

                let episodes = grid(episodes).spacing(16).fluid(EpisodeItem::WIDTH).height(
                    if self.episodes_shown {
                        grid::aspect_ratio(EpisodeItem::WIDTH, EpisodeItem::HEIGHT)
                    } else {
                        grid::Sizing::EvenlyDistribute(Length::Fixed(0.0))
                    },
                );

                self.episodes_expandable(episodes)
            };

            content.push(episodes)
        };

        let content = scrollable(content)
            .auto_scroll(true)
            .id(self.scroll.id.clone())
            .height(Length::Fill)
            .on_scroll(Message::Scroll);

        content.into()
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    pub fn movie_expandable<'a>(
        &self,
        movies: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let label = section_label("Movies", self.movies_shown);

        expandable(label, movies)
            .expanded(self.movies_shown)
            .on_expand(Message::ToggleMovies)
            .spacing(10.0)
            .into()
    }

    pub fn shows_expandable<'a>(
        &self,
        shows: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let label = section_label("Shows", self.shows_shown);

        expandable(label, shows)
            .expanded(self.shows_shown)
            .on_expand(Message::ToggleShows)
            .spacing(10.0)
            .into()
    }

    pub fn seasons_expandable<'a>(
        &self,
        seasons: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let label = section_label("Seasons", self.seasons_shown);

        expandable(label, seasons)
            .expanded(self.seasons_shown)
            .on_expand(Message::ToggleSeasons)
            .spacing(10.0)
            .into()
    }

    pub fn episodes_expandable<'a>(
        &self,
        episodes: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let label = section_label("Episodes", self.episodes_shown);

        expandable(label, episodes)
            .expanded(self.episodes_shown)
            .on_expand(Message::ToggleEpisodes)
            .spacing(10.0)
            .into()
    }
}

fn btn<'a>(
    id: CollectionId,
    unicode: char,
    label: &'static str,
    message: Message,
) -> Button<'a, CollectionMessage, Theme> {
    button(
        row!(icon(unicode).size(P), sized_medium(label, H7))
            .spacing(10.0)
            .align_y(Vertical::Center),
    )
    .padding([6, 12])
    .on_press(CollectionMessage { id, message })
    .style(theme::button::danger)
}

fn delete_btn<'a>(
    id: CollectionId,
    label: &'a str,
    message: Message,
) -> Button<'a, CollectionMessage, Theme> {
    utils::delete_btn(label).on_press(CollectionMessage { id, message })
}

fn add(item: impl Into<ItemId>) -> Message {
    Message::Add(item.into())
}

fn select(item: impl Into<ItemId>) -> Message {
    Message::DetailsItem(item.into())
}

fn hover(item: impl Into<ItemId>, hovered: bool) -> Message {
    Message::HoveredItem(hovered, item.into())
}

fn shown(item: impl Into<ItemId>, shown: bool) -> Message {
    Message::ShownItem(shown, item.into())
}

fn play(item: impl Into<ItemId>) -> Message {
    Message::PlayItem(item.into())
}

fn section_label<'a>(label: &'a str, shown: bool) -> Element<'a, Message> {
    let shown = if shown { CHEV_UP } else { CHEV_DOWN };
    let icon = icon(shown).size(H6);
    let label = h6(label);

    let label = row!(label, space::horizontal(), icon).align_y(Vertical::Center);

    column!(label, rule::horizontal(1.0))
        .spacing(4.0)
        .width(Length::Fill)
        .into()
}
