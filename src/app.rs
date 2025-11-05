use iced::{
    Element, Length, Subscription, Task, Theme, font,
    keyboard::{self, Key, Modifiers},
    time::Instant,
    widget::{Container, Slider, Text, center, column, image, row, text_input},
    window,
};
use iced_video_player::{Video, VideoPlayer};
use std::path::PathBuf;
use std::time::Duration;

use crate::home::{Home, HomeMessage};
use crate::player::{Manager as Player, ManagerMessage as PlayerMessage};
use crate::toast;
use crate::utils::{
    Action, FilterMode, Layout, PlayItem, PlayerAction, Playlist, Sort, VideoSettings, load_fonts,
};

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
    None,
}

pub struct App {
    now: Instant,
    toasts: Vec<toast::Toast>,
    window: Option<window::Id>,

    screen: Screen,
    home: Home,

    player: Option<Player>,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let load_font = load_fonts().map(Message::FontLoad);
        let load_id = window::oldest().map(Message::WindowId);

        let (home, home_tasks) = Home::boot(Layout::default(), FilterMode::default());
        let home_tasks = home_tasks.map(Message::Home);

        let new = Self::new(home);

        let tasks = Task::batch([load_font, load_id, home_tasks]);

        (new, tasks)
    }

    fn new(home: Home) -> Self {
        Self {
            screen: Screen::Home,
            now: Instant::now(),
            toasts: vec![],
            window: None,
            player: None,
            home,
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
            Message::Exit(id) => {
                let Some(own) = &self.window else {
                    return Task::none();
                };

                if id == *own {
                    self.player.take();
                    self.screen = Screen::Home;
                    window::close::<Message>(own.clone()).discard()
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
                (Screen::Home, Action::Home(action)) => self.home.action(action),
                (Screen::Home, Action::Back) => self.home.back(),
                (Screen::Home, Action::Forward) => self.home.forward(),
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


        Subscription::batch([animating, keys, exit, player])
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
        let (player, tasks) = Player::boot(self.window.clone(), VideoSettings::default(), playlist);
        self.player = Some(player);
        self.screen = Screen::Player;

        tasks.map(Message::Player)
    }
}

fn key_action(key: Key, modifiers: Modifiers) -> Option<Action> {
    match key {
        Key::Named(keyboard::key::Named::ArrowLeft) if modifiers.alt() => Some(Action::Back),
        Key::Named(keyboard::key::Named::ArrowRight) if modifiers.alt() => Some(Action::Forward),
        key => handle_keypress(key, modifiers).map(Action::Player),
    }
}

fn handle_keypress(key: Key, modifiers: Modifiers) -> Option<PlayerAction> {
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
