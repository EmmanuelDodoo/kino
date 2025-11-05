use chrono::{DateTime, Local};
use iced::{
    Element, Length, Padding, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    border::{Border, Radius},
    font, keyboard,
    time::Instant,
    widget::{
        button, center, column, container, grid, operation::scroll_to, pick_list, row, rule,
        scrollable, space, text, text_editor, text_input,
    },
    window,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

mod collection;
mod collections;
mod episode;
mod movie;
mod movies;
mod pages;
mod season;
mod series;
mod shared;
mod shows;

use crate::models::{
    Collection, CollectionId, CollectionView, Episode, EpisodeId, Movie, MovieId, Season, SeasonId,
    Show, ShowId, collection::ItemId,
};

use crate::app::Message;
use crate::models::Media;
use crate::utils::{
    self, HomeAction, Layout, PlayId, PlayItem, Sort, SortKind, empty, filter::*, icons, icons::*,
    load_fonts, typo::*,
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
use shared::{CARD_HEIGHT, CARD_WIDTH, CollectionThumbnail, Scroll, Thumbnail, filter_sort};
use shows::{TvShows, TvShowsMessage};

#[derive(Debug, Clone)]
enum CollectionState {
    Loading,
    Ready(HashSet<ItemId>),
}

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

#[derive(Debug, Clone, Copy)]
enum Focused {
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Episode(EpisodeId),
    Collection((CollectionView, CollectionId)),
}

#[derive(Debug, Clone)]
pub enum Fetch {
    Collections(Vec<CollectionThumbnail>),
    Shows(Vec<Thumbnail<Show>>),
    Movies(Vec<Thumbnail<Movie>>),
    Seasons(Vec<Thumbnail<Season>>),
    Episodes(Vec<Thumbnail<Episode>>),
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
    pub fn update(&self, collection: &mut Collection) {
        let Self {
            name,
            description,
            icon,
            view,
            theme,
            custom,
        } = self;

        collection.name = name.clone();
        let description = description.text();
        if description.is_empty() {
            collection.description = None;
        } else {
            collection.description = Some(description);
        }
        collection.icon = Some(icon.to_u32());
        collection.theme = *theme;
        collection.view = *view;
        collection.custom = custom.clone();
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
pub enum ViewMessage {
    CollectionConfig((CollectionView, CollectionId)),
    Add(ItemId),
    AddToCollection((CollectionView, CollectionId)),
}

#[derive(Debug, Clone)]
pub enum View {
    CollectionConfig(CollectionConfig, CollectionView, CollectionId),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    Search(String),
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
    Fetch(Fetch),
    Scroll(scrollable::Viewport),
    Refresh,
    Hovered(ItemId, bool),
    HoveredCollection((CollectionView, CollectionId), bool),
}

pub struct Home {
    forward: Vec<PageKind>,
    backward: Vec<PageKind>,
    current_page: Option<PageKind>,
    pages: HashMap<PageKind, Page>,

    movies: HashMap<MovieId, Thumbnail<Movie>>,
    shows: HashMap<ShowId, Thumbnail<Show>>,
    seasons: HashMap<SeasonId, Thumbnail<Season>>,
    episodes: HashMap<EpisodeId, Thumbnail<Episode>>,
    collections: BTreeMap<(CollectionView, CollectionId), CollectionThumbnail>,

    search: String,

    recent_movies: BTreeSet<(Option<DateTime<Local>>, MovieId)>,
    recent_shows: BTreeSet<(Option<DateTime<Local>>, ShowId)>,

    collection_states: HashMap<CollectionId, CollectionState>,

    layout: Layout,
    sort: Sort,
    filters: Filter,

    show_sorts: bool,
    show_filters: bool,

    focused: Option<Focused>,

    view: Option<View>,

    scroll: Scroll,
    pending: Vec<Task<HomeMessage>>,
}

impl Home {
    pub fn boot(layout: Layout, filter_mode: FilterMode) -> (Self, Task<HomeMessage>) {
        let movies = Task::perform(
            async {
                (0..3)
                    .map(|_| Movie::testing())
                    .chain((0..3).map(|_| Movie::testing2()))
                    .collect::<Vec<_>>()
            },
            |videos| {
                HomeMessage::Fetch(Fetch::Movies(
                    videos.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let shows = Task::perform(
            async {
                (0..3)
                    .map(|_| Show::testing())
                    .chain((0..3).map(|_| Show::testing1()))
                    .collect::<Vec<_>>()
            },
            |shows| {
                HomeMessage::Fetch(Fetch::Shows(
                    shows.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let seasons = Task::perform(
            async {
                (0..3)
                    .map(|_| Season::testing())
                    .chain((0..3).map(|_| Season::testing2()))
                    .collect::<Vec<_>>()
            },
            |seasons| {
                HomeMessage::Fetch(Fetch::Seasons(
                    seasons.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let episodes = Task::perform(
            async {
                (0..3)
                    .map(|_| Episode::testing())
                    .chain((0..3).map(|_| Episode::testing2()))
                    .collect::<Vec<_>>()
            },
            |episdoes| {
                HomeMessage::Fetch(Fetch::Episodes(
                    episdoes.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let collections = Task::perform(
            async {
                let (collection, _) = Collection::dummy();
                vec![collection]
            },
            |collections| {
                HomeMessage::Fetch(Fetch::Collections(
                    collections
                        .into_iter()
                        .map(CollectionThumbnail::new)
                        .collect(),
                ))
            },
        );

        let tasks = Task::batch([movies, shows, collections, seasons, episodes]);

        (Self::new(layout, filter_mode), tasks)
    }

    fn new(layout: Layout, filter_mode: FilterMode) -> Self {
        Self {
            forward: vec![],
            backward: vec![],
            search: String::default(),

            layout,
            sort: Sort::new_with_name(),
            filters: Filter::new(filter_mode),

            show_sorts: false,
            show_filters: false,
            movies: HashMap::default(),
            shows: HashMap::default(),
            seasons: HashMap::default(),
            episodes: HashMap::default(),
            focused: None,
            scroll: Scroll::new(),
            pages: HashMap::default(),
            current_page: None,
            pending: vec![],
            collections: BTreeMap::default(),
            collection_states: HashMap::default(),
            recent_shows: BTreeSet::default(),
            recent_movies: BTreeSet::default(),
            view: None,
        }
    }

    pub fn update(&mut self, message: HomeMessage, now: Instant) -> Task<Message> {
        match message {
            HomeMessage::None => Task::none(),
            HomeMessage::Search(input) => {
                self.search = input;
                Task::none()
            }
            HomeMessage::Settings => Task::none(),
            HomeMessage::Home => {
                if let Some(old) = self.current_page.take() {
                    self.backward.push(old);
                };
                self.forward.clear();
                self.focused = None;
                self.update_scroll().map(|_| Message::None)
            }
            HomeMessage::Goto(kind) => {
                if let Some(current) = self.current_page
                    && current == kind
                {
                    return Task::none();
                }

                self.backward.retain(|back| *back != kind);

                if let Some(old) = self.current_page.replace(kind) {
                    self.backward.push(old)
                };
                self.forward.clear();
                self.focused = None;

                if let Some(page) = self.pages.get_mut(&kind) {
                    let update = PageUpdate {
                        filters: self.filters,
                        sort: self.sort,
                        layout: self.layout,
                    };
                    page.page_update(update);
                    return page.update_scroll().map(|_| Message::None);
                }

                match kind {
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
                    PageKind::Movie(id) => match self.movies.get(&id) {
                        Some(movie) => {
                            let movie = MoviePage::new(&movie.media);

                            self.pages.insert(kind, Page::Movie { page: movie, id });

                            Task::none()
                        }
                        None => {
                            todo!("Fetch movie from db")
                        }
                    },
                    PageKind::Episode(id) => match self.episodes.get(&id) {
                        Some(episode) => {
                            let episode = EpisodePage::new(&episode.media);

                            self.pages.insert(kind, Page::Episode { page: episode, id });

                            Task::none()
                        }
                        None => {
                            todo!("Fetch episode from db")
                        }
                    },
                    PageKind::Show(id) => match self.shows.get(&id) {
                        Some(show) => {
                            let (show, task) =
                                ShowPage::boot(&show.media, self.sort, self.filters, self.layout);

                            self.pages.insert(kind, Page::Show { id, page: show });

                            task.map(|ssg| Message::Home(HomeMessage::ShowPage(ssg)))
                        }
                        None => {
                            todo!("Fetch show from db")
                        }
                    },
                    PageKind::Season(id) => match self.seasons.get(&id) {
                        Some(season) => {
                            let (season, task) = SeasonPage::boot(
                                &season.media,
                                self.sort,
                                self.filters,
                                self.layout,
                            );

                            self.pages.insert(kind, Page::Season { id, page: season });

                            task.map(|ssg| Message::Home(HomeMessage::SeasonPage(ssg)))
                        }
                        None => {
                            todo!("Fetch season from db")
                        }
                    },
                    PageKind::Collection(id) => match self
                        .collections
                        .iter()
                        .find(|((_, collection), _)| *collection == id)
                    {
                        Some((_, collection)) => {
                            let (collection, tasks) = CollectionPage::boot(
                                &collection.collection,
                                self.sort,
                                self.filters,
                                self.layout,
                            );

                            self.pages.insert(kind, Page::Collection { collection, id });

                            //todo: Fetch members
                            let state = CollectionState::Loading;
                            self.collection_states.insert(id, state);

                            tasks.map(|csg| Message::Home(HomeMessage::Collection(csg)))
                        }
                        None => {
                            todo!("fetch collection if not present also fetch members")
                        }
                    },
                    PageKind::Collections => {
                        let (collections, task) =
                            Collections::boot(self.sort, self.filters, self.layout);

                        self.pages.insert(kind, Page::Collections(collections));

                        task.map(|csg| Message::Home(HomeMessage::Collections(csg)))
                    }
                    _ => {
                        todo!()
                    }
                }
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
                    let Some(collection) = self.collections.get(&key) else {
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

                    self.view = Some(View::CollectionConfig(
                        config,
                        collection.collection.view,
                        collection.collection.id,
                    ));

                    Task::none()
                }
                ViewMessage::Add(item) => {
                    println!("Add item {item:?} to collection");
                    Task::none()
                }
                ViewMessage::AddToCollection((_view, id)) => {
                    println!("Add to collection {id:?}");
                    Task::none()
                }
            },
            HomeMessage::CollectionConfig(csg) => {
                let Some(View::CollectionConfig(mut config, view, id)) = self.view.take() else {
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
                        let Some(mut collection) = self.collections.remove(&(view, id)) else {
                            return Task::none();
                        };

                        if let Some(Page::Collection {
                            collection,
                            id: pid,
                        }) = self.current_page_mut()
                            && *pid == id
                        {
                            collection.name = config.name.clone();
                            collection.view = config.view;
                        }

                        config.update(&mut collection.collection);
                        self.collections.insert(
                            (collection.collection.view, collection.collection.id),
                            collection,
                        );

                        self.view = None;

                        return Task::none();
                    }
                }

                self.view = Some(View::CollectionConfig(config, view, id));

                Task::none()
            }
            HomeMessage::CloseView => {
                self.view = None;
                Task::none()
            }
            HomeMessage::Back => self.back(),
            HomeMessage::Forward => self.forward(),
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
            HomeMessage::Refresh => {
                todo!("Refreshing");
            }
            HomeMessage::Fetch(fsg) => {
                match fsg {
                    Fetch::Episodes(episodes) => {
                        for episode in episodes {
                            self.episodes.insert(episode.id(), episode);
                        }
                    }
                    Fetch::Seasons(seasons) => {
                        for season in seasons {
                            self.seasons.insert(season.id(), season);
                        }
                    }
                    Fetch::Shows(shows) => {
                        for show in shows {
                            self.recent_shows.insert((show.media.recent(), show.id()));
                            self.shows.insert(show.id(), show);
                        }
                    }
                    Fetch::Movies(movies) => {
                        for movie in movies {
                            self.recent_movies
                                .insert((movie.media.recent(), movie.id()));
                            self.movies.insert(movie.id(), movie);
                        }
                    }
                    Fetch::Collections(collections) => {
                        for collection in collections {
                            self.collections.insert(
                                (collection.collection.view, collection.collection.id),
                                collection,
                            );
                        }
                    }
                }
                Task::none()
            }
            HomeMessage::Play(item) => self.play(item),
            HomeMessage::PlayCollection { id, items } => {
                println!("Play {items:?} from collection {id:?}");
                Task::none()
            }
            HomeMessage::HoveredCollection(id, is_hovered) => {
                let Some(collection) = self.collections.get_mut(&id) else {
                    return Task::none();
                };

                collection.zoom.go_mut(is_hovered, now);
                self.focused = Some(Focused::Collection(id));
                Task::none()
            }
            HomeMessage::Hovered(item, is_hovered) => match item {
                ItemId::Movie(id) => {
                    let Some(media) = self.movies.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Movie(id));
                    Task::none()
                }
                ItemId::Show(id) => {
                    let Some(media) = self.shows.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Show(id));
                    Task::none()
                }
                ItemId::Season(id) => {
                    let Some(media) = self.seasons.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Season(id));
                    Task::none()
                }
                ItemId::Episode(id) => {
                    let Some(media) = self.episodes.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Episode(id));
                    Task::none()
                }
            },
            HomeMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                Task::none()
            }
        }
    }

    fn play(&self, item: ItemId) -> Task<Message> {
        match item {
            ItemId::Movie(id) => match self.movies.get(&id) {
                Some(movie) => {
                    let name = movie.media.name();
                    let path = &movie.media.full_path;

                    match play_item(PlayId::Movie(id), name, path) {
                        Ok(item) => Task::done(Message::PlayItem(item)),
                        Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                    }
                }
                None => {
                    todo!("Fetch movie")
                }
            },
            ItemId::Episode(id) => match self.episodes.get(&id) {
                Some(episode) => {
                    let name = episode.media.name();
                    let path = &episode.media.full_path;

                    match play_item(PlayId::Episode(id), name, path) {
                        Ok(item) => Task::done(Message::PlayItem(item)),
                        Err(error) => Task::done(Message::PushToast(error, toast::Status::Error)),
                    }
                }
                None => {
                    todo!("Fetch Episode")
                }
            },
            ItemId::Show(show) => {
                // todo: Might want to fetch the episodes from the db directly?
                let items = self.episodes.values().filter_map(|episode| {
                    if episode.media.show != show {
                        return None;
                    }

                    let id = PlayId::Episode(episode.media.id);
                    let name = episode.media.name();
                    let path = &episode.media.full_path;
                    Some(play_item(id, name, path))
                });

                play_helper(items)
            }
            ItemId::Season(season) => {
                let items = self.episodes.values().filter_map(|episode| {
                    let _ = season;
                    // todo
                    // if episode.media.season != season {
                    //     return None;
                    // }

                    let id = PlayId::Episode(episode.media.id);
                    let name = episode.media.name();
                    let path = &episode.media.full_path;
                    Some(play_item(id, name, path))
                });
                play_helper(items)
            }
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
                .filter_map(|((view, id), collection)| match view {
                    CollectionView::Pinned => {
                        let unicode = Icon::new(collection.collection.icon).unicode();
                        let content = collection_button(
                            unicode,
                            &collection.collection.name,
                            view_unicode(collection.collection.view),
                            HomeMessage::Goto(PageKind::Collection(*id)),
                            self.current_page()
                                .map(|page| page.is_collection(id))
                                .unwrap_or_default(),
                        );

                        Some(content)
                    }
                    CollectionView::Shown => {
                        let unicode = Icon::new(collection.collection.icon).unicode();
                        let content = icon_button(
                            unicode,
                            &collection.collection.name,
                            HomeMessage::Goto(PageKind::Collection(*id)),
                            self.current_page()
                                .map(|page| page.is_collection(id))
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

        let content = column!(collections, space::vertical(), bottom,)
            .padding([0, 5])
            .height(Length::Fill);

        let content = column!(header, space::vertical().height(24.0), content,)
            .width(275.0)
            .height(Length::Fill);

        content.into()
    }

    fn recents(&self, now: Instant) -> Element<'_, HomeMessage> {
        let movies = {
            let label = text("Recent Movies").size(H4);
            let label = column!(label, rule::horizontal(2.0)).spacing(4.0);

            let movies = self
                .recent_movies
                .iter()
                .filter_map(|(_, id)| self.movies.get(id));

            let movies = filter_sort(movies, &self.filters, &self.sort);

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

            let shows = self
                .recent_shows
                .iter()
                .filter_map(|(_, id)| self.shows.get(id));

            let shows = filter_sort(shows, &self.filters, &self.sort);

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

    fn inner(&self, now: Instant) -> Element<'_, HomeMessage> {
        match self.current_page() {
            None => self.recents(now),
            Some(Page::Shows(shows)) => {
                shows.view(now, self.shows.values()).map(HomeMessage::Shows)
            }
            Some(Page::Movies(movies)) => movies
                .view(now, self.movies.values())
                .map(HomeMessage::Movies),
            Some(Page::Movie { page, id }) => {
                // todo: while fetching the movie, this could be reached. Need to handle it better.
                let movie = self.movies.get(id).expect("Page movie not found");
                page.view(movie).map(HomeMessage::MoviePage)
            }
            Some(Page::Episode { page, id }) => {
                let episode = self.episodes.get(id).expect("Page episdoe not found");
                page.view(episode).map(HomeMessage::EpisodePage)
            }
            Some(Page::Season { page, id }) => {
                let season = self.seasons.get(id).expect("Page season not found");
                let episodes = self
                    .episodes
                    .values()
                    // todo
                    // .filter(|episode| episode.media.season == *id)
                    .filter(|_| true);

                page.view(now, season, episodes)
                    .map(HomeMessage::SeasonPage)
            }
            Some(Page::Show { page, id }) => {
                let show = self.shows.get(id).expect("Page show not found");
                let seasons = self
                    .seasons
                    .values()
                    // .filter(|season| season.media.show == *id)
                    .filter(|_| true);

                page.view(now, show, seasons).map(HomeMessage::ShowPage)
            }
            Some(Page::Collection {
                collection: page,
                id,
            }) => {
                match self
                    .collection_states
                    .get(id)
                    .expect("Goto Collection should add the state")
                {
                    CollectionState::Ready(items) => {
                        let collection = self
                            .collections
                            .get(&(page.view, *id))
                            .expect("Page collection not found");

                        let movies = self
                            .movies
                            .values()
                            .filter(|movie| {
                                let id = ItemId::Movie(movie.id());
                                items.contains(&id)
                            })
                            .peekable();
                        let shows = self
                            .shows
                            .values()
                            .filter(|show| {
                                let id = ItemId::Show(show.id());
                                items.contains(&id)
                            })
                            .peekable();
                        let seasons = self
                            .seasons
                            .values()
                            .filter(|season| {
                                let id = ItemId::Season(season.id());
                                items.contains(&id)
                            })
                            .peekable();
                        let episodes = self
                            .episodes
                            .values()
                            .filter(|episode| {
                                let id = ItemId::Episode(episode.id());
                                items.contains(&id)
                            })
                            .peekable();

                        page.view(now, collection, movies, shows, seasons, episodes)
                            .map(HomeMessage::Collection)
                    }
                    CollectionState::Loading => {
                        //todo
                        let collection = self
                            .collections
                            .get(&(page.view, *id))
                            .expect("Page collection not found");

                        let movies = self.movies.values().peekable();
                        let shows = self.shows.values().peekable();
                        let seasons = self.seasons.values().peekable();
                        let episodes = self.episodes.values().peekable();

                        page.view(now, collection, movies, shows, seasons, episodes)
                            .map(HomeMessage::Collection)
                    }
                }
            }
            Some(Page::Collections(collections)) => collections
                .view(now, self.collections.values())
                .map(HomeMessage::Collections),
            _ => todo!("Page view"),
        }
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
            icons::sized_button(icons::REFRESH, size).on_press(HomeMessage::Refresh),
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
        let title = self.current_page().map(Page::name).unwrap_or("Home");
        let title = container(text(title).size(H6)).max_width(400.0);

        let search = {
            let size = H7;
            let icon = text_input::Icon {
                font: icons::FONT,
                code_point: icons::SEARCH,
                side: text_input::Side::Right,
                size: Some(size.into()),
                spacing: 5.0,
            };

            text_input("Search", &self.search)
                .icon(icon)
                .size(size)
                .width(175.0)
                .on_input(HomeMessage::Search)
        };

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

        let content_area = container(self.inner(now))
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

    pub fn view(&self, now: Instant) -> Element<'_, HomeMessage> {
        let content = row!(self.side(), self.content_area(now))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([6, 5]);

        match &self.view {
            None => content.into(),
            Some(View::CollectionConfig(config, _view, _id)) => {
                let overlay = draw_config(config);

                modal(content, overlay)
                    .on_blur(HomeMessage::CloseView)
                    .into()
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        match &self.focused {
            Some(Focused::Show(id)) => self
                .shows
                .get(id)
                .map(|media| media.is_animating(now))
                .unwrap_or_default(),
            Some(Focused::Movie(id)) => self
                .movies
                .get(id)
                .map(|media| media.is_animating(now))
                .unwrap_or_default(),
            Some(Focused::Episode(id)) => self
                .episodes
                .get(id)
                .map(|media| media.is_animating(now))
                .unwrap_or_default(),
            Some(Focused::Season(id)) => self
                .seasons
                .get(id)
                .map(|media| media.is_animating(now))
                .unwrap_or_default(),
            Some(Focused::Collection(key)) => self
                .collections
                .get(key)
                .map(|collection| collection.is_animating(now))
                .unwrap_or_default(),
            None => false,
        }
    }

    pub fn back(&mut self) -> Task<Message> {
        self.focused = None;
        let update = PageUpdate {
            layout: self.layout,
            sort: self.sort,
            filters: self.filters,
        };

        match self.current_page.take() {
            Some(current) => {
                self.forward.push(current);

                match self.backward.pop() {
                    Some(new) => {
                        let page = self
                            .pages
                            .get_mut(&new)
                            .expect("Page cannot be in back without being recorded first");
                        self.current_page = Some(new);
                        page.page_update(update);
                        page.update_scroll().map(|_| Message::None)
                    }
                    None => self.update_scroll().map(|_| Message::None),
                }
            }
            None => {
                let Some(new) = self.backward.pop() else {
                    return Task::none();
                };
                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in back without being recorded first");
                self.current_page = Some(new);
                page.page_update(update);
                page.update_scroll().map(|_| Message::None)
            }
        }
    }

    pub fn forward(&mut self) -> Task<Message> {
        let update = PageUpdate {
            layout: self.layout,
            sort: self.sort,
            filters: self.filters,
        };

        match self.current_page.take() {
            Some(current) => {
                self.backward.push(current);
                let Some(new) = self.forward.pop() else {
                    return Task::none();
                };

                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in forward without being recorded");

                self.current_page = Some(new);
                page.page_update(update);
                page.update_scroll().map(|_| Message::None)
            }
            None => {
                let Some(new) = self.forward.pop() else {
                    return Task::none();
                };

                let page = self
                    .pages
                    .get_mut(&new)
                    .expect("Page cannot be in forward without being recorded");
                self.current_page = Some(new);
                page.page_update(update);
                page.update_scroll().map(|_| Message::None)
            }
        }
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

    pub fn action(&mut self, action: HomeAction) -> Task<Message> {
        match action {
            HomeAction::LayoutToggle => self.layout_toggle(),
        }
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

    let content = container(content)
        .padding([16, 24])
        .style(container::dark)
        .width(width)
        .height(height);

    content.into()
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
