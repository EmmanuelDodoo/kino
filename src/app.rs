use iced::{
    Element, Length, Subscription, Task, Theme, font,
    keyboard::{self, Key, Modifiers},
    time::{self, Duration, Instant},
    widget::{Container, Slider, Text, center, column, image, row, text_input},
    window,
};
use iced_video_player::{Video, VideoPlayer};
use std::path::PathBuf;

use crate::db;
use crate::home::{Home, HomeMessage};
use crate::models::{
    CollectionId, CollectionView, EpisodeId, ItemId, MovieId, SeasonId, ShowId, collection,
};
use crate::player::{Manager as Player, ManagerMessage as PlayerMessage};
use crate::toast;
use crate::utils::{
    Action, Filter, FilterMode, HomeAction, Layout, PlayItem, PlayerAction, Playlist, Sort,
    VideoSettings, load_fonts,
};

#[derive(Debug, Clone, Copy)]
pub enum FetchId {
    Recents,
    Shows,
    Movies,
    Collections(bool),
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Episode(EpisodeId),
    Collection((CollectionView, CollectionId)),
}

#[derive(Clone, Debug, Copy)]
pub enum Screen {
    Home,
    Player,
    // Settings,
    // Log,
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
    Action(Action),
    PlayItem(PlayItem),
    PlayItems(Vec<PlayItem>),
    Animate,
    Fetch {
        id: FetchId,
        filters: Filter,
        sort: Sort,
        limit: Option<i32>,
        offset: Option<i32>,
    },
    Refresh(Instant),
    None,
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
                self.toasts.remove(idx);

                Task::none()
            }
            Message::Home(hsg) => self.home.update(hsg, now),
            Message::Player(psg) => {
                let Some(player) = self.player.as_mut() else {
                    return Task::none();
                };

                player.update(psg, now)
            }
            Message::PlayItem(item) => self.play_item(std::iter::once(item)),
            Message::PlayItems(items) => self.play_item(items.into_iter()),
            Message::Action(action) => match (self.screen, action) {
                // todo: update last refresh
                (Screen::Home, Action::Home(action)) => self.home.action(action, now),
                (Screen::Home, Action::Back) => self.home.back(now),
                (Screen::Home, Action::Forward) => self.home.forward(now),
                (Screen::Home, _) => Task::none(),

                (Screen::Player, Action::Player(action)) => self
                    .player
                    .as_mut()
                    .map(|player| player.action(action, now))
                    .unwrap_or_default(),
                (Screen::Player, Action::Back) => {
                    self.player.take();
                    self.screen = Screen::Home;
                    Task::none()
                }
                (Screen::Player, _) => Task::none(),
            },
            Message::Fetch {
                id,
                filters: filter,
                sort,
                limit,
                offset,
            } => match id {
                FetchId::Shows => {
                    let shows = match self.db.get_shows(limit, offset, filter, sort) {
                        Ok(shows) => shows,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_shows(shows);

                    Task::none()
                }
                FetchId::Movies => {
                    let movies = match self.db.get_movies(limit, offset, filter, sort) {
                        Ok(movies) => movies,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_movies(movies);

                    Task::none()
                }
                FetchId::Recents => {
                    let movies = match self.db.get_movies(limit, offset, filter, sort) {
                        Ok(movies) => movies,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };
                    let shows = match self.db.get_shows(limit, offset, filter, sort) {
                        Ok(shows) => shows,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_recents(movies, shows);

                    Task::none()
                }
                FetchId::Show(id) => {
                    let show = match self.db.get_show(id) {
                        Ok(show) => show,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    let seasons = match self.db.get_show_seasons(id, limit, offset, filter, sort) {
                        Ok(seasons) => seasons,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_show(show, seasons);

                    Task::none()
                }
                FetchId::Season(id) => {
                    let season = match self.db.get_season(id) {
                        Ok(season) => season,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    let episodes =
                        match self.db.get_season_episodes(id, limit, offset, filter, sort) {
                            Ok(episodes) => episodes,
                            Err(error) => {
                                let msg =
                                    Message::PushToast(error.to_string(), toast::Status::Error);
                                return Task::done(msg);
                            }
                        };

                    self.home.fetched_season(season, episodes);

                    Task::none()
                }
                FetchId::Episode(id) => {
                    let episode = match self.db.get_episode(id) {
                        Ok(episode) => episode,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_episode(episode);

                    Task::none()
                }
                FetchId::Movie(id) => {
                    let movie = match self.db.get_movie(id) {
                        Ok(movie) => movie,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_movie(movie);

                    Task::none()
                }
                FetchId::Collections(state) => {
                    //todo: collection sorts
                    let collections = match self.db.get_collections(collection::Sort::default()) {
                        Ok(collection) => collection,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_collections(collections, state)
                }
                FetchId::Collection(key) => {
                    let items = match self
                        .db
                        .get_collection_items(key.1, limit, offset, filter, sort)
                    {
                        Ok(items) => items,
                        Err(error) => {
                            let msg = Message::PushToast(error.to_string(), toast::Status::Error);
                            return Task::done(msg);
                        }
                    };

                    self.home.fetched_collection(key, items)
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.screen {
            Screen::Home => self.home.view(self.now).map(Message::Home),
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

        let animating = if self.home.is_animating(self.now) || animating {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        };

        let keys = keyboard::on_key_press(key_action).map(Message::Action);

        let exit = window::close_requests().map(Message::Exit);

        let player = player.map(Message::Player);

        let refresh = time::every(self.refresh_interval).map(Message::Refresh);

        Subscription::batch([animating, keys, exit, player, refresh])
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

    pub fn push_toasts(&mut self, toasts: impl Iterator<Item = toast::Toast>) {
        for toast in toasts {
            self.push_toast(toast)
        }
    }

    fn play_item(&mut self, items: impl Iterator<Item = PlayItem>) -> Task<Message> {
        let playlist = Playlist::new(items);
        let (player, tasks) = Player::boot(self.window, VideoSettings::default(), playlist);
        self.player = Some(player);
        self.screen = Screen::Player;

        tasks.map(Message::Player)
    }
}

fn key_action(key: Key, modifiers: Modifiers) -> Option<Action> {
    match key {
        Key::Named(keyboard::key::Named::ArrowLeft) if modifiers.alt() => Some(Action::Back),
        Key::Named(keyboard::key::Named::ArrowRight) if modifiers.alt() => Some(Action::Forward),
        key => player_keypress(key, modifiers).map(Action::Player),
    }
}

fn home_keypress(key: Key, modifiers: Modifiers) -> Option<HomeAction> {
    use keyboard::key::Named;

    let action = match key {
        Key::Character(char) if char.as_str() == "l" => HomeAction::LayoutToggle,
        Key::Character(char) if char.as_str() == "r" && modifiers.shift() => HomeAction::Refresh,
        Key::Character(char) if char.as_str() == "r" => HomeAction::RefreshContent,
        _ => return None,
    };

    Some(action)
}

fn player_keypress(key: Key, modifiers: Modifiers) -> Option<PlayerAction> {
    use keyboard::key::Named;

    let action = match key {
        Key::Named(Named::Space) => PlayerAction::PlayToggle,
        Key::Named(Named::ArrowLeft) if modifiers.command() => PlayerAction::PlayPrevious,
        Key::Named(Named::ArrowRight) if modifiers.command() => PlayerAction::PlayNext,

        Key::Named(Named::Enter) => PlayerAction::FullscreenToggle,
        Key::Named(Named::Escape) => PlayerAction::FullscreenExit,
        Key::Character(char) if char.as_str() == "f" => PlayerAction::FullscreenToggle,

        Key::Named(Named::ArrowLeft) if modifiers.shift() => PlayerAction::SeekBackShift,
        Key::Named(Named::ArrowLeft) => PlayerAction::SeekBack,
        Key::Named(Named::ArrowRight) if modifiers.shift() => PlayerAction::SeekFrontShift,
        Key::Named(Named::ArrowRight) => PlayerAction::SeekFront,

        Key::Named(Named::ArrowUp) => PlayerAction::VolumeIncrease,
        Key::Named(Named::ArrowDown) => PlayerAction::VolumeDecrease,
        Key::Character(char) if char.as_str() == "m" => PlayerAction::MuteToggle,

        Key::Character(char) if char.as_str() == "c" => PlayerAction::SpeedIncrease,
        Key::Character(char) if char.as_str() == "x" => PlayerAction::SpeedDecrease,
        Key::Character(char) if char.as_str() == "z" => PlayerAction::SpeedReset,

        Key::Character(char) if char.as_str() == "s" && modifiers.shift() => {
            PlayerAction::VideoConfig
        }

        Key::Character(char) if char.as_str() == "s" => PlayerAction::SubtitlesToggle,

        Key::Character(char) if char.as_str() == "b" => PlayerAction::VideoComment,

        _ => return None,
    };

    Some(action)
}
