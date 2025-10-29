use super::{
    PageUpdate,
    movies::{self, MoviePreview},
    shared::*,
    shows::{self, EpisodePreview, Series, SeriesMessage, TvSeason, TvSeasonMessage},
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

const COLLAGE_HEIGHT: u32 = 200;
const COLLAGE_WIDTH: u32 = 200;

#[derive(Debug, Clone)]
enum Preview {
    Movie(MoviePreview),
    Show(Box<Series>),
    Season(Box<TvSeason>),
    Episode(EpisodePreview),
}

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
pub enum ThumbnailMessage {
    Movies(Vec<Thumbnail<Movie>>),
    Shows(Vec<Thumbnail<Show>>),
    Seasons(Vec<Thumbnail<Season>>),
    Episodes(Vec<Thumbnail<Episode>>),
}

#[derive(Debug, Clone)]
pub enum ItemMessage {
    PlayItem(ItemId),
    HoveredItem(bool, ItemId),
    DetailsItem(ItemId),
    AddCollection(ItemId),
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
    Thumbnails(ThumbnailMessage),
    Scroll(scrollable::Viewport),
    Item(ItemMessage),
    Show(SeriesMessage),
    Season(TvSeasonMessage),
    Tab(Tab),
    OpenConfig,
    CloseConfig,
    Config(ConfigMessage),
    Play(PlayMessage),
    AddItem,
    CloseModal,
    Animate,
    Toggle(bool),
    None,
}

#[derive(Debug, Clone)]
pub struct CollectionMessage {
    pub id: CollectionId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct CollectionPage {
    now: Instant,
    pub collection: Collection,
    collage: Option<image::Handle>,
    movies: HashMap<MovieId, Thumbnail<Movie>>,
    shows: HashMap<ShowId, Thumbnail<Show>>,
    seasons: HashMap<SeasonId, Thumbnail<Season>>,
    episodes: HashMap<EpisodeId, Thumbnail<Episode>>,
    layout: Layout,
    sort: Sort,
    filters: Filter,
    scroll: Scroll,
    focused: Option<ItemId>,
    selected: Option<Preview>,
    selected_prev: Option<Preview>,
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
        let movies = Task::perform(
            async {
                (0..3)
                    .map(|_| Movie::testing())
                    .chain((0..3).map(|_| Movie::testing2()))
                    .collect::<Vec<_>>()
            },
            |videos| {
                Message::Thumbnails(ThumbnailMessage::Movies(
                    videos.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        )
        .map(move |message| CollectionMessage { id, message });

        let shows = Task::perform(
            async {
                (0..3)
                    .map(|_| Show::testing())
                    .chain((0..3).map(|_| Show::testing1()))
                    .collect::<Vec<_>>()
            },
            |shows| {
                Message::Thumbnails(ThumbnailMessage::Shows(
                    shows.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        )
        .map(move |message| CollectionMessage { id, message });

        let seasons = Task::perform(
            async {
                (0..3)
                    .map(|_| Season::testing())
                    .chain((0..3).map(|_| Season::testing2()))
                    .collect::<Vec<_>>()
            },
            |season| {
                Message::Thumbnails(ThumbnailMessage::Seasons(
                    season.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        )
        .map(move |message| CollectionMessage { id, message });

        let episodes = Task::perform(
            async {
                (0..3)
                    .map(|_| Episode::testing())
                    .chain((0..3).map(|_| Episode::testing2()))
                    .collect::<Vec<_>>()
            },
            |episodes| {
                Message::Thumbnails(ThumbnailMessage::Episodes(
                    episodes.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        )
        .map(move |message| CollectionMessage { id, message });

        let new = Self::new(collection, sort, filter, layout);

        let scroll =
            operation::scroll_to(new.scroll.id.clone(), operation::AbsoluteOffset::default())
                .map(move |message| CollectionMessage { id, message });

        let tasks = Task::batch([movies, shows, seasons, episodes, scroll]);

        (new, tasks)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dummies(
        collection: Collection,
        sort: Sort,
        filter: Filter,
        layout: Layout,
        movies: Vec<Movie>,
        shows: Vec<Show>,
        seasons: Vec<Season>,
        episodes: Vec<Episode>,
    ) -> (Self, Task<CollectionMessage>) {
        let id = collection.id;

        let movies = Task::perform(async move { movies }, |videos| {
            Message::Thumbnails(ThumbnailMessage::Movies(
                videos.into_iter().map(Thumbnail::new).collect(),
            ))
        })
        .map(move |message| CollectionMessage { id, message });

        let shows = Task::perform(async move { shows }, |shows| {
            Message::Thumbnails(ThumbnailMessage::Shows(
                shows.into_iter().map(Thumbnail::new).collect(),
            ))
        })
        .map(move |message| CollectionMessage { id, message });

        let seasons = Task::perform(async move { seasons }, |season| {
            Message::Thumbnails(ThumbnailMessage::Seasons(
                season.into_iter().map(Thumbnail::new).collect(),
            ))
        })
        .map(move |message| CollectionMessage { id, message });

        let episodes = Task::perform(async move { episodes }, |episodes| {
            Message::Thumbnails(ThumbnailMessage::Episodes(
                episodes.into_iter().map(Thumbnail::new).collect(),
            ))
        })
        .map(move |message| CollectionMessage { id, message });

        let new = Self::new(collection, sort, filter, layout);

        let scroll =
            operation::scroll_to(new.scroll.id.clone(), operation::AbsoluteOffset::default())
                .map(move |message| CollectionMessage { id, message });

        let tasks = Task::batch([movies, shows, seasons, episodes, scroll]);

        (new, tasks)
    }

    fn new(collection: Collection, sort: Sort, filter: Filter, layout: Layout) -> Self {
        let paths = collection
            .posters
            .iter()
            .filter_map(|poster| poster.as_ref());

        let collage = collection_collage(paths, COLLAGE_WIDTH, COLLAGE_HEIGHT);

        Self {
            now: Instant::now(),
            collection,
            collage,
            movies: HashMap::default(),
            shows: HashMap::default(),
            seasons: HashMap::default(),
            episodes: HashMap::default(),
            layout,
            sort,
            filters: filter,
            scroll: Scroll::new(),
            focused: None,
            selected: None,
            selected_prev: None,
            config: None,
            view: View::None,
        }
    }

    pub fn update(&mut self, message: CollectionMessage, now: Instant) -> Task<CollectionMessage> {
        self.now = now;
        if message.id != self.collection.id {
            return Task::none();
        }

        match message.message {
            Message::None => Task::none(),
            Message::Toggle(_) => Task::none(),
            Message::Animate => Task::none(),
            Message::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                Task::none()
            }
            Message::Thumbnails(tsg) => {
                match tsg {
                    ThumbnailMessage::Movies(movies) => {
                        for movie in movies {
                            self.movies.insert(movie.id(), movie);
                        }
                    }
                    ThumbnailMessage::Shows(shows) => {
                        for show in shows {
                            self.shows.insert(show.id(), show);
                        }
                    }
                    ThumbnailMessage::Seasons(seasons) => {
                        for season in seasons {
                            self.seasons.insert(season.id(), season);
                        }
                    }
                    ThumbnailMessage::Episodes(episodes) => {
                        for episode in episodes {
                            self.episodes.insert(episode.id(), episode);
                        }
                    }
                }
                Task::none()
            }
            Message::Item(isg) => match isg {
                ItemMessage::PlayItem(ItemId::Movie(id)) => {
                    println!("Play Movie {id:?}");
                    Task::none()
                }
                ItemMessage::PlayItem(ItemId::Show(id)) => {
                    println!("Play Show {id:?}");
                    Task::none()
                }
                ItemMessage::PlayItem(ItemId::Season(id)) => {
                    println!("Play Season {id:?}");
                    Task::none()
                }
                ItemMessage::PlayItem(ItemId::Episode(id)) => {
                    println!("Play Episode {id:?}");
                    Task::none()
                }
                ItemMessage::HoveredItem(is_hovered, ItemId::Movie(id)) => {
                    let Some(movie) = self.movies.get_mut(&id) else {
                        return Task::none();
                    };

                    movie.zoom.go_mut(is_hovered, now);
                    self.focused = Some(ItemId::Movie(id));
                    Task::none()
                }
                ItemMessage::HoveredItem(is_hovered, ItemId::Show(id)) => {
                    let Some(show) = self.shows.get_mut(&id) else {
                        return Task::none();
                    };

                    show.zoom.go_mut(is_hovered, now);
                    self.focused = Some(ItemId::Show(id));
                    Task::none()
                }
                ItemMessage::HoveredItem(is_hovered, ItemId::Season(id)) => {
                    let Some(season) = self.seasons.get_mut(&id) else {
                        return Task::none();
                    };

                    season.zoom.go_mut(is_hovered, now);
                    self.focused = Some(ItemId::Season(id));
                    Task::none()
                }
                ItemMessage::HoveredItem(is_hovered, ItemId::Episode(id)) => {
                    let Some(episode) = self.episodes.get_mut(&id) else {
                        return Task::none();
                    };

                    episode.zoom.go_mut(is_hovered, now);
                    self.focused = Some(ItemId::Episode(id));
                    Task::none()
                }
                ItemMessage::DetailsItem(id) => self.preview(id).unwrap_or(Task::none()),
                ItemMessage::AddCollection(id) => {
                    println!("Add {id:?} to collection");
                    Task::none()
                }
            },
            Message::Show(ssg) => {
                let Some(Preview::Show(show)) = self.selected.as_mut() else {
                    return Task::none();
                };

                let id = self.collection.id;
                show.update(ssg, now).map(move |msg| CollectionMessage {
                    id,
                    message: Message::Show(msg),
                })
            }
            Message::Season(ssg) => {
                let Some(Preview::Season(season)) = self.selected.as_mut() else {
                    return Task::none();
                };

                let id = self.collection.id;
                season.update(ssg, now).map(move |msg| CollectionMessage {
                    id,
                    message: Message::Season(msg),
                })
            }
            Message::Tab(tab) => match self.selected.as_mut() {
                Some(Preview::Movie(movie)) => {
                    movie.tab = tab;
                    Task::none()
                }
                Some(Preview::Episode(episode)) => {
                    episode.tab = tab;
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Play(_) => todo!(),
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
                Task::none()
            }
            Message::CloseConfig => {
                self.config.take();
                self.view = View::None;
                Task::none()
            }
            Message::Config(csg) => {
                let Some(mut config) = self.config.take() else {
                    return Task::none();
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
                        return Task::none();
                    }
                    ConfigMessage::Save => {
                        config.update(&mut self.collection);
                        self.view = View::None;
                        return Task::none();
                    }
                }

                self.config = Some(config);
                Task::none()
            }
            Message::AddItem => {
                self.view = View::Add;
                Task::none()
            }
            Message::CloseModal => {
                self.view = View::None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, CollectionMessage> {
        let collection = self.collection.id;

        match &self.selected {
            Some(Preview::Movie(movie)) => {
                let thumbnail = self
                    .movies
                    .get(&movie.id)
                    .expect("Collection missing movie preview id");

                movie.view(
                    thumbnail,
                    |id| CollectionMessage {
                        id: collection,
                        message: Message::Item(ItemMessage::PlayItem(ItemId::Movie(id))),
                    },
                    |tab| CollectionMessage {
                        id: collection,
                        message: Message::Tab(tab),
                    },
                    |id| CollectionMessage {
                        id: collection,
                        message: Message::Item(ItemMessage::AddCollection(ItemId::Movie(id))),
                    },
                )
            }
            Some(Preview::Episode(episode)) => {
                let thumbnail = self
                    .episodes
                    .get(&episode.id)
                    .expect("Collection missing episode preview id");

                episode.view(
                    thumbnail,
                    |id| CollectionMessage {
                        id: collection,
                        message: Message::Item(ItemMessage::PlayItem(ItemId::Episode(id))),
                    },
                    |tab| CollectionMessage {
                        id: collection,
                        message: Message::Tab(tab),
                    },
                    |id| CollectionMessage {
                        id: collection,
                        message: Message::Item(ItemMessage::AddCollection(ItemId::Episode(id))),
                    },
                )
            }
            Some(Preview::Show(show)) => show.view().map(move |message| CollectionMessage {
                id: collection,
                message: Message::Show(message),
            }),
            Some(Preview::Season(season)) => season.view().map(move |message| CollectionMessage {
                id: collection,
                message: Message::Season(message),
            }),
            None => self.content(),
        }
    }

    pub fn subscription(&self) -> Subscription<CollectionMessage> {
        let id = self.collection.id;

        match &self.selected {
            Some(preview) => match preview {
                Preview::Show(show) => {
                    show.subscription()
                        .with(id)
                        .map(|(id, msg)| CollectionMessage {
                            id,
                            message: Message::Show(msg),
                        })
                }
                Preview::Season(season) => {
                    season
                        .subscription()
                        .with(id)
                        .map(|(id, msg)| CollectionMessage {
                            id,
                            message: Message::Season(msg),
                        })
                }
                Preview::Movie(_) | Preview::Episode(_) => Subscription::none(),
            },
            None => {
                let animating = match &self.focused {
                    Some(ItemId::Movie(id)) => self
                        .movies
                        .get(id)
                        .map(|media| media.is_animating(self.now))
                        .unwrap_or_default(),
                    Some(ItemId::Show(id)) => self
                        .shows
                        .get(id)
                        .map(|media| media.is_animating(self.now))
                        .unwrap_or_default(),
                    Some(ItemId::Season(id)) => self
                        .seasons
                        .get(id)
                        .map(|media| media.is_animating(self.now))
                        .unwrap_or_default(),
                    Some(ItemId::Episode(id)) => self
                        .episodes
                        .get(id)
                        .map(|media| media.is_animating(self.now))
                        .unwrap_or_default(),
                    None => false,
                };

                if animating {
                    window::frames().with(id).map(|(id, _)| CollectionMessage {
                        id,
                        message: Message::Animate,
                    })
                } else {
                    Subscription::none()
                }
            }
        }
    }

    fn add(&self, item: ItemId) -> CollectionMessage {
        CollectionMessage {
            id: self.collection.id,
            message: Message::Item(ItemMessage::AddCollection(item)),
        }
    }

    fn select(&self, item: ItemId) -> CollectionMessage {
        CollectionMessage {
            id: self.collection.id,
            message: Message::Item(ItemMessage::DetailsItem(item)),
        }
    }

    fn hover(&self, hovered: bool, item: ItemId) -> CollectionMessage {
        CollectionMessage {
            id: self.collection.id,
            message: Message::Item(ItemMessage::HoveredItem(hovered, item)),
        }
    }

    fn play(&self, item: ItemId) -> CollectionMessage {
        CollectionMessage {
            id: self.collection.id,
            message: Message::Item(ItemMessage::PlayItem(item)),
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
                        message: Message::Toggle(toggle),
                    })
                    .position(menu::Position::Bottom)
            };

            let actions = row!(
                play,
                btn(collection, ADD, "Add", Message::AddItem),
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

    fn content(&self) -> Element<'_, CollectionMessage> {
        let collection = self.collection.id;
        let content = match self.layout {
            Layout::List => self.list(),
            Layout::Grid => self.grid(),
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

    fn list(&self) -> Element<'_, CollectionMessage> {
        let label = |label: &'static str| -> Element<'_, CollectionMessage> {
            let label = text(label).size(H4);
            column!(label, rule::horizontal(2.0)).spacing(4.0).into()
        };

        let content = Column::new().spacing(40);

        let content = if self.movies.is_empty() {
            content
        } else {
            let movies = {
                let label = label("Movies");
                let movies = filter_sort(self.movies.values(), &self.filters, &self.sort);

                let movies: Element<'_, CollectionMessage> = {
                    let content = movies.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| self.add(ItemId::Movie(id)),
                            |id| self.select(ItemId::Movie(id)),
                            |id, hovered| self.hover(hovered, ItemId::Movie(id)),
                            |id| self.play(ItemId::Movie(id)),
                            movies::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                };

                column!(label, movies).spacing(10.0)
            };

            content.push(movies)
        };

        let content = if self.shows.is_empty() {
            content
        } else {
            let shows = {
                let label = label("Shows");
                let shows = filter_sort(self.shows.values(), &self.filters, &self.sort);

                let shows: Element<'_, CollectionMessage> = {
                    let content = shows.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| self.add(ItemId::Show(id)),
                            |id| self.select(ItemId::Show(id)),
                            |id, hovered| self.hover(hovered, ItemId::Show(id)),
                            |id| self.play(ItemId::Show(id)),
                            shows::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, shows).spacing(10.0)
            };

            content.push(shows)
        };

        let content = if self.seasons.is_empty() {
            content
        } else {
            let seasons = {
                let label = label("Seasons");
                let seasons = filter_sort(self.seasons.values(), &self.filters, &self.sort);

                let seasons: Element<'_, CollectionMessage> = {
                    let content = seasons.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| self.add(ItemId::Season(id)),
                            |id| self.select(ItemId::Season(id)),
                            |id, hovered| self.hover(hovered, ItemId::Season(id)),
                            |id| self.play(ItemId::Season(id)),
                            |_| empty(),
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, seasons).spacing(10.0)
            };

            content.push(seasons)
        };

        let content = if self.episodes.is_empty() {
            content
        } else {
            let episodes = {
                let label = label("Episodes");
                let episodes = filter_sort(self.episodes.values(), &self.filters, &self.sort);

                let episodes: Element<'_, CollectionMessage> = {
                    let content = episodes.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| self.add(ItemId::Episode(id)),
                            |id| self.select(ItemId::Episode(id)),
                            |id, hovered| self.hover(hovered, ItemId::Episode(id)),
                            |id| self.play(ItemId::Episode(id)),
                            |_| empty(),
                        )
                    });

                    column(content).spacing(16.0).into()
                };
                column!(label, episodes).spacing(10.0)
            };
            content.push(episodes)
        };

        content.into()
    }

    fn grid(&self) -> Element<'_, CollectionMessage> {
        let label = |label: &'static str| -> Element<'_, CollectionMessage> {
            let label = text(label).size(H4);
            column!(label, rule::horizontal(2.0)).spacing(4.0).into()
        };

        let content = Column::new().spacing(40.0);

        let content = if self.movies.is_empty() {
            content
        } else {
            let movies = {
                let label = label("Movies");
                let movies = filter_sort(self.movies.values(), &self.filters, &self.sort);

                let movies = movies.map(|thumbnail| {
                    thumbnail.card(
                        self.now,
                        |id| self.add(ItemId::Movie(id)),
                        |id| self.select(ItemId::Movie(id)),
                        |id, hovered| self.hover(hovered, ItemId::Movie(id)),
                        |id| self.play(ItemId::Movie(id)),
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

        let content = if self.shows.is_empty() {
            content
        } else {
            let shows = {
                let label = label("Shows");
                let shows = filter_sort(self.shows.values(), &self.filters, &self.sort);

                let shows = shows.map(|show| {
                    show.card(
                        self.now,
                        |id| self.add(ItemId::Show(id)),
                        |id| self.select(ItemId::Show(id)),
                        |id, hovered| self.hover(hovered, ItemId::Show(id)),
                        |id| self.play(ItemId::Show(id)),
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

        let content = if self.seasons.is_empty() {
            content
        } else {
            let seasons = {
                let label = label("Seasons");
                let seasons = filter_sort(self.seasons.values(), &self.filters, &self.sort);

                let seasons = seasons.map(|season| {
                    season.card(
                        self.now,
                        |id| self.add(ItemId::Season(id)),
                        |id| self.select(ItemId::Season(id)),
                        |id, hovered| self.hover(hovered, ItemId::Season(id)),
                        |id| self.play(ItemId::Season(id)),
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

        let content = if self.episodes.is_empty() {
            content
        } else {
            let episodes = {
                let label = label("Episodes");
                let episodes = filter_sort(self.episodes.values(), &self.filters, &self.sort);

                let episodes = episodes.map(|episode| {
                    episode.card(
                        self.now,
                        |id| self.add(ItemId::Episode(id)),
                        |id| self.select(ItemId::Episode(id)),
                        |id, hovered| self.hover(hovered, ItemId::Episode(id)),
                        |id| self.play(ItemId::Episode(id)),
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

    pub fn preview(&mut self, item: ItemId) -> Option<Task<CollectionMessage>> {
        self.focused = None;
        self.selected_prev = None;

        match item {
            ItemId::Movie(id) => match self.movies.get_mut(&id) {
                Some(thumbnail) => {
                    thumbnail.zoom.go_mut(false, self.now);
                    self.selected = Some(Preview::Movie(MoviePreview::new(
                        id,
                        thumbnail.media.name().to_owned(),
                    )));
                    None
                }
                None => {
                    todo!("Fetch missing media?")
                }
            },
            ItemId::Episode(id) => match self.episodes.get_mut(&id) {
                Some(thumbnail) => {
                    thumbnail.zoom.go_mut(false, self.now);
                    self.selected = Some(Preview::Episode(EpisodePreview::new(
                        id,
                        thumbnail.media.name().to_owned(),
                    )));
                    None
                }
                None => {
                    todo!("Fetch missing media?")
                }
            },
            ItemId::Show(id) => match self.shows.get_mut(&id) {
                Some(show) => {
                    let (show, tasks) =
                        Series::boot(show.media.clone(), self.sort, self.filters, self.layout);

                    self.selected = Some(Preview::Show(Box::new(show)));
                    let id = self.collection.id;

                    let tasks = tasks.map(move |message| CollectionMessage {
                        id,
                        message: Message::Show(message),
                    });

                    Some(tasks)
                }
                None => todo!("Fetch missing media?"),
            },
            ItemId::Season(id) => match self.seasons.get_mut(&id) {
                Some(season) => {
                    let (season, tasks) =
                        TvSeason::boot(season.media.clone(), self.sort, self.filters, self.layout);

                    self.selected = Some(Preview::Season(Box::new(season)));

                    let id = self.collection.id;
                    let tasks = tasks.map(move |message| CollectionMessage {
                        id,
                        message: Message::Season(message),
                    });

                    Some(tasks)
                }
                None => todo!("Fetch missing media?"),
            },
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

        match self.selected.as_mut() {
            Some(Preview::Show(show)) => show.page_update(update, now),
            Some(Preview::Season(season)) => season.page_update(update, now),

            _ => {}
        }
    }

    fn unfocus(&mut self) {
        let Some(id) = self.focused.take() else {
            return;
        };

        match id {
            ItemId::Movie(id) => {
                if let Some(thumbnail) = self.movies.get_mut(&id) {
                    thumbnail.zoom.go_mut(false, self.now);
                }
            }
            ItemId::Show(id) => {
                if let Some(thumbnail) = self.shows.get_mut(&id) {
                    thumbnail.zoom.go_mut(false, self.now);
                }
            }
            ItemId::Season(id) => {
                if let Some(thumbnail) = self.seasons.get_mut(&id) {
                    thumbnail.zoom.go_mut(false, self.now);
                }
            }
            ItemId::Episode(id) => {
                if let Some(thumbnail) = self.episodes.get_mut(&id) {
                    thumbnail.zoom.go_mut(false, self.now);
                }
            }
        }
    }

    pub fn name(&self) -> String {
        match &self.selected {
            Some(Preview::Movie(movie)) => movie.name.clone(),
            Some(Preview::Show(show)) => show.name(),
            Some(Preview::Season(season)) => season.name(),
            Some(Preview::Episode(episode)) => episode.name.clone(),
            None => self.collection.name.clone(),
        }
    }

    pub fn can_back(&self) -> bool {
        self.selected.is_some()
    }

    pub fn can_forward(&self) -> bool {
        let selected = match &self.selected {
            Some(Preview::Show(show)) => show.can_forward(),
            Some(Preview::Season(season)) => season.can_forward(),
            _ => false,
        };

        selected || self.selected_prev.is_some()
    }

    pub fn show_tools(&self) -> bool {
        match &self.selected {
            Some(Preview::Movie(_)) | Some(Preview::Episode(_)) => false,
            Some(Preview::Show(show)) => show.show_tools(),
            Some(Preview::Season(season)) => season.show_tools(),
            None => true,
        }
    }

    pub fn rand(&mut self) -> Task<CollectionMessage> {
        todo!()
    }

    pub fn refresh(&mut self) -> Task<CollectionMessage> {
        todo!()
    }

    pub fn back(&mut self) -> Option<Task<()>> {
        self.unfocus();
        let preview = self.selected.take()?;

        match preview {
            Preview::Show(mut show) => {
                if show.can_back() {
                    let task = show.back();
                    self.selected = Some(Preview::Show(show));
                    task
                } else {
                    self.selected_prev = Some(Preview::Show(show));
                    Some(self.update_scroll())
                }
            }
            Preview::Season(mut season) => {
                if season.can_back() {
                    let task = season.back();
                    self.selected = Some(Preview::Season(season));
                    task
                } else {
                    self.selected_prev = Some(Preview::Season(season));
                    Some(self.update_scroll())
                }
            }
            preview => {
                self.selected_prev = Some(preview);
                Some(self.update_scroll())
            }
        }
    }

    pub fn forward(&mut self) -> Option<Task<()>> {
        self.unfocus();
        match self.selected.as_mut() {
            Some(Preview::Show(show)) => {
                if show.can_forward() {
                    show.forward()
                } else {
                    None
                }
            }
            Some(Preview::Season(season)) => {
                if season.can_forward() {
                    season.forward()
                } else {
                    None
                }
            }
            Some(_) => None,
            None => {
                let prev = self.selected_prev.take()?;

                match prev {
                    Preview::Show(mut show) => {
                        let task = show.update_scroll();
                        self.selected = Some(Preview::Show(show));
                        Some(task)
                    }
                    Preview::Season(mut season) => {
                        let task = season.update_scroll();
                        self.selected = Some(Preview::Season(season));
                        Some(task)
                    }
                    preview => {
                        self.selected = Some(preview);
                        None
                    }
                }
            }
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self.selected.as_mut() {
            Some(Preview::Movie(_)) | Some(Preview::Episode(_)) => Task::none(),
            Some(Preview::Show(show)) => show.update_scroll(),
            Some(Preview::Season(season)) => season.update_scroll(),
            None => operation::scroll_to(self.scroll.id.clone(), self.scroll.offset),
        }
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
