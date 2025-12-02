use chrono::Local;
use iced::{
    Element, Subscription, Task, Theme, font,
    keyboard::{self, Key, Modifiers},
    time::{self, Instant},
    window,
};

use crate::db::{self, Query};
use crate::error::Error;
use crate::home::{Home, HomeMessage, shared};
use crate::models::{
    self, Collection, CollectionId, DirectoryId, Episode, EpisodeId, ItemId, Movie, MovieId,
    Season, SeasonId, Show, ShowId, SimpleCollection, collection, collection::Items,
};
use crate::player::{Manager as Player, ManagerMessage as PlayerMessage};
use crate::scan;
use crate::settings::{Settings, SettingsMessage};
use crate::toast;
use crate::utils::{
    Action, Config, Filter, KeyPress, Layout, PlayId, PlayItem, Playlist, Screen, Sort,
    filter::FilterMode, filter::SearchFilter, load_fonts,
};

const DB_PATH: &str = "test.db";

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
pub enum Message {
    FontLoad(Result<(), font::Error>),
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
        filters: Filter,
        sort: Sort,
        limit: Option<i32>,
        offset: Option<i32>,
    },
    FetchDirectories,
    Refresh(Instant),
    LastWatched(PlayId),
    VideoStats(PlayItem),
    Key {
        key: Key,
        modifiers: Modifiers,
    },
    Back,
    Random,
    SettingsOpen,
    SaveSettings,
    Layout(Layout),
    CaptureKeys(bool),
    Scan,
    ScanComplete(Vec<DirectoryId>),
    None,
}

impl Message {
    pub fn fetch_simple_collections() -> Self {
        Message::Fetch {
            id: FetchId::CollectionsSimple,
            filters: Filter::none(),
            sort: Sort::new(),
            limit: None,
            offset: None,
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

    config: Config,

    db: db::Database,

    is_capturing_keys: bool,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let load_font = load_fonts().map(Message::FontLoad);
        let load_id = window::oldest().map(Message::WindowId);
        let settings = Config::defaults();

        let (home, home_tasks) = Home::boot(
            settings.layout(),
            Filter::new(FilterMode::default()),
            Sort::new_with_name(),
            settings.general.recents_limit,
        );

        let new = Self::new(settings, home);

        let tasks = Task::batch([load_font, load_id, home_tasks]);

        (new, tasks)
    }

    fn new(settings: Config, home: Home) -> Self {
        let db = db::Database::open_test_db(DB_PATH).expect("Failed to open DB");

        Self {
            screen: Screen::Home,
            now: Instant::now(),
            last_refresh: Instant::now(),
            toasts: vec![],
            window: None,
            player: None,
            settings: None,
            config: settings,
            home,
            db,
            is_capturing_keys: false,
        }
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::None => Task::none(),
            Message::Animate => Task::none(),
            Message::FontLoad(Ok(_)) => Task::none(),
            Message::FontLoad(Err(_)) => {
                let msg = Message::PushToast("Font load error".to_owned(), toast::Status::Error);

                Task::done(msg)
            }
            Message::WindowId(window) => {
                self.window = window;
                Task::none()
            }
            Message::Refresh(refresh) => {
                if refresh.duration_since(self.last_refresh) >= self.config.refresh_interval() {
                    self.last_refresh = refresh;
                    self.home.refresh(now)
                } else {
                    Task::none()
                }
            }
            Message::Exit(id) => {
                let Some(own) = &self.window else {
                    return Task::none();
                };

                if id == *own {
                    self.player.take();
                    self.screen = Screen::Home;
                    window::close::<Message>(*own).discard()
                } else {
                    Task::none()
                }
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
                // todo
                let _res = match query.execute(&self.db) {
                    Ok(suc) => suc,
                    Err(error) => {
                        let msg = Message::PushToast(error.error.to_string(), toast::Status::Error);
                        return Task::done(msg);
                    }
                };

                Task::none()
            }
            Message::PlayItem(item) => self.play_items(std::iter::once(item)),
            Message::PlayItems(items) => self.play_items(items.into_iter()),
            Message::PlayCollectionItems { id, items } => {
                let items = match self.db.get_collection_items(id) {
                    Ok(items) => items,
                    Err(error) => {
                        let msg = Message::PushToast(error.to_string(), toast::Status::Error);
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

                self.play_items(items)
            }
            Message::Key { key, modifiers } => {
                let keypress = KeyPress::new(key, modifiers.into());

                if let Some(settings) = self.settings.as_mut()
                    && self.is_capturing_keys
                {
                    settings.captured_key(keypress)
                } else {
                    self.config
                        .keystore
                        .action(keypress, self.screen)
                        .map(|action| self.action(action, now))
                        .unwrap_or_default()
                }
            }
            Message::CaptureKeys(capture) => {
                self.is_capturing_keys = capture;
                Task::none()
            }
            Message::Back => match self.screen {
                Screen::Home => self.home.back(now),
                Screen::Player => {
                    self.screen = Screen::Home;
                    let stats = match self.player.take() {
                        Some(mut player) => {
                            self.config.video.speed = player.settings.speed;
                            self.config.video.show_subtitles = player.settings.show_subtitles;
                            self.config.video.volume = player.settings.volume;

                            player.stats()
                        }
                        None => None,
                    };

                    let stats = stats.map(Task::done).unwrap_or_default();

                    Task::batch([self.home.refresh(now), stats])
                }
                Screen::Settings => {
                    self.settings.take();
                    self.player.take();

                    self.screen = Screen::Home;

                    self.home.refresh(now)
                }
            },
            Message::Fetch {
                id,
                filters: filter,
                sort,
                limit,
                offset,
            } => match id {
                FetchId::CollectionsSimple => {
                    let collections = match self
                        .db
                        .get_collections(collection::Sort::View, SimpleCollection::from_row)
                    {
                        Ok(collection) => collection,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    match self.player.as_mut() {
                        Some(player) => player.fetched_collections(collections),
                        None => self.home.fetch_collections_simple(collections),
                    }
                }
                FetchId::Shows => {
                    let shows =
                        match self
                            .db
                            .get_shows(limit, offset, filter, sort, shared::Thumbnail::new)
                        {
                            Ok(shows) => shows,
                            Err(error) => {
                                let msg =
                                    Message::PushToast(error.to_string(), toast::Status::Error);
                                return Task::done(msg);
                            }
                        };

                    self.home.fetched_shows(shows)
                }
                FetchId::Movies => {
                    let movies = match self.db.get_movies(
                        limit,
                        offset,
                        filter,
                        sort,
                        shared::Thumbnail::new,
                    ) {
                        Ok(movies) => movies,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_movies(movies)
                }
                FetchId::Recents => {
                    let movies = match self.db.get_movies(
                        limit,
                        offset,
                        filter,
                        sort,
                        shared::Thumbnail::new,
                    ) {
                        Ok(movies) => movies,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };
                    let shows =
                        match self
                            .db
                            .get_shows(limit, offset, filter, sort, shared::Thumbnail::new)
                        {
                            Ok(shows) => shows,
                            Err(error) => {
                                let msg =
                                    Message::PushToast(error.to_string(), toast::Status::Error);
                                return Task::done(msg);
                            }
                        };

                    self.home.fetched_recents(movies, shows)
                }
                FetchId::Show(id) => {
                    let show = match self.db.get_show(id, show_map) {
                        Ok(show) => show,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    let seasons = match self
                        .db
                        .get_show_seasons(id, limit, offset, filter, sort, season_map)
                    {
                        Ok(seasons) => seasons,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_show(show, seasons)
                }
                FetchId::Season(id) => {
                    let season = match self.db.get_season(id, season_map) {
                        Ok(season) => season,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    let episodes = match self.db.get_season_episodes(
                        id,
                        limit,
                        offset,
                        filter,
                        sort,
                        episode_map,
                    ) {
                        Ok(episodes) => episodes,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_season(season, episodes)
                }
                FetchId::Episode(id) => {
                    let episode = match self.db.get_episode(id, episode_map) {
                        Ok(episode) => episode,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_episode(episode)
                }
                FetchId::Movie(id) => {
                    let movie = match self.db.get_movie(id, movie_map) {
                        Ok(movie) => movie,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_movie(movie)
                }
                FetchId::Collections => {
                    //todo: collection sorts
                    let collections = match self
                        .db
                        .get_collections(collection::Sort::default(), Collection::from_row)
                    {
                        Ok(collection) => collection,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_collections(collections)
                }
                FetchId::Collection(id) => {
                    let collection = match self.db.get_collection(id, Collection::from_row) {
                        Ok(collection) => collection,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    let items = match self.db.get_collection_members(
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
                        Ok(items) => items,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_collection(collection, items)
                }
            },
            Message::FetchDirectories => {
                let Some(settings) = self.settings.as_mut() else {
                    return Task::none();
                };

                let dirs = match self.db.get_directories() {
                    Ok(dirs) => dirs,
                    Err(error) => {
                        let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                        return Task::done(msg);
                    }
                };

                settings.fetched_directories(dirs);

                Task::none()
            }
            Message::LoadSearch(search, filter) => {
                let items = match self.db.search(
                    search,
                    filter,
                    self.config.search_limit(),
                    shared::SearchView::new,
                ) {
                    Ok(items) => items,
                    Err(error) => {
                        let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                        return Task::done(msg);
                    }
                };

                self.home.loaded_search(items)
            }
            Message::FetchMembershipIds(item) => {
                let memberships = match self.db.get_item_membership_ids(item) {
                    Ok(memberships) => memberships,
                    Err(error) => {
                        let msg = Message::PushToast(error.to_string(), toast::Status::Error);
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
                        Ok(memberships) => memberships,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                self.home.fetched_memberships(memberships)
            }
            Message::ToggleMembership { item, collections } => {
                let msg = match self.db.toggle_membership(item, collections) {
                    Ok(true) => Message::PushToast(
                        "Collections Updated!".to_owned(),
                        toast::Status::Success,
                    ),
                    Ok(false) => Message::None,
                    Err(error) => Message::PushToast(error.to_string(), toast::Status::Error),
                };

                let refresh = self.home.content_refresh(now);
                Task::batch([Task::done(msg), refresh])
            }
            Message::LastWatched(id) => {
                let now = Local::now();
                let now = models::datetime_to_sql(&now);

                match id {
                    PlayId::Movie(id) => match self.db.last_watched_movie(id, now) {
                        Ok(_) => Task::none(),
                        Err(error) => {
                            Task::done(Message::PushToast(error.to_string(), toast::Status::Error))
                        }
                    },
                    PlayId::Episode(id) => match self.db.last_watched_episode(id, now) {
                        Ok(_) => Task::none(),
                        Err(error) => {
                            Task::done(Message::PushToast(error.to_string(), toast::Status::Error))
                        }
                    },
                }
            }
            Message::VideoStats(item) => match item.id {
                PlayId::Movie(id) => {
                    match self.db.update_movie_stats(
                        id,
                        item.watch_count,
                        item.progress,
                        item.duration,
                    ) {
                        Ok(_) => Task::none(),
                        Err(error) => {
                            Task::done(Message::PushToast(error.to_string(), toast::Status::Error))
                        }
                    }
                }
                PlayId::Episode(id) => {
                    match self.db.update_episode_stats(
                        id,
                        item.watch_count,
                        item.progress,
                        item.duration,
                    ) {
                        Ok(_) => Task::none(),
                        Err(error) => {
                            Task::done(Message::PushToast(error.to_string(), toast::Status::Error))
                        }
                    }
                }
            },
            Message::Random => {
                let random = match self.db.get_random() {
                    Ok(random) => random,
                    Err(error) => {
                        let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                        return Task::done(msg);
                    }
                };

                self.home.goto(random.into(), now)
            }
            Message::SettingsOpen => self.settings(),
            Message::SaveSettings => {
                // todo: Directories changes
                self.screen = Screen::Home;

                let Some(settings) = self.settings.take() else {
                    return Task::none();
                };

                self.home.layout(settings.config.layout());
                self.home
                    .recents_limit(settings.config.general.recents_limit);
                self.config = settings.config;

                // todo: Scan dirs
                let dirs = settings.directories;

                let msg = match self.db.toggle_directories(dirs) {
                    Ok(true) => Message::PushToast(
                        "Directories Updated!".to_owned(),
                        toast::Status::Success,
                    ),
                    Ok(false) => Message::None,
                    Err(error) => Message::PushToast(error.to_string(), toast::Status::Error),
                };

                let refresh = self.home.refresh(now);

                Task::batch([refresh, Task::done(msg)])
            }
            Message::Layout(layout) => {
                self.config.general.layout = layout;
                Task::none()
            }
            Message::Scan => {
                let dirs = match self.db.get_directories() {
                    Ok(dirs) => dirs,
                    Err(error) => {
                        return Task::done(Message::PushToast(
                            error.to_string(),
                            toast::Status::Error,
                        ));
                    }
                };

                let discoverer = self.config.general.scan_discoverer;
                let home_task = self.home.scanning(true, now);

                let scan = Task::perform(
                    async move { scan::scan_dirs(DB_PATH, dirs, discoverer) },
                    |(todo, res)| {
                        if let Some(res) = todo {
                            dbg!(res.successes.len());
                            dbg!(res.failures.len());
                        }
                        Message::ScanComplete(res)
                    },
                );

                Task::batch([home_task, scan])
            }
            Message::ScanComplete(scanned) => {
                let last_scan = Local::now();
                let last_scan = models::datetime_to_sql(&last_scan);

                let _todo = match self.db.last_scans(scanned, last_scan) {
                    Ok(rows) => rows,
                    Err(error) => {
                        return Task::done(Message::PushToast(
                            error.to_string(),
                            toast::Status::Error,
                        ));
                    }
                };

                self.home.scanning(false, now)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = self.theme().unwrap();
        let content: Element<'_, Message> = match self.screen {
            Screen::Home => self.home.view(&theme, self.now).map(Message::Home),
            Screen::Player => {
                let player = self.player.as_ref().unwrap();

                player.view(self.now).map(Message::Player)
            }
            Screen::Settings => {
                let settings = self.settings.as_ref().unwrap();
                settings.view().map(Message::Settings)
            }
        };

        toast::manager(content, &self.toasts, Message::CloseToast).into()
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

        let keys = keyboard::listen().map(|event| {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return Message::None;
            };

            Message::Key { key, modifiers }
        });

        let exit = window::close_requests().map(Message::Exit);

        let home = self.home.subscription();

        let refresh = time::every(self.config.refresh_interval()).map(Message::Refresh);

        Subscription::batch([animating, keys, exit, player, refresh, home])
    }

    fn settings(&mut self) -> Task<Message> {
        let (settings, tasks) = Settings::boot(self.config.clone());
        self.screen = Screen::Settings;
        self.settings = Some(settings);

        tasks
    }

    fn push_toast(&mut self, toast: toast::Toast) {
        // todo
        // match toast.status {
        //     Status::Info => info!(toast.body),
        //     Status::Warn => warn!(toast.body),
        //     Status::Success => info!(toast.body),
        //     Status::Error => error!(toast.body),
        // }

        self.toasts.push(dbg!(toast));
    }

    fn push_toasts(&mut self, toasts: impl Iterator<Item = toast::Toast>) {
        for toast in toasts {
            self.push_toast(toast)
        }
    }

    fn play_season(&self, season: SeasonId) -> Result<(Playlist, Vec<String>), Error> {
        let recent = self.db.get_season(season, EpisodeId::from_recents)?;
        let items = self.db.get_season_episodes(
            season,
            None,
            None,
            Filter::none(),
            Sort::release(),
            PlayItem::from_episode,
        )?;

        let pos = recent
            .and_then(|recent| {
                items
                    .iter()
                    .position(|item| item.id == PlayId::Episode(recent))
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
        let mut playlist = Playlist::new(valid);
        playlist.position(pos);

        let invalid = invalid
            .into_iter()
            .map(|error| error.unwrap_err().to_string())
            .collect::<Vec<_>>();

        Ok((playlist, invalid))
    }

    fn play_show(&self, show: ShowId) -> Result<(Playlist, Vec<String>), Error> {
        let recent = self.db.get_show(show, SeasonId::from_recents)?;
        let seasons = self.db.get_show_seasons(
            show,
            None,
            None,
            Filter::none(),
            Sort::release(),
            SeasonId::from_row,
        )?;

        let mut errors = vec![];
        let mut playlist = Playlist::empty();

        for season in seasons {
            let (season_playlist, mut season_errors) = self.play_season(season)?;
            errors.append(&mut season_errors);
            playlist = playlist.merge(season_playlist, recent == Some(season));
        }

        Ok((playlist, errors))
    }

    fn play_item(&mut self, item: ItemId) -> Result<(Playlist, Vec<String>), Error> {
        match item {
            ItemId::Movie(id) => {
                let item = self.db.get_movie(id, PlayItem::from_movie)?;
                if item.path.try_exists()? {
                    Ok((Playlist::single(item), vec![]))
                } else {
                    Err(Error::Raw(format!(
                        "{} does not exist",
                        item.path.to_string_lossy()
                    )))
                }
            }
            ItemId::Episode(id) => {
                let item = self.db.get_episode(id, PlayItem::from_episode)?;
                if item.path.try_exists()? {
                    Ok((Playlist::single(item), vec![]))
                } else {
                    Err(Error::Raw(format!(
                        "{} does not exist",
                        item.path.to_string_lossy()
                    )))
                }
            }
            ItemId::Season(id) => self.play_season(id),
            ItemId::Show(id) => self.play_show(id),
        }
    }

    fn play_items(&mut self, items: impl Iterator<Item = ItemId>) -> Task<Message> {
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
                playlist = playlist.merge(item_playlist, false)
            }
        }

        let (player, player_tasks) = Player::boot(self.window, self.config.video, playlist);
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

fn movie_map(row: &rusqlite::Row<'_>) -> rusqlite::Result<shared::Thumbnail<Movie>> {
    Movie::from_row(row).map(shared::Thumbnail::new)
}

fn show_map(row: &rusqlite::Row<'_>) -> rusqlite::Result<shared::Thumbnail<Show>> {
    Show::from_row(row).map(shared::Thumbnail::new)
}

fn season_map(row: &rusqlite::Row<'_>) -> rusqlite::Result<shared::Thumbnail<Season>> {
    Season::from_row(row).map(shared::Thumbnail::new)
}

fn episode_map(row: &rusqlite::Row<'_>) -> rusqlite::Result<shared::Thumbnail<Episode>> {
    Episode::from_row(row).map(shared::Thumbnail::new)
}
