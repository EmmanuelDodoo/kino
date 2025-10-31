use super::{
    CollectionThumbnail, HomeMessage, MoviePage, PageKind, PageUpdate, SeasonPage, ViewMessage,
    movies, shared::*, shows, view_unicode,
};
use crate::models::{
    Collection, CollectionId, CollectionView, Episode, EpisodeId, Media, Movie, MovieId, Season,
    SeasonId, Show, ShowId, collection::ItemId,
};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort, empty};
use crate::widgets::{menu, modal};
use iced::widget::Space;
use iced::{
    Color, ContentFit, Element, Length, Shadow, Subscription, Task,
    alignment::{Horizontal, Vertical},
    font::{Family, Font, Style, Weight},
    time::Instant,
    widget::{
        Button, Column, Row, bottom_center, button, center, center_x, column, container, grid,
        image,
        operation::{self, scroll_to},
        row, rule, scrollable, space, stack, text, text_editor, text_input,
    },
    window,
};
use std::collections::{HashMap, hash_map};
use std::iter::Peekable;

#[derive(Debug, Clone, Copy)]
pub enum Items {
    Movies,
    Shows,
    Seasons,
    Episodes,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scroll(scrollable::Viewport),
    PlayItem(ItemId),
    HoveredItem(bool, ItemId),
    DetailsItem(ItemId),
    Add(ItemId),
    Play(Items),
    OpenConfig,
    AddNewItem,
    MenuToggle(bool),
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
    pub name: String,
    pub view: CollectionView,
    layout: Layout,
    sort: Sort,
    filters: Filter,
    scroll: Scroll,
}

impl CollectionPage {
    pub fn boot(
        collection: &Collection,
        sort: Sort,
        filter: Filter,
        layout: Layout,
    ) -> (Self, Task<CollectionMessage>) {
        let id = collection.id;

        let new = Self::new(collection, sort, filter, layout);

        let scroll =
            operation::scroll_to(new.scroll.id.clone(), operation::AbsoluteOffset::default())
                .map(move |message| CollectionMessage { id, message });

        (new, scroll)
    }

    fn new(collection: &Collection, sort: Sort, filters: Filter, layout: Layout) -> Self {
        Self {
            id: collection.id,
            name: collection.name.clone(),
            view: collection.view,
            layout,
            sort,
            filters,
            scroll: Scroll::new(),
        }
    }

    pub fn update(&mut self, message: CollectionMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::None => None,
            Message::MenuToggle(_) => None,
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
                let msg =
                    HomeMessage::OpenView(ViewMessage::CollectionConfig((self.view, self.id)));

                Some(msg)
            }
            Message::AddNewItem => {
                let msg = HomeMessage::OpenView(ViewMessage::AddToCollection((self.view, self.id)));

                Some(msg)
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        now: Instant,
        collection: &'a CollectionThumbnail,
        movies: Peekable<impl Iterator<Item = &'a Thumbnail<Movie>>>,
        shows: Peekable<impl Iterator<Item = &'a Thumbnail<Show>>>,
        seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
        episodes: Peekable<impl Iterator<Item = &'a Thumbnail<Episode>>>,
    ) -> Element<'a, CollectionMessage> {
        let id = self.id;
        let content = match self.layout {
            Layout::List => self.list(now, movies, shows, seasons, episodes),
            Layout::Grid => self.grid(now, movies, shows, seasons, episodes),
        };

        let content = scrollable(content)
            .spacing(16.0)
            .id(self.scroll.id.clone())
            .on_scroll(move |viewport| CollectionMessage {
                id,
                message: Message::Scroll(viewport),
            });

        let content = column!(self.top(collection), content)
            .spacing(10)
            .padding(10);

        content.into()
    }

    fn top<'a>(&self, collection: &'a CollectionThumbnail) -> Element<'a, CollectionMessage> {
        let id = self.id;

        let img_height = CollectionThumbnail::HEIGHT;

        let img = collection.collage();

        let header = {
            let title = text(&collection.collection.name).size(H3);

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
            let description = container(text(description))
                .max_width(750)
                .height(Length::Fill);

            let play = {
                let base = btn(id, PLAY, "Play", Message::None);

                let actions = column!(
                    btn(id, PLAY, "Play movies", Message::Play(Items::Movies)),
                    btn(id, PLAY, "Play shows", Message::Play(Items::Shows)),
                    btn(id, PLAY, "Play seasons", Message::Play(Items::Seasons)),
                    btn(id, PLAY, "Play episodes", Message::Play(Items::Episodes)),
                )
                .spacing(8);

                let overlay = container(actions).padding([8, 12]).style(|theme| {
                    let default = container::rounded_box(theme);
                    let border = default.border.rounded(8);

                    container::Style { border, ..default }
                });

                menu(base, overlay)
                    .on_toggle(move |toggle| CollectionMessage {
                        id,
                        message: Message::MenuToggle(toggle),
                    })
                    .position(menu::Position::Bottom)
            };

            let actions = row!(
                play,
                btn(id, ADD, "Add", Message::AddNewItem),
                btn(id, EDIT, "Edit", Message::OpenConfig)
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
            .style(|theme| {
                let default = container::dark(theme);
                let background = default
                    .background
                    .map(|background| background.scale_alpha(0.45));

                container::Style {
                    background,
                    ..default
                }
            });

        content.into()
    }

    fn list<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a Thumbnail<Movie>>>,
        mut shows: Peekable<impl Iterator<Item = &'a Thumbnail<Show>>>,
        mut seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
        mut episodes: Peekable<impl Iterator<Item = &'a Thumbnail<Episode>>>,
    ) -> Element<'a, CollectionMessage> {
        let label = |label: &'a str| -> Element<'a, CollectionMessage> {
            let label = text(label).size(H4);
            column!(label, rule::horizontal(2.0)).spacing(4.0).into()
        };
        let collection = self.id;

        let content = Column::new().spacing(40);

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let label = label("Movies");
                let movies = filter_sort(movies, &self.filters, &self.sort);

                let movies: Element<'_, CollectionMessage> = {
                    let content = movies.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            move |id| add(collection, ItemId::Movie(id)),
                            move |id| select(collection, ItemId::Movie(id)),
                            move |id, hovered| hover(collection, hovered, ItemId::Movie(id)),
                            move |id| play(collection, ItemId::Movie(id)),
                            movies::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                };

                column!(label, movies).spacing(10.0)
            };

            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                let label = label("Shows");
                let shows = filter_sort(shows, &self.filters, &self.sort);

                let shows: Element<'_, CollectionMessage> = {
                    let content = shows.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            move |id| add(collection, ItemId::Show(id)),
                            move |id| select(collection, ItemId::Show(id)),
                            move |id, hovered| hover(collection, hovered, ItemId::Show(id)),
                            move |id| play(collection, ItemId::Show(id)),
                            shows::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, shows).spacing(10.0)
            };

            content.push(shows)
        };

        let content = if seasons.peek().is_none() {
            content
        } else {
            let seasons = {
                let label = label("Seasons");
                let seasons = filter_sort(seasons, &self.filters, &self.sort);

                let seasons: Element<'_, CollectionMessage> = {
                    let content = seasons.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            move |id| add(collection, ItemId::Season(id)),
                            move |id| select(collection, ItemId::Season(id)),
                            move |id, hovered| hover(collection, hovered, ItemId::Season(id)),
                            move |id| play(collection, ItemId::Season(id)),
                            |_| empty(),
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, seasons).spacing(10.0)
            };

            content.push(seasons)
        };

        let content = if episodes.peek().is_none() {
            content
        } else {
            let episodes = {
                let label = label("Episodes");
                let episodes = filter_sort(episodes, &self.filters, &self.sort);

                let episodes: Element<'_, CollectionMessage> = {
                    let content = episodes.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            move |id| add(collection, ItemId::Episode(id)),
                            move |id| select(collection, ItemId::Episode(id)),
                            move |id, hovered| hover(collection, hovered, ItemId::Episode(id)),
                            move |id| play(collection, ItemId::Episode(id)),
                            |_| empty(),
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, episodes).spacing(10.0)
            };
            content.push(episodes)
        };

        let content = content;

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        mut movies: Peekable<impl Iterator<Item = &'a Thumbnail<Movie>>>,
        mut shows: Peekable<impl Iterator<Item = &'a Thumbnail<Show>>>,
        mut seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
        mut episodes: Peekable<impl Iterator<Item = &'a Thumbnail<Episode>>>,
    ) -> Element<'a, CollectionMessage> {
        let label = |label: &'a str| -> Element<'a, CollectionMessage> {
            let label = text(label).size(H4);
            column!(label, rule::horizontal(2.0)).spacing(4.0).into()
        };

        let collection = self.id;

        let content = Column::new().spacing(40.0);

        let content = if movies.peek().is_none() {
            content
        } else {
            let movies = {
                let label = label("Movies");
                let movies = filter_sort(movies, &self.filters, &self.sort);

                let movies = movies.map(|thumbnail| {
                    thumbnail.card(
                        now,
                        move |id| add(collection, ItemId::Movie(id)),
                        move |id| select(collection, ItemId::Movie(id)),
                        move |id, hovered| hover(collection, hovered, ItemId::Movie(id)),
                        move |id| play(collection, ItemId::Movie(id)),
                    )
                });

                let movies = grid(movies)
                    .spacing(16)
                    .fluid(CARD_WIDTH)
                    .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

                column!(label, movies).spacing(10.0)
            };
            content.push(movies)
        };

        let content = if shows.peek().is_none() {
            content
        } else {
            let shows = {
                let label = label("Shows");
                let shows = filter_sort(shows, &self.filters, &self.sort);

                let shows = shows.map(|show| {
                    show.card(
                        now,
                        move |id| add(collection, ItemId::Show(id)),
                        move |id| select(collection, ItemId::Show(id)),
                        move |id, hovered| hover(collection, hovered, ItemId::Show(id)),
                        move |id| play(collection, ItemId::Show(id)),
                    )
                });

                let shows = grid(shows)
                    .spacing(16)
                    .fluid(CARD_WIDTH)
                    .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

                column!(label, shows).spacing(10.0)
            };

            content.push(shows)
        };

        let content = if seasons.peek().is_none() {
            content
        } else {
            let seasons = {
                let label = label("Seasons");
                let seasons = filter_sort(seasons, &self.filters, &self.sort);

                let seasons = seasons.map(|season| {
                    season.card(
                        now,
                        move |id| add(collection, ItemId::Season(id)),
                        move |id| select(collection, ItemId::Season(id)),
                        move |id, hovered| hover(collection, hovered, ItemId::Season(id)),
                        move |id| play(collection, ItemId::Season(id)),
                    )
                });

                let seasons = grid(seasons)
                    .spacing(16)
                    .fluid(CARD_WIDTH)
                    .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

                column!(label, seasons).spacing(10.0)
            };

            content.push(seasons)
        };

        let content = if episodes.peek().is_none() {
            content
        } else {
            let episodes = {
                let label = label("Episodes");
                let episodes = filter_sort(episodes, &self.filters, &self.sort);

                let episodes = episodes.map(|episode| {
                    episode.card(
                        now,
                        move |id| add(collection, ItemId::Episode(id)),
                        move |id| select(collection, ItemId::Episode(id)),
                        move |id, hovered| hover(collection, hovered, ItemId::Episode(id)),
                        move |id| play(collection, ItemId::Episode(id)),
                    )
                });

                let episodes = grid(episodes)
                    .spacing(16)
                    .fluid(CARD_WIDTH)
                    .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

                column!(label, episodes).spacing(10.0)
            };

            content.push(episodes)
        };

        content.into()
    }

    pub fn page_update(&mut self, update: PageUpdate) {
        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.layout = layout;
        self.filters = filters;
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

fn btn<'a>(
    id: CollectionId,
    unicode: char,
    label: &'static str,
    message: Message,
) -> Button<'a, CollectionMessage> {
    button(
        row!(icon(unicode).size(P), text(label).size(H7))
            .spacing(10.0)
            .align_y(Vertical::Center),
    )
    .padding([6, 12])
    .on_press(CollectionMessage { id, message })
    .style(|theme, status| {
        let default = button::subtle(theme, status);
        let border = default.border.rounded(5);

        button::Style { border, ..default }
    })
}

fn add(id: CollectionId, item: ItemId) -> CollectionMessage {
    CollectionMessage {
        id,
        message: Message::Add(item),
    }
}

fn select(id: CollectionId, item: ItemId) -> CollectionMessage {
    CollectionMessage {
        id,
        message: Message::DetailsItem(item),
    }
}

fn hover(id: CollectionId, hovered: bool, item: ItemId) -> CollectionMessage {
    CollectionMessage {
        id,
        message: Message::HoveredItem(hovered, item),
    }
}

fn play(id: CollectionId, item: ItemId) -> CollectionMessage {
    CollectionMessage {
        id,
        message: Message::PlayItem(item),
    }
}
