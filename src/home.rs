use iced::{
    Color, Element, Length, Padding, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    animation::Animation,
    border::Border,
    time::{Duration, Instant},
    widget::{
        self, button, center, column, container, grid,
        operation::{self, scroll_to},
        pick_list, row, rule, scrollable, space, text, text_editor, text_input, tooltip as tp,
    },
    window,
};
use std::collections::{HashMap, HashSet};

mod collection;
mod collections;
mod draws;
mod episode;
mod movie;
mod movies;
mod pages;
mod season;
mod series;
pub mod shared;
mod shows;

use draws::*;
use registry::models::{
    Collection, CollectionId, CollectionView, Episode, EpisodeId, Media, Movie, MovieId, Season,
    SeasonId, Show, ShowId, SimpleCollection,
    collection::{
        ItemId, Items,
        triggers::{self, Comparison, DeleteId, DeleteTrigger, InsertId, InsertTrigger, Logic},
    },
};
use registry::{
    filter::{self, Filter, FilterMode, SearchFilter},
    sort::{Sort, SortKind},
};

use crate::app::{MediaUpdate, MediaUpdateKind};

use crate::app::{FetchId, Message};
use crate::utils::{
    self, HomeAction, Layout, Scroll, empty, icons,
    icons::*,
    loading_animation, loading_svg,
    modal::{self, modal},
    picklist_handle, styles, tooltip,
    typo::*,
};

use collection::{CollectionMessage, CollectionPage};
use collections::{Collections, CollectionsMessage};
use episode::{EpisodePage, EpisodePageMessage};
use movie::{MoviePage, MoviePageMessage};
use movies::{Movies, MoviesMessage};
use pages::{Page, PageKind};
use season::{SeasonPage, SeasonPageMessage};
use series::{ShowPage, ShowPageMessage};
use shared::{CARD_HEIGHT, CARD_WIDTH, CollectionThumbnail, Icon, SearchView, Thumbnail};
use shows::{TvShows, TvShowsMessage};
use widgets::marquee;
use widgets::menu::{Position, menu};

const SIDE_ICON_SPACING: f32 = 8.0;

#[derive(Debug, Clone)]
pub enum FilterMessage {
    Mode,
    Clear,
    ProgressKind(filter::ProgressKind),
    ProgressComp,
    RatingKind(filter::RatingKind),
    RatingComp,
    CommentsNum(String),
    CommentsComp,
    ReleaseYear(String),
    ReleaseComp,
    DurationHours(String),
    DurationMinutes(String),
    DurationComp,
}

#[derive(Debug, Clone, Copy)]
pub enum SortMessage {
    AddSort(SortKind),
    RemoveSort(SortKind),
    ReverseSort(SortKind),
    Clear,
    ToggleReverse,
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Name(String),
    Description(text_editor::Action),
    View(CollectionView),
    Icon(Icon),
    Save,
}

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    id: Option<CollectionId>,
    name: String,
    empty_name: bool,
    description: text_editor::Content,
    icon: Icon,
    view: CollectionView,
    theme: Option<u32>,
    name_input: widget::Id,
}

impl CollectionConfig {
    pub fn update(self, collection: &mut Collection) {
        let Self {
            id: _id,
            name,
            description,
            icon,
            view,
            theme,
            name_input: _text_input,
            empty_name: _empty_name,
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
    }

    pub fn from_collection(collection: &Collection) -> (Self, widget::Id) {
        let description =
            text_editor::Content::with_text(collection.description.as_deref().unwrap_or_default());

        let name_input = widget::Id::unique();

        (
            Self {
                id: Some(collection.id),
                name: collection.name.clone(),
                empty_name: collection.name.is_empty(),
                description,
                view: collection.view,
                icon: Icon::new(collection.icon),
                theme: collection.theme,
                name_input: name_input.clone(),
            },
            name_input,
        )
    }

    pub fn new() -> (Self, widget::Id) {
        let name_input = widget::Id::unique();

        (
            Self {
                id: None,
                name: String::new(),
                empty_name: true,
                description: text_editor::Content::new(),
                icon: Icon::new(None),
                view: CollectionView::Shown,
                theme: None,
                name_input: name_input.clone(),
            },
            name_input,
        )
    }
}

#[derive(Debug)]
pub struct SearchState {
    items: Vec<SearchView>,
    search: String,
    last_edit: Option<Instant>,
    filter: Option<SearchFilter>,
    text_input: widget::Id,
}

#[derive(Debug, Clone)]
pub enum SearchMessage {
    Load,
    Search(String),
    Searching,
    ClearFilter,
}

#[derive(Debug, Clone)]
pub struct CollectionAddState {
    item: ItemId,
    initial: HashSet<CollectionId>,
    selected: HashSet<CollectionId>,
}

#[derive(Debug, Clone)]
pub enum CollectionAddMessage {
    Toggle(bool, CollectionId),
    Save,
}

#[derive(Debug, Clone)]
pub enum LogicMessage {
    Name(String),
    NameComp(bool),
    Synopsis(String),
    SynopsisComp(bool),
    Tags(String),
    TagsComp(bool),
    Dir(String),
    DirComp(bool),
    Last(String),
    LastComp(Comparison),
    Duration(String),
    DurationComp(Comparison),
    Progress(String),
    ProgressComp(Comparison),
    Watch(String),
    WatchComp(Comparison),
    Release(String),
    ReleaseComp(Comparison),
    Rating(String),
    RatingComp(Comparison),
    Comment(String),
    CommentComp(Comparison),
}

#[derive(Debug, Clone)]
pub enum TriggerMessage {
    Save,
    GenerateDelete(InsertId),
    ToggleExpandInsert(InsertId, bool),
    ToggleExpandDelete(DeleteId, bool),
    Tab,
    AddInsert,
    AddDelete,
    DuplicateInsert(InsertId),
    DuplicateDelete(DeleteId),
    RemoveInsert(InsertId),
    RemoveDelete(DeleteId),
    NameInsert(InsertId, String),
    NameDelete(DeleteId, String),
    MediaInsert(InsertId, triggers::Media),
    MediaDelete(DeleteId, triggers::Media),
    ROEInsert(InsertId, bool),
    ROEDelete(DeleteId, bool),
    ToggleROEInsert(InsertId),
    ToggleROEDelete(DeleteId),
    LogicInsert(InsertId, LogicMessage),
    LogicDelete(DeleteId, LogicMessage),
}

#[derive(Debug, Clone)]
pub enum Rating {
    Value(f32),
    Input { id: widget::Id, input: String },
}

#[derive(Debug, Clone)]
pub enum RatingMessage {
    Type,
    Submit,
    Star(u8),
    Input(String),
}

#[derive(Debug, Clone)]
pub enum RenameMessage {
    Input(String),
    Submit,
}

#[derive(Debug, Clone)]
pub enum SynopsisMessage {
    Action(text_editor::Action),
    Submit,
}

#[derive(Debug, Clone)]
pub enum TMDBMessage {
    Input(String),
    Submit,
}

#[derive(Debug, Clone)]
pub enum SelectionMessage {
    Select(ItemId),
    Cancel,
    Play,
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    CollectionConfig,
    CollectionTriggers,
    Add(ItemId),
    AddToCollection(CollectionId),
    Search,
    Rating(ItemId, Option<f32>),
    Rename { id: ItemId, old: String },
    Synopsis { id: ItemId, old: String },
    TMDBId { id: ItemId, top_level: bool },
    RemoveMedia { id: ItemId, name: String },
    RemoveCollection { id: CollectionId, name: String },
    Selection,
}

#[derive(Debug)]
pub enum View {
    CollectionConfig(CollectionConfig),
    Search(SearchState, Option<CollectionId>),
    CollectionAdd(CollectionAddState),
    Rating {
        id: ItemId,
        rating: Rating,
    },
    Rename {
        id: ItemId,
        input: widget::Id,
        old: String,
        value: String,
        empty: bool,
    },
    Synopsis {
        id: ItemId,
        editor: widget::Id,
        content: text_editor::Content,
    },
    TMDBId {
        id: ItemId,
        input: widget::Id,
        value: String,
        top_level: bool,
    },
    RemoveMedia {
        id: ItemId,
        name: String,
    },
    RemoveCollection {
        id: CollectionId,
        name: String,
    },
    CollectionTriggers {
        id: CollectionId,
        view_inserts: bool,
        itriggers: Vec<(bool, InsertTrigger, bool, String, String)>,
        removed_inserts: Vec<InsertTrigger>,
        dtriggers: Vec<(bool, DeleteTrigger, bool, String, String)>,
        removed_deletes: Vec<DeleteTrigger>,
    },
    Selection(Vec<ItemId>),
}

#[derive(Debug, Clone)]
enum State {
    Loading(Animation<bool>),
    Recent {
        shows: Vec<Thumbnail<Show>>,
        movies: Vec<Thumbnail<Movie>>,
    },
    Shows(Vec<Thumbnail<Show>>),
    Movies(Vec<Thumbnail<Movie>>),
    Collections(Vec<CollectionThumbnail>),
    Movie {
        movie: Thumbnail<Movie>,
        memberships: Vec<SimpleCollection>,
    },
    Show {
        show: Thumbnail<Show>,
        memberships: Vec<SimpleCollection>,
        seasons: Vec<Thumbnail<Season>>,
    },
    Season {
        season: Thumbnail<Season>,
        memberships: Vec<SimpleCollection>,
        episodes: Vec<Thumbnail<Episode>>,
    },
    Episode {
        episode: Thumbnail<Episode>,
        memberships: Vec<SimpleCollection>,
    },
    Collection {
        collection: Box<CollectionThumbnail>,
        itriggers: Vec<InsertTrigger>,
        dtriggers: Vec<DeleteTrigger>,
        shows: Vec<Thumbnail<Show>>,
        movies: Vec<Thumbnail<Movie>>,
        seasons: Vec<Thumbnail<Season>>,
        episodes: Vec<Thumbnail<Episode>>,
    },
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    ToggleSort,
    ToggleFilter,
    Filter(FilterMessage),
    Sort(SortMessage),
    Movies(MoviesMessage),
    Shows(TvShowsMessage),
    Collection(CollectionMessage),
    Collections(CollectionsMessage),
    MoviePage(MoviePageMessage),
    EpisodePage(EpisodePageMessage),
    ShowPage(ShowPageMessage),
    SeasonPage(SeasonPageMessage),
    Settings,
    Random,
    Back,
    Forward,
    CollectionConfig(ConfigMessage),
    RemoveCollection,
    RemoveCollectionItems(CollectionId, Items),
    SearchMessage(SearchMessage),
    CollectionAdd(CollectionAddMessage),
    Rating(RatingMessage),
    Rename(RenameMessage),
    Synopsis(SynopsisMessage),
    TMDBId(TMDBMessage),
    Selection(SelectionMessage),
    RemoveMedia,
    Refetch(ItemId),
    OpenView(ViewMessage),
    AddCollection(ItemId, CollectionId),
    CloseView,
    Play(ItemId),
    PlayCollection {
        id: CollectionId,
        items: Items,
    },
    ToggleLayout,
    Goto(PageKind),
    NewCollection,
    None,
    Scroll(scrollable::Viewport),
    RefreshContent,
    Hovered(ItemId, bool),
    FetchedCollections(Vec<CollectionThumbnail>),
    FetchedCollection {
        collection: Box<CollectionThumbnail>,
        itriggers: Vec<InsertTrigger>,
        dtriggers: Vec<DeleteTrigger>,
        movies: Vec<Thumbnail<Movie>>,
        shows: Vec<Thumbnail<Show>>,
        seasons: Vec<Thumbnail<Season>>,
        episodes: Vec<Thumbnail<Episode>>,
    },
    Trigger(TriggerMessage),
}

pub struct Home {
    forward: Vec<PageKind>,
    backward: Vec<PageKind>,
    current_page: Option<PageKind>,
    pages: HashMap<PageKind, Page>,

    state: State,

    collections: Vec<SimpleCollection>,

    layout: Layout,
    sort: Sort,
    filters: Filter,

    show_sorts: bool,
    show_filters: bool,

    view: Option<View>,

    scroll: Scroll,

    recent_limit: Option<i32>,

    scanning: Option<Animation<bool>>,

    focused: Option<ItemId>,

    pub command: bool,
}

impl Home {
    pub fn boot(
        layout: Layout,
        filters: Filter,
        sort: Sort,
        recent_limit: Option<i32>,
    ) -> (Self, Task<Message>) {
        let recents = Task::done(Message::Fetch {
            id: FetchId::Recents,
            filters,
            sort: Sort::recents(),
            limit: recent_limit,
            offset: None,
        });

        let collections = Task::done(Message::Fetch {
            id: FetchId::CollectionsSimple,
            filters,
            sort,
            limit: None,
            offset: None,
        });

        let tasks = collections.chain(recents);

        (Self::new(layout, filters, sort, recent_limit), tasks)
    }

    fn new(layout: Layout, filters: Filter, sort: Sort, recent_limit: Option<i32>) -> Self {
        Self {
            forward: vec![],
            backward: vec![],
            current_page: Some(PageKind::Home),
            pages: [(PageKind::Home, Page::Home)].into(),

            layout,
            sort,
            filters,

            show_sorts: false,
            show_filters: false,
            state: State::Loading(loading_animation(Instant::now())),
            scroll: Scroll::new(),
            collections: Vec::default(),
            view: None,
            recent_limit,
            scanning: None,
            focused: None,
            command: false,
        }
    }

    fn unfocus(&mut self, now: Instant) {
        let Some(id) = self.focused.take() else {
            return;
        };

        match (id, &mut self.state) {
            (_, State::Loading(_)) => {}
            (ItemId::Show(id), State::Recent { shows, .. })
            | (ItemId::Show(id), State::Shows(shows))
            | (ItemId::Show(id), State::Collection { shows, .. }) => {
                if let Some(item) = shows.iter_mut().find(|show| show.media.id() == id) {
                    item.go_mut(false, now);
                }
            }
            (ItemId::Movie(id), State::Recent { movies, .. })
            | (ItemId::Movie(id), State::Movies(movies))
            | (ItemId::Movie(id), State::Collection { movies, .. }) => {
                if let Some(item) = movies.iter_mut().find(|movie| movie.media.id() == id) {
                    item.go_mut(false, now);
                }
            }
            (ItemId::Season(id), State::Show { seasons, .. })
            | (ItemId::Season(id), State::Collection { seasons, .. }) => {
                if let Some(item) = seasons.iter_mut().find(|season| season.media.id() == id) {
                    item.go_mut(false, now);
                }
            }
            (ItemId::Episode(id), State::Season { episodes, .. })
            | (ItemId::Episode(id), State::Collection { episodes, .. }) => {
                if let Some(item) = episodes.iter_mut().find(|episode| episode.media.id() == id) {
                    item.go_mut(false, now);
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, message: HomeMessage, now: Instant) -> Task<Message> {
        match message {
            HomeMessage::None => Task::none(),
            HomeMessage::Settings => {
                self.unfocus(now);
                Task::done(Message::SettingsOpen)
            }
            HomeMessage::FetchedCollections(collections) => {
                self.state = State::Collections(collections);

                self.update_page_scroll()
            }
            HomeMessage::FetchedCollection {
                collection,
                itriggers,
                dtriggers,
                mut movies,
                mut shows,
                mut seasons,
                mut episodes,
            } => {
                if let Some(View::Selection(selected)) = &self.view {
                    for media in &mut movies {
                        media.selected = selected.contains(&media.media.id.into());
                    }
                    for media in &mut shows {
                        media.selected = selected.contains(&media.media.id.into());
                    }
                    for media in &mut seasons {
                        media.selected = selected.contains(&media.media.id.into());
                    }
                    for media in &mut episodes {
                        media.selected = selected.contains(&media.media.id.into());
                    }
                }

                self.state = State::Collection {
                    collection,
                    itriggers,
                    dtriggers,
                    shows,
                    movies,
                    seasons,
                    episodes,
                };

                self.update_page_scroll()
            }
            HomeMessage::Goto(kind) => self.goto(kind, now),
            HomeMessage::Movies(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.movies_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::Shows(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.shows_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::Collections(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.collections_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::MoviePage(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.movie_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::ShowPage(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.show_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::SeasonPage(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.season_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::EpisodePage(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.episode_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::Collection(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.collection_update(message)
                    .map(|hsg| Task::done(Message::Home(hsg)))
                    .unwrap_or_default()
            }
            HomeMessage::OpenView(view) => {
                self.unfocus(now);
                match view {
                    ViewMessage::CollectionConfig => {
                        let State::Collection { collection, .. } = &self.state else {
                            return Task::none();
                        };

                        let (config, name_input) =
                            CollectionConfig::from_collection(&collection.collection);

                        self.view = Some(View::CollectionConfig(config));
                        let focus = operation::focus(name_input);

                        Task::batch([focus, self.update_page_scroll()])
                    }
                    ViewMessage::CollectionTriggers => {
                        let State::Collection {
                            collection,
                            itriggers,
                            dtriggers,
                            ..
                        } = &self.state
                        else {
                            return Task::none();
                        };

                        self.view = Some(View::CollectionTriggers {
                            id: collection.collection.id,
                            view_inserts: true,
                            itriggers: itriggers
                                .clone()
                                .into_iter()
                                .map(|trigger| {
                                    let last = trigger
                                        .logic
                                        .last_watched
                                        .as_ref()
                                        .map(|(_, last)| last.format("%F %R").to_string())
                                        .unwrap_or_default();

                                    let release = trigger
                                        .logic
                                        .release
                                        .as_ref()
                                        .map(|(_, release)| release.format("%F").to_string())
                                        .unwrap_or_default();

                                    (false, trigger, false, last, release)
                                })
                                .collect(),
                            dtriggers: dtriggers
                                .clone()
                                .into_iter()
                                .map(|trigger| {
                                    let last = trigger
                                        .logic
                                        .last_watched
                                        .as_ref()
                                        .map(|(_, last)| last.format("%F %R").to_string())
                                        .unwrap_or_default();

                                    let release = trigger
                                        .logic
                                        .release
                                        .as_ref()
                                        .map(|(_, release)| release.format("%F").to_string())
                                        .unwrap_or_default();

                                    (false, trigger, false, last, release)
                                })
                                .collect(),
                            removed_inserts: vec![],
                            removed_deletes: vec![],
                        });

                        self.update_page_scroll()
                    }
                    ViewMessage::Add(item) => {
                        let state = CollectionAddState {
                            item,
                            selected: HashSet::new(),
                            initial: HashSet::new(),
                        };
                        self.view = Some(View::CollectionAdd(state));

                        Task::done(Message::FetchMembershipIds(item))
                    }
                    ViewMessage::AddToCollection(id) => self.toggle_search(Some(id), now),
                    ViewMessage::Search => self.toggle_search(None, now),
                    ViewMessage::Rating(id, rating) => {
                        let rating = Rating::Value(rating.unwrap_or_default());
                        self.view = Some(View::Rating { id, rating });

                        self.update_page_scroll()
                    }
                    ViewMessage::Rename { id, old } => {
                        let input = widget::Id::unique();
                        self.view = Some(View::Rename {
                            id,
                            empty: old.is_empty(),
                            input: input.clone(),
                            value: old.clone(),
                            old,
                        });

                        Task::batch([self.update_page_scroll(), operation::focus(input)])
                    }
                    ViewMessage::Synopsis { id, old } => {
                        let editor = widget::Id::unique();

                        self.view = Some(View::Synopsis {
                            id,
                            editor: editor.clone(),
                            content: text_editor::Content::with_text(&old),
                        });

                        Task::batch([self.update_page_scroll(), operation::focus(editor)])
                    }
                    ViewMessage::TMDBId { id, top_level } => {
                        let input = widget::Id::unique();
                        self.view = Some(View::TMDBId {
                            id,
                            input: input.clone(),
                            value: String::new(),
                            top_level,
                        });

                        Task::batch([self.update_page_scroll(), operation::focus(input)])
                    }
                    ViewMessage::RemoveMedia { id, name } => {
                        self.view = Some(View::RemoveMedia { id, name });

                        self.update_page_scroll()
                    }
                    ViewMessage::RemoveCollection { id, name } => {
                        self.view = Some(View::RemoveCollection { id, name });

                        self.update_page_scroll()
                    }
                    ViewMessage::Selection => self.selection(),
                }
            }
            HomeMessage::CollectionConfig(csg) => {
                let Some(View::CollectionConfig(mut config)) = self.view.take() else {
                    return Task::none();
                };

                match csg {
                    ConfigMessage::Name(name) => {
                        config.empty_name = name.is_empty();
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
                    ConfigMessage::Save if config.id.is_some() => {
                        let close_view = self.close_view(true);

                        let State::Collection { collection, .. } = &mut self.state else {
                            return Task::none();
                        };

                        if config.empty_name {
                            let focus = operation::focus(config.name_input.clone());
                            self.view = Some(View::CollectionConfig(config));
                            return focus;
                        }

                        if let Some(collection) = self
                            .collections
                            .iter_mut()
                            .find(|own| own.id == collection.collection.id)
                        {
                            collection.name = config.name.clone();
                            collection.view = config.view;
                            collection.icon = Some(config.icon.to_u32());
                        }

                        config.update(&mut collection.collection);

                        sort_collections(&mut self.collections);

                        let query = collection.collection.save();

                        return Task::batch([Task::done(Message::Query(query)), close_view]);
                    }
                    ConfigMessage::Save => {
                        if config.empty_name {
                            let focus = operation::focus(config.name_input.clone());
                            self.view = Some(View::CollectionConfig(config));
                            return focus;
                        }

                        let CollectionConfig {
                            name,
                            description,
                            icon,
                            view,
                            theme,
                            id: _unused1,
                            name_input: _unused2,
                            empty_name: _empty_name,
                        } = config;

                        let description = description.text();
                        let description = if description.is_empty() {
                            None
                        } else {
                            Some(description)
                        };

                        let (new, query) =
                            Collection::new(name, description, view, Some(icon.to_u32()), theme);

                        let new_id = new.id;
                        let simple = SimpleCollection::from_collection(&new);
                        self.collections.push(simple);
                        sort_collections(&mut self.collections);

                        let close_view = self.close_view(true);
                        self.state = State::Collection {
                            collection: Box::new(CollectionThumbnail::new(new)),
                            itriggers: vec![],
                            dtriggers: vec![],
                            shows: vec![],
                            movies: vec![],
                            episodes: vec![],
                            seasons: vec![],
                        };

                        if let Some(old) = self.current_page.replace(PageKind::Collection(new_id)) {
                            self.backward.push(old)
                        };
                        self.forward.clear();

                        let msg = {
                            let (collection, tasks) = CollectionPage::boot(new_id);

                            self.pages.insert(
                                PageKind::Collection(new_id),
                                Page::Collection {
                                    collection,
                                    id: new_id,
                                },
                            );

                            tasks.map(|csg| Message::Home(HomeMessage::Collection(csg)))
                        };

                        return Task::batch([Task::done(Message::Query(query)), msg, close_view]);
                    }
                }

                self.view = Some(View::CollectionConfig(config));

                Task::none()
            }
            HomeMessage::RemoveCollection => {
                self.unfocus(now);
                let Some(View::RemoveCollection { id, .. }) = self.view.as_ref() else {
                    return Task::none();
                };

                let Some((index, _)) = self
                    .collections
                    .iter()
                    .enumerate()
                    .find(|(_, collection)| collection.id == *id)
                else {
                    return Task::none();
                };

                let old = self.collections.remove(index);
                let remove = Message::RemoveCollection(old.id).tasked();

                Task::batch([remove, self.back(now, true), self.close_view(true)])
            }
            HomeMessage::RemoveCollectionItems(collection, items) => {
                self.unfocus(now);

                let remove = Message::RemoveCollectionItems { collection, items }.tasked();

                Task::batch([remove, self.content_refresh(now)])
            }
            HomeMessage::SearchMessage(ssg) => {
                let Some(View::Search(state, _)) = self.view.as_mut() else {
                    return Task::none();
                };

                match ssg {
                    SearchMessage::Search(mut search) => {
                        state.last_edit = Some(now);

                        let filter = search.find(":").and_then(|pos| {
                            SearchFilter::new(&search[0..pos]).map(|filter| (pos, filter))
                        });

                        match filter {
                            Some((pos, filter)) => {
                                search.replace_range(0..=pos, "");
                                state.filter = Some(filter);
                                state.search = search;

                                operation::focus(state.text_input.clone())
                            }
                            None => match state.filter {
                                Some(filter) if search.is_empty() && state.search.is_empty() => {
                                    state.filter = None;

                                    state.search = filter.to_str().to_owned();

                                    operation::focus(state.text_input.clone())
                                }
                                _ => {
                                    state.search = search;

                                    Task::none()
                                }
                            },
                        }
                    }
                    SearchMessage::ClearFilter => {
                        state.filter = None;

                        self.load()
                    }
                    SearchMessage::Load => self.load(),
                    SearchMessage::Searching => {
                        if state
                            .last_edit
                            .map(|last_edit| {
                                now.duration_since(last_edit) < Duration::from_millis(500)
                            })
                            .unwrap_or_default()
                        {
                            return Task::none();
                        }

                        if state.last_edit.is_none() {
                            return Task::none();
                        }

                        self.load()
                    }
                }
            }
            HomeMessage::CollectionAdd(csg) => {
                let Some(View::CollectionAdd(mut state)) = self.view.take() else {
                    return Task::none();
                };

                match csg {
                    CollectionAddMessage::Toggle(selected, id) => {
                        if selected {
                            state.selected.remove(&id);
                        } else {
                            state.selected.insert(id);
                        }

                        self.view = Some(View::CollectionAdd(state));
                        Task::none()
                    }
                    CollectionAddMessage::Save => {
                        let mut new = state
                            .selected
                            .iter()
                            .filter_map(|collection| {
                                (!state.initial.contains(collection)).then_some((*collection, true))
                            })
                            .collect::<Vec<_>>();

                        let remove = state.initial.iter().filter_map(|init| {
                            let selected = state.selected.contains(init);
                            if !selected {
                                Some((*init, false))
                            } else {
                                None
                            }
                        });

                        new.extend(remove);

                        Task::done(Message::ToggleMembership {
                            item: state.item,
                            collections: new,
                        })
                    }
                }
            }
            HomeMessage::Rating(rsg) => {
                let Some(View::Rating { rating, id }) = self.view.as_mut() else {
                    return Task::none();
                };

                match rsg {
                    RatingMessage::Type => {
                        let Rating::Value(value) = &rating else {
                            return Task::none();
                        };
                        let id = widget::Id::unique();
                        *rating = Rating::Input {
                            id: id.clone(),
                            input: value.to_string(),
                        };

                        operation::focus(id)
                    }
                    RatingMessage::Star(val) => {
                        match rating {
                            Rating::Value(old) => {
                                *old = val as f32;
                            }
                            Rating::Input { input, .. } => {
                                *input = val.to_string();
                            }
                        }

                        let value = val as f32;
                        let msg = Message::MediaUpdate(MediaUpdate {
                            id: *id,
                            kind: MediaUpdateKind::Rating(value),
                        });

                        let close = self.close_view(true);

                        Task::batch([Task::done(msg), close])
                    }
                    RatingMessage::Submit => {
                        let Rating::Input { input, .. } = &rating else {
                            return Task::none();
                        };

                        let value = input.parse::<f32>().unwrap_or(0.0).clamp(0.0, 5.0);
                        *rating = Rating::Value(value);

                        let msg = Message::MediaUpdate(MediaUpdate {
                            id: *id,
                            kind: MediaUpdateKind::Rating(value),
                        });

                        let close = self.close_view(true);

                        Task::batch([Task::done(msg), close])
                    }
                    RatingMessage::Input(value) => {
                        if let Rating::Input { input, .. } = rating {
                            *input = value;
                        }

                        Task::none()
                    }
                }
            }
            HomeMessage::Rename(rsg) => {
                let Some(View::Rename {
                    id, value, empty, ..
                }) = self.view.as_mut()
                else {
                    return Task::none();
                };

                match rsg {
                    RenameMessage::Input(new) => {
                        *empty = new.is_empty();
                        *value = new;
                        Task::none()
                    }
                    RenameMessage::Submit => {
                        let msg = Message::MediaUpdate(MediaUpdate {
                            id: *id,
                            kind: MediaUpdateKind::Name(value.clone()),
                        });
                        let close = self.close_view(true);

                        Task::batch([Task::done(msg), close])
                    }
                }
            }
            HomeMessage::Synopsis(ssg) => {
                let Some(View::Synopsis { id, content, .. }) = self.view.as_mut() else {
                    return Task::none();
                };

                match ssg {
                    SynopsisMessage::Submit => {
                        let msg = Message::MediaUpdate(MediaUpdate {
                            id: *id,
                            kind: MediaUpdateKind::Synopsis(content.text()),
                        });

                        Task::batch([Task::done(msg), self.close_view(true)])
                    }
                    SynopsisMessage::Action(action) => {
                        content.perform(action);
                        Task::none()
                    }
                }
            }
            HomeMessage::TMDBId(tsg) => {
                let Some(View::TMDBId {
                    id,
                    value,
                    top_level,
                    ..
                }) = self.view.as_mut()
                else {
                    return Task::none();
                };

                match tsg {
                    TMDBMessage::Input(new) => {
                        *value = new;
                        Task::none()
                    }
                    TMDBMessage::Submit => {
                        let tmdb_id = match value.parse::<u32>() {
                            Ok(tmdb_id) => tmdb_id,
                            Err(error) => return Task::done(Message::error(error)),
                        };

                        if !(*top_level) && tmdb_id == 0 {
                            return Task::done(Message::error(
                                "Cannot have a 0 season/episode number",
                            ));
                        }

                        let msg = Message::MediaUpdate(MediaUpdate {
                            id: *id,
                            kind: MediaUpdateKind::TMDBId(tmdb_id),
                        });

                        Task::batch([Task::done(msg), self.close_view(true)])
                    }
                }
            }
            HomeMessage::Selection(ssg) => {
                let Some(View::Selection(mut selected)) = self.view.take() else {
                    return Task::none();
                };

                match ssg {
                    SelectionMessage::Select(item) => {
                        let new = !selected.contains(&item);

                        if new {
                            selected.push(item);
                        } else {
                            selected.retain(|selected| *selected != item);
                        }

                        match (item, &mut self.state) {
                            (ItemId::Movie(id), State::Recent { movies, .. })
                            | (ItemId::Movie(id), State::Movies(movies))
                            | (ItemId::Movie(id), State::Collection { movies, .. }) => {
                                if let Some(media) =
                                    movies.iter_mut().find(|item| item.media.id == id)
                                {
                                    media.selected = new;
                                }
                            }
                            (ItemId::Show(id), State::Recent { shows, .. })
                            | (ItemId::Show(id), State::Shows(shows))
                            | (ItemId::Show(id), State::Collection { shows, .. }) => {
                                if let Some(media) =
                                    shows.iter_mut().find(|item| item.media.id == id)
                                {
                                    media.selected = new;
                                }
                            }
                            (ItemId::Season(id), State::Show { seasons, .. })
                            | (ItemId::Season(id), State::Collection { seasons, .. }) => {
                                if let Some(media) =
                                    seasons.iter_mut().find(|item| item.media.id == id)
                                {
                                    media.selected = new;
                                }
                            }
                            (ItemId::Episode(id), State::Season { episodes, .. })
                            | (ItemId::Episode(id), State::Collection { episodes, .. }) => {
                                if let Some(media) =
                                    episodes.iter_mut().find(|item| item.media.id == id)
                                {
                                    media.selected = new;
                                }
                            }
                            _ => {}
                        }

                        self.view = Some(View::Selection(selected));

                        self.update_page_scroll()
                    }
                    SelectionMessage::Cancel => {
                        match &mut self.state {
                            State::Loading(_)
                            | State::Collections(_)
                            | State::Movie { .. }
                            | State::Episode { .. } => {}
                            State::Recent { movies, shows } => {
                                for media in movies {
                                    media.selected = false;
                                }
                                for media in shows {
                                    media.selected = false;
                                }
                            }
                            State::Movies(movies) => {
                                for media in movies {
                                    media.selected = false;
                                }
                            }
                            State::Shows(shows) => {
                                for media in shows {
                                    media.selected = false;
                                }
                            }
                            State::Show { seasons, .. } => {
                                for media in seasons {
                                    media.selected = false;
                                }
                            }
                            State::Season { episodes, .. } => {
                                for media in episodes {
                                    media.selected = false;
                                }
                            }
                            State::Collection {
                                movies,
                                shows,
                                seasons,
                                episodes,
                                ..
                            } => {
                                for media in movies {
                                    media.selected = false;
                                }
                                for media in shows {
                                    media.selected = false;
                                }
                                for media in seasons {
                                    media.selected = false;
                                }

                                for media in episodes {
                                    media.selected = false;
                                }
                            }
                        }

                        self.close_view(true)
                    }
                    SelectionMessage::Play => {
                        let play = selected.to_vec();
                        let play = Message::PlayItems(play).tasked();

                        Task::batch([play, self.close_view(true)])
                    }
                }
            }
            HomeMessage::Refetch(id) => {
                let msg = Message::MediaUpdate(MediaUpdate {
                    id,
                    kind: MediaUpdateKind::Refetch,
                });

                Task::done(msg)
            }
            HomeMessage::RemoveMedia => {
                let Some(View::RemoveMedia { id, .. }) = self.view.as_mut() else {
                    return Task::none();
                };

                let msg = Message::MediaUpdate(MediaUpdate {
                    id: *id,
                    kind: MediaUpdateKind::Remove,
                });

                let remove = Task::done(msg);

                Task::batch([self.back(now, true), remove, self.close_view(true)])
            }
            HomeMessage::Trigger(tsg) => {
                let Some(View::CollectionTriggers {
                    id,
                    view_inserts,
                    itriggers,
                    dtriggers,
                    removed_inserts,
                    removed_deletes,
                }) = self.view.as_mut()
                else {
                    return Task::none();
                };

                match tsg {
                    TriggerMessage::Tab => {
                        *view_inserts = !*view_inserts;
                        Task::none()
                    }
                    TriggerMessage::AddInsert => {
                        let new = InsertTrigger::new(
                            *id,
                            "Insert Rule",
                            Logic::default(),
                            triggers::Media::Movies,
                        );
                        itriggers.push((true, new, false, String::default(), String::default()));

                        Task::none()
                    }
                    TriggerMessage::AddDelete => {
                        let new = DeleteTrigger::new(
                            *id,
                            "Delete Rule",
                            Logic::default(),
                            triggers::Media::Movies,
                        );
                        dtriggers.push((true, new, false, String::default(), String::default()));

                        Task::none()
                    }
                    TriggerMessage::NameInsert(id, name) => {
                        if let Some((_, trigger, _, _, _)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            trigger.name = name;
                        }

                        Task::none()
                    }
                    TriggerMessage::NameDelete(id, name) => {
                        if let Some((_, trigger, _, _, _)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            trigger.name = name;
                        }

                        Task::none()
                    }
                    TriggerMessage::MediaInsert(id, media) => {
                        if let Some((_, trigger, _, _, _)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            trigger.media = media;
                        }

                        Task::none()
                    }
                    TriggerMessage::MediaDelete(id, media) => {
                        if let Some((_, trigger, _, _, _)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            trigger.media = media;
                        }

                        Task::none()
                    }
                    TriggerMessage::ROEInsert(id, checked) => {
                        if let Some((_, _, roe, _, _)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *roe = checked;
                        }

                        Task::none()
                    }
                    TriggerMessage::ROEDelete(id, checked) => {
                        if let Some((_, _, roe, _, _)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *roe = checked;
                        }

                        Task::none()
                    }
                    TriggerMessage::ToggleROEInsert(id) => {
                        if let Some((_, _, roe, _, _)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *roe = !*roe;
                        }

                        Task::none()
                    }
                    TriggerMessage::ToggleROEDelete(id) => {
                        if let Some((_, _, roe, _, _)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *roe = !*roe;
                        }

                        Task::none()
                    }
                    TriggerMessage::DuplicateInsert(id) => {
                        if let Some((idx, (open, trigger, roe, last, release))) = itriggers
                            .iter()
                            .enumerate()
                            .find(|(_, (_, trigger, _, _, _))| trigger.id == id)
                        {
                            let name = format!("{} - Copy", trigger.name);
                            let copy = InsertTrigger::new(
                                trigger.collection,
                                name,
                                trigger.logic.clone(),
                                trigger.media,
                            );

                            itriggers.insert(
                                idx + 1,
                                (*open, copy, *roe, last.clone(), release.clone()),
                            );
                        }

                        Task::none()
                    }
                    TriggerMessage::DuplicateDelete(id) => {
                        if let Some((idx, (open, trigger, roe, last, release))) = dtriggers
                            .iter()
                            .enumerate()
                            .find(|(_, (_, trigger, _, _, _))| trigger.id == id)
                        {
                            let name = format!("{} - Copy", trigger.name);
                            let copy = DeleteTrigger::new(
                                trigger.collection,
                                name,
                                trigger.logic.clone(),
                                trigger.media,
                            );

                            dtriggers.insert(
                                idx + 1,
                                (*open, copy, *roe, last.clone(), release.clone()),
                            );
                        }

                        Task::none()
                    }
                    TriggerMessage::RemoveInsert(id) => {
                        if let Some((idx, _)) = itriggers
                            .iter()
                            .enumerate()
                            .find(|(_, (_, trigger, _, _, _))| trigger.id == id)
                        {
                            let (_, trigger, _, _, _) = itriggers.remove(idx);
                            removed_inserts.push(trigger);
                        }

                        Task::none()
                    }
                    TriggerMessage::RemoveDelete(id) => {
                        if let Some((idx, _)) = dtriggers
                            .iter()
                            .enumerate()
                            .find(|(_, (_, trigger, _, _, _))| trigger.id == id)
                        {
                            let (_, trigger, _, _, _) = dtriggers.remove(idx);
                            removed_deletes.push(trigger);
                        }

                        Task::none()
                    }
                    TriggerMessage::ToggleExpandInsert(id, toggle) => {
                        if let Some((open, _, _, _, _)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *open = toggle;
                        }

                        Task::none()
                    }
                    TriggerMessage::ToggleExpandDelete(id, toggle) => {
                        if let Some((open, _, _, _, _)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        {
                            *open = toggle;
                        }

                        Task::none()
                    }
                    TriggerMessage::LogicInsert(id, lsg) => {
                        let Some((_, trigger, _, last, release)) = itriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        else {
                            return Task::none();
                        };

                        match lsg {
                            LogicMessage::Name(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.name = None;
                                    return Task::none();
                                }

                                match trigger.logic.name.take() {
                                    Some((not, _)) => {
                                        trigger.logic.name = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.name = Some((false, pattern));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::NameComp(new) => {
                                if let Some((not, _)) = trigger.logic.name.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Synopsis(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.synopsis = None;
                                    return Task::none();
                                }

                                match trigger.logic.synopsis.take() {
                                    Some((not, _)) => {
                                        trigger.logic.synopsis = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.synopsis = Some((false, pattern));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::SynopsisComp(new) => {
                                if let Some((not, _)) = trigger.logic.synopsis.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Tags(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.tags = None;
                                    return Task::none();
                                }

                                match trigger.logic.tags.take() {
                                    Some((not, _)) => {
                                        trigger.logic.tags = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.tags = Some((false, pattern));
                                    }
                                };

                                if matches!(
                                    trigger.media,
                                    triggers::Media::Episodes | triggers::Media::Seasons
                                ) {
                                    trigger.media = triggers::Media::Shows;
                                }

                                Task::none()
                            }
                            LogicMessage::TagsComp(new) => {
                                if let Some((not, _)) = trigger.logic.tags.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Dir(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.dir = None;
                                    return Task::none();
                                }

                                match trigger.logic.dir.take() {
                                    Some((not, _)) => {
                                        trigger.logic.dir = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.dir = Some((false, pattern));
                                    }
                                };

                                if matches!(
                                    trigger.media,
                                    triggers::Media::Episodes | triggers::Media::Seasons
                                ) {
                                    trigger.media = triggers::Media::Shows;
                                }

                                Task::none()
                            }
                            LogicMessage::DirComp(new) => {
                                if let Some((not, _)) = trigger.logic.dir.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::LastComp(new) => {
                                if let Some((comp, _)) = trigger.logic.last_watched.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Last(new) => {
                                *last = new.clone();
                                if new.is_empty() {
                                    trigger.logic.last_watched = None;
                                    last.clear();
                                    return Task::none();
                                }

                                let new = new.trim();

                                match humantime::parse_duration(new) {
                                    Ok(duration) => {
                                        if let Some(last_watched) = chrono::TimeDelta::from_std(
                                            duration,
                                        )
                                        .ok()
                                        .and_then(|duration| {
                                            chrono::Local::now().checked_sub_signed(duration)
                                        }) {
                                            *last = last_watched.format("%F %R").to_string();
                                            trigger.logic.last_watched =
                                                Some((Comparison::default(), last_watched))
                                        };

                                        Task::none()
                                    }
                                    Err(_) => {
                                        let Ok(last_watched) =
                                            chrono::NaiveDateTime::parse_from_str(new, "%F %R")
                                                .or_else(|_| {
                                                    chrono::NaiveDateTime::parse_from_str(new, "%F")
                                                })
                                        else {
                                            return Message::warn(format!("Invalid input: {new}"))
                                                .tasked();
                                        };

                                        let last_watched = last_watched.and_utc();

                                        *last = last_watched.format("%F %R").to_string();
                                        trigger.logic.last_watched =
                                            Some((Comparison::Equal, last_watched.into()));

                                        Task::none()
                                    }
                                }
                            }
                            LogicMessage::Duration(new) => {
                                if new.is_empty() {
                                    trigger.logic.duration = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u64>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.duration.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.duration = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.duration = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::DurationComp(new) => {
                                if let Some((comp, _)) = trigger.logic.duration.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Progress(new) => {
                                if new.is_empty() {
                                    trigger.logic.progress = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<f32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                let new = new.min(1.0);

                                match trigger.logic.progress.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.progress = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.progress = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::ProgressComp(new) => {
                                if let Some((comp, _)) = trigger.logic.progress.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Watch(new) => {
                                if new.is_empty() {
                                    trigger.logic.watch_count = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.watch_count.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.watch_count = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.watch_count =
                                            Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::WatchComp(new) => {
                                if let Some((comp, _)) = trigger.logic.watch_count.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Release(new) => {
                                *release = new.clone();
                                if new.is_empty() {
                                    trigger.logic.release = None;
                                    release.clear();
                                    return Task::none();
                                }

                                let new = new.trim();

                                match humantime::parse_duration(new) {
                                    Ok(duration) => {
                                        if let Some(new) = chrono::TimeDelta::from_std(duration)
                                            .ok()
                                            .and_then(|duration| {
                                                chrono::Local::now().checked_sub_signed(duration)
                                            })
                                        {
                                            *release = new.format("%F").to_string();
                                            trigger.logic.release =
                                                Some((Comparison::default(), new.date_naive()))
                                        };

                                        Task::none()
                                    }
                                    Err(_) => {
                                        let Ok(new) = chrono::DateTime::parse_from_str(new, "%F")
                                        else {
                                            return Message::warn(format!("Invalid input: {new}"))
                                                .tasked();
                                        };

                                        *release = new.format("%F").to_string();
                                        trigger.logic.release =
                                            Some((Comparison::Equal, new.date_naive()));

                                        Task::none()
                                    }
                                }
                            }
                            LogicMessage::ReleaseComp(new) => {
                                if let Some((comp, _)) = trigger.logic.release.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Rating(new) => {
                                if new.is_empty() {
                                    trigger.logic.rating = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<f32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                let new = new.clamp(0.0, 5.0);

                                match trigger.logic.rating.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.rating = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.rating = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::RatingComp(new) => {
                                if let Some((comp, _)) = trigger.logic.rating.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Comment(new) => {
                                if new.is_empty() {
                                    trigger.logic.comment = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.comment.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.comment = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.comment = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::CommentComp(new) => {
                                if let Some((comp, _)) = trigger.logic.comment.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                        }
                    }
                    TriggerMessage::LogicDelete(id, lsg) => {
                        let Some((_, trigger, _, last, release)) = dtriggers
                            .iter_mut()
                            .find(|(_, trigger, _, _, _)| trigger.id == id)
                        else {
                            return Task::none();
                        };

                        match lsg {
                            LogicMessage::Name(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.name = None;
                                    return Task::none();
                                }

                                match trigger.logic.name.take() {
                                    Some((not, _)) => {
                                        trigger.logic.name = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.name = Some((false, pattern));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::NameComp(new) => {
                                if let Some((not, _)) = trigger.logic.name.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Synopsis(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.synopsis = None;
                                    return Task::none();
                                }

                                match trigger.logic.synopsis.take() {
                                    Some((not, _)) => {
                                        trigger.logic.synopsis = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.synopsis = Some((false, pattern));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::SynopsisComp(new) => {
                                if let Some((not, _)) = trigger.logic.synopsis.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Tags(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.tags = None;
                                    return Task::none();
                                }

                                match trigger.logic.tags.take() {
                                    Some((not, _)) => {
                                        trigger.logic.tags = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.tags = Some((false, pattern));
                                    }
                                };

                                if matches!(
                                    trigger.media,
                                    triggers::Media::Episodes | triggers::Media::Seasons
                                ) {
                                    trigger.media = triggers::Media::Shows;
                                }

                                Task::none()
                            }
                            LogicMessage::TagsComp(new) => {
                                if let Some((not, _)) = trigger.logic.tags.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Dir(pattern) => {
                                if pattern.is_empty() {
                                    trigger.logic.dir = None;
                                    return Task::none();
                                }

                                match trigger.logic.dir.take() {
                                    Some((not, _)) => {
                                        trigger.logic.dir = Some((not, pattern));
                                    }
                                    None => {
                                        trigger.logic.dir = Some((false, pattern));
                                    }
                                };

                                if matches!(
                                    trigger.media,
                                    triggers::Media::Episodes | triggers::Media::Seasons
                                ) {
                                    trigger.media = triggers::Media::Shows;
                                }

                                Task::none()
                            }
                            LogicMessage::DirComp(new) => {
                                if let Some((not, _)) = trigger.logic.dir.as_mut() {
                                    *not = new;
                                }

                                Task::none()
                            }
                            LogicMessage::LastComp(new) => {
                                if let Some((comp, _)) = trigger.logic.last_watched.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Last(new) => {
                                *last = new.clone();
                                if new.is_empty() {
                                    trigger.logic.last_watched = None;
                                    last.clear();
                                    return Task::none();
                                }

                                let new = new.trim();

                                match humantime::parse_duration(new) {
                                    Ok(duration) => {
                                        if let Some(last_watched) = chrono::TimeDelta::from_std(
                                            duration,
                                        )
                                        .ok()
                                        .and_then(|duration| {
                                            chrono::Local::now().checked_sub_signed(duration)
                                        }) {
                                            *last = last_watched.format("%F %R").to_string();
                                            trigger.logic.last_watched =
                                                Some((Comparison::default(), last_watched))
                                        };

                                        Task::none()
                                    }
                                    Err(_) => {
                                        let Ok(last_watched) =
                                            chrono::NaiveDateTime::parse_from_str(new, "%F %R")
                                                .or_else(|_| {
                                                    chrono::NaiveDateTime::parse_from_str(new, "%F")
                                                })
                                        else {
                                            return Message::warn(format!("Invalid input: {new}"))
                                                .tasked();
                                        };

                                        let last_watched = last_watched.and_utc();
                                        *last = last_watched.format("%F %R").to_string();
                                        trigger.logic.last_watched =
                                            Some((Comparison::Equal, last_watched.into()));

                                        Task::none()
                                    }
                                }
                            }
                            LogicMessage::Duration(new) => {
                                if new.is_empty() {
                                    trigger.logic.duration = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u64>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.duration.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.duration = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.duration = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::DurationComp(new) => {
                                if let Some((comp, _)) = trigger.logic.duration.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Progress(new) => {
                                if new.is_empty() {
                                    trigger.logic.progress = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<f32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                let new = new.min(1.0);

                                match trigger.logic.progress.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.progress = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.progress = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::ProgressComp(new) => {
                                if let Some((comp, _)) = trigger.logic.progress.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Watch(new) => {
                                if new.is_empty() {
                                    trigger.logic.watch_count = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.watch_count.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.watch_count = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.watch_count =
                                            Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::WatchComp(new) => {
                                if let Some((comp, _)) = trigger.logic.watch_count.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Release(new) => {
                                *release = new.clone();
                                if new.is_empty() {
                                    trigger.logic.release = None;
                                    release.clear();
                                    return Task::none();
                                }

                                let new = new.trim();

                                match humantime::parse_duration(new) {
                                    Ok(duration) => {
                                        if let Some(new) = chrono::TimeDelta::from_std(duration)
                                            .ok()
                                            .and_then(|duration| {
                                                chrono::Local::now().checked_sub_signed(duration)
                                            })
                                        {
                                            *release = new.format("%F").to_string();
                                            trigger.logic.release =
                                                Some((Comparison::default(), new.date_naive()))
                                        };

                                        Task::none()
                                    }
                                    Err(_) => {
                                        let Ok(new) = chrono::DateTime::parse_from_str(new, "%F")
                                        else {
                                            return Message::warn(format!("Invalid input: {new}"))
                                                .tasked();
                                        };

                                        *release = new.format("%F").to_string();
                                        trigger.logic.release =
                                            Some((Comparison::Equal, new.date_naive()));

                                        Task::none()
                                    }
                                }
                            }
                            LogicMessage::ReleaseComp(new) => {
                                if let Some((comp, _)) = trigger.logic.release.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Rating(new) => {
                                if new.is_empty() {
                                    trigger.logic.rating = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<f32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                let new = new.clamp(0.0, 5.0);

                                match trigger.logic.rating.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.rating = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.rating = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::RatingComp(new) => {
                                if let Some((comp, _)) = trigger.logic.rating.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                            LogicMessage::Comment(new) => {
                                if new.is_empty() {
                                    trigger.logic.comment = None;
                                    return Task::none();
                                }

                                let new = new.trim();

                                let Ok(new) = new.parse::<u32>() else {
                                    let msg = Message::error(format!("Invalid input: {new}"));
                                    return Task::done(msg);
                                };

                                match trigger.logic.comment.take() {
                                    Some((comp, _)) => {
                                        trigger.logic.comment = Some((comp, new));
                                    }
                                    None => {
                                        trigger.logic.comment = Some((Comparison::default(), new));
                                    }
                                };

                                Task::none()
                            }
                            LogicMessage::CommentComp(new) => {
                                if let Some((comp, _)) = trigger.logic.comment.as_mut() {
                                    *comp = new;
                                }

                                Task::none()
                            }
                        }
                    }
                    TriggerMessage::GenerateDelete(id) => {
                        let Some((idx, (_, trigger, roe, last, release))) = itriggers
                            .iter()
                            .enumerate()
                            .find(|(_, (_, trigger, _, _, _))| trigger.id == id)
                        else {
                            return Task::none();
                        };

                        let name = format!("{} - Delete Rule", trigger.name);
                        let logic = trigger.logic.not();

                        let delete =
                            DeleteTrigger::new(trigger.collection, name, logic, trigger.media);

                        let idx = idx.min(dtriggers.len());

                        dtriggers.insert(idx, (true, delete, *roe, last.clone(), release.clone()));
                        *view_inserts = false;

                        Task::none()
                    }
                    TriggerMessage::Save => {
                        let inserts = itriggers
                            .iter()
                            .map(|(_, trigger, roe, _, _)| (trigger.clone(), *roe))
                            .collect();

                        let deletes = dtriggers
                            .iter()
                            .map(|(_, trigger, roe, _, _)| (trigger.clone(), *roe))
                            .collect();

                        let msg = Message::Triggers {
                            inserts,
                            deletes,
                            removed_inserts: removed_inserts.clone(),
                            removed_deletes: removed_deletes.clone(),
                        }
                        .tasked();

                        let close_view = self.close_view(true);

                        Task::batch([close_view, msg])
                    }
                }
            }
            HomeMessage::CloseView => self.close_view(true),
            HomeMessage::Back => self.back(now, false),
            HomeMessage::Forward => self.forward(now),
            HomeMessage::ToggleLayout => self.layout_toggle(),
            HomeMessage::Sort(ssg) => {
                match ssg {
                    SortMessage::AddSort(kind) => self.sort.push(kind),
                    SortMessage::RemoveSort(kind) => self.sort.remove(kind),
                    SortMessage::ReverseSort(kind) => self.sort.reverse_kind(kind),
                    SortMessage::Clear => self.sort.clear(),
                    SortMessage::ToggleReverse => self.sort.reverse(),
                }

                self.content_refresh(now)
            }
            HomeMessage::ToggleSort => {
                self.show_sorts = !self.show_sorts;
                Task::none()
            }
            HomeMessage::ToggleFilter => {
                self.show_filters = !self.show_filters;
                Task::none()
            }
            HomeMessage::Filter(fsg) => {
                match fsg {
                    FilterMessage::Mode => self.filters.mode.toggle(),
                    FilterMessage::ProgressKind(kind) => {
                        self.filters.progress.kind = kind;
                    }
                    FilterMessage::ProgressComp => {
                        self.filters.progress.comp.toggle();
                    }
                    FilterMessage::RatingKind(kind) => {
                        self.filters.rating.kind = kind;
                    }
                    FilterMessage::RatingComp => {
                        self.filters.rating.comp.toggle();
                    }
                    FilterMessage::CommentsNum(number) => {
                        let number = number.trim();
                        if number.is_empty() {
                            self.filters.comments = None;
                            return self.content_refresh(now);
                        }

                        let Ok(number) = number.parse::<u32>() else {
                            let msg = Message::error(format!("Invalid input: {number}"));
                            return Task::done(msg);
                        };

                        match self.filters.comments.as_mut() {
                            Some(comments) => {
                                comments.number = number;
                            }
                            None => {
                                self.filters.comments = Some(filter::Comments {
                                    number,
                                    comp: filter::Comp::default(),
                                })
                            }
                        }
                    }
                    FilterMessage::CommentsComp => {
                        if let Some(comments) = self.filters.comments.as_mut() {
                            comments.comp.toggle();
                        }
                    }
                    FilterMessage::DurationMinutes(minutes) => {
                        let minutes = minutes.trim();

                        if minutes.is_empty() {
                            if let Some(duration) = self.filters.duration.as_mut() {
                                duration.secs = (duration.secs / 3600) * 3600;
                                if duration.secs == 0 {
                                    self.filters.duration = None;
                                }
                            }

                            return self.content_refresh(now);
                        }

                        let Ok(minutes) = minutes.parse::<u64>() else {
                            let msg = Message::error(format!("Invalid input: {minutes}"));
                            return Task::done(msg);
                        };

                        let secs = minutes * 60;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let hours = (duration.secs / 3600) * 3600;

                                duration.secs = hours + secs;
                            }
                            None => {
                                self.filters.duration = Some(filter::Duration {
                                    secs,
                                    comp: filter::Comp::default(),
                                });
                            }
                        }
                    }
                    FilterMessage::DurationHours(hours) => {
                        let hours = hours.trim();

                        if hours.is_empty() {
                            if let Some(duration) = self.filters.duration.as_mut() {
                                duration.secs %= 3600;
                                if duration.secs == 0 {
                                    self.filters.duration = None;
                                }
                            }

                            return self.content_refresh(now);
                        }

                        let Ok(hours) = hours.parse::<u64>() else {
                            let msg = Message::error(format!("Invalid input: {hours}"));
                            return Task::done(msg);
                        };

                        let secs = hours * 3600;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let minutes = duration.secs % 3600;
                                duration.secs = secs + minutes;
                            }
                            None => {
                                self.filters.duration = Some(filter::Duration {
                                    secs,
                                    comp: filter::Comp::default(),
                                });
                            }
                        }
                    }
                    FilterMessage::DurationComp => {
                        if let Some(duration) = self.filters.duration.as_mut() {
                            duration.comp.toggle();
                        }
                    }
                    FilterMessage::ReleaseYear(year) => {
                        let year = year.trim();

                        if year.is_empty() {
                            self.filters.release = None;
                            return self.content_refresh(now);
                        }

                        let Ok(year) = year.parse::<i32>() else {
                            let msg = Message::error(format!("Invalid input: {year}"));
                            return Task::done(msg);
                        };

                        match self.filters.release.as_mut() {
                            Some(release) => release.year = year,
                            None => {
                                self.filters.release = Some(filter::Release {
                                    year,
                                    comp: filter::Comp::default(),
                                })
                            }
                        }
                    }
                    FilterMessage::ReleaseComp => {
                        if let Some(release) = self.filters.release.as_mut() {
                            release.comp.toggle();
                        }
                    }
                    FilterMessage::Clear => {
                        self.filters.clear();
                    }
                }

                self.content_refresh(now)
            }
            HomeMessage::NewCollection => {
                let (config, name_input) = CollectionConfig::new();

                self.view = Some(View::CollectionConfig(config));
                let focus = operation::focus(name_input);

                Task::batch([focus, self.update_page_scroll()])
            }
            HomeMessage::Random => Task::done(Message::Random),
            HomeMessage::RefreshContent => self.content_refresh(now),
            HomeMessage::Play(item) => {
                if matches!(self.view, Some(View::Selection(_))) {
                    Message::Home(HomeMessage::Selection(SelectionMessage::Select(item))).tasked()
                } else if self.command {
                    let selection =
                        Message::Home(HomeMessage::OpenView(ViewMessage::Selection)).tasked();

                    let selected =
                        Message::Home(HomeMessage::Selection(SelectionMessage::Select(item)))
                            .tasked();

                    selection.chain(selected)
                } else {
                    let close_view = self.close_view(true);
                    Task::batch([Task::done(Message::PlayItem(item)), close_view])
                }
            }
            HomeMessage::PlayCollection { id, items } => {
                Task::done(Message::PlayCollectionItems { id, items })
            }
            HomeMessage::Hovered(id, is_hovered) => {
                let is_hovered = is_hovered && !matches!(self.view, Some(View::Selection(_)));

                match (&mut self.state, id) {
                    (State::Loading(_), _)
                    | (State::Episode { .. }, _)
                    | (State::Movie { .. }, _)
                    | (State::Collections(_), _) => Task::none(),
                    (State::Recent { shows, .. }, ItemId::Show(id)) => {
                        if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                            show.go_mut(is_hovered, now);
                        };
                        self.focused = Some(ItemId::Show(id));
                        Task::none()
                    }
                    (State::Recent { movies, .. }, ItemId::Movie(id)) => {
                        if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                            movie.go_mut(is_hovered, now);
                        }

                        self.focused = Some(ItemId::Movie(id));
                        Task::none()
                    }
                    (State::Recent { .. }, _) => Task::none(),
                    (State::Shows(shows), ItemId::Show(id)) => {
                        if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                            show.go_mut(is_hovered, now);
                        };

                        self.focused = Some(ItemId::Show(id));
                        Task::none()
                    }
                    (State::Shows(_), _) => Task::none(),
                    (State::Movies(movies), ItemId::Movie(id)) => {
                        if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                            movie.go_mut(is_hovered, now);
                        }

                        self.focused = Some(ItemId::Movie(id));
                        Task::none()
                    }
                    (State::Movies(_), _) => Task::none(),
                    (State::Show { seasons, .. }, ItemId::Season(id)) => {
                        if let Some(season) =
                            seasons.iter_mut().find(|season| season.media.id == id)
                        {
                            season.go_mut(is_hovered, now);
                        }
                        self.focused = Some(ItemId::Season(id));
                        Task::none()
                    }
                    (State::Show { .. }, _) => Task::none(),
                    (State::Season { episodes, .. }, ItemId::Episode(id)) => {
                        if let Some(episode) =
                            episodes.iter_mut().find(|episode| episode.media.id == id)
                        {
                            episode.go_mut(is_hovered, now);
                        }

                        self.focused = Some(ItemId::Episode(id));
                        Task::none()
                    }
                    (State::Season { .. }, _) => Task::none(),
                    (State::Collection { shows, .. }, ItemId::Show(id)) => {
                        if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                            show.go_mut(is_hovered, now);
                        };

                        self.focused = Some(ItemId::Show(id));
                        Task::none()
                    }
                    (State::Collection { movies, .. }, ItemId::Movie(id)) => {
                        if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                            movie.go_mut(is_hovered, now);
                        }

                        self.focused = Some(ItemId::Movie(id));
                        Task::none()
                    }
                    (State::Collection { seasons, .. }, ItemId::Season(id)) => {
                        if let Some(season) = seasons.iter_mut().find(|show| show.media.id == id) {
                            season.go_mut(is_hovered, now);
                        };

                        self.focused = Some(ItemId::Season(id));
                        Task::none()
                    }
                    (State::Collection { episodes, .. }, ItemId::Episode(id)) => {
                        if let Some(episode) =
                            episodes.iter_mut().find(|episode| episode.media.id == id)
                        {
                            episode.go_mut(is_hovered, now);
                        }

                        self.focused = Some(ItemId::Episode(id));
                        Task::none()
                    }
                }
            }
            HomeMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                Task::none()
            }
            HomeMessage::AddCollection(item, collection) => {
                self.view = None;
                Task::done(Message::ToggleMembership {
                    item,
                    collections: vec![(collection, true)],
                })
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let searching = match &self.view {
            Some(View::Search(state, _)) => state.last_edit.is_some(),
            _ => false,
        };
        if searching {
            window::frames()
                .map(|_| Message::Home(HomeMessage::SearchMessage(SearchMessage::Searching)))
        } else {
            Subscription::none()
        }
    }

    pub fn layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    pub fn recents_limit(&mut self, recent_limit: Option<i32>) {
        self.recent_limit = recent_limit;
    }

    fn update_scroll(&mut self) -> Task<()> {
        scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    pub fn update_page_scroll(&mut self) -> Task<Message> {
        match self.current_page_mut() {
            None | Some(Page::Home) => self.update_scroll().discard(),
            Some(page) => page.update_scroll().discard(),
        }
    }

    fn current_page(&self) -> Option<&Page> {
        self.current_page
            .as_ref()
            .and_then(|kind| self.pages.get(kind))
    }

    fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.current_page
            .as_ref()
            .and_then(|kind| self.pages.get_mut(kind))
    }

    fn side(&self) -> Element<'_, HomeMessage> {
        let header = {
            let color = |theme: &Theme| {
                let color = theme
                    .extended_palette()
                    .primary
                    .base
                    .color
                    .scale_alpha(0.85);

                text::Style { color: Some(color) }
            };

            // let icon = icons::icon(icons::LOGO).size(H3).style(color);
            let text = display("kino").style(color);

            container(
                row!(text)
                    .padding([5, 10])
                    .align_y(Vertical::Center)
                    .spacing(12.0),
            )
            // .center_x(Length::Fill)
        };

        let collections = self
            .collections
            .iter()
            .filter_map(|collection| match collection.view {
                CollectionView::Pinned => {
                    let unicode = Icon::new(collection.icon).unicode();
                    let content = icon_button(
                        unicode,
                        &collection.name,
                        HomeMessage::Goto(PageKind::Collection(collection.id)),
                        self.current_page()
                            .map(|page| page.is_collection(&collection.id))
                            .unwrap_or_default(),
                        Some(view_unicode(collection.view)),
                    );

                    Some(content)
                }
                CollectionView::Shown => {
                    let unicode = Icon::new(collection.icon).unicode();
                    let content = icon_button(
                        unicode,
                        &collection.name,
                        HomeMessage::Goto(PageKind::Collection(collection.id)),
                        self.current_page()
                            .map(|page| page.is_collection(&collection.id))
                            .unwrap_or_default(),
                        None,
                    );

                    Some(content)
                }
                CollectionView::Hidden => None,
            });

        let collections = column!(
            icon_button(
                icons::HOME,
                "Home",
                HomeMessage::Goto(PageKind::Home),
                self.current_page().map(Page::is_home).unwrap_or_default(),
                None,
            ),
            icon_button(
                icons::SHOW,
                "Shows",
                HomeMessage::Goto(Page::goto_shows()),
                self.current_page().map(Page::is_shows).unwrap_or_default(),
                None,
            ),
            icon_button(
                icons::MOVIE,
                "Movies",
                HomeMessage::Goto(Page::goto_movies()),
                self.current_page().map(Page::is_movies).unwrap_or_default(),
                None,
            ),
        )
        .extend(collections)
        .push(icon_button(
            icons::ADD,
            "New collection",
            HomeMessage::NewCollection,
            false,
            None,
        ))
        .spacing(16.0)
        .width(Length::Fill);
        let collections = scrollable(collections)
            .width(Length::Fill)
            .height(Length::Fill);

        let bottom = column!(
            icon_button(
                icons::LIBRARY,
                "Collections",
                HomeMessage::Goto(PageKind::Collections),
                self.current_page()
                    .map(Page::is_collections)
                    .unwrap_or_default(),
                None
            ),
            // icon_button(
            //     icons::COMMENT,
            //     "Comments",
            //     HomeMessage::Goto(Page::goto_comments()),
            //     self.current_page()
            //         .map(Page::is_comments)
            //         .unwrap_or_default(),
            //         None
            // ),
            icon_button(
                icons::SETTINGS,
                "Settings",
                HomeMessage::Settings,
                false,
                None
            )
        )
        .spacing(12.0);

        let content = column!(collections, rule::horizontal(1.0), bottom,)
            .spacing(4.0)
            .padding([0, 5])
            .height(Length::Fill);

        let content = column!(header, space::vertical().height(24.0), content,)
            .width(300.0)
            .height(Length::Fill);

        let content = container(content).style(styles::container::bw3);

        content.into()
    }

    fn recents<'a>(
        &self,
        now: Instant,
        movies: &'a [Thumbnail<Movie>],
        shows: &'a [Thumbnail<Show>],
    ) -> Element<'a, HomeMessage> {
        let movies = {
            let label = h6("Recent Movies");
            let label = column!(label, rule::horizontal(1.0)).spacing(4.0);

            let movies = movies.iter();

            let movies: Element<'_, HomeMessage> = match self.layout {
                Layout::Grid => {
                    let content = movies.map(|thumbnail| {
                        thumbnail.card(
                            now,
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(id))),
                            |id| HomeMessage::Goto(PageKind::Movie(id)),
                            |id, hovered| HomeMessage::Hovered(ItemId::Movie(id), hovered),
                            |id| HomeMessage::Play(ItemId::Movie(id)),
                        )
                    });

                    grid(content)
                        .spacing(16)
                        .fluid(CARD_WIDTH)
                        .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT))
                        .into()
                }
                Layout::List => {
                    let content = movies.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(id))),
                            |id| HomeMessage::Goto(PageKind::Movie(id)),
                            |id, hovered| HomeMessage::Hovered(ItemId::Movie(id), hovered),
                            |id| HomeMessage::Play(ItemId::Movie(id)),
                            movies::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                }
                Layout::Compact => {
                    let content = movies.map(|thumbnail| {
                        thumbnail.compact(
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(id))),
                            |id| HomeMessage::Goto(PageKind::Movie(id)),
                            |id| HomeMessage::Play(ItemId::Movie(id)),
                        )
                    });

                    column(content).spacing(16.0).into()
                }
            };

            column!(label, movies).spacing(10.0)
        };

        let shows = {
            let label = h6("Recent Shows");
            let label = column!(label, rule::horizontal(1.0)).spacing(4.0);

            let shows = shows.iter();

            let shows: Element<'_, HomeMessage> = match self.layout {
                Layout::Grid => {
                    let shows = shows.map(|show| {
                        show.card(
                            now,
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(id))),
                            |id| HomeMessage::Goto(PageKind::Show(id)),
                            |id, hovered| HomeMessage::Hovered(ItemId::Show(id), hovered),
                            |id| HomeMessage::Play(ItemId::Show(id)),
                        )
                    });

                    grid(shows)
                        .spacing(16)
                        .fluid(CARD_WIDTH)
                        .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT))
                        .into()
                }
                Layout::List => {
                    let content = shows.map(|thumbnail| {
                        thumbnail.list(
                            now,
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(id))),
                            |id| HomeMessage::Goto(PageKind::Show(id)),
                            |id, hovered| HomeMessage::Hovered(ItemId::Show(id), hovered),
                            |id| HomeMessage::Play(ItemId::Show(id)),
                            shows::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                }
                Layout::Compact => {
                    let content = shows.map(|thumbnail| {
                        thumbnail.compact(
                            |id| HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(id))),
                            |id| HomeMessage::Goto(PageKind::Show(id)),
                            |id| HomeMessage::Play(ItemId::Show(id)),
                        )
                    });

                    column(content).spacing(16.0).into()
                }
            };

            column!(label, shows).spacing(10.0)
        };

        let content = if matches!(self.layout, Layout::Grid) {
            scrollable(
                column!(movies, shows)
                    .height(Length::Fill)
                    .spacing(40.0)
                    .padding(iced::Padding::new(10.0).right(16)),
            )
            .auto_scroll(true)
            .id(self.scroll.id.clone())
            .on_scroll(HomeMessage::Scroll)
        } else {
            scrollable(
                column!(movies, shows)
                    .spacing(40.0)
                    .padding(Padding::new(10.0).bottom(0)),
            )
            .auto_scroll(true)
            .spacing(0.5)
            .id(self.scroll.id.clone())
            .on_scroll(HomeMessage::Scroll)
        };

        content.into()
    }

    fn navigation(&self) -> Element<'_, HomeMessage> {
        let can_back = !self.backward.is_empty();

        let can_forward = !self.forward.is_empty();

        let navigation = row!(
            icons::text_button(icons::BACK).on_press_maybe(can_back.then_some(HomeMessage::Back)),
            icons::text_button(icons::FORWARD)
                .on_press_maybe(can_forward.then_some(HomeMessage::Forward))
        )
        .spacing(5.0);

        navigation.into()
    }

    fn filters_view(&self) -> Element<'_, HomeMessage> {
        let size = H8;
        let padding = Padding::new(2.0).horizontal(5.0);
        let spacing = 2.0;
        let picklist_font = regular_font();
        let input_font = mono_font();

        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);
        let comp = |icon: char, msg: FilterMessage| {
            icons::sized_button(icon, size * RATIO)
                .padding([5, 5])
                .style(styles::button::subtlest)
                .on_press(HomeMessage::Filter(msg))
        };

        let progress = {
            let text = sized_medium("Progress", H8);
            let progress = pick_list(
                Some(self.filters.progress.kind),
                filter::ProgressKind::ALL,
                ToString::to_string,
            )
            .padding(padding)
            .on_select(|selected| HomeMessage::Filter(FilterMessage::ProgressKind(selected)))
            .width(60.0)
            .handle(picklist_handle(size))
            .font(picklist_font)
            .text_size(size);

            let comp = comp(
                comp_icon(self.filters.progress.comp),
                FilterMessage::ProgressComp,
            );

            row!(text, comp, progress)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let rating = {
            let text = sized_medium("Rating", H8);
            let rating = pick_list(
                Some(self.filters.rating.kind),
                filter::RatingKind::ALL,
                ToString::to_string,
            )
            .padding(padding)
            .font(picklist_font)
            .on_select(|selected| HomeMessage::Filter(FilterMessage::RatingKind(selected)))
            .width(52.0)
            .handle(picklist_handle(size))
            .text_size(size);

            let comp = comp(
                comp_icon(self.filters.rating.comp),
                FilterMessage::RatingComp,
            );

            row!(text, comp, rating)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let comments = {
            let text = sized_medium("Comments", H8);
            let icon = self
                .filters
                .comments
                .map(|comments| comp_icon(comments.comp))
                .unwrap_or(comp_icon(filter::Comp::default()));
            let comp = comp(icon, FilterMessage::CommentsComp);

            let content = self
                .filters
                .comments
                .map(|comments| comments.number.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(32.0)
                .size(size)
                .font(input_font)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::CommentsNum(input)));

            row!(text, comp, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let release = {
            let text = sized_medium("Release", H8);
            let icon = self
                .filters
                .release
                .map(|release| comp_icon(release.comp))
                .unwrap_or(comp_icon(filter::Comp::default()));
            let comp = comp(icon, FilterMessage::ReleaseComp);

            let content = self
                .filters
                .release
                .map(|release| release.year.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(48.0)
                .font(input_font)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::ReleaseYear(input)));

            row!(text, comp, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let duration = {
            let hr = sized_regular("hrs", size);
            let min = sized_regular("mins", size);
            let text = sized_medium("Duration", H8);
            let icon = self
                .filters
                .duration
                .map(|duration| comp_icon(duration.comp))
                .unwrap_or(comp_icon(filter::Comp::default()));
            let comp = comp(icon, FilterMessage::DurationComp);

            let hours = self
                .filters
                .duration
                .map(|duration| format!("{}", duration.secs / 3600))
                .unwrap_or_default();
            let hours = text_input("", &hours)
                .width(28.0)
                .size(size)
                .font(input_font)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::DurationHours(input)));

            let minutes = self
                .filters
                .duration
                .map(|duration| format!("{}", (duration.secs % 3600) / 60))
                .unwrap_or_default();
            let minutes = text_input("", &minutes)
                .width(28.0)
                .size(size)
                .font(input_font)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::DurationMinutes(input)));

            let duration = row!(hours, hr, minutes, min)
                .spacing(4.0)
                .align_y(Vertical::Center);

            row!(text, comp, duration)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let mode = {
            let mode = sized_medium(self.filters.mode.to_string(), H8);

            let button = button(mode)
                .style(styles::button::background)
                .padding(padding)
                .on_press(HomeMessage::Filter(FilterMessage::Mode));

            tooltip(button, "Filter combination mode", tp::Position::Bottom)
        };

        let clear = button(sized_medium("Clear", H8))
            .padding(padding)
            .style(styles::button::text)
            .on_press(HomeMessage::Filter(FilterMessage::Clear));

        let content = row!(
            progress,
            vertical_rule(),
            rating,
            vertical_rule(),
            comments,
            vertical_rule(),
            release,
            vertical_rule(),
            duration,
            vertical_rule(),
            mode,
            vertical_rule(),
            clear,
        )
        .spacing(8.0)
        .align_y(Vertical::Center)
        .wrap();

        content.into()
    }

    #[allow(clippy::const_is_empty)]
    fn sort_view(&self) -> Element<'_, HomeMessage> {
        let size = H8;
        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);
        let view_sort = |sort: SortKind, position: Option<(usize, bool)>| {
            let enable = position.is_none();
            let msg = if enable {
                HomeMessage::Sort(SortMessage::AddSort(sort))
            } else {
                HomeMessage::Sort(SortMessage::RemoveSort(sort))
            };

            let content = match position {
                Some((order, asc)) => {
                    let label = format!("{sort} {}", order + 1);
                    let content = sized_medium(label, H8);

                    let unicode = if asc { UPS } else { DOWNS };
                    let icon = icon(unicode).size(10.0);

                    let icon = button(icon)
                        .padding(0)
                        .on_press(HomeMessage::Sort(SortMessage::ReverseSort(sort)))
                        .style(styles::button::text_primary);

                    let content = row!(content, icon).spacing(2.0).align_y(Vertical::Center);

                    button(content)
                }
                None => {
                    let label = format!("{sort}");
                    let content = sized_regular(label, size);

                    button(content)
                }
            }
            .on_press(msg)
            .style(move |theme, status| {
                let default = if enable {
                    styles::button::background(theme, status)
                } else if SortKind::HIDDEN.is_empty() {
                    styles::button::subtler(theme, status)
                } else {
                    styles::button::subtle_primary(theme, status)
                };
                let border = Border::default().width(2.0).rounded(5.0);

                button::Style { border, ..default }
            });

            Element::from(content)
        };

        let clear = button(h8("Clear"))
            .padding([2, 5])
            .style(styles::button::text)
            .on_press(HomeMessage::Sort(SortMessage::Clear));

        let reverse = button(h8("Reverse"))
            .padding([2, 5])
            .style(styles::button::text)
            .on_press(HomeMessage::Sort(SortMessage::ToggleReverse));

        let base = icon(ELLIPSIS_HOR).size(size);

        let more: Element<'_, HomeMessage> = if SortKind::HIDDEN.is_empty() {
            empty()
        } else {
            let hidden = container(
                column(SortKind::HIDDEN.iter().map(|sort| {
                    let position = self.sort.position(*sort);

                    view_sort(*sort, position)
                }))
                .spacing(8),
            )
            .style(styles::container::bw2)
            .padding([3, 6]);

            menu(base, hidden)
                .auto_close(false)
                .position(Position::Bottom)
                .on_toggle(|_| HomeMessage::None)
                .into()
        };

        row!(
            h8("Sort by: "),
            row(SortKind::VISIBLE.iter().map(|sort| {
                let order = self.sort.position(*sort);
                view_sort(*sort, order)
            }))
            .spacing(5.0),
            if !SortKind::HIDDEN.is_empty() {
                vertical_rule().into()
            } else {
                empty()
            },
            if !SortKind::HIDDEN.is_empty() {
                more
            } else {
                empty()
            },
            vertical_rule(),
            reverse,
            clear,
        )
        .align_y(Vertical::Center)
        .spacing(8.0)
        .into()
    }

    fn toolbar(&self) -> Element<'_, HomeMessage> {
        let size = P;
        let tp = tp::Position::Top;

        let filter = {
            let icon = if self.show_filters {
                icons::CHEV_UP
            } else {
                icons::CHEV_DOWN
            };
            let icon = icons::icon(icon).size(size).line_height(0.5);
            let text = icons::icon(icons::FILTER).size(size);

            let content = row!(text, icon).spacing(2.0).align_y(Vertical::Center);

            tooltip(
                button(content)
                    .style(if self.filters.is_any() {
                        styles::button::background
                    } else {
                        styles::button::text_primary
                    })
                    .on_press(HomeMessage::ToggleFilter)
                    .padding([5, 5]),
                "Filters",
                tp,
            )
        };

        let sort = {
            let icon = if self.show_sorts {
                icons::CHEV_UP
            } else {
                icons::CHEV_DOWN
            };
            let icon = icons::icon(icon).size(size).line_height(0.5);
            let text = icons::icon(icons::SORT).size(size);

            let content = row!(text, icon).spacing(2.0).align_y(Vertical::Center);

            tooltip(
                button(content)
                    .style(if self.sort.is_empty() {
                        styles::button::background
                    } else {
                        styles::button::text_primary
                    })
                    .on_press(HomeMessage::ToggleSort)
                    .padding([5, 5]),
                "Sort",
                tp,
            )
        };

        let curr_filters: Element<'_, HomeMessage> = if !self.show_filters {
            empty()
        } else {
            self.filters_view()
        };

        let curr_sorts: Element<'_, HomeMessage> = if !self.show_sorts {
            empty()
        } else {
            self.sort_view()
        };

        let left = row!(filter, sort).align_y(Vertical::Center).spacing(10.0);

        let right = row!(
            tooltip(
                icons::sized_button(icons::REFRESH, size).on_press(HomeMessage::RefreshContent),
                "Refresh",
                tp
            ),
            tooltip(
                icons::sized_button(icons::RAND, size).on_press(HomeMessage::Random),
                "Random media",
                tp
            ),
            tooltip(
                icons::sized_button(self.layout.icon(), size).on_press(HomeMessage::ToggleLayout),
                self.layout.to_string(),
                tp
            ),
        )
        .align_y(Vertical::Center)
        .spacing(5.0);

        let tools = row!(left, space::horizontal(), right).width(Length::Fill);

        let sorts_rule = if self.show_sorts {
            rule::horizontal(1.0).into()
        } else {
            empty()
        };

        let filters_rule = if self.show_filters {
            rule::horizontal(1.0).into()
        } else {
            empty()
        };

        let tools = column!(tools, sorts_rule, curr_sorts, filters_rule, curr_filters)
            .width(Length::Fill)
            .spacing(2.0)
            .padding([2, 5]);

        let tools = column!(tools, rule::horizontal(1.0));

        let content = container(tools).width(Length::Fill);

        content.into()
    }

    fn content_area(&self, now: Instant) -> Element<'_, HomeMessage> {
        let (title, items) = match &self.state {
            State::Recent { shows, movies } => ("Recents", shows.len() + movies.len()),
            State::Shows(shows) => ("Shows", shows.len()),
            State::Movies(movies) => ("Movies", movies.len()),
            State::Show { show, seasons, .. } => (show.media.name(), seasons.len()),
            State::Movie { movie, .. } => (movie.media.name(), 0),
            State::Season {
                season, episodes, ..
            } => (season.media.name(), episodes.len()),
            State::Episode { episode, .. } => (episode.media.name(), 0),
            State::Loading(_) => ("Loading", 0),
            State::Collections(collections) => ("Collections", collections.len()),
            State::Collection {
                collection,
                itriggers: _itriggers,
                dtriggers: _dtriggers,
                movies,
                shows,
                seasons,
                episodes,
            } => (
                collection.collection.name.as_str(),
                movies.len() + shows.len() + seasons.len() + episodes.len(),
            ),
        };
        let title = container(h4(title)).clip(true).center_y(40);

        let search =
            sized_button(icons::SEARCH, H6).on_press(HomeMessage::OpenView(ViewMessage::Search));

        let top = row!(
            self.navigation(),
            space::horizontal(),
            title,
            space::horizontal(),
            search,
        )
        .spacing(5.0)
        .padding(Padding::ZERO.right(5))
        .align_y(Vertical::Center)
        .width(Length::Fill);

        let top = container(column!(top, rule::horizontal(1.0),));

        let content_area = container(self.content(now))
            .height(Length::Fill)
            .width(Length::Fill);

        let show_tools = self
            .current_page()
            .map(|page| page.show_tools())
            .unwrap_or(true);

        let bottom: Element<'_, HomeMessage> = if items > 0 || self.scanning.is_some() {
            let size = H8;
            let padding = Padding::new(3.0).right(10);

            let items = sized_medium(format!("{items} items"), size);

            let scanning: Element<'_, HomeMessage> =
                match &self.scanning {
                    Some(scanning) => {
                        let label = sized_medium("Scanning Directories", size);

                        let svg = loading_svg(scanning, now).height(size).width(size).style(
                            |theme, _| widget::svg::Style {
                                color: Some(theme.extended_palette().primary.base.color),
                            },
                        );

                        row!(svg, label)
                            .spacing(4.0)
                            .align_y(Vertical::Center)
                            .into()
                    }
                    None => empty(),
                };

            let content = row!(scanning, space::horizontal(), items).align_y(Vertical::Center);

            container(content)
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Right)
                .padding(padding)
                .style(|theme| {
                    let default = styles::container::bw3(theme);
                    let background = default.background.map(|back| back.scale_alpha(0.25));

                    container::Style {
                        background,
                        ..default
                    }
                })
                .into()
        } else {
            empty()
        };

        let content = container(column!(
            top,
            if show_tools { self.toolbar() } else { empty() },
            content_area,
            bottom,
        ))
        .clip(true)
        .height(Length::Fill)
        .width(Length::Fill);

        content.into()
    }

    pub fn content(&self, now: Instant) -> Element<'_, HomeMessage> {
        match (&self.state, self.current_page()) {
            (State::Loading(animation), _) => center(loading_svg(animation, now)).into(),
            (State::Recent { shows, movies }, Some(Page::Home)) => self.recents(now, movies, shows),
            (State::Shows(shows), Some(Page::Shows(page))) => page
                .view(now, self.layout, shows.iter())
                .map(HomeMessage::Shows),
            (State::Movies(movies), Some(Page::Movies(page))) => page
                .view(now, self.layout, movies.iter())
                .map(HomeMessage::Movies),
            (State::Collections(collections), Some(Page::Collections(page))) => {
                page.view(collections.iter()).map(HomeMessage::Collections)
            }
            // todo: Needed?
            (State::Collections(_), None) => center("Loading").into(),
            (
                State::Episode {
                    episode,
                    memberships,
                },
                Some(Page::Episode { page, .. }),
            ) => page
                .view(episode, memberships.iter())
                .map(HomeMessage::EpisodePage),
            (State::Movie { movie, memberships }, Some(Page::Movie { page, .. })) => page
                .view(movie, memberships.iter())
                .map(HomeMessage::MoviePage),
            (
                State::Season {
                    season,
                    episodes,
                    memberships,
                },
                Some(Page::Season { page, .. }),
            ) => page
                .view(
                    now,
                    self.layout,
                    season,
                    episodes.iter(),
                    memberships.iter(),
                )
                .map(HomeMessage::SeasonPage),
            (
                State::Show {
                    show,
                    seasons,
                    memberships,
                },
                Some(Page::Show { page, .. }),
            ) => page
                .view(now, self.layout, show, seasons.iter(), memberships.iter())
                .map(HomeMessage::ShowPage),
            (
                State::Collection {
                    collection,
                    itriggers: _itriggers,
                    dtriggers: _dtriggers,
                    shows,
                    movies,
                    seasons,
                    episodes,
                },
                Some(Page::Collection {
                    collection: page, ..
                }),
            ) => page
                .view(
                    now,
                    self.layout,
                    collection,
                    movies.iter().peekable(),
                    shows.iter().peekable(),
                    seasons.iter().peekable(),
                    episodes.iter().peekable(),
                )
                .map(HomeMessage::Collection),
            unreached => {
                todo!("{unreached:?}")
            }
        }
    }

    pub fn view(&self, theme: &Theme, now: Instant) -> Element<'_, HomeMessage> {
        let content = container(
            row!(self.side(), self.content_area(now))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0),
        )
        .style(styles::container::bb);

        match &self.view {
            None => content.into(),
            Some(view) => {
                let overlay: Element<'_, HomeMessage> = match view {
                    View::CollectionConfig(config) => draw_config(config),
                    View::Search(state, None) => {
                        draw_search(state, |id| HomeMessage::Goto(id.into()), theme, true)
                    }
                    View::Search(state, Some(collection)) => draw_search(
                        state,
                        |item| HomeMessage::AddCollection(item, *collection),
                        theme,
                        false,
                    ),
                    View::CollectionAdd(state) => draw_collection_add(
                        state,
                        self.collections.iter(),
                        self.collections.is_empty(),
                    ),
                    View::Rating { rating, .. } => draw_rating(rating),
                    View::Rename {
                        input,
                        old,
                        value,
                        empty,
                        ..
                    } => draw_rename(input, old, value, *empty),
                    View::Synopsis {
                        editor, content, ..
                    } => draw_synopsis(editor, content),
                    View::TMDBId {
                        input,
                        value,
                        top_level,
                        ..
                    } => draw_tmdb(input, value, *top_level),
                    View::RemoveMedia { name, .. } => {
                        draw_delete_confirm(name, HomeMessage::RemoveMedia)
                    }
                    View::RemoveCollection { name, .. } => {
                        draw_delete_confirm(name, HomeMessage::RemoveCollection)
                    }
                    View::CollectionTriggers {
                        itriggers,
                        dtriggers,
                        view_inserts,
                        id: _id,
                        removed_inserts: _removed_inserts,
                        removed_deletes: _removed_deletes,
                    } => draw_collection_triggers(*view_inserts, itriggers, dtriggers),
                    View::Selection(selected) => {
                        let selection = draw_selection(selected.len());

                        return modal::transparent(content, selection);
                    }
                };

                modal(content, overlay, HomeMessage::CloseView)
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let state = match &self.state {
            State::Loading(animation) => animation.is_animating(now),
            State::Recent { shows, movies } => {
                let shows = shows.iter().any(|show| show.is_animating(now));
                let movies = movies.iter().any(|movie| movie.is_animating(now));

                shows || movies
            }
            State::Shows(shows) => shows.iter().any(|show| show.is_animating(now)),
            State::Movies(movies) => movies.iter().any(|movie| movie.is_animating(now)),
            State::Show { seasons, .. } => seasons.iter().any(|season| season.is_animating(now)),
            State::Season { episodes, .. } => {
                episodes.iter().any(|episode| episode.is_animating(now))
            }
            State::Collections(_) => false,
            State::Movie { .. } | State::Episode { .. } => false,
            State::Collection {
                shows,
                movies,
                seasons,
                episodes,
                ..
            } => {
                let shows = shows.iter().any(|show| show.is_animating(now));
                let movies = movies.iter().any(|movie| movie.is_animating(now));
                let seasons = seasons.iter().any(|season| season.is_animating(now));
                let episodes = episodes.iter().any(|episode| episode.is_animating(now));

                shows || movies || seasons || episodes
            }
        };

        let scan = self
            .scanning
            .as_ref()
            .map(|scan| scan.is_animating(now))
            .unwrap_or_default();

        state || scan
    }

    pub fn back(&mut self, now: Instant, clear: bool) -> Task<Message> {
        self.unfocus(now);
        let Some(new) = self.backward.pop() else {
            return Task::none();
        };

        let (task, msg) = match self.current_page.take() {
            Some(current) => {
                self.state = State::Loading(loading_animation(now));
                if clear {
                    self.forward.clear();
                } else {
                    self.forward.push(current);
                }

                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in back without being recorded first");
                self.current_page = Some(new);
                let task = page.update_scroll().map(|_| Message::None);
                let limit = if matches!(new, PageKind::Home) {
                    self.recent_limit
                } else {
                    None
                };

                (task, fetch_kind(new, self.filters, self.sort, limit, None))
            }
            None => {
                unreachable!("current page is always non-empty");
            }
        };

        Task::batch([msg.tasked(), task])
    }

    pub fn forward(&mut self, now: Instant) -> Task<Message> {
        self.unfocus(now);
        let Some(new) = self.forward.pop() else {
            return Task::none();
        };

        let (task, msg) = match self.current_page.take() {
            Some(current) => {
                self.backward.push(current);

                self.state = State::Loading(loading_animation(now));
                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in forward without being recorded");

                self.current_page = Some(new);
                let task = page.update_scroll().map(|_| Message::None);
                let limit = if matches!(new, PageKind::Home) {
                    self.recent_limit
                } else {
                    None
                };

                (task, fetch_kind(new, self.filters, self.sort, limit, None))
            }
            None => {
                unreachable!("current page is always non-empty");
            }
        };

        Task::batch([msg.tasked(), task])
    }

    pub fn content_refresh(&mut self, now: Instant) -> Task<Message> {
        let (id, limit) = match &self.state {
            State::Loading(_) => return Task::none(),
            State::Recent { .. } => (FetchId::Recents, self.recent_limit),
            State::Movies(_) => (FetchId::Movies, None),
            State::Shows(_) => (FetchId::Shows, None),
            State::Show { show, .. } => (FetchId::Show(show.media.id), None),
            State::Season { season, .. } => (FetchId::Season(season.media.id), None),
            State::Episode { episode, .. } => (FetchId::Episode(episode.media.id), None),
            State::Movie { movie, .. } => (FetchId::Movie(movie.media.id), None),
            State::Collections(_) => (FetchId::Collections, None),
            State::Collection { collection, .. } => {
                (FetchId::Collection(collection.collection.id), None)
            }
        };

        self.state = State::Loading(loading_animation(now));

        let msg = fetch_kind_aux(id, self.filters, self.sort, limit, None);

        Task::done(msg)
    }

    pub fn refresh(&mut self, now: Instant) -> Task<Message> {
        let rsg = Message::Fetch {
            id: FetchId::CollectionsSimple,
            filters: self.filters,
            sort: self.sort,
            limit: None,
            offset: None,
        };
        let rsg = Task::done(rsg);

        Task::batch([rsg, self.content_refresh(now)])
    }

    fn layout_toggle(&mut self) -> Task<Message> {
        self.layout = match self.layout {
            Layout::Grid => Layout::List,
            Layout::List => Layout::Compact,
            Layout::Compact => Layout::Grid,
        };

        Task::done(Message::Layout(self.layout))
    }

    fn close_view(&mut self, selected: bool) -> Task<Message> {
        let scroll = self.update_page_scroll();

        match self.view.take() {
            Some(View::Selection(items)) if selected => {
                self.view = Some(View::Selection(items));
                self.command = false;

                let clear = HomeMessage::Selection(SelectionMessage::Cancel);

                Task::batch([scroll, Message::Home(clear).tasked()])
            }
            Some(View::Selection(items)) => {
                self.command = true;
                self.view = Some(View::Selection(items));

                scroll
            }
            _ => {
                self.command = false;
                scroll
            }
        }
    }

    pub fn goto(&mut self, kind: PageKind, now: Instant) -> Task<Message> {
        self.unfocus(now);
        if let Some(current) = self.current_page
            && current == kind
        {
            return self.close_view(false);
        }

        let close_view = self.close_view(false);

        if let Some(old) = self.current_page.replace(kind) {
            self.backward.push(old)
        };
        self.forward.clear();
        self.state = State::Loading(loading_animation(now));

        let limit = if matches!(kind, PageKind::Home) {
            self.recent_limit
        } else {
            None
        };
        let msg = fetch_kind(kind, self.filters, self.sort, limit, None).tasked();

        if let Some(page) = self.pages.get_mut(&kind) {
            let scroll = page.update_scroll().discard();

            let tsk = msg.chain(scroll);

            return Task::batch([tsk, close_view]);
        }

        let task = match kind {
            PageKind::Home => self.update_scroll().discard(),
            PageKind::Movies => {
                let (movies, task) = Movies::boot();

                self.pages.insert(kind, Page::Movies(movies));

                task.map(|msg| Message::Home(HomeMessage::Movies(msg)))
            }
            PageKind::Shows => {
                let (shows, tasks) = TvShows::boot();

                self.pages.insert(kind, Page::Shows(shows));

                tasks.map(|ssg| Message::Home(HomeMessage::Shows(ssg)))
            }
            PageKind::Movie(id) => {
                let movie = MoviePage::new(id);

                self.pages.insert(kind, Page::Movie { page: movie, id });

                Task::none()
            }
            PageKind::Episode(id) => {
                let episode = EpisodePage::new(id);

                self.pages.insert(kind, Page::Episode { page: episode, id });

                Task::none()
            }
            PageKind::Show(id) => {
                let (show, task) = ShowPage::boot(id);

                self.pages.insert(kind, Page::Show { id, page: show });

                task.map(|ssg| Message::Home(HomeMessage::ShowPage(ssg)))
            }
            PageKind::Season(id) => {
                let (season, task) = SeasonPage::boot(id);

                self.pages.insert(kind, Page::Season { id, page: season });

                task.map(|ssg| Message::Home(HomeMessage::SeasonPage(ssg)))
            }
            PageKind::Collection(id) => {
                let (collection, tasks) = CollectionPage::boot(id);

                self.pages.insert(kind, Page::Collection { collection, id });

                tasks.map(|csg| Message::Home(HomeMessage::Collection(csg)))
            }
            PageKind::Collections => {
                let (collections, task) = Collections::boot();

                self.pages.insert(kind, Page::Collections(collections));

                task.map(|csg| Message::Home(HomeMessage::Collections(csg)))
            }
        };

        Task::batch([msg, close_view, task])
    }

    fn selection(&mut self) -> Task<Message> {
        self.view = Some(View::Selection(vec![]));

        self.update_page_scroll()
    }

    pub fn action(&mut self, action: HomeAction, now: Instant) -> Task<Message> {
        match action {
            HomeAction::SettingsOpen => Task::done(Message::SettingsOpen),
            HomeAction::LayoutToggle => self.layout_toggle(),
            HomeAction::RefreshContent => self.content_refresh(now),
            HomeAction::Refresh => self.refresh(now),
            HomeAction::SearchToggle => self.toggle_search(None, now),
            HomeAction::CloseModal => self.close_view(true),
            HomeAction::Back => self.back(now, false),
            HomeAction::Forward => self.forward(now),
            HomeAction::SelectionStart => self.selection(),
        }
    }

    pub fn fetched_recents(
        &mut self,
        mut movies: Vec<Thumbnail<Movie>>,
        mut shows: Vec<Thumbnail<Show>>,
    ) -> Task<Message> {
        if let Some(View::Selection(selected)) = &self.view {
            for media in &mut movies {
                media.selected = selected.contains(&media.media.id.into());
            }
            for media in &mut shows {
                media.selected = selected.contains(&media.media.id.into());
            }
        }

        let state = State::Recent { shows, movies };

        self.state = state;

        self.update_page_scroll()
    }

    pub fn fetch_collections_simple(
        &mut self,
        mut collections: Vec<SimpleCollection>,
    ) -> Task<Message> {
        sort_collections(&mut collections);

        self.collections = collections;
        self.update_page_scroll()
    }

    pub fn fetched_shows(&mut self, mut shows: Vec<Thumbnail<Show>>) -> Task<Message> {
        if let Some(View::Selection(selected)) = &self.view {
            for media in &mut shows {
                media.selected = selected.contains(&media.media.id.into());
            }
        }
        self.state = State::Shows(shows);

        self.update_page_scroll()
    }

    pub fn fetched_movies(&mut self, mut movies: Vec<Thumbnail<Movie>>) -> Task<Message> {
        if let Some(View::Selection(selected)) = &self.view {
            for media in &mut movies {
                media.selected = selected.contains(&media.media.id.into());
            }
        }
        self.state = State::Movies(movies);
        self.update_page_scroll()
    }

    pub fn fetched_show(
        &mut self,
        show: Thumbnail<Show>,
        mut seasons: Vec<Thumbnail<Season>>,
    ) -> Task<Message> {
        if let Some(View::Selection(selected)) = &self.view {
            for media in &mut seasons {
                media.selected = selected.contains(&media.media.id.into());
            }
        }

        let memberships = Message::FetchMemberships(show.media.id.into());

        self.state = State::Show {
            show,
            seasons,
            memberships: vec![],
        };

        Task::batch([self.update_page_scroll(), Task::done(memberships)])
    }

    pub fn fetched_movie(&mut self, movie: Thumbnail<Movie>) -> Task<Message> {
        let memberships = Message::FetchMemberships(movie.media.id.into());

        self.state = State::Movie {
            movie,
            memberships: vec![],
        };

        Task::batch([self.update_page_scroll(), Task::done(memberships)])
    }

    pub fn fetched_season(
        &mut self,
        season: Thumbnail<Season>,
        mut episodes: Vec<Thumbnail<Episode>>,
    ) -> Task<Message> {
        if let Some(View::Selection(selected)) = &self.view {
            for media in &mut episodes {
                media.selected = selected.contains(&media.media.id.into());
            }
        }
        let memberships = Message::FetchMemberships(season.media.id.into());

        self.state = State::Season {
            season,
            episodes,
            memberships: vec![],
        };

        Task::batch([self.update_page_scroll(), Task::done(memberships)])
    }

    pub fn fetched_episode(&mut self, episode: Thumbnail<Episode>) -> Task<Message> {
        let memberships = Message::FetchMemberships(episode.media.id.into());

        self.state = State::Episode {
            episode,
            memberships: vec![],
        };

        Task::batch([self.update_page_scroll(), Task::done(memberships)])
    }

    pub fn fetched_collections(&mut self, collections: Vec<Collection>) -> Task<Message> {
        Task::perform(
            async move {
                collections
                    .into_iter()
                    .map(CollectionThumbnail::new)
                    .collect::<Vec<_>>()
            },
            move |collections| Message::Home(HomeMessage::FetchedCollections(collections)),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetched_collection(
        &mut self,
        collection: Collection,
        itriggers: Vec<InsertTrigger>,
        dtriggers: Vec<DeleteTrigger>,
        movies: Vec<Thumbnail<Movie>>,
        shows: Vec<Thumbnail<Show>>,
        seasons: Vec<Thumbnail<Season>>,
        episodes: Vec<Thumbnail<Episode>>,
    ) -> Task<Message> {
        Task::perform(
            async move {
                let collection = CollectionThumbnail::new(collection);
                (
                    collection, itriggers, dtriggers, movies, shows, seasons, episodes,
                )
            },
            move |(collection, itriggers, dtriggers, movies, shows, seasons, episodes)| {
                Message::Home(HomeMessage::FetchedCollection {
                    collection: Box::new(collection),
                    itriggers,
                    dtriggers,
                    movies,
                    shows,
                    seasons,
                    episodes,
                })
            },
        )
    }

    pub fn fetched_memberships_ids(&mut self, memberships: Vec<CollectionId>) -> Task<Message> {
        if let Some(View::CollectionAdd(CollectionAddState {
            selected, initial, ..
        })) = self.view.as_mut()
        {
            for collection in memberships {
                initial.insert(collection);
                selected.insert(collection);
            }
        }

        self.update_page_scroll()
    }

    pub fn fetched_memberships(&mut self, mut fetched: Vec<SimpleCollection>) -> Task<Message> {
        sort_collections(&mut fetched);

        match &mut self.state {
            State::Movie { memberships, .. } => {
                *memberships = fetched;
                self.update_page_scroll()
            }
            State::Show { memberships, .. } => {
                *memberships = fetched;
                self.update_page_scroll()
            }
            State::Season { memberships, .. } => {
                *memberships = fetched;
                self.update_page_scroll()
            }
            State::Episode { memberships, .. } => {
                *memberships = fetched;
                self.update_page_scroll()
            }
            _ => Task::none(),
        }
    }

    pub fn movie_sample(&mut self, id: MovieId, sample: Option<(Color, Color)>) -> Task<Message> {
        match &mut self.state {
            State::Movies(movies)
            | State::Recent { movies, .. }
            | State::Collection { movies, .. } => {
                if let Some(movie) = movies.iter_mut().find(|thumbnail| thumbnail.media.id == id) {
                    movie.sample(sample);
                }
            }
            State::Movie { movie, .. } if movie.media.id == id => {
                movie.sample(sample);
            }
            _ => {}
        }

        Task::none()
    }

    pub fn show_sample(&mut self, id: ShowId, sample: Option<(Color, Color)>) -> Task<Message> {
        match &mut self.state {
            State::Shows(shows) | State::Recent { shows, .. } | State::Collection { shows, .. } => {
                if let Some(show) = shows.iter_mut().find(|thumbnail| thumbnail.media.id == id) {
                    show.sample(sample);
                }
            }
            State::Show { show, .. } if show.media.id == id => {
                show.sample(sample);
            }
            _ => {}
        }

        Task::none()
    }

    pub fn season_sample(&mut self, id: SeasonId, sample: Option<(Color, Color)>) -> Task<Message> {
        match &mut self.state {
            State::Show { seasons, .. } | State::Collection { seasons, .. } => {
                if let Some(season) = seasons
                    .iter_mut()
                    .find(|thumbnail| thumbnail.media.id == id)
                {
                    season.sample(sample);
                }
            }
            State::Season { season, .. } if season.media.id == id => {
                season.sample(sample);
            }
            _ => {}
        }

        Task::none()
    }

    pub fn episode_sample(
        &mut self,
        id: EpisodeId,
        sample: Option<(Color, Color)>,
    ) -> Task<Message> {
        match &mut self.state {
            State::Season { episodes, .. } | State::Collection { episodes, .. } => {
                if let Some(episode) = episodes
                    .iter_mut()
                    .find(|thumbnail| thumbnail.media.id == id)
                {
                    episode.sample(sample);
                }
            }
            State::Episode { episode, .. } if episode.media.id == id => {
                episode.sample(sample);
            }
            _ => {}
        }

        Task::none()
    }

    pub fn load(&mut self) -> Task<Message> {
        let Some(View::Search(state, _)) = self.view.as_mut() else {
            return Task::none();
        };

        state.last_edit = None;

        if state.search.trim().is_empty() {
            state.items.clear();
            return Task::none();
        }

        Task::done(Message::LoadSearch(state.search.clone(), state.filter))
    }

    pub fn loaded_search(&mut self, items: Vec<SearchView>) -> Task<Message> {
        let Some(View::Search(state, _)) = self.view.as_mut() else {
            return self.update_page_scroll();
        };

        state.items = items;

        self.update_page_scroll()
    }

    pub fn toggle_search(
        &mut self,
        collection: Option<CollectionId>,
        now: Instant,
    ) -> Task<Message> {
        self.unfocus(now);
        let text_input = widget::Id::unique();
        let state = SearchState {
            items: vec![],
            search: String::default(),
            last_edit: None,
            filter: None,
            text_input: text_input.clone(),
        };

        self.view = Some(View::Search(state, collection));

        let focus = operation::focus(text_input);

        Task::batch([focus, self.update_page_scroll()])
    }

    pub fn scanning(&mut self, scanning: bool, now: Instant) -> Task<Message> {
        if scanning {
            self.scanning = Some(loading_animation(now));
            Task::none()
        } else {
            self.scanning.take();
            self.content_refresh(now)
        }
    }
}

fn icon_button<'a>(
    left_icon: char,
    value: &'a str,
    message: HomeMessage,
    current: bool,
    right_icon: Option<char>,
) -> Element<'a, HomeMessage> {
    let size = H6;
    let icon = icons::icon(left_icon).size(size);
    let text = if current {
        marquee(value).size(H6).font(bold_font())
    } else {
        marquee(value).size(H6).font(medium_font())
    };

    let content = row!(icon, text)
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .spacing(SIDE_ICON_SPACING);

    let content = match right_icon {
        Some(icon) => {
            let icon = icons::icon(icon).size(size);

            content.push(icon)
        }
        None => content,
    };

    container(
        button(content)
            .style(move |theme, status| {
                if current {
                    styles::button::background_primary(theme, status)
                } else {
                    styles::button::subtlest(theme, status)
                }
            })
            .on_press(message),
    )
    .clip(true)
    .max_height(56.0)
    .into()
}

pub fn view_unicode(view: CollectionView) -> char {
    match view {
        CollectionView::Shown => EYE,
        CollectionView::Pinned => PIN,
        CollectionView::Hidden => HIDE,
    }
}

fn fetch_kind(
    kind: PageKind,
    filters: Filter,
    sort: Sort,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Message {
    let id = match kind {
        PageKind::Home => FetchId::Recents,
        PageKind::Shows => FetchId::Shows,
        PageKind::Movies => FetchId::Movies,
        PageKind::Collections => FetchId::Collections,
        PageKind::Show(id) => FetchId::Show(id),
        PageKind::Season(id) => FetchId::Season(id),
        PageKind::Episode(id) => FetchId::Episode(id),
        PageKind::Movie(id) => FetchId::Movie(id),
        PageKind::Collection(id) => FetchId::Collection(id),
    };

    fetch_kind_aux(id, filters, sort, limit, offset)
}

fn fetch_kind_aux(
    id: FetchId,
    filters: Filter,
    sort: Sort,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Message {
    Message::Fetch {
        id,
        filters,
        // sort,
        sort: if matches!(id, FetchId::Recents) {
            Sort::recents()
        } else {
            sort
        },
        limit,
        offset,
    }
}

fn sort_collections(collections: &mut [SimpleCollection]) {
    collections.sort_by(|x, y| {
        x.view.cmp(&y.view).then(alphanumeric_sort::compare_str(
            x.name.to_lowercase(),
            y.name.to_lowercase(),
        ))
    });
}

fn comp_icon(comp: filter::Comp) -> char {
    use filter::Comp;

    match comp {
        Comp::Equal => EQUALS,
        Comp::Greater => CHEV_LEFT,
        Comp::Less => CHEV_RIGHT,
    }
}
