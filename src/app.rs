use chrono::{DateTime, Local};
use iced::{
    Element, Subscription, Task, Theme, event, font,
    keyboard::{self, Key, Modifiers},
    time::{self, Duration, Instant},
    window,
};

use crate::db::{self, Query};
use crate::error::Error;
use crate::home::{Home, HomeMessage, shared};
use crate::models::{
    self, Collection, CollectionId, Episode, EpisodeId, ItemId, Movie, MovieId, Season, SeasonId,
    Show, ShowId, SimpleCollection, collection, collection::Items,
};
use crate::player::{Manager as Player, ManagerMessage as PlayerMessage};
use crate::toast;
use crate::utils::{
    Filter, FilterMode, HomeAction, Layout, PlayId, PlayItem, PlayerAction, Playlist, SearchFilter,
    Sort, VideoSettings, load_fonts,
};

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

#[derive(Clone, Debug, Copy)]
pub enum Screen {
    Home,
    Player,
    // Settings,
    // Log,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Back(Screen),
    Forward,
    Home(HomeAction),
    Player(PlayerAction),
}

impl From<PlayerAction> for Action {
    fn from(value: PlayerAction) -> Self {
        Self::Player(value)
    }
}

impl From<HomeAction> for Action {
    fn from(value: HomeAction) -> Self {
        Self::Home(value)
    }
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
    Refresh(Instant),
    LastWatched(PlayId),
    VideoStats(PlayItem),
    Key {
        key: Key,
        modifiers: Modifiers,
        press: bool,
    },
    Back,
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

    player: Option<Player>,

    last_refresh: Instant,
    refresh_interval: Duration,

    db: db::Database,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let load_font = load_fonts().map(Message::FontLoad);
        let load_id = window::oldest().map(Message::WindowId);

        let (home, home_tasks) = Home::boot(
            Layout::default(),
            Filter::new(FilterMode::default()),
            Sort::new_with_name(),
            Some(5),
        );

        let new = Self::new(home);

        let tasks = Task::batch([load_font, load_id, home_tasks]);

        (new, tasks)
    }

    fn new(home: Home) -> Self {
        let db = db::Database::open_test_db().expect("Failed to open DB");

        Self {
            screen: Screen::Home,
            now: Instant::now(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(75),
            toasts: vec![],
            window: None,
            player: None,
            home,
            db,
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
                if refresh.duration_since(self.last_refresh) >= self.refresh_interval {
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
            Message::Query(query) => {
                // todo
                let _res = match query.execute(&self.db) {
                    Ok(suc) => suc,
                    Err(error) => {
                        dbg!(&error.error);
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
            Message::Key {
                key,
                modifiers,
                press,
            } => key_action(key, modifiers, self.screen, press)
                .map(|action| self.action(action, now))
                .unwrap_or_default(),
            Message::Back => self.action(Action::Back(self.screen), now),
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
            Message::LoadSearch(search, filter) => {
                let items = match self
                    .db
                    .search(search, filter, Some(5), shared::SearchView::new)
                {
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
                        Err(error) => Task::done(Message::PushToast(
                            dbg!(error).to_string(),
                            toast::Status::Error,
                        )),
                    },
                    PlayId::Episode(id) => match self.db.last_watched_episode(id, now) {
                        Ok(_) => Task::none(),
                        Err(error) => Task::done(Message::PushToast(
                            dbg!(error).to_string(),
                            toast::Status::Error,
                        )),
                    },
                }
            }
            Message::VideoStats(item) => match item.id {
                PlayId::Movie(id) => {
                    match self
                        .db
                        .update_movie_stats(id, item.watch_count, item.progress)
                    {
                        Ok(_) => Task::none(),
                        Err(error) => Task::done(Message::PushToast(
                            dbg!(error).to_string(),
                            toast::Status::Error,
                        )),
                    }
                }
                PlayId::Episode(id) => {
                    match self
                        .db
                        .update_episode_stats(id, item.watch_count, item.progress)
                    {
                        Ok(_) => Task::none(),
                        Err(error) => Task::done(Message::PushToast(
                            dbg!(error).to_string(),
                            toast::Status::Error,
                        )),
                    }
                }
            },
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
        };

        toast::manager(content, &self.toasts, Message::CloseToast).into()
    }

    pub fn theme(&self) -> Option<Theme> {
        Some(Theme::TokyoNight)
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

        let keys = keyboard::on_key_press(|key, modifiers| {
            Some(Message::Key {
                key,
                modifiers,
                press: true,
            })
        });

        let exit = window::close_requests().map(Message::Exit);

        let home = self.home.subscription();

        let refresh = time::every(self.refresh_interval).map(Message::Refresh);

        Subscription::batch([animating, keys, exit, player, refresh, home])
    }

    fn push_toast(&mut self, toast: toast::Toast) {
        // todo
        // match toast.status {
        //     Status::Info => info!(toast.body),
        //     Status::Warn => warn!(toast.body),
        //     Status::Success => info!(toast.body),
        //     Status::Error => error!(toast.body),
        // }

        self.toasts.push(toast);
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

        let (player, player_tasks) = Player::boot(self.window, VideoSettings::default(), playlist);
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
            Action::Forward => self.home.forward(now),
            Action::Back(Screen::Home) => self.home.back(now),
            Action::Back(Screen::Player) => {
                self.screen = Screen::Home;
                let stats = self
                    .player
                    .take()
                    .as_mut()
                    .and_then(|player| player.stats().map(Task::done))
                    .unwrap_or_default();

                Task::batch([self.home.refresh(now), stats])
            }
        }
    }
}

fn key_action(key: Key, modifiers: Modifiers, screen: Screen, press: bool) -> Option<Action> {
    use keyboard::key::Named;
    match key {
        Key::Named(Named::ArrowLeft) if modifiers.alt() => Some(Action::Back(screen)),
        Key::Named(Named::NavigateNext) | Key::Named(Named::BrowserForward) => {
            Some(Action::Forward)
        }
        Key::Named(Named::NavigatePrevious) | Key::Named(Named::BrowserBack) => {
            Some(Action::Back(screen))
        }
        Key::Named(Named::ArrowRight) if modifiers.alt() => Some(Action::Forward),
        key => match screen {
            Screen::Player => player_keypress(key, modifiers, press).map(Action::Player),
            Screen::Home => home_keypress(key, modifiers, press).map(Action::Home),
        },
    }
}

fn home_keypress(key: Key, modifiers: Modifiers, _press: bool) -> Option<HomeAction> {
    use keyboard::key::Named;

    let action = match key {
        Key::Character(char) if char.as_str() == "l" => HomeAction::LayoutToggle,
        Key::Character(char) if char.as_str() == "r" && modifiers.shift() => HomeAction::Refresh,
        Key::Character(char) if char.as_str() == "r" => HomeAction::RefreshContent,
        Key::Character(char) if char.as_str() == "s" => HomeAction::SearchToggle,
        Key::Character(char) if char.as_str() == "f" && modifiers.command() => {
            HomeAction::SearchToggle
        }
        Key::Named(Named::Escape) => HomeAction::CloseModal,
        _ => return None,
    };

    Some(action)
}

fn player_keypress(key: Key, modifiers: Modifiers, _press: bool) -> Option<PlayerAction> {
    use keyboard::key::Named;

    let action = match key {
        Key::Named(Named::Space) => PlayerAction::PlayToggle,
        Key::Named(Named::MediaPlayPause) => PlayerAction::PlayToggle,
        Key::Named(Named::ArrowLeft) if modifiers.command() => PlayerAction::PlayPrevious,
        Key::Named(Named::MediaTrackPrevious) => PlayerAction::PlayPrevious,
        Key::Named(Named::ArrowRight) if modifiers.command() => PlayerAction::PlayNext,
        Key::Named(Named::MediaTrackNext) => PlayerAction::PlayNext,

        Key::Named(Named::Enter) => PlayerAction::FullscreenToggle,
        Key::Named(Named::Escape) => PlayerAction::Exit,
        Key::Character(char) if char.as_str() == "f" => PlayerAction::FullscreenToggle,

        Key::Named(Named::ArrowLeft) if modifiers.shift() => PlayerAction::SeekBackShift,
        Key::Named(Named::ArrowLeft) => PlayerAction::SeekBack,
        Key::Named(Named::ArrowRight) if modifiers.shift() => PlayerAction::SeekFrontShift,
        Key::Named(Named::ArrowRight) => PlayerAction::SeekFront,

        Key::Named(Named::ArrowUp) => PlayerAction::VolumeIncrease,
        Key::Named(Named::ArrowDown) => PlayerAction::VolumeDecrease,
        Key::Character(char) if char.as_str() == "m" => PlayerAction::MuteToggle,

        Key::Character(char) if char.as_str() == "c" => PlayerAction::SpeedIncrease,
        Key::Named(Named::PlaySpeedUp) => PlayerAction::SpeedIncrease,
        Key::Character(char) if char.as_str() == "x" => PlayerAction::SpeedDecrease,
        Key::Named(Named::PlaySpeedDown) => PlayerAction::SpeedDecrease,
        Key::Character(char) if char.as_str() == "z" => PlayerAction::SpeedReset,
        Key::Named(Named::PlaySpeedReset) => PlayerAction::SpeedReset,

        Key::Character(char) if char.as_str() == "s" && modifiers.shift() => {
            PlayerAction::VideoConfig
        }

        Key::Character(char) if char.as_str() == "s" => PlayerAction::SubtitlesToggle,
        Key::Named(Named::Subtitle) => PlayerAction::SubtitlesToggle,

        Key::Character(char) if char.as_str() == "b" => PlayerAction::VideoComment,

        _ => return None,
    };

    Some(action)
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
