use super::{HomeMessage, MoviePage, PageKind, PageUpdate, SeasonPage, movies, shared::*, shows};
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

const COLLAGE_HEIGHT: u32 = 200;
const COLLAGE_WIDTH: u32 = 200;

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub description: text_editor::Content,
    pub icon: Icon,
    pub view: CollectionView,
    pub theme: Option<u32>,
    pub custom: Option<String>,
}

impl Config {
    pub fn update(self, collection: &mut Collection) {
        let Self {
            name,
            description,
            icon,
            view,
            theme,
            custom,
        } = self;

        collection.name = name;
        let description = description.text();
        if description.is_empty() {
            collection.description = None;
        } else {
            collection.description = Some(description);
        }
        collection.icon = Some(icon.to_u32());
        collection.theme = theme;
        collection.view = view;
        collection.custom = custom;
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
        }
    }

    pub fn to_u32(self) -> u32 {
        self.id as u32
    }

    pub fn all() -> [Self; 13] {
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
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum View {
    None,
    Config,
    Add,
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Name(String),
    Description(text_editor::Action),
    View(CollectionView),
    Icon(Icon),
    Script(String),
    Cancel,
    Save,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayMessage {
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
    OpenConfig,
    CloseConfig,
    Config(ConfigMessage),
    Play(PlayMessage),
    AddNewItem,
    CloseModal,
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
    pub collection: Collection,
    collage: Option<image::Handle>,
    layout: Layout,
    sort: Sort,
    filters: Filter,
    scroll: Scroll,
    pub config: Option<Config>,
    view: View,
}

impl CollectionPage {
    pub fn boot(
        collection: Collection,
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

    #[allow(clippy::too_many_arguments)]
    pub fn dummies(
        collection: Collection,
        sort: Sort,
        filters: Filter,
        layout: Layout,
    ) -> (Self, Task<CollectionMessage>) {
        let id = collection.id;

        let new = Self::new(collection, sort, filters, layout);

        let scroll =
            operation::scroll_to(new.scroll.id.clone(), operation::AbsoluteOffset::default())
                .map(move |message| CollectionMessage { id, message });

        (new, scroll)
    }

    fn new(collection: Collection, sort: Sort, filters: Filter, layout: Layout) -> Self {
        let paths = collection
            .posters
            .iter()
            .filter_map(|poster| poster.as_deref());

        let collage = collection_collage(paths, COLLAGE_WIDTH, COLLAGE_HEIGHT);

        Self {
            collection,
            collage,
            layout,
            sort,
            filters,
            scroll: Scroll::new(),
            config: None,
            view: View::None,
        }
    }

    pub fn update(&mut self, message: CollectionMessage) -> Option<HomeMessage> {
        if message.id != self.collection.id {
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
                let msg = HomeMessage::Add(item);
                Some(msg)
            }
            Message::Play(_) => todo!("Play collection"),
            Message::OpenConfig => {
                let description = text_editor::Content::with_text(
                    self.collection.description.as_deref().unwrap_or_default(),
                );

                let config = Config {
                    name: self.collection.name.clone(),
                    description,
                    view: self.collection.view,
                    icon: Icon::new(self.collection.icon),
                    theme: self.collection.theme,
                    custom: self.collection.custom.clone(),
                };
                self.config = Some(config);
                self.view = View::Config;
                None
            }
            Message::CloseConfig => {
                self.config.take();
                self.view = View::None;
                None
            }
            Message::Config(csg) => {
                let Some(mut config) = self.config.take() else {
                    return None;
                };

                match csg {
                    ConfigMessage::Name(name) => {
                        config.name = name;
                    }
                    ConfigMessage::Description(action) => {
                        config.description.perform(action);
                    }
                    ConfigMessage::View(view) => {
                        config.view = view;
                    }
                    ConfigMessage::Icon(icon) => {
                        config.icon = icon;
                    }
                    ConfigMessage::Script(script) => {
                        if script.is_empty() {
                            config.custom = None
                        } else {
                            config.custom = Some(script)
                        }
                    }
                    ConfigMessage::Cancel => {
                        self.view = View::None;
                        return None;
                    }
                    ConfigMessage::Save => {
                        config.update(&mut self.collection);
                        self.view = View::None;
                        return None;
                    }
                }

                self.config = Some(config);
                None
            }
            Message::AddNewItem => {
                self.view = View::Add;
                None
            }
            Message::CloseModal => {
                self.view = View::None;
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        now: Instant,
        movies: Peekable<impl Iterator<Item = &'a Thumbnail<Movie>>>,
        shows: Peekable<impl Iterator<Item = &'a Thumbnail<Show>>>,
        seasons: Peekable<impl Iterator<Item = &'a Thumbnail<Season>>>,
        episodes: Peekable<impl Iterator<Item = &'a Thumbnail<Episode>>>,
    ) -> Element<'a, CollectionMessage> {
        let collection = self.collection.id;
        let content = match self.layout {
            Layout::List => self.list(now, movies, shows, seasons, episodes),
            Layout::Grid => self.grid(now, movies, shows, seasons, episodes),
        };

        let content = scrollable(content)
            .spacing(16.0)
            .id(self.scroll.id.clone())
            .on_scroll(move |view| CollectionMessage {
                id: collection,
                message: Message::Scroll(view),
            });

        let content = column!(self.top(), content).spacing(10).padding(10);

        match self.view {
            View::None => content.into(),
            View::Add => {
                let overlay = container("work in progress");

                modal(content, overlay)
                    .on_blur(CollectionMessage {
                        id: collection,
                        message: Message::CloseConfig,
                    })
                    .into()
            }
            View::Config => modal(content, self.config())
                .on_blur(CollectionMessage {
                    id: collection,
                    message: Message::Config(ConfigMessage::Cancel),
                })
                .into(),
        }
    }

    fn top(&self) -> Element<'_, CollectionMessage> {
        let collection = self.collection.id;

        let img_height = COLLAGE_HEIGHT;
        let img_width = COLLAGE_WIDTH;
        let img: Element<'_, CollectionMessage> = {
            match &self.collage {
                Some(handle) => image(handle)
                    .border_radius(10)
                    .height(img_height)
                    .width(img_width)
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
                        .height(img_height)
                        .width(img_width)
                        .style(|theme| {
                            let default = container::dark(theme);
                            let border = default.border.rounded(10.0);

                            container::Style { border, ..default }
                        })
                        .into()
                }
            }
        };

        let header = {
            let title = text(&self.collection.name).size(H3);

            let title = row!(title)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(10.0);

            let title = if matches!(self.collection.view, CollectionView::Hidden) {
                let view = icon(view_unicode(self.collection.view)).size(H3);

                title.push(view)
            } else {
                title
            };

            let description = self.collection.description.as_deref().unwrap_or_default();
            let description = container(text(description))
                .max_width(750)
                .height(Length::Fill);

            let play = {
                let base = btn(collection, PLAY, "Play", Message::None);

                let actions = column!(
                    btn(
                        collection,
                        PLAY,
                        "Play movies",
                        Message::Play(PlayMessage::Movies)
                    ),
                    btn(
                        collection,
                        PLAY,
                        "Play shows",
                        Message::Play(PlayMessage::Shows)
                    ),
                    btn(
                        collection,
                        PLAY,
                        "Play seasons",
                        Message::Play(PlayMessage::Seasons)
                    ),
                    btn(
                        collection,
                        PLAY,
                        "Play episodes",
                        Message::Play(PlayMessage::Episodes)
                    ),
                )
                .spacing(8);

                let overlay = container(actions).padding([8, 12]).style(|theme| {
                    let default = container::rounded_box(theme);
                    let border = default.border.rounded(8);

                    container::Style { border, ..default }
                });

                menu(base, overlay)
                    .on_toggle(move |toggle| CollectionMessage {
                        id: collection,
                        message: Message::MenuToggle(toggle),
                    })
                    .position(menu::Position::Bottom)
            };

            let actions = row!(
                play,
                btn(collection, ADD, "Add", Message::AddNewItem),
                btn(collection, EDIT, "Edit", Message::OpenConfig)
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

    fn config(&self) -> Element<'_, CollectionMessage> {
        let config = self.config.as_ref().expect("Config should have been set");
        let collection = self.collection.id;
        let width = 500;
        let height = 500;

        let icon_height = 40.0;
        let icon_width = 40.0;

        let name = {
            let label = text("Name");

            let value = config.name.as_str();

            let input = text_input("", value)
                .on_input(move |input| CollectionMessage {
                    id: collection,
                    message: Message::Config(ConfigMessage::Name(input)),
                })
                .width(Length::Fill);

            column!(label, input).spacing(2)
        };

        let description = {
            let label = text("Description");

            let content = &config.description;
            let editor = text_editor(content)
                .on_action(move |action| CollectionMessage {
                    id: collection,
                    message: Message::Config(ConfigMessage::Description(action)),
                })
                .height(height as f32 * 0.2);

            column!(label, editor).spacing(2)
        };

        let view = {
            let selected = config.view;

            let label = text("Visibility");

            let views = [
                CollectionView::Pinned,
                CollectionView::Shown,
                CollectionView::Hidden,
            ]
            .into_iter()
            .map(|view| view_draw(collection, view, view == selected));

            let views = grid(views)
                .spacing(16)
                .fluid(icon_width)
                .height(grid::aspect_ratio(icon_width, icon_height));

            column!(label, views).spacing(2)
        };

        let icons = {
            let selected = config.icon;

            let label = text("Icon");

            let icons = Icon::all()
                .into_iter()
                .map(|icon| icon_draw(collection, icon, icon == selected));

            let icons = grid(icons)
                .spacing(16)
                .fluid(icon_width)
                .height(grid::aspect_ratio(icon_width, icon_height));

            column!(label, icons).spacing(2)
        };

        let actions = {
            let save = button("Save").on_press(CollectionMessage {
                id: collection,
                message: Message::Config(ConfigMessage::Save),
            });

            let cancel = button("Cancel").on_press(CollectionMessage {
                id: collection,
                message: Message::Config(ConfigMessage::Cancel),
            });

            column!(row!(save, cancel).spacing(80))
                .align_x(Horizontal::Center)
                .width(Length::Fill)
        };

        let content =
            column!(name, description, view, icons, space::vertical(), actions).spacing(16);

        let content = container(content)
            .padding([16, 24])
            .style(container::dark)
            .width(width)
            .height(height);

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
        let collection = self.collection.id;

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

        let collection = self.collection.id;

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

    pub fn name(&self) -> &str {
        &self.collection.name
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
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

pub fn view_unicode(view: CollectionView) -> char {
    match view {
        CollectionView::Shown => EYE,
        CollectionView::Pinned => PIN,
        CollectionView::Hidden => HIDE,
    }
}

fn view_draw<'a>(
    collection: CollectionId,
    view: CollectionView,
    selected: bool,
) -> Element<'a, CollectionMessage> {
    let unicode = view_unicode(view);

    let content = center(icon(unicode).size(P));

    button(content)
        .on_press(CollectionMessage {
            id: collection,
            message: Message::Config(ConfigMessage::View(view)),
        })
        .style(move |theme, status| {
            let default = if selected {
                button::secondary(theme, status)
            } else {
                button::background(theme, status)
            };
            let border = default.border.rounded(10.0);

            button::Style { border, ..default }
        })
        .into()
}

fn icon_draw<'a>(
    collection: CollectionId,
    value: Icon,
    selected: bool,
) -> Element<'a, CollectionMessage> {
    let content = center(icon(value.unicode()).size(P));

    button(content)
        .on_press(CollectionMessage {
            id: collection,
            message: Message::Config(ConfigMessage::Icon(value)),
        })
        .style(move |theme, status| {
            let default = if selected {
                button::secondary(theme, status)
            } else {
                button::background(theme, status)
            };
            let border = default.border.rounded(10.0);

            button::Style { border, ..default }
        })
        .into()
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
        id: id,
        message: Message::PlayItem(item),
    }
}
