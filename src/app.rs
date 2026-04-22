use chrono::Local;
use iced::{
    Element, Subscription, Task, Theme, event,
    font::Family,
    keyboard::{self, Key, Modifiers},
    mouse,
    time::{self, Instant},
    window,
};
use tokio::sync::mpsc;

use crate::home::{Home, HomeMessage, shared};
use crate::player::{Comment, Manager as Player, ManagerMessage as PlayerMessage, Playlist};
use crate::settings::{Settings, SettingsMessage};
use crate::utils::{Action, Config, KeyPress, Layout, Screen, icons, typo};
use core::Error;
use registry::db::{self, Query};
use registry::{
    filter::{self, FilterMode, SearchFilter},
    sort::{self, Sort, SortKind},
};
use shared::ThumbnailTask;

use devutils::{scan, source};
use registry::models::{
    self, Collection, CollectionId, CollectionView, Directory, DirectoryId, Episode, EpisodeId,
    ItemId, Media, Movie, MovieId, Season, SeasonId, Show, ShowId, SimpleCollection, Video,
    VideoId, collection,
    collection::{
        Items,
        triggers::{DeleteTrigger, InsertTrigger},
    },
};
use widgets::toast;

#[derive(Debug, Clone, Copy)]
pub enum FetchId {
    Recents,
    Shows,
    Movies,
    CollectionsSimple,
    Collections,
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Episode(EpisodeId),
    Collection(CollectionId),
}

#[derive(Clone, Debug)]
pub enum MediaUpdateKind {
    Rating(f32),
    Name(String),
    Synopsis(String),
    Refetch(source::SourceSet),
    Remove,
    TMDBId { id: u32, source: source::SourceSet },
}

#[derive(Clone, Debug)]
pub struct MediaUpdate {
    pub id: ItemId,
    pub kind: MediaUpdateKind,
}

#[derive(Clone, Debug)]
pub enum Message {
    ExitRequested(window::Id),
    Exit(window::Id),
    WindowId(Option<window::Id>),
    CloseToast(usize),
    PushToast(String, toast::Status),
    PushToasts(Vec<(String, toast::Status)>),
    Home(HomeMessage),
    Player(PlayerMessage),
    Settings(SettingsMessage),
    PlayItem(ItemId),
    PlayItems(Vec<ItemId>),
    PlayCollectionItems {
        id: CollectionId,
        items: Items,
    },
    MediaUpdate(MediaUpdate),
    //todo
    Query(Query<'static>),
    FetchMembershipIds(ItemId),
    FetchMemberships(ItemId),
    ToggleMembership {
        item: ItemId,
        collections: Vec<(CollectionId, bool)>,
    },
    LoadSearch(String, Option<SearchFilter>),
    Animate,
    Fetch {
        id: FetchId,
        filters: filter::Filter,
        sort: Sort,
        limit: Option<i32>,
        offset: Option<i32>,
    },
    FetchDirectories,
    FetchComments(VideoId),
    SaveComments(Vec<models::Comment>),
    Refresh(Instant, bool),
    LastWatched(VideoId),
    VideoStats(Video),
    KeyPress {
        key: Key,
        modifiers: Modifiers,
    },
    KeyRelease {
        key: Key,
        modifiers: Modifiers,
    },
    Back,
    Forward,
    Random,
    SettingsOpen,
    SaveSettings,
    Layout(Layout),
    CaptureKeys(bool),
    Scan,
    ScanComplete(Vec<DirectoryId>),
    MovieTask(ThumbnailTask<Movie>),
    ShowTask(ThumbnailTask<Show>),
    SeasonTask(ThumbnailTask<Season>),
    EpisodeTask(ThumbnailTask<Episode>),
    Triggers {
        inserts: Vec<(InsertTrigger, bool)>,
        deletes: Vec<(DeleteTrigger, bool)>,
        removed_inserts: Vec<InsertTrigger>,
        removed_deletes: Vec<DeleteTrigger>,
    },
    RemoveCollectionItems {
        collection: CollectionId,
        items: Items,
    },
    RemoveCollection(CollectionId),
    PlaylistSave(Playlist),
    GeneratedPoster {
        id: VideoId,
        img: devutils::Image,
    },
    AvailableFonts(Result<Vec<Family>, iced::font::Error>),
    None,
}

impl Message {
    pub fn fetch_simple_collections() -> Self {
        Message::Fetch {
            id: FetchId::CollectionsSimple,
            filters: filter::Filter::none(),
            sort: Sort::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn error(error: impl std::fmt::Display) -> Self {
        Message::PushToast(error.to_string(), toast::Status::Error)
    }

    pub fn warn(warning: impl std::fmt::Display) -> Self {
        Message::PushToast(warning.to_string(), toast::Status::Warn)
    }

    pub fn success(message: impl std::fmt::Display) -> Self {
        Message::PushToast(message.to_string(), toast::Status::Success)
    }

    pub fn tasked(self) -> Task<Self> {
        match self {
            Message::None => Task::none(),
            other => Task::done(other),
        }
    }
}

pub struct App {
    now: Instant,
    toasts: Vec<toast::Toast>,
    window: Option<window::Id>,

    screen: Screen,
    home: Home,
    settings: Option<Settings>,
    player: Option<Player>,

    last_refresh: Instant,

    available_fonts: Vec<Family>,
    config: Config,

    db: db::Database,

    is_capturing_keys: bool,

    auth_tx: mpsc::Sender<String>,
    rating_tx: mpsc::Sender<bool>,
}

impl App {
    pub fn boot(
        config: Config,
        db: db::Database,
        errors: impl IntoIterator<Item = String>,
    ) -> (Self, Task<Message>) {
        let load_errors = Task::done(Message::PushToasts(
            errors
                .into_iter()
                .map(|error| (error, toast::Status::Error))
                .collect(),
        ));

        let load_id = window::oldest().map(Message::WindowId);

        let fonts = iced::font::list().map(Message::AvailableFonts);

        let (tmdb_auth_tx, tmdb_auth_rx) = mpsc::channel(2);
        let tmdb_auth = config.tmdb_auth();

        let (rating_tx, rating_rx) = mpsc::channel(2);
        let rating = config.general.tmdb_rating;

        let img_proc =
            Task::future(devutils::image_ops::image_processor(config.db_path())).discard();

        let sources = {
            let tmdb = Task::future(source::tmdb::run(
                config.db_path(),
                tmdb_auth_rx,
                tmdb_auth,
                rating_rx,
                rating,
                config.images_path(),
                config.fetching_interval(),
            ))
            .discard();

            Task::batch([tmdb])
        };

        let (home, home_tasks) = Home::boot(
            config.layout(),
            filter::Filter::new(FilterMode::default()),
            Sort::name(),
            config.general.recents_limit,
        );

        let new = Self::new(config, db, home, tmdb_auth_tx, rating_tx);

        let tasks = Task::batch([load_errors, load_id, home_tasks, sources, img_proc, fonts]);

        (new, tasks)
    }

    fn new(
        config: Config,
        db: db::Database,
        home: Home,
        auth_tx: mpsc::Sender<String>,
        rating_tx: mpsc::Sender<bool>,
    ) -> Self {
        Self {
            screen: Screen::Home,
            now: Instant::now(),
            last_refresh: Instant::now(),
            toasts: vec![],
            window: None,
            player: None,
            settings: None,
            available_fonts: vec![],
            config,
            home,
            db,
            is_capturing_keys: false,
            auth_tx,
            rating_tx,
        }
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => Task::none(),
            Message::Animate => Task::none(),
            Message::WindowId(window) => {
                tracing::debug!("Window id obtained");
                self.window = window;
                Task::none()
            }
            Message::Refresh(refresh, force) => {
                tracing::debug!("Refreshing");
                if force
                    || refresh.duration_since(self.last_refresh) >= self.config.refresh_interval()
                {
                    self.last_refresh = refresh;
                    self.home.refresh().chain(self.home.update_page_scroll())
                } else {
                    Task::none()
                }
            }
            Message::ExitRequested(id) => {
                let Some(own) = &self.window else {
                    return Task::none();
                };
                tracing::debug!("Initiating App Exit sequence");

                if id != *own {
                    return Task::none();
                }

                self.screen = Screen::Home;
                let stats = match self.player.take() {
                    Some(mut player) => {
                        tracing::debug!("Exiting player");
                        let stats = player.stats();

                        self.config.video = player.settings;

                        stats
                    }
                    None => Task::none(),
                };

                match self.config.save() {
                    Ok(_) => stats.chain(Task::done(Message::Exit(id))),
                    Err(error) => stats.chain(Task::done(Message::error(error))),
                }
            }
            Message::Exit(id) => {
                tracing::debug!("Exiting App");
                window::close::<Message>(id).discard()
            }
            Message::AvailableFonts(Ok(fonts)) => {
                tracing::debug!("Acquired available fonts");
                self.available_fonts = fonts;

                Task::none()
            }
            Message::AvailableFonts(Err(error)) => {
                tracing::debug!("Failed to get available fonts. \n{error:?}");
                Task::none()
            }
            Message::PushToast(message, status) => {
                self.push_toast(toast::Toast::new(message, status));
                Task::none()
            }
            Message::PushToasts(toasts) => {
                let toasts = toasts
                    .into_iter()
                    .map(|(message, status)| toast::Toast::new(message, status));

                self.push_toasts(toasts);

                Task::none()
            }
            Message::CloseToast(idx) => {
                tracing::debug!("Closed toast {idx}");
                self.toasts
                    .remove(idx.min(self.toasts.len().saturating_sub(1)));

                Task::none()
            }
            Message::Home(hsg) => self.home.update(hsg, now),
            Message::Player(psg) => {
                let Some(player) = self.player.as_mut() else {
                    return Task::none();
                };

                player.update(psg, now)
            }
            Message::Settings(ssg) => {
                let Some(settings) = self.settings.as_mut() else {
                    return Task::none();
                };

                settings.update(ssg)
            }
            Message::Query(query) => {
                let _todo = match query.execute(&self.db) {
                    Ok(suc) => {
                        tracing::debug!("{suc:?}");
                        suc
                    }
                    Err(error) => {
                        let msg = Message::error(error.error);
                        return Task::done(msg);
                    }
                };

                Task::none()
            }
            Message::MediaUpdate(MediaUpdate { id, kind }) => {
                let query = match kind {
                    MediaUpdateKind::Rating(value) => match id {
                        ItemId::Show(id) => Show::set_rating(id, value),
                        ItemId::Movie(id) => Movie::set_rating(id, value),
                        ItemId::Season(id) => Season::set_rating(id, value),
                        ItemId::Episode(id) => Episode::set_rating(id, value),
                    },
                    MediaUpdateKind::Name(value) => match id {
                        ItemId::Show(id) => Show::set_name(id, value),
                        ItemId::Movie(id) => Movie::set_name(id, value),
                        ItemId::Season(id) => Season::set_name(id, value),
                        ItemId::Episode(id) => Episode::set_name(id, value),
                    },
                    MediaUpdateKind::Synopsis(value) => match id {
                        ItemId::Show(id) => Show::set_synopsis(id, value),
                        ItemId::Movie(id) => Movie::set_synopsis(id, value),
                        ItemId::Season(id) => Season::set_synopsis(id, value),
                        ItemId::Episode(id) => Episode::set_synopsis(id, value),
                    },
                    MediaUpdateKind::Refetch(source) => match source.refetch(id) {
                        Some(query) => {
                            match query.execute(&self.db) {
                                Ok(todo) => {
                                    tracing::debug!("{todo:?}");
                                    let msg = Message::success("Refetch queued").tasked();
                                    return Task::batch([self.home.content_refresh(), msg]);
                                }
                                Err(error) => {
                                    // todo
                                    let msg = Message::error(error.error);

                                    return Task::done(msg);
                                }
                            }
                        }
                        None => {
                            return Task::none();
                        }
                    },
                    MediaUpdateKind::Remove => match id {
                        ItemId::Show(id) => Show::remove(id),
                        ItemId::Movie(id) => Movie::remove(id),
                        ItemId::Season(id) => Season::remove(id),
                        ItemId::Episode(id) => Episode::remove(id),
                    },
                    MediaUpdateKind::TMDBId {
                        id: tmdb_id,
                        source,
                    } => {
                        let query = match id {
                            ItemId::Movie(id) => source.set_tmdb_id(id, tmdb_id),

                            ItemId::Show(id) => source.set_tmdb_id(id, tmdb_id),
                            ItemId::Season(id) => source.set_tmdb_number(id, tmdb_id as u16),
                            ItemId::Episode(id) => source.set_tmdb_number(id, tmdb_id as u16),
                        };

                        match query {
                            Some(query) => {
                                match query.execute(&self.db) {
                                    Ok(todo) => {
                                        tracing::debug!("{todo:?}");
                                        let msg = Message::success("Refetch queued").tasked();
                                        return Task::batch([self.home.content_refresh(), msg]);
                                    }
                                    Err(error) => {
                                        // todo
                                        let msg = Message::error(error.error);

                                        return Task::done(msg);
                                    }
                                }
                            }
                            None => {
                                return Task::none();
                            }
                        }
                    }
                };

                match query.execute(&self.db) {
                    Ok(todo) => {
                        tracing::debug!("{todo:?}");
                        self.home.content_refresh()
                    }
                    Err(error) => {
                        // todo
                        let msg = Message::error(error.error);

                        Task::done(msg)
                    }
                }
            }
            Message::PlayItem(item) => self.play_items(std::iter::once(item), true),
            Message::PlayItems(items) => self.play_items(items.into_iter(), false),
            Message::PlayCollectionItems { id, items } => {
                let items = match self.db.get_collection_items(id) {
                    Ok(items) => items,
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                }
                .into_iter()
                .filter(|item| {
                    matches!(
                        (item, items),
                        (_, Items::All)
                            | (ItemId::Movie(_), Items::Movies)
                            | (ItemId::Show(_), Items::Shows)
                            | (ItemId::Season(_), Items::Seasons)
                            | (ItemId::Episode(_), Items::Episodes)
                    )
                });

                self.play_items(items, false)
            }
            Message::KeyPress { key, modifiers } => {
                let keypress = KeyPress::with_modifiers(key, modifiers);

                if let Some(settings) = self.settings.as_mut()
                    && self.is_capturing_keys
                {
                    settings.captured_key(keypress)
                } else {
                    match self
                        .config
                        .keystore
                        .action(keypress, self.screen)
                        .map(|action| self.action(action, now))
                    {
                        Some(action) => action,
                        None if matches!(self.screen, Screen::Home) => {
                            self.home.command = modifiers.command();
                            Task::none()
                        }
                        _ => Task::none(),
                    }
                }
            }
            Message::KeyRelease { modifiers, .. } => {
                self.home.command = modifiers.command();

                Task::none()
            }
            Message::CaptureKeys(capture) => {
                self.is_capturing_keys = capture;
                Task::none()
            }
            Message::Back => match self.screen {
                Screen::Home => self.home.back(now, false),
                Screen::Player => {
                    self.screen = Screen::Home;
                    let task = match self.player.take() {
                        Some(mut player) => {
                            let fullscreen = player.fullscreen_exit();
                            let stats = player.stats();
                            tracing::debug!("Exiting player");

                            self.config.video = player.settings;

                            Task::batch([fullscreen, stats])
                        }
                        None => Task::none(),
                    };

                    task.chain(Message::Refresh(now, true).tasked())
                }
                Screen::Settings => {
                    self.settings.take();
                    self.player.take();

                    self.screen = Screen::Home;

                    Message::Refresh(now, true).tasked()
                }
            },
            Message::Forward => {
                if !matches!(self.screen, Screen::Home) {
                    Task::none()
                } else {
                    self.home.forward(now)
                }
            }
            Message::Fetch {
                id,
                filters: filter,
                sort,
                limit,
                offset,
            } => {
                self.last_refresh = now;

                match id {
                    FetchId::CollectionsSimple => {
                        let collections = match self
                            .db
                            .get_collections(collection::Sort::View, SimpleCollection::from_row)
                        {
                            Ok(collections) => {
                                tracing::debug!("Fetched {} Simple Collections", collections.len());
                                collections
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        match self.player.as_mut() {
                            Some(player) => player.fetched_collections(collections),
                            None => self.home.fetch_collections_simple(collections),
                        }
                    }
                    FetchId::Shows => {
                        let thumbnails =
                            match self.db.get_shows(limit, offset, filter, sort, show_map) {
                                Ok(shows) => {
                                    tracing::debug!("Fetched {} Shows", shows.len());
                                    shows
                                }
                                Err(error) => {
                                    let msg = Message::error(error);
                                    return Task::done(msg);
                                }
                            };

                        let mut shows = Vec::with_capacity(thumbnails.len());
                        let mut tasks = Vec::with_capacity(thumbnails.len());

                        for (show, task) in thumbnails {
                            shows.push(show);
                            tasks.push(task.map(Message::ShowTask));
                        }

                        let home_tasks = self.home.fetched_shows(shows);
                        let samples = Task::batch(tasks);

                        Task::batch([home_tasks, samples])
                    }
                    FetchId::Movies => {
                        let thumbnails =
                            match self.db.get_movies(limit, offset, filter, sort, movie_map) {
                                Ok(movies) => {
                                    tracing::debug!("Fetched {} Movies", movies.len());
                                    movies
                                }
                                Err(error) => {
                                    let msg = Message::error(error);
                                    return Task::done(msg);
                                }
                            };

                        let mut movies = Vec::with_capacity(thumbnails.len());
                        let mut tasks = Vec::with_capacity(thumbnails.len());

                        for (movie, task) in thumbnails {
                            movies.push(movie);
                            tasks.push(task.map(Message::MovieTask));
                        }

                        let home_tasks = self.home.fetched_movies(movies);
                        let samples = Task::batch(tasks);

                        Task::batch([home_tasks, samples])
                    }
                    FetchId::Recents => {
                        let thumbnails_movies =
                            match self.db.get_movies(limit, offset, filter, sort, movie_map) {
                                Ok(movies) => {
                                    tracing::debug!("Fetched {} Recent Movies", movies.len());
                                    movies
                                }
                                Err(error) => {
                                    let msg = Message::error(error);
                                    return Task::done(msg);
                                }
                            };

                        let thumbnails_shows =
                            match self.db.get_shows(limit, offset, filter, sort, show_map) {
                                Ok(shows) => {
                                    tracing::debug!("Fetched {} Recent Shows", shows.len());
                                    shows
                                }
                                Err(error) => {
                                    let msg = Message::error(error);
                                    return Task::done(msg);
                                }
                            };
                        let mut shows = Vec::with_capacity(thumbnails_shows.len());
                        let mut movies = Vec::with_capacity(thumbnails_movies.len());
                        let mut tasks =
                            Vec::with_capacity(thumbnails_shows.len() + thumbnails_movies.len());

                        for (show, task) in thumbnails_shows {
                            shows.push(show);
                            tasks.push(task.map(Message::ShowTask));
                        }

                        for (movie, task) in thumbnails_movies {
                            movies.push(movie);
                            tasks.push(task.map(Message::MovieTask));
                        }

                        let samples = Task::batch(tasks);
                        let home_tasks = self.home.fetched_recents(movies, shows);

                        Task::batch([home_tasks, samples])
                    }
                    FetchId::Show(id) => {
                        let (show, show_task) = match self.db.get_show(id, show_map) {
                            Ok(show) => {
                                tracing::debug!("Fetched Show {}", show.0.media.name());
                                show
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let thumbnail_seasons = match self
                            .db
                            .get_show_seasons(id, limit, offset, filter, sort, season_map)
                        {
                            Ok(seasons) => {
                                tracing::debug!("Fetched {} show Seasons", seasons.len());
                                seasons
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let mut seasons = Vec::with_capacity(thumbnail_seasons.len());
                        let mut tasks = Vec::with_capacity(1 + thumbnail_seasons.len());

                        tasks.push(show_task.map(Message::ShowTask));

                        for (season, task) in thumbnail_seasons {
                            seasons.push(season);
                            tasks.push(task.map(Message::SeasonTask));
                        }

                        let home_task = self.home.fetched_show(show, seasons);
                        let samples = Task::batch(tasks);

                        Task::batch([home_task, samples])
                    }
                    FetchId::Season(id) => {
                        let (season, season_task) = match self.db.get_season(id, season_map) {
                            Ok(season) => {
                                tracing::debug!("Fetched season {}", season.0.media.name());
                                season
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let thumbnail_episodes = match self.db.get_season_episodes(
                            id,
                            limit,
                            offset,
                            filter,
                            sort,
                            episode_map,
                        ) {
                            Ok(episodes) => {
                                tracing::debug!("Fetched {} season episodes", episodes.len());
                                episodes
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let mut episodes = Vec::with_capacity(thumbnail_episodes.len());
                        let mut tasks = Vec::with_capacity(1 + thumbnail_episodes.len());
                        tasks.push(season_task.map(Message::SeasonTask));

                        for (episode, task) in thumbnail_episodes {
                            episodes.push(episode);
                            tasks.push(task.map(Message::EpisodeTask))
                        }

                        let samples = Task::batch(tasks);
                        let home_task = self.home.fetched_season(season, episodes);

                        Task::batch([home_task, samples])
                    }
                    FetchId::Episode(id) => {
                        let (episode, sample) = match self.db.get_episode(id, episode_map) {
                            Ok(episode) => {
                                tracing::debug!("Fetched Episode {}", episode.0.media.name());
                                episode
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        Task::batch([
                            self.home.fetched_episode(episode),
                            sample.map(Message::EpisodeTask),
                        ])
                    }
                    FetchId::Movie(id) => {
                        let (movie, task) = match self.db.get_movie(id, movie_map) {
                            Ok(movie) => {
                                tracing::debug!("Fetched Movie {}", movie.0.media.name());
                                movie
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        Task::batch([self.home.fetched_movie(movie), task.map(Message::MovieTask)])
                    }
                    FetchId::Collections => {
                        //todo: collection sorts
                        let collections = match self
                            .db
                            .get_collections(collection::Sort::default(), Collection::from_row)
                        {
                            Ok(collections) => {
                                tracing::debug!("Fetched {} Collections", collections.len());
                                collections
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        self.home.fetched_collections(collections)
                    }
                    FetchId::Collection(id) => {
                        let collection = match self.db.get_collection(id, Collection::from_row) {
                            Ok(collection) => {
                                tracing::debug!("Fetched Collection {}", collection.name);
                                collection
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let itriggers =
                            match self.db.get_collection_inserts(id, InsertTrigger::from_row) {
                                Ok(triggers) => {
                                    tracing::debug!(
                                        "Fetched {} collection insert triggers",
                                        triggers.len()
                                    );
                                    triggers
                                }
                                Err(error) => {
                                    return Message::error(error).tasked();
                                }
                            };

                        let dtriggers =
                            match self.db.get_collection_deletes(id, DeleteTrigger::from_row) {
                                Ok(triggers) => {
                                    tracing::debug!(
                                        "Fetched {} collection delete triggers",
                                        triggers.len()
                                    );
                                    triggers
                                }
                                Err(error) => {
                                    return Message::error(error).tasked();
                                }
                            };

                        let (
                            thumbnail_movies,
                            thumbnail_shows,
                            thumbnail_seasons,
                            thumbnail_episodes,
                        ) = match self.db.get_collection_members(
                            id,
                            limit,
                            offset,
                            filter,
                            sort,
                            movie_map,
                            show_map,
                            season_map,
                            episode_map,
                        ) {
                            Ok(items) => {
                                tracing::debug!("Fetched Collection items");
                                items
                            }
                            Err(error) => {
                                let msg = Message::error(error);
                                return Task::done(msg);
                            }
                        };

                        let mut movies = Vec::with_capacity(thumbnail_movies.len());
                        let mut shows = Vec::with_capacity(thumbnail_shows.len());
                        let mut seasons = Vec::with_capacity(thumbnail_seasons.len());
                        let mut episodes = Vec::with_capacity(thumbnail_episodes.len());
                        let mut tasks = Vec::with_capacity(
                            thumbnail_movies.len()
                                + thumbnail_shows.len()
                                + thumbnail_seasons.len()
                                + thumbnail_episodes.len(),
                        );

                        for (movie, task) in thumbnail_movies {
                            movies.push(movie);
                            tasks.push(task.map(Message::MovieTask));
                        }

                        for (show, task) in thumbnail_shows {
                            shows.push(show);
                            tasks.push(task.map(Message::ShowTask));
                        }

                        for (season, task) in thumbnail_seasons {
                            seasons.push(season);
                            tasks.push(task.map(Message::SeasonTask));
                        }

                        for (episode, task) in thumbnail_episodes {
                            episodes.push(episode);
                            tasks.push(task.map(Message::EpisodeTask));
                        }

                        let home_task = self.home.fetched_collection(
                            collection, itriggers, dtriggers, movies, shows, seasons, episodes,
                        );
                        let samples = Task::batch(tasks);

                        Task::batch([home_task, samples])
                    }
                }
            }
            Message::FetchDirectories => {
                let Some(settings) = self.settings.as_mut() else {
                    return Task::none();
                };

                let dirs = match self.db.get_directories() {
                    Ok(dirs) => {
                        tracing::debug!("Fetched {} Directories", dirs.len());
                        dirs
                    }
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                };

                settings.fetched_directories(dirs);

                Task::none()
            }
            Message::FetchComments(id) => {
                if matches!(self.screen, Screen::Settings) {
                    return Task::none();
                }

                let comments = match self.db.get_video_comments(
                    id,
                    None,
                    None,
                    filter::comments::Filter::default(),
                    sort::comments::Sort::default(),
                    |comment| Comment::load(comment, None),
                ) {
                    Ok(comments) => {
                        tracing::debug!("Fetched {} comments for {id}", comments.len());
                        comments
                    }
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                };

                match self.screen {
                    Screen::Player => match self.player.as_mut() {
                        Some(player) => player.fetched_comments(id, comments),
                        None => Task::none(),
                    },
                    Screen::Settings => unreachable!(),
                    _ => todo!(),
                }
            }
            Message::SaveComments(comments) => {
                for comment in comments {
                    let query = comment.insert();
                    match query.execute(&self.db) {
                        Ok(succ) => {
                            tracing::debug!("{succ:?}");
                        }
                        Err(fail) => {
                            tracing::error!("{fail:?}");
                        }
                    }
                }

                Task::none()
            }
            Message::LoadSearch(search, filter) => {
                let items = match self.db.search(
                    search.clone(),
                    filter,
                    self.config.search_limit(),
                    shared::SearchView::new,
                ) {
                    Ok(items) => {
                        tracing::debug!("Fetched Search {search} items");
                        items
                    }
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                };

                self.home.loaded_search(items)
            }
            Message::FetchMembershipIds(item) => {
                let memberships = match self.db.get_item_membership_ids(item) {
                    Ok(memberships) => {
                        tracing::debug!("Fetched Item {item:?} memberships ids");
                        memberships
                    }
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                };

                match self.player.as_mut() {
                    Some(player) => player.fetched_membership_ids(memberships),

                    None => self.home.fetched_memberships_ids(memberships),
                }
            }
            Message::FetchMemberships(item) => {
                let memberships =
                    match self
                        .db
                        .get_item_membership_ids(item)
                        .and_then(|collections| {
                            self.db.get_memberships(
                                collections,
                                None,
                                None,
                                collection::Sort::default(),
                                SimpleCollection::from_row,
                            )
                        }) {
                        Ok(memberships) => {
                            tracing::debug!("Fetched Item {item:?} memberships ids");
                            memberships
                        }
                        Err(error) => {
                            let msg = Message::error(error);
                            return Task::done(msg);
                        }
                    };

                self.home.fetched_memberships(memberships)
            }
            Message::ToggleMembership { item, collections } => {
                let msg = match self.db.toggle_membership(item, collections) {
                    Ok(true) => Message::success("Collections Updated!"),
                    Ok(false) => Message::None,
                    Err(error) => Message::error(error),
                };

                let refresh = self.home.content_refresh();
                Task::batch([Task::done(msg), refresh])
            }
            Message::LastWatched(id) => {
                let now = Local::now();
                let now = models::datetime_to_sql(&now);

                match id {
                    VideoId::Movie(id) => match self.db.last_watched_movie(id, now) {
                        Ok(_) => {
                            tracing::debug!("Updated {id:?} last watched");
                            Task::none()
                        }
                        Err(error) => Task::done(Message::error(error)),
                    },
                    VideoId::Episode(id) => match self.db.last_watched_episode(id, now) {
                        Ok(_) => {
                            tracing::debug!("Updated {id:?} last watched");
                            Task::none()
                        }
                        Err(error) => Task::done(Message::error(error)),
                    },
                }
            }
            Message::VideoStats(item) => {
                for sub in item.subtitles {
                    if let Err(error) = sub.insert().execute(&self.db) {
                        tracing::error!("Saving subtitle {} failed. \n{}", sub.id, error.error);
                    }
                }

                match self.db.update_video_stats(
                    item.id,
                    item.watch_count,
                    item.progress,
                    item.duration,
                    item.subtitle_id,
                    item.audio_id,
                ) {
                    Ok(_) => {
                        tracing::debug!("Updated {:?} statistics", item.id);
                    }
                    Err(error) => return Task::done(Message::error(error)),
                }

                Task::none()
            }
            Message::Random => {
                let random = match self.db.get_random() {
                    Ok(random) => {
                        tracing::debug!("Fetched random media {random:?}");
                        random
                    }
                    Err(error) => {
                        let msg = Message::error(error);
                        return Task::done(msg);
                    }
                };

                self.home.goto(random.into(), now)
            }
            Message::SettingsOpen => self.settings(),
            Message::SaveSettings => {
                self.screen = Screen::Home;

                let Some(settings) = self.settings.take() else {
                    return Task::none();
                };

                tracing::debug!("Saving settings");

                self.home.layout(settings.config.layout());
                self.home
                    .recents_limit(settings.config.general.recents_limit);

                let home_scroll = self.home.update_page_scroll();

                let (config, directories) = settings.save();
                let writer = self.config.span_writer.take();

                let prev_rating = self.config.general.tmdb_rating;
                let new_rating = config.general.tmdb_rating;

                self.config = config;
                self.config.span_writer = writer;

                let mut scans = vec![];
                let mut dirs = vec![];

                for (dir, op, scan) in directories {
                    if scan {
                        scans.push(dir.clone())
                    }

                    if let Some(op) = op {
                        dirs.push((dir, op))
                    }
                }

                let dir = match self.db.toggle_directories(dirs) {
                    Ok(true) => Message::success("Directories Updated!").tasked(),
                    Ok(false) => Message::None.tasked(),
                    Err(error) => Message::error(error).tasked(),
                };

                let home_task = if !scans.is_empty() {
                    self.home.scanning(true)
                } else {
                    Task::none()
                };
                let discoverer = self.config.general.scan_discoverer;
                let db_path = self.config.db_path();
                let movie_depth = self.config.general.movie_depth;
                let restore = self.config.general.restore_deleted;
                let preferred_sub = self.config.general.preferred_subtitle_codec.clone();
                let preferred_audio = self.config.general.preferred_audio_codec.clone();
                let default_source = self.config.general.default_source;

                let scans = scan_task(
                    db_path,
                    scans,
                    default_source,
                    discoverer,
                    movie_depth,
                    restore,
                    preferred_sub,
                    preferred_audio,
                );

                let auth = self.config.tmdb_auth();

                let auth = if !auth.is_empty() {
                    tracing::debug!("Updating TMDB API token");
                    let auth_tx = self.auth_tx.clone();

                    Task::perform(async move { auth_tx.send(auth).await }, |_| Message::None)
                } else {
                    Task::none()
                };

                let rating = if prev_rating != new_rating {
                    tracing::debug!("Updating TMDB rating option");
                    let rating_tx = self.rating_tx.clone();

                    Task::perform(async move { rating_tx.send(new_rating).await }, |_| {
                        Message::None
                    })
                } else {
                    Task::none()
                };

                Task::batch([auth, home_scroll, rating, dir, home_task]).chain(scans)
            }
            Message::Layout(layout) => {
                self.config.general.layout = layout;
                Task::none()
            }
            Message::Scan => {
                let dirs = match self.db.get_directories() {
                    Ok(dirs) => dirs,
                    Err(error) => {
                        return Task::done(Message::error(error));
                    }
                };

                let home_task = self.home.scanning(true);
                let discoverer = self.config.general.scan_discoverer;
                let db_path = self.config.db_path();
                let movie_depth = self.config.general.movie_depth;
                let restore = self.config.general.restore_deleted;
                let preferred_sub = self.config.general.preferred_subtitle_codec.clone();
                let preferred_audio = self.config.general.preferred_audio_codec.clone();
                let default_source = self.config.general.default_source;

                let scan = scan_task(
                    db_path,
                    dirs,
                    default_source,
                    discoverer,
                    movie_depth,
                    restore,
                    preferred_sub,
                    preferred_audio,
                );

                Task::batch([home_task, scan])
            }
            Message::ScanComplete(scanned) => {
                tracing::debug!("Directory scan complete");
                let last_scan = Local::now();
                let last_scan = models::datetime_to_sql(&last_scan);

                let _todo = match self.db.last_scans(scanned, last_scan) {
                    Ok(rows) => {
                        tracing::debug!("Directories last scanned updated {rows} rows");
                        rows
                    }
                    Err(error) => {
                        return Task::done(Message::error(error));
                    }
                };

                self.home.scanning(false)
            }
            Message::MovieTask(ThumbnailTask { id, kind }) => self.home.movie_task(id, kind, now),
            Message::ShowTask(ThumbnailTask { id, kind }) => self.home.show_task(id, kind, now),
            Message::SeasonTask(ThumbnailTask { id, kind }) => self.home.season_task(id, kind, now),
            Message::EpisodeTask(ThumbnailTask { id, kind }) => {
                self.home.episode_task(id, kind, now)
            }
            Message::Triggers {
                inserts,
                deletes,
                removed_inserts,
                removed_deletes,
            } => {
                let mut succs = vec![];
                let mut fails = vec![];

                for (trigger, roe) in inserts {
                    match trigger.insert().execute(&self.db) {
                        Ok(succ) => {
                            tracing::debug!("{succ:?}");
                            succs.push(succ);
                        }
                        Err(fail) => {
                            tracing::error!("{fail:?}");
                            fails.push((
                                format!("{} error.\n{}", trigger.name, fail.error),
                                toast::Status::Error,
                            ))
                        }
                    };

                    match trigger.save(&self.db) {
                        Ok(_) => {
                            tracing::debug!("{} saved", trigger.name)
                        }
                        Err(error) => {
                            tracing::error!("Trigger {} save {error:?}", trigger.name);
                            fails.push((
                                format!("{} save error.\n{}", trigger.name, error),
                                toast::Status::Error,
                            ))
                        }
                    }

                    if roe {
                        match trigger.run_on_existing(&mut self.db) {
                            Ok(_) => {
                                tracing::debug!("{} run-on-existing successful", trigger.name)
                            }
                            Err(error) => {
                                tracing::error!("{error:?}");
                                fails.push((
                                    format!("{} run-on-existing error.\n{}", trigger.name, error),
                                    toast::Status::Error,
                                ))
                            }
                        }
                    }
                }

                for (trigger, roe) in deletes {
                    match trigger.insert().execute(&self.db) {
                        Ok(succ) => {
                            tracing::debug!("{succ:?}");
                            succs.push(succ);
                        }
                        Err(fail) => {
                            tracing::error!("{fail:?}");
                            fails.push((
                                format!("{} error.\n{}", trigger.name, fail.error),
                                toast::Status::Error,
                            ))
                        }
                    };

                    match trigger.save(&self.db) {
                        Ok(_) => {
                            tracing::debug!("{} saved", trigger.name)
                        }
                        Err(error) => {
                            tracing::error!("Trigger {} save {error:?}", trigger.name);
                            fails.push((
                                format!("{} save error.\n{}", trigger.name, error),
                                toast::Status::Error,
                            ))
                        }
                    }

                    if roe {
                        match trigger.run_on_existing(&mut self.db) {
                            Ok(_) => {
                                tracing::debug!("{} run-on-existing successful", trigger.name)
                            }
                            Err(error) => {
                                tracing::error!("{error:?}");
                                fails.push((
                                    format!("{} run-on-existing error.\n{}", trigger.name, error),
                                    toast::Status::Error,
                                ))
                            }
                        }
                    }
                }

                for trigger in removed_inserts {
                    let name = trigger.name.clone();
                    match trigger.remove(&self.db) {
                        Ok(_) => {
                            tracing::debug!("{} removed", name);
                        }
                        Err(error) => {
                            tracing::error!("Trigger {} remove {error:?}", name);
                            fails.push((
                                format!("{} remove error.\n{}", name, error),
                                toast::Status::Error,
                            ))
                        }
                    }
                }

                for trigger in removed_deletes {
                    let name = trigger.name.clone();
                    match trigger.remove(&self.db) {
                        Ok(_) => {
                            tracing::debug!("{} removed", name);
                        }
                        Err(error) => {
                            tracing::error!("Trigger {} remove {error:?}", name);
                            fails.push((
                                format!("{} remove error.\n{}", name, error),
                                toast::Status::Error,
                            ))
                        }
                    }
                }

                let succ = if !succs.is_empty() {
                    Task::done(Message::success(format!(
                        "{} trigger successes",
                        succs.len()
                    )))
                } else {
                    Task::none()
                };

                let fails = Task::done(Message::PushToasts(fails));

                Task::batch([succ, fails, Message::Refresh(now, true).tasked()])
            }
            Message::RemoveCollection(id) => match self.db.remove_collection(id) {
                Ok(rows) if rows > 0 => Message::success("Collection Deleted").tasked(),
                Ok(_) => Task::none(),
                Err(error) => Message::error(error).tasked(),
            },
            Message::RemoveCollectionItems { collection, items } => {
                match self.db.remove_collection_items(collection, items) {
                    Ok(rows) if rows > 0 => Message::success("Collection Items removed").tasked(),
                    Ok(_) => Task::none(),
                    Err(error) => Message::error(error).tasked(),
                }
            }
            Message::PlaylistSave(playlist) => {
                let now = Local::now();
                let name = format!("Saved Playlist#{}", now.format("%Y/%m/%d"));
                let icon = shared::Icon::playlist();

                let (new, query) =
                    Collection::new(name, None, CollectionView::Shown, Some(icon), None);

                match query.execute(&self.db) {
                    Ok(suc) => {
                        tracing::debug!("{suc:?}");
                    }
                    Err(error) => return Message::error(error.error).tasked(),
                };

                match self.db.insert_collection_items(new.id, playlist.origins) {
                    Ok(rows) => {
                        tracing::debug!("Inserted {rows} playlist collection items");
                        Message::success("Saved Playlist").tasked()
                    }
                    Err(error) => Message::error(error).tasked(),
                }
            }
            Message::GeneratedPoster { id, img } => {
                tracing::debug!("Saving generated thumbnail on {id}");

                let db = self.config.db_path();

                let path = self.config.images_path();

                Task::future(async move {
                    devutils::image_ops::save_generated_poster(id, img, db, path);
                })
                .discard()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = self.theme().unwrap();
        let content: Element<'_, Message> = match self.screen {
            Screen::Home => self.home.view(&theme, self.now).map(Message::Home),
            Screen::Player => {
                let player = self.player.as_ref().unwrap();

                player.view(&theme, self.now).map(Message::Player)
            }
            Screen::Settings => {
                let settings = self.settings.as_ref().unwrap();
                settings.view().map(Message::Settings)
            }
        };

        toast::manager(
            content,
            &self.toasts,
            Message::CloseToast,
            toast::Settings {
                text_size: typo::H7,
                close_icon: icons::CANCEL,
                close_size: typo::P,
                close_font: icons::FONT,
            },
        )
        .into()
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.settings.as_ref() {
            Some(settings) => Some(settings.config.theme()),
            None => Some(self.config.theme()),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let (animating, player) = self
            .player
            .as_ref()
            .map(|player| (player.is_animating(self.now), player.subscription()))
            .unwrap_or((false, Subscription::none()));
        let player = player.map(Message::Player);

        let animating = if self.home.is_animating(self.now) || animating {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        };

        let keys = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPress { key, modifiers }
            }
            keyboard::Event::KeyReleased { key, modifiers, .. } => {
                Message::KeyRelease { key, modifiers }
            }
            _ => Message::None,
        });

        let mouse = event::listen_with(|event, status, _todo| {
            if !matches!(status, event::Status::Ignored) {
                return None;
            }

            let event::Event::Mouse(mouse::Event::ButtonPressed(button)) = event else {
                return None;
            };

            match button {
                mouse::Button::Forward => Some(Message::Forward),
                mouse::Button::Back => Some(Message::Back),
                _ => None,
            }
        });

        let exit = window::close_requests().map(Message::ExitRequested);

        let home = self.home.subscription();

        let refresh =
            time::every(self.config.refresh_interval()).map(|at| Message::Refresh(at, false));

        Subscription::batch([animating, keys, mouse, exit, player, refresh, home])
    }

    fn settings(&mut self) -> Task<Message> {
        let (settings, tasks) = Settings::boot(self.config.clone(), self.available_fonts.clone());
        self.screen = Screen::Settings;
        self.settings = Some(settings);

        tasks
    }

    fn push_toast(&mut self, toast: toast::Toast) {
        use toast::Status;
        use tracing::{debug, error, info, warn};

        match toast.status {
            Status::Info => info!(toast.message),
            Status::Warn => warn!(toast.message),
            Status::Success => debug!(toast.message),
            Status::Error => error!(toast.message),
        }

        self.toasts.push(toast);
    }

    fn push_toasts(&mut self, toasts: impl Iterator<Item = toast::Toast>) {
        for toast in toasts {
            self.push_toast(toast)
        }
    }

    fn play_season(
        &self,
        season: SeasonId,
        origin: Option<ItemId>,
    ) -> Result<(Playlist, Vec<String>), Error> {
        let recent = self.db.get_season(season, EpisodeId::from_recents)?;

        let sort = {
            let mut sort = Sort::release();
            sort.push(SortKind::Name);

            sort
        };

        let items = self
            .db
            .get_season_videos(season, None, None, filter::Filter::none(), sort)?;

        let pos = recent
            .and_then(|recent| {
                items
                    .iter()
                    .position(|item| item.id == VideoId::Episode(recent))
            })
            .unwrap_or_default();

        let (valid, invalid): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|item| match item.path.try_exists() {
                Ok(true) => Ok(item),
                Ok(false) => Err(Error::Raw(format!(
                    "{} does not exist",
                    item.path.to_string_lossy()
                ))),
                Err(error) => Err(Error::IO(error)),
            })
            .partition(Result::is_ok);

        let valid = valid.into_iter().map(Result::unwrap);
        let mut playlist = Playlist::new(valid, origin.unwrap_or(season.into()));
        playlist.position(pos);

        let invalid = invalid
            .into_iter()
            .map(|error| error.unwrap_err().to_string())
            .collect::<Vec<_>>();

        Ok((playlist, invalid))
    }

    fn play_show(&self, show: ShowId) -> Result<(Playlist, Vec<String>), Error> {
        let recent = self.db.get_show(show, SeasonId::from_recents)?;

        let sort = {
            let mut sort = Sort::release();
            sort.push(SortKind::Name);

            sort
        };

        let seasons = self.db.get_show_seasons(
            show,
            None,
            None,
            filter::Filter::none(),
            sort,
            SeasonId::from_row,
        )?;

        let mut errors = vec![];
        let mut playlist = Playlist::empty();

        for season in seasons {
            let (season_playlist, mut season_errors) =
                self.play_season(season, Some(show.into()))?;
            errors.append(&mut season_errors);
            playlist = playlist.merge(season_playlist, recent == Some(season));
        }

        Ok((playlist, errors))
    }

    fn play_item(&mut self, item: ItemId) -> Result<(Playlist, Vec<String>), Error> {
        match item {
            ItemId::Movie(id) => {
                let item = self.db.get_video(id)?;
                if item.path.try_exists()? {
                    tracing::debug!("Movie {} Play item fetched", item.name);
                    Ok((Playlist::single(item), vec![]))
                } else {
                    Err(Error::Raw(format!(
                        "{} does not exist",
                        item.path.display()
                    )))
                }
            }
            ItemId::Episode(id) => {
                let item = self.db.get_video(id)?;
                if item.path.try_exists()? {
                    tracing::debug!("Episode {} Play item fetched", item.name);
                    Ok((Playlist::single(item), vec![]))
                } else {
                    Err(Error::Raw(format!(
                        "{} does not exist",
                        item.path.display()
                    )))
                }
            }
            ItemId::Season(id) => self.play_season(id, None),
            ItemId::Show(id) => self.play_show(id),
        }
    }

    fn play_items(&mut self, items: impl Iterator<Item = ItemId>, flip: bool) -> Task<Message> {
        let mut errors = vec![];
        let mut playlist = Playlist::empty();

        for item in items {
            let (item_playlist, invalid_paths) = match self.play_item(item) {
                Ok(list) => list,
                Err(error) => {
                    let msg = (error.to_string(), toast::Status::Error);
                    errors.push(msg);
                    continue;
                }
            };
            if item_playlist.is_empty() {
                let invalids = invalid_paths
                    .into_iter()
                    .map(|message| (message, toast::Status::Error));

                errors.extend(invalids)
            } else {
                let invalids = invalid_paths
                    .into_iter()
                    .map(|message| (message, toast::Status::Warn));
                errors.extend(invalids);
                playlist = playlist.merge(item_playlist, flip)
            }
        }

        let (player, player_tasks) = Player::boot(
            self.window,
            self.config.video.clone(),
            playlist,
            self.available_fonts.clone(),
        );
        self.player = Some(player);
        self.screen = Screen::Player;

        Task::batch([
            player_tasks.map(Message::Player),
            Task::done(Message::PushToasts(errors)),
        ])
    }

    fn action(&mut self, action: Action, now: Instant) -> Task<Message> {
        match action {
            Action::Home(hat) => self.home.action(hat, now),
            Action::Player(pat) => self
                .player
                .as_mut()
                .map(|player| player.action(pat, now))
                .unwrap_or_default(),
            Action::Settings(sat) => self
                .settings
                .as_mut()
                .map(|settings| settings.action(sat))
                .unwrap_or_default(),
        }
    }
}

fn movie_map(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(shared::Thumbnail<Movie>, Task<ThumbnailTask<Movie>>)> {
    Movie::from_row(row).map(shared::Thumbnail::new)
}

fn show_map(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(shared::Thumbnail<Show>, Task<ThumbnailTask<Show>>)> {
    Show::from_row(row).map(shared::Thumbnail::new)
}

fn season_map(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(shared::Thumbnail<Season>, Task<ThumbnailTask<Season>>)> {
    Season::from_row(row).map(shared::Thumbnail::new)
}

fn episode_map(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(shared::Thumbnail<Episode>, Task<ThumbnailTask<Episode>>)> {
    Episode::from_row(row).map(shared::Thumbnail::new)
}

fn scan_task(
    db_path: std::path::PathBuf,
    dirs: Vec<Directory>,
    default_source: source::SourceSet,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_sub: Option<String>,
    preferred_audio: Option<String>,
) -> Task<Message> {
    Task::perform(
        async move {
            scan::scan_dirs(
                db_path,
                dirs,
                default_source,
                discoverer,
                movie_depth,
                restore,
                preferred_sub,
                preferred_audio,
            )
        },
        |(batch, res)| {
            if let Some(batch) = batch {
                batch.log()
            }
            Message::ScanComplete(res)
        },
    )
}
