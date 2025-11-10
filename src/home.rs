use iced::{
    Element, Length, Padding, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    animation::Animation,
    border::{Border, Radius},
    time::{Duration, Instant},
    widget::{
        self, Container, button, center, column, container, grid,
        operation::{self, scroll_to},
        pick_list, row, rule, scrollable, space, text, text_editor, text_input,
    },
    window,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    self, Collection, CollectionId, CollectionView, Episode, EpisodeId, Movie, MovieId, SearchItem,
    Season, SeasonId, Show, ShowId, collection::ItemId,
};

use crate::app::{FetchId, Message};
use crate::models::Media;
use crate::utils::{
    self, HomeAction, Layout, PlayId, PlayItem, Sort, SortKind, empty, filter::*, icons, icons::*,
    load_fonts, loading_animation, loading_svg, typo::*,
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
    CARD_HEIGHT, CARD_WIDTH, CollectionThumbnail, Scroll, SearchView, Thumbnail, filter_sort,
};
use shows::{TvShows, TvShowsMessage};

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
    Cancel,
    Save,
}

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub name: String,
    pub description: text_editor::Content,
    pub icon: Icon,
    pub view: CollectionView,
    pub theme: Option<u32>,
    pub custom: Option<String>,
}

impl CollectionConfig {
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

#[derive(Debug, Clone)]
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
    Hovered(ItemId, bool),
    Searching,
    ClearFilter,
}

#[derive(Debug, Clone)]
pub struct CollectionAddState {
    item: ItemId,
    selected: HashSet<CollectionId>,
}

#[derive(Debug, Clone)]
pub enum CollectionAddMessage {
    Toggle(bool, CollectionId),
    Save,
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    CollectionConfig(CollectionId),
    Add(ItemId),
    AddToCollection(CollectionId),
    Search,
}

#[derive(Debug, Clone)]
pub enum View {
    CollectionConfig(CollectionConfig, CollectionId),
    Search(SearchState, Option<CollectionId>),
    CollectionAdd(CollectionAddState),
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
    Collections,
    Movie(Thumbnail<Movie>),
    Show {
        show: Thumbnail<Show>,
        seasons: Vec<Thumbnail<Season>>,
    },
    Season {
        season: Thumbnail<Season>,
        episodes: Vec<Thumbnail<Episode>>,
    },
    Episode(Thumbnail<Episode>),
    Collection {
        id: CollectionId,
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
    OpenView(ViewMessage),
    CloseView,
    Play(ItemId),
    PlayCollection {
        id: CollectionId,
        items: collection::Items,
    },
    ToggleLayout,
    Home,
    Goto(PageKind),
    NewCollection,
    None,
    Scroll(scrollable::Viewport),
    RefreshContent,
    Refresh,
    Hovered(ItemId, bool),
    HoveredCollection(CollectionId, bool),
    FetchedCollections(bool, Vec<CollectionThumbnail>),
}

pub struct Home {
    forward: Vec<PageKind>,
    backward: Vec<PageKind>,
    current_page: Option<PageKind>,
    pages: HashMap<PageKind, Page>,

    state: State,

    collections: Vec<CollectionThumbnail>,

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
            id: FetchId::Collections(false),
            filters,
            sort,
            limit: None,
            offset: None,
        });

        let tasks = collections.chain(recents);
        // let tasks = Task::batch([collections, recents]);
        // let tasks = recents;
        // let tasks = Task::none();

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
            HomeMessage::Settings => Task::none(),
            HomeMessage::FetchedCollections(state, collections) => {
                self.collections = collections;
                sort_collections(&mut self.collections);
                if state {
                    self.state = State::Collections;
                }

                Task::none()
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
            HomeMessage::Goto(kind) => {
                if let Some(current) = self.current_page
                    && current == kind
                {
                    return Task::none();
                }

                self.view = None;
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

                    return Task::done(msg).chain(scroll);
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
                        let (season, task) =
                            SeasonPage::boot(id, self.sort, self.filters, self.layout);

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
                        let (collections, task) =
                            Collections::boot(self.sort, self.filters, self.layout);

                        self.pages.insert(kind, Page::Collections(collections));
                        self.state = State::Collections;

                        return task.map(|csg| Message::Home(HomeMessage::Collections(csg)));
                    }
                };

                Task::batch([Task::done(msg), task])
            }
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
                ViewMessage::CollectionConfig(key) => {
                    let Some(collection) = self
                        .collections
                        .iter()
                        .find(|thumbnail| thumbnail.collection.id == key)
                    else {
                        return Task::none();
                    };

                    let description = text_editor::Content::with_text(
                        collection
                            .collection
                            .description
                            .as_deref()
                            .unwrap_or_default(),
                    );

                    let config = CollectionConfig {
                        name: collection.collection.name.clone(),
                        description,
                        view: collection.collection.view,
                        icon: Icon::new(collection.collection.icon),
                        theme: collection.collection.theme,
                        custom: collection.collection.custom.clone(),
                    };

                    self.view = Some(View::CollectionConfig(config, collection.collection.id));

                    Task::none()
                }
                ViewMessage::Add(item) => {
                    let state = CollectionAddState {
                        item,
                        selected: HashSet::new(),
                    };
                    self.view = Some(View::CollectionAdd(state));

                    Task::done(Message::FetchMemberShip(item))
                }
                ViewMessage::AddToCollection(id) => self.toggle_search(Some(id)),
                ViewMessage::Search => self.toggle_search(None),
            },
            HomeMessage::CollectionConfig(csg) => {
                let Some(View::CollectionConfig(mut config, id)) = self.view.take() else {
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
                        self.view = None;
                        return Task::none();
                    }
                    ConfigMessage::Save => {
                        let Some(collection) = self
                            .collections
                            .iter_mut()
                            .find(|thumbnail| thumbnail.collection.id == id)
                        else {
                            return Task::none();
                        };

                        config.update(&mut collection.collection);

                        if let State::Collection { id: key, .. } = &mut self.state {
                            *key = collection.collection.id;
                        }

                        sort_collections(&mut self.collections);
                        self.view = None;

                        // todo: Save in DB
                        return Task::none();
                    }
                }

                self.view = Some(View::CollectionConfig(config, id));

                Task::none()
            }
            HomeMessage::SearchMessage(ssg) => {
                let Some(View::Search(state, _)) = self.view.as_mut() else {
                    return Task::none();
                };

                match ssg {
                    SearchMessage::Hovered(id, is_hovered) => {
                        if let Some(item) = state.items.iter_mut().find(|view| view.item.id == id) {
                            item.animation.go_mut(is_hovered, now);
                        };

                        Task::none()
                    }
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
                        Task::none()
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
                let Some(View::CollectionAdd(state)) = self.view.as_mut() else {
                    return Task::none();
                };

                match csg {
                    CollectionAddMessage::Toggle(selected, id) => {
                        if selected {
                            state.selected.remove(&id);
                        } else {
                            state.selected.insert(id);
                        }

                        Task::none()
                    }
                    CollectionAddMessage::Save => todo!(),
                }
            }
            HomeMessage::CloseView => {
                self.view = None;
                Task::none()
            }
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
                Task::none()
            }
            HomeMessage::NewCollection => Task::none(),
            HomeMessage::Random => {
                todo!("Random");
            }
            HomeMessage::RefreshContent => self.content_refresh(now),
            HomeMessage::Refresh => self.refresh(now),
            HomeMessage::Play(item) => {
                self.view = None;
                self.play(item)
            }
            HomeMessage::PlayCollection { id, items } => {
                println!("Play {items:?} from collection {id:?}");
                todo!()
            }
            HomeMessage::HoveredCollection(key, is_hovered) => {
                let State::Collections = &mut self.state else {
                    return Task::none();
                };

                let Some(collection) = self
                    .collections
                    .iter_mut()
                    .find(|thumbnail| thumbnail.collection.id == key)
                else {
                    return Task::none();
                };

                collection.zoom.go_mut(is_hovered, now);

                Task::none()
            }
            HomeMessage::Hovered(id, is_hovered) => match (&mut self.state, id) {
                (State::Loading(_), _)
                | (State::Episode(_), _)
                | (State::Movie(_), _)
                | (State::Collections, _) => Task::none(),
                (State::Recent { shows, .. }, ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.zoom.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Recent { movies, .. }, ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.zoom.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Recent { .. }, _) => Task::none(),
                (State::Shows(shows), ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.zoom.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Shows(_), _) => Task::none(),
                (State::Movies(movies), ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.zoom.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Movies(_), _) => Task::none(),
                (State::Show { seasons, .. }, ItemId::Season(id)) => {
                    if let Some(season) = seasons.iter_mut().find(|season| season.media.id == id) {
                        season.zoom.go_mut(is_hovered, now);
                    }
                    Task::none()
                }
                (State::Show { .. }, _) => Task::none(),
                (State::Season { episodes, .. }, ItemId::Episode(id)) => {
                    if let Some(episode) =
                        episodes.iter_mut().find(|episode| episode.media.id == id)
                    {
                        episode.zoom.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Season { .. }, _) => Task::none(),
                (State::Collection { shows, .. }, ItemId::Show(id)) => {
                    if let Some(show) = shows.iter_mut().find(|show| show.media.id == id) {
                        show.zoom.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Collection { movies, .. }, ItemId::Movie(id)) => {
                    if let Some(movie) = movies.iter_mut().find(|movie| movie.media.id == id) {
                        movie.zoom.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
                (State::Collection { seasons, .. }, ItemId::Season(id)) => {
                    if let Some(season) = seasons.iter_mut().find(|show| show.media.id == id) {
                        season.zoom.go_mut(is_hovered, now);
                    };

                    Task::none()
                }
                (State::Collection { episodes, .. }, ItemId::Episode(id)) => {
                    if let Some(episode) =
                        episodes.iter_mut().find(|episode| episode.media.id == id)
                    {
                        episode.zoom.go_mut(is_hovered, now);
                    }

                    Task::none()
                }
            },
            HomeMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                Task::none()
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

    fn play(&self, item: ItemId) -> Task<Message> {
        match (&self.state, item) {
            (State::Loading(_), _) => Task::none(),
            (State::Recent { movies, .. }, ItemId::Movie(id)) => {
                let Some(movie) = movies.iter().find(|movie| movie.media.id == id) else {
                    return Task::none();
                };

                let name = movie.media.name();
                let path = &movie.media.full_path;

                match play_item(PlayId::Movie(id), name, path) {
                    Ok(item) => Task::done(Message::PlayItem(item)),
                    Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                }
            }
            (State::Season { episodes, .. }, ItemId::Episode(id)) => {
                let Some(episode) = episodes.iter().find(|episode| episode.media.id == id) else {
                    return Task::none();
                };

                let name = episode.media.name();
                let path = &episode.media.full_path;

                match play_item(PlayId::Episode(id), name, path) {
                    Ok(item) => Task::done(Message::PlayItem(item)),
                    Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                }
            }
            (State::Movie(movie), ItemId::Movie(id)) => {
                let name = movie.media.name();
                let path = &movie.media.full_path;

                match play_item(PlayId::Movie(id), name, path) {
                    Ok(item) => Task::done(Message::PlayItem(item)),
                    Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                }
            }
            (State::Episode(episode), ItemId::Episode(id)) => {
                let name = episode.media.name();
                let path = &episode.media.full_path;

                match play_item(PlayId::Episode(id), name, path) {
                    Ok(item) => Task::done(Message::PlayItem(item)),
                    Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                }
            }
            unreached => todo!("Needs rework after search implementation {unreached:?}"),
        }
    }

    fn update_scroll(&mut self) -> Task<()> {
        scroll_to(self.scroll.id.clone(), self.scroll.offset)
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

        let collections =
            self.collections
                .iter()
                .filter_map(|collection| match collection.collection.view {
                    CollectionView::Pinned => {
                        let unicode = Icon::new(collection.collection.icon).unicode();
                        let content = collection_button(
                            unicode,
                            &collection.collection.name,
                            view_unicode(collection.collection.view),
                            HomeMessage::Goto(PageKind::Collection(collection.collection.id)),
                            self.current_page()
                                .map(|page| page.is_collection(&collection.collection.id))
                                .unwrap_or_default(),
                        );

                        Some(content)
                    }
                    CollectionView::Shown => {
                        let unicode = Icon::new(collection.collection.icon).unicode();
                        let content = icon_button(
                            unicode,
                            &collection.collection.name,
                            HomeMessage::Goto(PageKind::Collection(collection.collection.id)),
                            self.current_page()
                                .map(|page| page.is_collection(&collection.collection.id))
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
            let label = column!(label, rule::horizontal(2.0)).spacing(4.0);

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
            };

            column!(label, movies).spacing(10.0)
        };

        let shows = {
            let label = text("Recent Shows").size(H4);
            let label = column!(label, rule::horizontal(2.0)).spacing(4.0);

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
        let size = H7;
        let padding = Padding::new(2.0).left(5.0).right(5.0);

        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);
        let comp = |icon: char, msg: FilterMessage| {
            icons::sized_button(icon, size)
                .padding([5, 5])
                .style(button::background)
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
                .spacing(5.0)
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
                .spacing(5.0)
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
                .spacing(5.0)
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
                .spacing(5.0)
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
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let mode = {
            let mode = text(self.filters.mode.to_string()).size(size);
            let text = text("Combination mode:").size(size);

            let button = button(mode)
                .style(button::background)
                .padding(padding)
                .on_press(HomeMessage::Filter(FilterMessage::Mode));

            row!(text, button).spacing(5.0).align_y(Vertical::Center)
        };

        let clear = button(text("Clear").size(size))
            .padding(padding)
            .style(button::text)
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
        .spacing(10.0)
        .align_y(Vertical::Center)
        .wrap();

        let content = column!(text("Filters").size(size), content).spacing(5.0);

        content.into()
    }

    fn sort_view(&self) -> Element<'_, HomeMessage> {
        let size = H7;
        let vertical_rule = || container(rule::vertical(2.0)).height(20.0);

        let clear = button(text("Clear").size(size))
            .padding([2, 5])
            .style(button::text)
            .on_press(HomeMessage::Sort(SortMessage::Clear));

        let reverse = button(text("Reverse").size(size))
            .padding([2, 5])
            .style(button::text)
            .on_press(HomeMessage::Sort(SortMessage::ToggleReverse));

        let base = row!(text("More").size(size), icon(ELLIPSIS_VER).size(size))
            .spacing(2.0)
            .align_y(Vertical::Center);

        let hidden = {
            container(
                column(SortKind::HIDDEN.iter().map(|kind| {
                    let order = self.sort.position(*kind);

                    kind.view(
                        |kind| HomeMessage::Sort(SortMessage::AddSort(kind)),
                        |kind| HomeMessage::Sort(SortMessage::RemoveSort(kind)),
                        order,
                    )
                }))
                .spacing(8),
            )
            .style(container::bordered_box)
            .padding([3, 6])
        };

        let more = menu(base, hidden)
            .auto_close(false)
            .position(Position::Right)
            .on_toggle(|_| HomeMessage::None);

        row!(
            text("Sort by: ").size(size),
            row(SortKind::VISIBLE.iter().map(|sort| {
                let order = self.sort.position(*sort);

                sort.view(
                    |kind| HomeMessage::Sort(SortMessage::AddSort(kind)),
                    |kind| HomeMessage::Sort(SortMessage::RemoveSort(kind)),
                    order,
                )
            }))
            .spacing(5.0),
            vertical_rule(),
            more,
            vertical_rule(),
            reverse,
            clear,
        )
        .align_y(Vertical::Center)
        .spacing(10.0)
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

            button(content)
                .style(if self.filters.is_any() {
                    button::subtle
                } else {
                    button::background
                })
                .on_press(HomeMessage::ToggleFilter)
                .padding([5, 5])
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

            button(content)
                .style(if self.sort.is_empty() {
                    button::subtle
                } else {
                    button::background
                })
                .on_press(HomeMessage::ToggleSort)
                .padding([5, 5])
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
            icons::sized_button(icons::REFRESH, size).on_press(HomeMessage::RefreshContent),
            icons::sized_button(icons::RAND, size).on_press(HomeMessage::Random),
            icons::sized_button(self.layout.icon(), size).on_press(HomeMessage::ToggleLayout),
        )
        .align_y(Vertical::Center)
        .spacing(5.0);

        let tools = row!(left, space::horizontal(), right).width(Length::Fill);

        let sorts_rule = if self.show_sorts {
            rule::horizontal(2.0).into()
        } else {
            empty()
        };

        let filters_rule = if self.show_filters {
            rule::horizontal(2.0).into()
        } else {
            empty()
        };

        let tools = column!(tools, sorts_rule, curr_sorts, filters_rule, curr_filters)
            .width(Length::Fill)
            .spacing(5.0)
            .padding(Padding::default().top(5.0).right(5.0).bottom(8.0).left(5.0));

        let content = container(tools).width(Length::Fill).style(container_style);

        content.into()
    }

    fn content_area(&self, now: Instant) -> Element<'_, HomeMessage> {
        let title = match &self.state {
            State::Recent { .. } => "Recents",
            State::Shows(_) => "Shows",
            State::Movies(_) => "Movies",
            State::Show { show, .. } => show.media.name(),
            State::Movie(movie) => movie.media.name(),
            State::Season { season, .. } => season.media.name(),
            State::Episode(episode) => episode.media.name(),
            State::Loading(_) => "Loading",
            State::Collections => "Collections",
            State::Collection { id, .. } => self
                .collections
                .iter()
                .find(|thumbnail| thumbnail.collection.id == *id)
                .map(|collection| collection.collection.name.as_str())
                .unwrap_or_default(),
        };
        let title = container(text(title).size(H6)).height(30);

        let search =
            sized_button(icons::SEARCH, H6).on_press(HomeMessage::OpenView(ViewMessage::Search));

        let top = container(
            row!(
                self.navigation(),
                space::horizontal(),
                title,
                space::horizontal(),
                search,
            )
            .padding(Padding::ZERO.right(5))
            .align_y(Vertical::Center)
            .height(H2 * 1.50)
            .width(Length::Fill),
        )
        .style(container_style);

        let content_area = container(self.content(now))
            .style(container_style)
            .height(Length::Fill)
            .width(Length::Fill);

        let show_tools = self
            .current_page()
            .map(|page| page.show_tools())
            .unwrap_or(true);

        let content = column!(
            top,
            if show_tools { self.toolbar() } else { empty() },
            content_area
        )
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
            (State::Collections, Some(Page::Collections(page))) => page
                .view(now, self.collections.iter())
                .map(HomeMessage::Collections),
            (State::Collections, None) => center("Loading").into(),
            (State::Episode(episode), Some(Page::Episode { page, .. })) => {
                page.view(episode).map(HomeMessage::EpisodePage)
            }
            (State::Movie(movie), Some(Page::Movie { page, .. })) => {
                page.view(movie).map(HomeMessage::MoviePage)
            }
            (State::Season { season, episodes }, Some(Page::Season { page, .. })) => page
                .view(now, season, episodes.iter())
                .map(HomeMessage::SeasonPage),
            (State::Show { show, seasons }, Some(Page::Show { page, .. })) => page
                .view(now, show, seasons.iter())
                .map(HomeMessage::ShowPage),
            (
                State::Collection {
                    id,
                    shows,
                    movies,
                    seasons,
                    episodes,
                },
                Some(Page::Collection {
                    collection: page, ..
                }),
            ) => {
                let collection = self
                    .collections
                    .iter()
                    .find(|thumbnail| thumbnail.collection.id == *id)
                    .expect("Trying to display an unrecorded collection");

                page.view(
                    now,
                    collection,
                    movies.iter().peekable(),
                    shows.iter().peekable(),
                    seasons.iter().peekable(),
                    episodes.iter().peekable(),
                )
                .map(HomeMessage::Collection)
            }
            unreached => {
                todo!("{unreached:?}")
            }
        }
    }

    pub fn view(&self, theme: &Theme, now: Instant) -> Element<'_, HomeMessage> {
        let content = row!(self.side(), self.content_area(now))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([6, 5]);

        match &self.view {
            None => content.into(),
            Some(View::CollectionConfig(config, _id)) => {
                let overlay = draw_config(config);

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
            }
            Some(View::Search(state, None)) => {
                let overlay =
                    draw_search(state, |id| HomeMessage::Goto(id.into()), theme, now, true);

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
            }
            Some(View::Search(state, Some(_collection))) => {
                // todo
                let overlay = draw_search(state, |_| HomeMessage::None, theme, now, false);

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
            }
            Some(View::CollectionAdd(state)) => {
                let overlay = draw_collection_add(
                    state,
                    self.collections
                        .iter()
                        .map(|thumbnail| &thumbnail.collection),
                );

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
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
            State::Collections => self
                .collections
                .iter()
                .any(|collection| collection.is_animating(now)),
            State::Movie(_) | State::Episode(_) => false,
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

        let searching = match &self.view {
            Some(View::Search(state, _)) => state.items.iter().any(|item| item.is_animating(now)),
            _ => false,
        };

        searching || state
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

                        if matches!(new, PageKind::Collections) {
                            self.state = State::Collections;
                            return task;
                        }

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

                if matches!(new, PageKind::Collections) {
                    self.state = State::Collections;
                    return task;
                }

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

                if matches!(new, PageKind::Collections) {
                    self.state = State::Collections;
                    return task;
                }

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

                if matches!(new, PageKind::Collections) {
                    self.state = State::Collections;
                    return task;
                }

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

    fn content_refresh(&mut self, now: Instant) -> Task<Message> {
        let (id, limit) = match &self.state {
            State::Loading(_) => return Task::none(),
            State::Recent { .. } => (FetchId::Recents, self.recent_limit),
            State::Movies(_) => (FetchId::Movies, None),
            State::Shows(_) => (FetchId::Shows, None),
            State::Show { show, .. } => (FetchId::Show(show.media.id), None),
            State::Season { season, .. } => (FetchId::Season(season.media.id), None),
            State::Episode(episode) => (FetchId::Episode(episode.media.id), None),
            State::Movie(movie) => (FetchId::Movie(movie.media.id), None),
            State::Collections => (FetchId::Collections(true), None),
            State::Collection { id, .. } => (FetchId::Collection(*id), None),
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
            id: FetchId::Collections(false),
            filters: self.filters,
            sort: self.sort,
            limit: None,
            offset: None,
        };
        let rsg = Task::done(rsg);

        Task::batch([rsg, self.content_refresh(now)])
    }

    fn layout_toggle(&mut self) -> Task<Message> {
        if self.layout == Layout::Grid {
            self.layout = Layout::List
        } else {
            self.layout = Layout::Grid
        }

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

    pub fn action(&mut self, action: HomeAction, now: Instant) -> Task<Message> {
        match action {
            HomeAction::LayoutToggle => self.layout_toggle(),
            HomeAction::RefreshContent => self.content_refresh(now),
            HomeAction::Refresh => self.refresh(now),
            HomeAction::SearchToggle => self.toggle_search(None),
        }
    }

    pub fn fetched_recents(&mut self, movies: Vec<Thumbnail<Movie>>, shows: Vec<Thumbnail<Show>>) {
        let state = State::Recent { shows, movies };

        self.state = state;
    }

    pub fn fetched_shows(&mut self, shows: Vec<Thumbnail<Show>>) {
        self.state = State::Shows(shows)
    }

    pub fn fetched_movies(&mut self, movies: Vec<Thumbnail<Movie>>) {
        self.state = State::Movies(movies)
    }

    pub fn fetched_show(&mut self, show: Thumbnail<Show>, seasons: Vec<Thumbnail<Season>>) {
        self.state = State::Show { show, seasons }
    }

    pub fn fetched_movie(&mut self, movie: Thumbnail<Movie>) {
        self.state = State::Movie(movie)
    }

    pub fn fetched_season(&mut self, season: Thumbnail<Season>, episodes: Vec<Thumbnail<Episode>>) {
        self.state = State::Season { season, episodes }
    }

    pub fn fetched_episode(&mut self, episode: Thumbnail<Episode>) {
        self.state = State::Episode(episode)
    }

    pub fn fetched_collections(
        &mut self,
        collections: Vec<Collection>,
        state: bool,
    ) -> Task<Message> {
        Task::perform(
            async move {
                collections
                    .into_iter()
                    .map(CollectionThumbnail::new)
                    .collect::<Vec<_>>()
            },
            move |collections| Message::Home(HomeMessage::FetchedCollections(state, collections)),
        )
    }

    pub fn fetched_collection(
        &mut self,
        id: CollectionId,
        items: (
            Vec<Thumbnail<Movie>>,
            Vec<Thumbnail<Show>>,
            Vec<Thumbnail<Season>>,
            Vec<Thumbnail<Episode>>,
        ),
    ) -> Task<Message> {
        use models::collection::Item;
        let (movies, shows, seasons, episodes) = items;

        self.state = State::Collection {
            id,
            shows,
            movies,
            seasons,
            episodes,
        };

        Task::none()
    }

    pub fn fetched_memberships(&mut self, memberships: Vec<CollectionId>) {
        if let Some(View::CollectionAdd(CollectionAddState { selected, .. })) = self.view.as_mut() {
            selected.extend(memberships.into_iter());
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

    pub fn loaded_search(&mut self, items: Vec<SearchView>) {
        let Some(View::Search(state, _)) = self.view.as_mut() else {
            return;
        };

        state.items = items
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

        operation::focus(text_input)
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
                .spacing(16.0),
        )
        .style(move |theme, status| {
            use button::{Status, Style, background};
            let default = background(theme, status);

            match status {
                Status::Active if current => {
                    let background = theme.extended_palette().background.weakest;
                    Style {
                        background: Some(background.color.into()),
                        text_color: background.text,
                        ..default
                    }
                }
                _ => default,
            }
        })
        .on_press(message),
    )
    .max_height(60.0)
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
    let text = container(text(value).size(size)).max_height(60.0);
    let view = icons::icon(view).size(size);

    button(
        row!(icon, text, view)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .spacing(16.0),
    )
    .style(move |theme, status| {
        use button::{Status, Style, background};
        let default = background(theme, status);

        match status {
            Status::Active if current => {
                let background = theme.extended_palette().background.weakest;
                Style {
                    background: Some(background.color.into()),
                    text_color: background.text,
                    ..default
                }
            }
            _ => default,
        }
    })
    .on_press(message)
    .into()
}

fn container_style(theme: &Theme) -> container::Style {
    let style = container::bordered_box(theme);
    let border = Border {
        radius: Radius::default(),
        ..style.border
    };

    container::Style { border, ..style }
}

pub fn view_unicode(view: CollectionView) -> char {
    match view {
        CollectionView::Shown => EYE,
        CollectionView::Pinned => PIN,
        CollectionView::Hidden => HIDE,
    }
}

fn view_draw<'a>(view: CollectionView, selected: bool) -> Element<'a, HomeMessage> {
    let unicode = view_unicode(view);

    let content = center(icon(unicode).size(P));

    button(content)
        .on_press(HomeMessage::CollectionConfig(ConfigMessage::View(view)))
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

fn icon_draw<'a>(value: Icon, selected: bool) -> Element<'a, HomeMessage> {
    let content = center(icon(value.unicode()).size(P));

    button(content)
        .on_press(HomeMessage::CollectionConfig(ConfigMessage::Icon(value)))
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

fn draw_config(config: &CollectionConfig) -> Element<'_, HomeMessage> {
    let width = 500;
    let height = 500;

    let icon_height = 40.0;
    let icon_width = 40.0;

    let name = {
        let label = text("Name");

        let value = config.name.as_str();

        let input = text_input("", value)
            .on_input(move |input| HomeMessage::CollectionConfig(ConfigMessage::Name(input)))
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
        .map(|view| view_draw(view, view == selected));

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
            .map(|icon| icon_draw(icon, icon == selected));

        let icons = grid(icons)
            .spacing(16)
            .fluid(icon_width)
            .height(grid::aspect_ratio(icon_width, icon_height));

        column!(label, icons).spacing(2)
    };

    let actions = {
        let save = button("Save").on_press(HomeMessage::CollectionConfig(ConfigMessage::Save));

        let cancel =
            button("Cancel").on_press(HomeMessage::CollectionConfig(ConfigMessage::Cancel));

        column!(row!(save, cancel).spacing(80))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
    };

    let content = column!(name, description, view, icons, space::vertical(), actions).spacing(16);

    modal_container(content).width(width).height(height).into()
}

fn play_item(id: PlayId, name: &str, path: &Path) -> Result<PlayItem, String> {
    match path.try_exists() {
        Ok(false) => Err(format!("{} does not exist", path.to_string_lossy())),
        Err(error) => Err(format!("{error}")),
        Ok(true) => {
            let play = PlayItem {
                id,
                name: name.to_owned(),
                path: path.to_path_buf(),
            };

            Ok(play)
        }
    }
}

fn play_helper(items: impl Iterator<Item = Result<PlayItem, String>>) -> Task<Message> {
    let mut toasts = vec![];
    let mut plays = vec![];

    for item in items {
        match item {
            Ok(item) => plays.push(item),
            Err(error) => toasts.push(error),
        }
    }

    if plays.is_empty() && toasts.is_empty() {
        return Task::done(Message::PushToast(
            "No content to play".to_owned(),
            toast::Status::Error,
        ));
    }

    let status = if plays.is_empty() {
        toast::Status::Error
    } else {
        toast::Status::Warn
    };

    let play = if !plays.is_empty() {
        Task::done(Message::PlayItems(plays))
    } else {
        Task::none()
    };

    let toasts = if !toasts.is_empty() {
        let toasts = toasts
            .into_iter()
            .map(|message| (message, status))
            .collect::<Vec<_>>();
        Task::done(Message::PushToasts(toasts))
    } else {
        Task::none()
    };

    Task::batch([play, toasts])
}

fn fetch_kind(kind: PageKind) -> FetchId {
    match kind {
        PageKind::Shows => FetchId::Shows,
        PageKind::Movies => FetchId::Movies,
        PageKind::Collections => FetchId::Collections(true),
        PageKind::Show(id) => FetchId::Show(id),
        PageKind::Season(id) => FetchId::Season(id),
        PageKind::Episode(id) => FetchId::Episode(id),
        PageKind::Movie(id) => FetchId::Movie(id),
        PageKind::Collection(id) => FetchId::Collection(id),
    }
}

fn modal_container<'a>(content: impl Into<Element<'a, HomeMessage>>) -> Container<'a, HomeMessage> {
    container(content)
        .padding([12, 20])
        .style(|theme| {
            let default = container::dark(theme);
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
    now: Instant,
    set_play: bool,
) -> Element<'a, HomeMessage> {
    let items = state.items.iter().map(|item| {
        item.view(
            now,
            &theme,
            HomeMessage::Play,
            primary.clone(),
            |id, hovered| HomeMessage::SearchMessage(SearchMessage::Hovered(id, hovered)),
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
                        let default = button::primary(theme, status);
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

fn sort_collections(collections: &mut [CollectionThumbnail]) {
    collections.sort_by(|x, y| {
        x.collection
            .view
            .cmp(&y.collection.view)
            .then(alphanumeric_sort::compare_str(
                &x.collection.name,
                &y.collection.name,
            ))
    });
}

fn draw_collection_add<'a>(
    state: &'a CollectionAddState,
    collections: impl Iterator<Item = &'a Collection>,
) -> Element<'a, HomeMessage> {
    let title = text("Add to Collection").size(H6);

    fn btn(collection: &Collection, selected: bool) -> Element<'_, HomeMessage> {
        button(container(text(&collection.name)))
            .on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Toggle(
                selected,
                collection.id,
            )))
            .style(move |theme, status| {
                let default = if selected {
                    button::secondary(theme, status)
                } else {
                    button::background(theme, status)
                };
                let border = default.border.rounded(5.0);

                button::Style { border, ..default }
            })
            .into()
    }

    let collections =
        collections.map(|collection| btn(collection, state.selected.contains(&collection.id)));

    let collections = grid(collections)
        .fluid(200)
        .height(Length::Shrink)
        .spacing(12);

    let collections = scrollable(
        container(collections)
            .padding([6, 8])
            .style(|theme: &Theme| {
                let color = theme.extended_palette().background.strong.color;
                let default = container::transparent(theme);
                let border = default.border.rounded(5).color(color).width(2.0);

                container::Style { border, ..default }
            }),
    )
    .spacing(16.0);

    let actions = {
        let save = button("Save")
            .on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Save))
            .style(button::subtle);

        let cancel = button("Cancel")
            .on_press(HomeMessage::CloseView)
            .style(button::subtle);

        row!(save, cancel).spacing(100)
    };
    let content = column!(title, collections, actions)
        .spacing(20)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(500).into()
}
