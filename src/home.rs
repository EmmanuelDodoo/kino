use iced::{
    Element, Length, Padding, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    animation::Animation,
    border::{Border, Radius},
    mouse,
    time::{Duration, Instant},
    widget::{
        self, Container, button, center, checkbox, column, container, grid, mouse_area,
        operation::{self, scroll_to},
        pick_list, row, rule, scrollable, space, text, text_editor, text_input, tooltip as tp,
    },
    window,
};
use std::collections::{HashMap, HashSet};

mod collection;
mod collections;
mod episode;
mod movie;
mod movies;
mod pages;
mod season;
mod series;
pub mod shared;
mod shows;

use crate::models::{
    Collection, CollectionId, CollectionView, Episode, Movie, Season, Show, SimpleCollection,
    collection::ItemId, collection::Items,
};

use crate::app::{FetchId, Message};
use crate::models::Media;
use crate::utils::{
    self, HomeAction, Layout, Sort, SortKind, empty, filter::*, icons, icons::*, loading_animation,
    loading_svg, styles, tooltip, typo::*,
};
use crate::widgets::{
    menu::{Position, menu},
    modal, toast,
};
use collection::{CollectionMessage, CollectionPage};
use collections::{Collections, CollectionsMessage};
use episode::{EpisodePage, EpisodePageMessage};
use movie::{MoviePage, MoviePageMessage};
use movies::{Movies, MoviesMessage};
use pages::{Page, PageKind, PageUpdate};
use season::{SeasonPage, SeasonPageMessage};
use series::{ShowPage, ShowPageMessage};
use shared::{
    CARD_HEIGHT, CARD_WIDTH, CollectionThumbnail, Icon, Scroll, SearchView, Thumbnail, filter_sort,
};
use shows::{TvShows, TvShowsMessage};

const SIDE_ICON_SPACING: f32 = 8.0;

#[derive(Debug, Clone)]
pub enum FilterMessage {
    Mode,
    Clear,
    ProgressKind(ProgressKind),
    ProgressComp,
    RatingKind(RatingKind),
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
    Clear,
    ToggleReverse,
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Name(String),
    Description(text_editor::Action),
    View(CollectionView),
    Icon(Icon),
    Script(String),
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
    custom: Option<String>,
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
            custom,
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
        collection.custom = custom;
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
                custom: collection.custom.clone(),
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
                custom: None,
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
pub enum ViewMessage {
    CollectionConfig,
    Add(ItemId),
    AddToCollection(CollectionId),
    Search,
    Rating(ItemId, Option<f32>),
}

#[derive(Debug)]
pub enum View {
    CollectionConfig(CollectionConfig),
    Search(SearchState, Option<CollectionId>),
    CollectionAdd(CollectionAddState),
    Rating { id: ItemId, rating: Rating },
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
    SearchMessage(SearchMessage),
    CollectionAdd(CollectionAddMessage),
    Rating(RatingMessage),
    OpenView(ViewMessage),
    AddCollection(ItemId, CollectionId),
    CloseView,
    Play(ItemId),
    PlayCollection {
        id: CollectionId,
        items: Items,
    },
    ToggleLayout,
    Home,
    Goto(PageKind),
    NewCollection,
    None,
    Scroll(scrollable::Viewport),
    RefreshContent,
    Hovered(ItemId, bool),
    FetchedCollections(Vec<CollectionThumbnail>),
    FetchedCollection {
        collection: Box<CollectionThumbnail>,
        movies: Vec<Thumbnail<Movie>>,
        shows: Vec<Thumbnail<Show>>,
        seasons: Vec<Thumbnail<Season>>,
        episodes: Vec<Thumbnail<Episode>>,
    },
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
    pending: Vec<Task<HomeMessage>>,

    recent_limit: Option<i32>,
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
            sort,
            limit: Some(5),
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

            layout,
            sort,
            filters,

            show_sorts: false,
            show_filters: false,
            state: State::Loading(loading_animation(Instant::now())),
            scroll: Scroll::new(),
            pages: HashMap::default(),
            current_page: None,
            pending: vec![],
            collections: Vec::default(),
            view: None,
            recent_limit,
        }
    }

    pub fn update(&mut self, message: HomeMessage, now: Instant) -> Task<Message> {
        match message {
            HomeMessage::None => Task::none(),
            HomeMessage::Settings => todo!(),
            HomeMessage::FetchedCollections(collections) => {
                self.state = State::Collections(collections);

                self.update_page_scroll()
            }
            HomeMessage::FetchedCollection {
                collection,
                movies,
                shows,
                seasons,
                episodes,
            } => {
                self.state = State::Collection {
                    collection,
                    shows,
                    movies,
                    seasons,
                    episodes,
                };

                self.update_page_scroll()
            }
            HomeMessage::Home => {
                if let Some(old) = self.current_page.take() {
                    self.backward.push(old);
                };
                self.forward.clear();
                self.state = State::Loading(loading_animation(now));

                let scroll = self.update_scroll().map(|_| Message::None);

                let msg = Message::Fetch {
                    id: FetchId::Recents,
                    filters: self.filters,
                    sort: self.sort,
                    limit: self.recent_limit,
                    offset: None,
                };

                Task::batch([Task::done(msg), scroll])
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
            HomeMessage::OpenView(view) => match view {
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
                ViewMessage::Add(item) => {
                    let state = CollectionAddState {
                        item,
                        selected: HashSet::new(),
                        initial: HashSet::new(),
                    };
                    self.view = Some(View::CollectionAdd(state));

                    Task::done(Message::FetchMembershipIds(item))
                }
                ViewMessage::AddToCollection(id) => self.toggle_search(Some(id)),
                ViewMessage::Search => self.toggle_search(None),
                ViewMessage::Rating(id, rating) => {
                    let rating = Rating::Value(rating.unwrap_or_default());
                    self.view = Some(View::Rating { id, rating });

                    self.update_page_scroll()
                }
            },
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
                    ConfigMessage::Script(script) => {
                        if script.is_empty() {
                            config.custom = None
                        } else {
                            config.custom = Some(script)
                        }
                    }
                    ConfigMessage::Save if config.id.is_some() => {
                        let close_view = self.close_view();

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
                            custom,
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

                        let (new, query) = Collection::new(
                            name,
                            description,
                            view,
                            Some(icon.to_u32()),
                            theme,
                            custom,
                        );

                        let new_id = new.id;
                        let simple = SimpleCollection::from_collection(&new);
                        self.collections.push(simple);
                        sort_collections(&mut self.collections);

                        let close_view = self.close_view();
                        self.state = State::Collection {
                            collection: Box::new(CollectionThumbnail::new(new)),
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
                            let (collection, tasks) =
                                CollectionPage::boot(new_id, self.sort, self.filters, self.layout);

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
            HomeMessage::SearchMessage(ssg) => {
                let Some(View::Search(state, _)) = self.view.as_mut() else {
                    return Task::none();
                };

                match ssg {
                    SearchMessage::Search(mut search) => {
                        state.last_edit = Some(now);
                        match search.find(":").and_then(|pos| {
                            SearchFilter::new(&search[0..pos]).map(|filter| (pos, filter))
                        }) {
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

                        match &mut self.state {
                            State::Movie {
                                movie: Thumbnail { media, .. },
                                ..
                            } if ItemId::Movie(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Show {
                                show: Thumbnail { media, .. },
                                ..
                            } if ItemId::Show(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Season {
                                season: Thumbnail { media, .. },
                                ..
                            } if ItemId::Season(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Episode {
                                episode: Thumbnail { media, .. },
                                ..
                            } if ItemId::Episode(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            _ => Task::none(),
                        }
                    }
                    RatingMessage::Submit => {
                        let Rating::Input { input, .. } = &rating else {
                            return Task::none();
                        };

                        let value = input.parse::<f32>().unwrap_or(0.0).clamp(0.0, 5.0);
                        *rating = Rating::Value(value);

                        match &mut self.state {
                            State::Movie {
                                movie: Thumbnail { media, .. },
                                ..
                            } if ItemId::Movie(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Show {
                                show: Thumbnail { media, .. },
                                ..
                            } if ItemId::Show(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Season {
                                season: Thumbnail { media, .. },
                                ..
                            } if ItemId::Season(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            State::Episode {
                                episode: Thumbnail { media, .. },
                                ..
                            } if ItemId::Episode(media.id) == *id => {
                                let query = media.set_rating(value);

                                Task::done(Message::Query(query))
                            }
                            _ => Task::none(),
                        }
                    }
                    RatingMessage::Input(value) => {
                        if let Rating::Input { input, .. } = rating {
                            *input = value;
                        }

                        Task::none()
                    }
                }
            }
            HomeMessage::CloseView => self.close_view(),
            HomeMessage::Back => self.back(now),
            HomeMessage::Forward => self.forward(now),
            HomeMessage::ToggleLayout => self.layout_toggle(),
            HomeMessage::Sort(ssg) => {
                match ssg {
                    SortMessage::AddSort(kind) => self.sort.push(kind),
                    SortMessage::RemoveSort(kind) => self.sort.remove(kind),
                    SortMessage::Clear => self.sort.clear(),
                    SortMessage::ToggleReverse => self.sort.reverse(),
                }

                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                if let Some(page) = self.current_page_mut() {
                    page.page_update(update);
                };

                // todo: Persist changes
                Task::none()
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
                            return Task::none();
                        }

                        let Ok(number) = number.parse::<u32>() else {
                            let msg = Message::PushToast(
                                format!("Invalid input: {number}"),
                                toast::Status::Error,
                            );
                            return Task::done(msg);
                        };

                        match self.filters.comments.as_mut() {
                            Some(comments) => {
                                comments.number = number;
                            }
                            None => {
                                self.filters.comments = Some(Comments {
                                    number,
                                    comp: Comp::default(),
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

                            return Task::none();
                        }

                        let Ok(minutes) = minutes.parse::<u64>() else {
                            let msg = Message::PushToast(
                                format!("Invalid input: {minutes}"),
                                toast::Status::Error,
                            );
                            return Task::done(msg);
                        };

                        let secs = minutes * 60;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let hours = (duration.secs / 3600) * 3600;

                                duration.secs = hours + secs;
                            }
                            None => {
                                self.filters.duration = Some(utils::Duration {
                                    secs,
                                    comp: Comp::default(),
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

                            return Task::none();
                        }

                        let Ok(hours) = hours.parse::<u64>() else {
                            let msg = Message::PushToast(
                                format!("Invalid input: {hours}"),
                                toast::Status::Error,
                            );
                            return Task::done(msg);
                        };

                        let secs = hours * 3600;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let minutes = duration.secs % 3600;
                                duration.secs = secs + minutes;
                            }
                            None => {
                                self.filters.duration = Some(utils::Duration {
                                    secs,
                                    comp: Comp::default(),
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
                            return Task::none();
                        }

                        let Ok(year) = year.parse::<i32>() else {
                            let msg = Message::PushToast(
                                format!("Invalid input: {year}"),
                                toast::Status::Error,
                            );
                            return Task::done(msg);
                        };

                        match self.filters.release.as_mut() {
                            Some(release) => release.year = year,
                            None => {
                                self.filters.release = Some(Release {
                                    year,
                                    comp: Comp::default(),
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

                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                if let Some(page) = self.current_page_mut() {
                    page.page_update(update);
                };

                // todo: Persist changes
                Task::none()
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
                let close_view = self.close_view();
                Task::batch([Task::done(Message::PlayItem(item)), close_view])
            }
            HomeMessage::PlayCollection { id, items } => {
                Task::done(Message::PlayCollectionItems { id, items })
            }
            HomeMessage::Hovered(id, is_hovered) => match (&mut self.state, id) {
                (State::Loading(_), _)
                | (State::Episode { .. }, _)
                | (State::Movie { .. }, _)
                | (State::Collections(_), _) => Task::none(),
                (State::Recent { shows, .. }, ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Recent { movies, .. }, ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Recent { .. }, _) => Task::none(),
                (State::Shows(shows), ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Shows(_), _) => Task::none(),
                (State::Movies(movies), ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Movies(_), _) => Task::none(),
                (State::Show { seasons, .. }, ItemId::Season(id)) => {
                    if let Some(season) = seasons.iter_mut().find(|season| season.media.id == id) {
                        season.go_mut(is_hovered, now);
                    }
                    Task::none()
                }
                (State::Show { .. }, _) => Task::none(),
                (State::Season { episodes, .. }, ItemId::Episode(id)) => {
                    if let Some(episode) =
                        episodes.iter_mut().find(|episode| episode.media.id == id)
                    {
                        episode.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Season { .. }, _) => Task::none(),
                (State::Collection { shows, .. }, ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Collection { movies, .. }, ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Collection { seasons, .. }, ItemId::Season(id)) => {
                    if let Some(season) = seasons.iter_mut().find(|show| show.media.id == id) {
                        season.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Collection { episodes, .. }, ItemId::Episode(id)) => {
                    if let Some(episode) =
                        episodes.iter_mut().find(|episode| episode.media.id == id)
                    {
                        episode.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
            },
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

    fn update_scroll(&mut self) -> Task<()> {
        scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn update_page_scroll(&mut self) -> Task<Message> {
        match self.current_page_mut() {
            None => self.update_scroll().discard(),
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
            let icon = icons::icon(icons::LOGO).size(H2);
            let text = text("Kino").size(H2);

            row!(icon, text)
                .padding([5, 10])
                .align_y(Vertical::Center)
                .spacing(12.0)
        };

        let collections = self
            .collections
            .iter()
            .filter_map(|collection| match collection.view {
                CollectionView::Pinned => {
                    let unicode = Icon::new(collection.icon).unicode();
                    let content = collection_button(
                        unicode,
                        &collection.name,
                        view_unicode(collection.view),
                        HomeMessage::Goto(PageKind::Collection(collection.id)),
                        self.current_page()
                            .map(|page| page.is_collection(&collection.id))
                            .unwrap_or_default(),
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
                    );

                    Some(content)
                }
                CollectionView::Hidden => None,
            });

        let collections = column!(
            icon_button(
                icons::HOME,
                "Home",
                HomeMessage::Home,
                self.current_page().is_none()
            ),
            icon_button(
                icons::SHOW,
                "Shows",
                HomeMessage::Goto(Page::goto_shows()),
                self.current_page().map(Page::is_shows).unwrap_or_default()
            ),
            icon_button(
                icons::MOVIE,
                "Movies",
                HomeMessage::Goto(Page::goto_movies()),
                self.current_page().map(Page::is_movies).unwrap_or_default(),
            ),
        )
        .extend(collections)
        .push(icon_button(
            icons::ADD,
            "New collection",
            HomeMessage::NewCollection,
            false,
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
                    .unwrap_or_default()
            ),
            // icon_button(
            //     icons::COMMENT,
            //     "Comments",
            //     HomeMessage::Goto(Page::goto_comments()),
            //     self.current_page()
            //         .map(Page::is_comments)
            //         .unwrap_or_default()
            // ),
            icon_button(icons::SETTINGS, "Settings", HomeMessage::Settings, false)
        )
        .spacing(16.0);

        let content = column!(collections, bottom,)
            .padding([0, 5])
            .height(Length::Fill);

        let content = column!(header, space::vertical().height(24.0), content,)
            .width(275.0)
            .height(Length::Fill);

        content.into()
    }

    fn recents<'a>(
        &self,
        now: Instant,
        movies: &'a [Thumbnail<Movie>],
        shows: &'a [Thumbnail<Show>],
    ) -> Element<'a, HomeMessage> {
        let movies = {
            let label = text("Recent Movies").size(H4);
            let label = column!(label, rule::horizontal(1.0)).spacing(4.0);

            let movies = filter_sort(movies.iter(), &self.filters, &self.sort);

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
            let label = text("Recent Shows").size(H4);
            let label = column!(label, rule::horizontal(1.0)).spacing(4.0);

            let shows = filter_sort(shows.iter(), &self.filters, &self.sort);

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

        let content = scrollable(column!(movies, shows).spacing(40.0).padding(10))
            .spacing(16.0)
            .id(self.scroll.id.clone())
            .on_scroll(HomeMessage::Scroll);

        content.into()
    }

    fn navigation(&self) -> Element<'_, HomeMessage> {
        let can_back =
            !self.backward.is_empty() || (self.backward.is_empty() && self.current_page.is_some());

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

        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);
        let comp = |icon: char, msg: FilterMessage| {
            icons::sized_button(icon, size * RATIO)
                .padding([5, 5])
                .style(styles::button::background)
                .on_press(HomeMessage::Filter(msg))
        };

        let up = pick_list::Icon {
            font: icons::FONT,
            code_point: icons::CHEV_UP,
            size: Some(size.into()),
            line_height: text::LineHeight::Relative(1.0),
            shaping: text::Shaping::Basic,
        };

        let down = pick_list::Icon {
            font: icons::FONT,
            code_point: icons::CHEV_DOWN,
            size: Some(size.into()),
            line_height: text::LineHeight::Relative(1.0),
            shaping: text::Shaping::Basic,
        };

        let handle = pick_list::Handle::Dynamic {
            closed: down,
            open: up,
        };

        let progress = {
            let text = text("Progress:").size(size);
            let progress = pick_list(
                ProgressKind::ALL,
                Some(self.filters.progress.kind),
                |selected| HomeMessage::Filter(FilterMessage::ProgressKind(selected)),
            )
            .padding(padding)
            .width(60.0)
            .handle(handle.clone())
            .text_size(size);

            let comp = comp(
                self.filters.progress.comp.icon(),
                FilterMessage::ProgressComp,
            );

            row!(text, comp, progress)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let rating = {
            let text = text("Rating:").size(size);
            let rating = pick_list(
                RatingKind::ALL,
                Some(self.filters.rating.kind),
                |selected| HomeMessage::Filter(FilterMessage::RatingKind(selected)),
            )
            .padding(padding)
            .width(52.0)
            .handle(handle)
            .text_size(size);

            let comp = comp(self.filters.rating.comp.icon(), FilterMessage::RatingComp);

            row!(text, comp, rating)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let comments = {
            let text = text("Comments:").size(size);
            let icon = self
                .filters
                .comments
                .map(|comments| comments.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::CommentsComp);

            let content = self
                .filters
                .comments
                .map(|comments| comments.number.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(32.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::CommentsNum(input)));

            row!(text, comp, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let release = {
            let text = text("Release:").size(size);
            let icon = self
                .filters
                .release
                .map(|release| release.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::ReleaseComp);

            let content = self
                .filters
                .release
                .map(|release| release.year.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(48.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::ReleaseYear(input)));

            row!(text, comp, input)
                .spacing(spacing)
                .align_y(Vertical::Center)
        };

        let duration = {
            let hr = text("hrs").size(size);
            let min = text("mins").size(size);
            let text = text("Duration:").size(size);
            let icon = self
                .filters
                .duration
                .map(|duration| duration.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::DurationComp);

            let hours = self
                .filters
                .duration
                .map(|duration| format!("{}", duration.secs / 3600))
                .unwrap_or_default();
            let hours = text_input("", &hours)
                .width(28.0)
                .size(size)
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
            let mode = text(self.filters.mode.to_string()).size(size);

            let button = button(mode)
                .style(styles::button::background)
                .padding(padding)
                .on_press(HomeMessage::Filter(FilterMessage::Mode));

            tooltip(button, "Filter combination mode", tp::Position::Bottom)
        };

        let clear = button(text("Clear").size(size))
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

    fn sort_view(&self) -> Element<'_, HomeMessage> {
        let size = H8;
        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);
        let view_sort = |sort: SortKind, order: Option<usize>| {
            let enable = order.is_none();
            let msg = if enable {
                HomeMessage::Sort(SortMessage::AddSort(sort))
            } else {
                HomeMessage::Sort(SortMessage::RemoveSort(sort))
            };

            let label = sort.view(order);
            let content = text(label).size(size);

            Element::from(button(content).on_press(msg).style(move |theme, status| {
                let default = if enable {
                    styles::button::background(theme, status)
                } else if SortKind::HIDDEN.is_empty() {
                    styles::button::subtler(theme, status)
                } else {
                    styles::button::subtle_primary(theme, status)
                };
                let border = Border::default().width(2.0).rounded(5.0);

                button::Style { border, ..default }
            }))
        };

        let clear = button(text("Clear").size(size))
            .padding([2, 5])
            .style(styles::button::text)
            .on_press(HomeMessage::Sort(SortMessage::Clear));

        let reverse = button(text("Reverse").size(size))
            .padding([2, 5])
            .style(styles::button::text)
            .on_press(HomeMessage::Sort(SortMessage::ToggleReverse));

        let base = icon(ELLIPSIS_HOR).size(size);

        let more: Element<'_, HomeMessage> = if SortKind::HIDDEN.is_empty() {
            empty()
        } else {
            let hidden = container(
                column(SortKind::HIDDEN.iter().map(|sort| {
                    let order = self.sort.position(*sort);
                    view_sort(*sort, order)
                }))
                .spacing(8),
            )
            .style(styles::container::bw)
            .padding([3, 6]);

            menu(base, hidden)
                .auto_close(false)
                .position(Position::Bottom)
                .on_toggle(|_| HomeMessage::None)
                .into()
        };

        row!(
            text("Sort by: ").size(size),
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
                        styles::button::subtler
                    })
                    .on_press(HomeMessage::ToggleFilter)
                    .padding([5, 5]),
                "Filters",
                tp::Position::Bottom,
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
                        styles::button::subtler
                    })
                    .on_press(HomeMessage::ToggleSort)
                    .padding([5, 5]),
                "Sort",
                tp::Position::Bottom,
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
                tp::Position::Bottom
            ),
            tooltip(
                icons::sized_button(icons::RAND, size).on_press(HomeMessage::Random),
                "Random media",
                tp::Position::Bottom
            ),
            tooltip(
                icons::sized_button(self.layout.icon(), size).on_press(HomeMessage::ToggleLayout),
                self.layout.str(),
                tp::Position::Bottom
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
        let title = match &self.state {
            State::Recent { .. } => "Recents",
            State::Shows(_) => "Shows",
            State::Movies(_) => "Movies",
            State::Show { show, .. } => show.media.name(),
            State::Movie { movie, .. } => movie.media.name(),
            State::Season { season, .. } => season.media.name(),
            State::Episode { episode, .. } => episode.media.name(),
            State::Loading(_) => "Loading",
            State::Collections(_) => "Collections",
            State::Collection { collection, .. } => &collection.collection.name,
        };
        let title = container(text(title).size(H6)).clip(true).height(24);

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
        .height(H2 * RATIO)
        .width(Length::Fill);

        let top = container(column!(top, rule::horizontal(1.0),));

        let content_area = container(self.content(now))
            .height(Length::Fill)
            .width(Length::Fill);

        let show_tools = self
            .current_page()
            .map(|page| page.show_tools())
            .unwrap_or(true);

        let content = container(column!(
            top,
            if show_tools { self.toolbar() } else { empty() },
            content_area
        ))
        .clip(true)
        .style(styles::container::bb)
        .height(Length::Fill)
        .width(Length::Fill);

        content.into()
    }

    pub fn content(&self, now: Instant) -> Element<'_, HomeMessage> {
        match (&self.state, self.current_page()) {
            (State::Loading(animation), _) => center(loading_svg(animation, now)).into(),
            (State::Recent { shows, movies }, None) => self.recents(now, movies, shows),
            (State::Shows(shows), Some(Page::Shows(page))) => {
                page.view(now, shows.iter()).map(HomeMessage::Shows)
            }
            (State::Movies(movies), Some(Page::Movies(page))) => {
                page.view(now, movies.iter()).map(HomeMessage::Movies)
            }
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
                .view(now, season, episodes.iter(), memberships.iter())
                .map(HomeMessage::SeasonPage),
            (
                State::Show {
                    show,
                    seasons,
                    memberships,
                },
                Some(Page::Show { page, .. }),
            ) => page
                .view(now, show, seasons.iter(), memberships.iter())
                .map(HomeMessage::ShowPage),
            (
                State::Collection {
                    collection,
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
                .padding(3),
        )
        .style(styles::container::bw);

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
                    View::CollectionAdd(state) => {
                        draw_collection_add(state, self.collections.iter())
                    }
                    View::Rating { rating, .. } => draw_rating(rating),
                };

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        match &self.state {
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
        }
    }

    pub fn back(&mut self, now: Instant) -> Task<Message> {
        let update = PageUpdate {
            layout: self.layout,
            sort: self.sort,
            filters: self.filters,
        };

        let (task, id, limit) = match self.current_page.take() {
            Some(current) => {
                self.state = State::Loading(loading_animation(now));
                self.forward.push(current);

                match self.backward.pop() {
                    Some(new) => {
                        let page = self
                            .pages
                            .get_mut(&new)
                            .expect("Page cannot be in back without being recorded first");
                        self.current_page = Some(new);
                        page.page_update(update);
                        let task = page.update_scroll().map(|_| Message::None);

                        (task, fetch_kind(new), None)
                    }
                    None => (
                        self.update_scroll().map(|_| Message::None),
                        FetchId::Recents,
                        self.recent_limit,
                    ),
                }
            }
            None => {
                let Some(new) = self.backward.pop() else {
                    return Task::none();
                };

                self.state = State::Loading(loading_animation(now));
                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in back without being recorded first");
                self.current_page = Some(new);
                page.page_update(update);
                let task = page.update_scroll().map(|_| Message::None);

                (task, fetch_kind(new), None)
            }
        };

        let msg = Message::Fetch {
            id,
            filters: self.filters,
            sort: self.sort,
            limit,
            offset: None,
        };

        Task::batch([Task::done(msg), task])
    }

    pub fn forward(&mut self, now: Instant) -> Task<Message> {
        let update = PageUpdate {
            layout: self.layout,
            sort: self.sort,
            filters: self.filters,
        };

        let (task, id) = match self.current_page.take() {
            Some(current) => {
                self.backward.push(current);
                let Some(new) = self.forward.pop() else {
                    return Task::none();
                };

                self.state = State::Loading(loading_animation(now));
                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in forward without being recorded");

                self.current_page = Some(new);
                page.page_update(update);
                let task = page.update_scroll().map(|_| Message::None);

                (task, fetch_kind(new))
            }
            None => {
                let Some(new) = self.forward.pop() else {
                    return Task::none();
                };

                self.state = State::Loading(loading_animation(now));
                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in forward without being recorded");
                self.current_page = Some(new);
                page.page_update(update);
                let task = page.update_scroll().map(|_| Message::None);

                (task, fetch_kind(new))
            }
        };

        let msg = Message::Fetch {
            id,
            filters: self.filters,
            sort: self.sort,
            limit: None,
            offset: None,
        };

        Task::batch([Task::done(msg), task])
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

        let msg = Message::Fetch {
            id,
            filters: self.filters,
            sort: self.sort,
            limit,
            offset: None,
        };

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

        let update = PageUpdate {
            layout: self.layout,
            sort: self.sort,
            filters: self.filters,
        };

        if let Some(page) = self.current_page_mut() {
            page.page_update(update);
        };

        Task::none()
    }

    fn close_view(&mut self) -> Task<Message> {
        self.view.take();
        self.update_page_scroll()
    }

    pub fn goto(&mut self, kind: PageKind, now: Instant) -> Task<Message> {
        if let Some(current) = self.current_page
            && current == kind
        {
            self.view = None;
            return Task::none();
        }

        let close_view = self.close_view();
        self.backward.retain(|back| *back != kind);

        if let Some(old) = self.current_page.replace(kind) {
            self.backward.push(old)
        };
        self.forward.clear();
        self.state = State::Loading(loading_animation(now));

        let fid = fetch_kind(kind);

        let msg = Message::Fetch {
            id: fid,
            filters: self.filters,
            sort: self.sort,
            limit: None,
            offset: None,
        };

        if let Some(page) = self.pages.get_mut(&kind) {
            let update = PageUpdate {
                filters: self.filters,
                sort: self.sort,
                layout: self.layout,
            };
            page.page_update(update);

            let scroll = page.update_scroll().discard();

            let tsk = Task::done(msg).chain(scroll);

            return Task::batch([tsk, close_view]);
        }

        let task = match kind {
            PageKind::Movies => {
                let (movies, task) = Movies::boot(self.sort, self.filters, self.layout);

                self.pages.insert(kind, Page::Movies(movies));

                task.map(|msg| Message::Home(HomeMessage::Movies(msg)))
            }
            PageKind::Shows => {
                let (shows, tasks) = TvShows::boot(self.sort, self.filters, self.layout);

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
                let (show, task) = ShowPage::boot(id, self.sort, self.filters, self.layout);

                self.pages.insert(kind, Page::Show { id, page: show });

                task.map(|ssg| Message::Home(HomeMessage::ShowPage(ssg)))
            }
            PageKind::Season(id) => {
                let (season, task) = SeasonPage::boot(id, self.sort, self.filters, self.layout);

                self.pages.insert(kind, Page::Season { id, page: season });

                task.map(|ssg| Message::Home(HomeMessage::SeasonPage(ssg)))
            }
            PageKind::Collection(id) => {
                let (collection, tasks) =
                    CollectionPage::boot(id, self.sort, self.filters, self.layout);

                self.pages.insert(kind, Page::Collection { collection, id });

                tasks.map(|csg| Message::Home(HomeMessage::Collection(csg)))
            }
            PageKind::Collections => {
                let (collections, task) = Collections::boot(self.sort, self.filters, self.layout);

                self.pages.insert(kind, Page::Collections(collections));

                task.map(|csg| Message::Home(HomeMessage::Collections(csg)))
            }
        };

        Task::batch([Task::done(msg), close_view, task])
    }

    pub fn action(&mut self, action: HomeAction, now: Instant) -> Task<Message> {
        match action {
            HomeAction::LayoutToggle => self.layout_toggle(),
            HomeAction::RefreshContent => self.content_refresh(now),
            HomeAction::Refresh => self.refresh(now),
            HomeAction::SearchToggle => self.toggle_search(None),
            HomeAction::CloseModal => self.close_view(),
        }
    }

    pub fn fetched_recents(
        &mut self,
        movies: Vec<Thumbnail<Movie>>,
        shows: Vec<Thumbnail<Show>>,
    ) -> Task<Message> {
        let state = State::Recent { shows, movies };

        self.state = state;

        self.update_page_scroll()
    }

    pub fn fetch_collections_simple(
        &mut self,
        collections: Vec<SimpleCollection>,
    ) -> Task<Message> {
        self.collections = collections;
        self.update_page_scroll()
    }

    pub fn fetched_shows(&mut self, shows: Vec<Thumbnail<Show>>) -> Task<Message> {
        self.state = State::Shows(shows);

        self.update_page_scroll()
    }

    pub fn fetched_movies(&mut self, movies: Vec<Thumbnail<Movie>>) -> Task<Message> {
        self.state = State::Movies(movies);
        self.update_page_scroll()
    }

    pub fn fetched_show(
        &mut self,
        show: Thumbnail<Show>,
        seasons: Vec<Thumbnail<Season>>,
    ) -> Task<Message> {
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
        episodes: Vec<Thumbnail<Episode>>,
    ) -> Task<Message> {
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

    #[allow(clippy::type_complexity)]
    pub fn fetched_collection(
        &mut self,
        collection: Collection,
        items: (
            Vec<Thumbnail<Movie>>,
            Vec<Thumbnail<Show>>,
            Vec<Thumbnail<Season>>,
            Vec<Thumbnail<Episode>>,
        ),
    ) -> Task<Message> {
        Task::perform(
            async move {
                let collection = CollectionThumbnail::new(collection);
                (collection, items)
            },
            move |(collection, items)| {
                let (movies, shows, seasons, episodes) = items;
                Message::Home(HomeMessage::FetchedCollection {
                    collection: Box::new(collection),
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

    pub fn fetched_memberships(&mut self, fetched: Vec<SimpleCollection>) -> Task<Message> {
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

    pub fn toggle_search(&mut self, collection: Option<CollectionId>) -> Task<Message> {
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
}

fn icon_button<'a>(
    unicode: char,
    value: &'a str,
    message: HomeMessage,
    current: bool,
) -> Element<'a, HomeMessage> {
    let size = H6;
    let icon = icons::icon(unicode).size(size);
    let text = text(value).size(size);

    container(
        button(
            row!(icon, text)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(SIDE_ICON_SPACING),
        )
        .style(move |theme, status| {
            if current {
                styles::button::subtle_primary(theme, status)
            } else {
                styles::button::subtler(theme, status)
            }
        })
        .on_press(message),
    )
    .clip(true)
    .max_height(48.0)
    .into()
}

fn collection_button<'a>(
    icon: char,
    value: &'a str,
    view: char,
    message: HomeMessage,
    current: bool,
) -> Element<'a, HomeMessage> {
    let size = H6;
    let icon = icons::icon(icon).size(size);
    let text = container(text(value).size(size))
        .max_height(48.0)
        .clip(true);
    let view = icons::icon(view).size(size);

    button(
        row!(icon, text, view)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .spacing(SIDE_ICON_SPACING),
    )
    .style(move |theme, status| {
        if current {
            styles::button::subtle_primary(theme, status)
        } else {
            styles::button::subtler(theme, status)
        }
    })
    .on_press(message)
    .into()
}

pub fn view_unicode(view: CollectionView) -> char {
    match view {
        CollectionView::Shown => EYE,
        CollectionView::Pinned => PIN,
        CollectionView::Hidden => HIDE,
    }
}

fn draw_config(config: &CollectionConfig) -> Element<'_, HomeMessage> {
    let width = 550;
    let height = 550;
    let radius = 5.0;
    let padding = Padding::from([6, 6]);

    let icon_height = 40.0;
    let icon_width = 40.0;

    fn icon_btn<'a>(
        content: impl Into<Element<'a, HomeMessage>>,
        selected: bool,
        message: ConfigMessage,
        label: &'a str,
    ) -> Element<'a, HomeMessage> {
        let radius = 5.0;
        tooltip(
            button(content)
                .padding([0, 0])
                .on_press(HomeMessage::CollectionConfig(message))
                .style(move |theme, status| {
                    let default = if selected {
                        styles::button::subtle_primary(theme, status)
                    } else {
                        styles::button::subtle(theme, status)
                    };
                    let border = default.border.rounded(radius);

                    button::Style { border, ..default }
                }),
            label,
            tp::Position::Top,
        )
        .into()
    }

    let name = {
        let label = text("Name");

        let value = config.name.as_str();
        let is_empty = config.empty_name;

        let input = text_input("", value)
            .id(config.name_input.clone())
            .on_input(move |input| HomeMessage::CollectionConfig(ConfigMessage::Name(input)))
            .padding(padding)
            .style(move |theme: &Theme, status| {
                let error = theme.extended_palette().danger.strong.color;
                let default = text_input::default(theme, status);
                let border = default.border.rounded(radius);
                let border = if is_empty && matches!(status, text_input::Status::Focused { .. }) {
                    border.color(error)
                } else {
                    border
                };

                text_input::Style { border, ..default }
            })
            .width(Length::Fill);

        column!(label, input).spacing(2)
    };

    let description = {
        let label = text("Description");

        let content = &config.description;
        let editor = text_editor(content)
            .on_action(move |action| {
                HomeMessage::CollectionConfig(ConfigMessage::Description(action))
            })
            .padding(padding)
            .style(move |theme, status| {
                let default = text_editor::default(theme, status);
                let border = default.border.rounded(radius);

                text_editor::Style { border, ..default }
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
        .map(|view| {
            let unicode = view_unicode(view);

            let content = center(icon(unicode).size(P));
            let label = match view {
                CollectionView::Shown => "Shown",
                CollectionView::Pinned => "Pinned",
                CollectionView::Hidden => "Hidden",
            };

            icon_btn(content, view == selected, ConfigMessage::View(view), label)
        });

        let views = grid(views)
            .spacing(16)
            .fluid(icon_width)
            .height(grid::aspect_ratio(icon_width, icon_height));

        let views = container(views)
            .padding(padding)
            .style(move |theme: &Theme| {
                let color = theme.extended_palette().secondary.strong.color;
                let default = styles::container::transparent(theme);
                let border = default.border.rounded(radius).color(color).width(1.5);

                container::Style { border, ..default }
            });

        column!(label, views).spacing(2)
    };

    let icons = {
        let selected = config.icon;

        let label = text("Icon");

        let icons = Icon::all().into_iter().map(|value| {
            let content = center(icon(value.unicode()).size(P));

            icon_btn(
                content,
                value == selected,
                ConfigMessage::Icon(value),
                value.label(),
            )
        });

        let icons = grid(icons)
            .spacing(16)
            .fluid(icon_width)
            .height(grid::aspect_ratio(icon_width, icon_height));

        let icons = container(icons)
            .padding(padding)
            .style(move |theme: &Theme| {
                let color = theme.extended_palette().secondary.strong.color;
                let default = styles::container::transparent(theme);
                let border = default.border.rounded(radius).color(color).width(1.5);

                container::Style { border, ..default }
            });

        column!(label, icons).spacing(2)
    };

    let actions = {
        let save = button("Save")
            .on_press(HomeMessage::CollectionConfig(ConfigMessage::Save))
            .style(styles::button::primary);

        let cancel = button("Cancel")
            .on_press(HomeMessage::CloseView)
            .style(styles::button::primary);

        column!(row!(save, cancel).spacing(80))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
    };

    let content = column!(name, description, view, icons, space::vertical(), actions).spacing(16);

    modal_container(content).width(width).height(height).into()
}

fn fetch_kind(kind: PageKind) -> FetchId {
    match kind {
        PageKind::Shows => FetchId::Shows,
        PageKind::Movies => FetchId::Movies,
        PageKind::Collections => FetchId::Collections,
        PageKind::Show(id) => FetchId::Show(id),
        PageKind::Season(id) => FetchId::Season(id),
        PageKind::Episode(id) => FetchId::Episode(id),
        PageKind::Movie(id) => FetchId::Movie(id),
        PageKind::Collection(id) => FetchId::Collection(id),
    }
}

fn modal_container<'a>(content: impl Into<Element<'a, HomeMessage>>) -> Container<'a, HomeMessage> {
    container(content)
        .padding([8, 12])
        .style(|theme| {
            let default = styles::container::bw(theme);
            let border = default.border.rounded(5.0);

            container::Style { border, ..default }
        })
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
}

fn draw_search<'a, F: Fn(ItemId) -> HomeMessage + Clone>(
    state: &'a SearchState,
    primary: F,
    theme: &Theme,
    set_play: bool,
) -> Element<'a, HomeMessage> {
    let items = state.items.iter().map(|item| {
        item.view(
            theme,
            HomeMessage::Play,
            primary.clone(),
            |_| HomeMessage::None,
            set_play,
        )
    });

    let input = {
        let filter: Element<'_, HomeMessage> = match state.filter {
            Some(filter) => {
                let content = row!(filter.to_str(), icon(CANCEL))
                    .align_y(Vertical::Center)
                    .spacing(4.0);

                button(content)
                    .on_press(HomeMessage::SearchMessage(SearchMessage::ClearFilter))
                    .style(|theme, status| {
                        let default = styles::button::subtle_primary(theme, status);
                        let border = default.border.rounded(5);

                        button::Style { border, ..default }
                    })
                    .into()
            }
            None => empty(),
        };

        let size = H6;
        let icon = text_input::Icon {
            font: icons::FONT,
            code_point: icons::SEARCH,
            side: text_input::Side::Right,
            size: Some(size.into()),
            spacing: 5.0,
        };
        let input = text_input("Search Media", &state.search)
            .id(state.text_input.clone())
            .size(size)
            .icon(icon)
            .on_input(|search| HomeMessage::SearchMessage(SearchMessage::Search(search)))
            .on_submit(HomeMessage::SearchMessage(SearchMessage::Load));

        row!(filter, input).spacing(10.0).align_y(Vertical::Center)
    };

    let content = column!(input).extend(items).spacing(16.0);

    modal_container(content)
        .max_width(550)
        .height(Length::Shrink)
        .into()
}

fn sort_collections(collections: &mut [SimpleCollection]) {
    collections.sort_by(|x, y| {
        x.view.cmp(&y.view).then(alphanumeric_sort::compare_str(
            x.name.to_lowercase(),
            y.name.to_lowercase(),
        ))
    });
}

fn draw_collection_add<'a>(
    state: &'a CollectionAddState,
    collections: impl Iterator<Item = &'a SimpleCollection>,
) -> Element<'a, HomeMessage> {
    let title = text("Collections").size(H6);

    fn btn(collection: &SimpleCollection, selected: bool) -> Element<'_, HomeMessage> {
        let size = P;
        let unicode = Icon::new(collection.icon).unicode();
        let icon = icons::icon(unicode).size(size);
        let text = container(text(&collection.name).size(size))
            .max_height(48.0)
            .max_width(275);
        let check = checkbox("", selected).on_toggle(|value| {
            HomeMessage::CollectionAdd(CollectionAddMessage::Toggle(!value, collection.id))
        });

        button(
            row!(icon, text, space::horizontal(), check)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(8.0),
        )
        .padding([8, 12])
        .on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Toggle(
            selected,
            collection.id,
        )))
        .style(move |theme, status| {
            let default = if selected {
                styles::button::subtle(theme, status)
            } else {
                styles::button::subtlest(theme, status)
            };

            let border = default.border.rounded(5.0);

            button::Style { border, ..default }
        })
        .into()
    }

    let collections = column(
        collections.map(|collection| btn(collection, state.selected.contains(&collection.id))),
    )
    .spacing(8.0);

    let collections = scrollable(collections).spacing(16.0);

    let collections = container(collections)
        .padding([6, 8])
        .style(|theme: &Theme| {
            let color = theme.extended_palette().secondary.strong.color;
            let default = styles::container::transparent(theme);
            let border = default.border.rounded(5).color(color).width(1.5);

            container::Style { border, ..default }
        });

    let new = button(
        row!(icons::icon(icons::ADD).size(H7), text("New").size(H7))
            .align_y(Vertical::Center)
            .spacing(8),
    )
    .padding([2, 4])
    .on_press(HomeMessage::NewCollection)
    .style(move |theme, status| {
        let default = styles::button::text(theme, status);

        let border = default.border.rounded(5.0);

        button::Style { border, ..default }
    });

    let collections = column!(new, collections)
        .spacing(5.0)
        .align_x(Horizontal::Right);

    let actions = {
        let save = button("Save")
            .on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Save))
            .style(styles::button::primary);

        let cancel = button("Cancel")
            .on_press(HomeMessage::CloseView)
            .style(styles::button::primary);

        row!(save, cancel).spacing(100)
    };

    let content = column!(title, collections, actions)
        .spacing(24)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(400).into()
}

fn draw_rating<'a>(state: &Rating) -> Element<'a, HomeMessage> {
    let title = text("Rating").size(H4);

    let size = H6;

    let value: Element<'_, HomeMessage> = {
        let size = H6;
        let extra = text("/5").size(H7);

        let value: Element<'_, HomeMessage> = match state {
            Rating::Value(value) => {
                let rating = (value * 100.0).round() / 100.0;
                mouse_area(text(format!("{rating:.2}")).size(size))
                    .interaction(mouse::Interaction::Text)
                    .on_press(HomeMessage::Rating(RatingMessage::Type))
                    .into()
            }
            Rating::Input { id, input } => text_input("", input)
                .id(id.clone())
                .size(size)
                .width(48.0)
                .on_submit(HomeMessage::Rating(RatingMessage::Submit))
                .on_input(|input| HomeMessage::Rating(RatingMessage::Input(input)))
                .into(),
        };

        row!(value, extra)
            .spacing(2.0)
            .align_y(Vertical::Center)
            .into()
    };

    let ratings = {
        let rating = match state {
            Rating::Value(value) => *value,
            Rating::Input { input, .. } => input.parse::<f32>().unwrap_or_default().clamp(0.0, 5.0),
        };

        let stars = (rating.trunc() as u8).clamp(0, 5);
        let rem = 5 - stars;
        let frac = rating.fract() >= 0.5;
        let unstars = if frac { rem.saturating_sub(1) } else { rem };
        let frac = rem - unstars;

        let color = |theme: &Theme| -> text::Style {
            let color = theme.extended_palette().primary.strong.color;
            text::Style { color: Some(color) }
        };

        let stars = (0..stars).map(|_| Element::from(icon(STAR).size(size).style(color)));
        let frac = (0..frac).map(|_| Element::from(icon(HALF_STAR).size(size).style(color)));
        let unstars = (0..unstars).map(|_| Element::from(icon(UNSTAR).size(size).style(color)));

        let stars = stars
            .chain(frac)
            .chain(unstars)
            .enumerate()
            .map(|(idx, elem)| {
                Element::from(
                    button(elem)
                        .on_press(HomeMessage::Rating(RatingMessage::Star((idx + 1) as u8)))
                        .padding(0)
                        .style(styles::button::text),
                )
            });

        row(stars).spacing(6.0).align_y(Vertical::Center)
    };

    let content = column!(title, value, ratings)
        .spacing(16.0)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(400).into()
}
