use iced::{
    Element, Length, Subscription, Task,
    time::Instant,
    widget::{center, text},
};

use super::HomeMessage;
use super::movies::{Movies, MoviesMessage};
use super::shows::{TvShows, TvShowsMessage};
use crate::utils::{Filter, Layout, Sort};

#[derive(Debug, Clone, PartialEq)]
pub struct PageUpdate {
    pub layout: Layout,
    pub sort: Sort,
    pub filters: Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum PageKind {
    Home,
    Shows,
    Movies,
    Comments,
    Search,
    Custom,
}

#[derive(Debug, Clone)]
pub enum Page {
    Shows(Box<TvShows>),
    Movies(Box<Movies>),
    Comments(()),
    Search(()),
    Custom(()),
}

impl Page {
    pub fn goto_shows() -> PageKind {
        PageKind::Shows
    }

    pub fn goto_movies() -> PageKind {
        PageKind::Movies
    }

    pub fn goto_comments() -> PageKind {
        PageKind::Comments
    }

    pub fn is_shows(&self) -> bool {
        matches!(self, Self::Shows(_))
    }

    pub fn is_movies(&self) -> bool {
        matches!(self, Self::Movies(_))
    }

    pub fn is_comments(&self) -> bool {
        matches!(self, Self::Comments(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn movies_update(&mut self, message: MoviesMessage, now: Instant) -> Task<MoviesMessage> {
        match self {
            Self::Movies(movies) => movies.update(message, now),
            _ => Task::none(),
        }
    }

    pub fn shows_update(&mut self, message: TvShowsMessage, now: Instant) -> Task<TvShowsMessage> {
        match self {
            Self::Shows(shows) => shows.update(message, now),
            _ => Task::none(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Movies(movies) => movies.name(),
            Self::Shows(shows) => shows.name(),
            _ => todo!(),
        }
    }

    pub fn show_tools(&self) -> bool {
        match self {
            Self::Movies(movies) => movies.show_tools(),
            Self::Shows(shows) => shows.show_tools(),
            _ => todo!(),
        }
    }

    pub fn rand(&mut self) {
        match self {
            Self::Movies(movies) => movies.rand(),
            Self::Shows(shows) => shows.rand(),
            _ => todo!(),
        }
    }

    pub fn refresh(&mut self) -> Task<HomeMessage> {
        match self {
            Self::Movies(movies) => movies.refresh().map(HomeMessage::Movies),
            Self::Shows(shows) => shows.refresh().map(HomeMessage::Shows),
            _ => todo!(),
        }
    }

    /// Returns true if the collection can go to a previous page
    pub fn can_back(&self) -> bool {
        match self {
            Self::Movies(movies) => movies.can_back(),
            Self::Shows(shows) => shows.can_back(),
            _ => todo!(),
        }
    }

    /// Returns true if the collection can go to a next page
    pub fn can_forward(&self) -> bool {
        match self {
            Self::Movies(movies) => movies.can_forward(),
            Self::Shows(shows) => shows.can_forward(),
            _ => todo!(),
        }
    }

    /// Navigates to the previous page of the collection.
    /// Returning `None` causes the entire collection to be navigated past.
    pub fn back(&mut self, update: PageUpdate, now: Instant) -> Option<Task<()>> {
        self.page_update(update, now);
        match self {
            Self::Movies(movies) => movies.back(),
            Self::Shows(shows) => shows.back(),
            _ => todo!(),
        }
    }

    /// Navigates to the next page of the collection.
    /// Returning `None` causes the entire collection to be navigated past.
    pub fn forward(&mut self, update: PageUpdate, now: Instant) -> Option<Task<()>> {
        self.page_update(update, now);

        match self {
            Self::Movies(movies) => movies.forward(),
            Self::Shows(shows) => shows.forward(),
            _ => todo!(),
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self {
            Self::Movies(page) => page.update_scroll(),
            Self::Shows(page) => page.update_scroll(),
            _ => todo!(),
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        match self {
            Self::Movies(movies) => movies.page_update(update, now),
            Self::Shows(shows) => shows.page_update(update, now),
            _ => todo!(),
        }
    }

    pub fn subscription(&self) -> Subscription<HomeMessage> {
        match self {
            Self::Movies(movies) => movies.subscription().map(HomeMessage::Movies),
            Self::Shows(shows) => shows.subscription().map(HomeMessage::Shows),
            _ => todo!(),
        }
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {
        match self {
            Self::Shows(shows) => shows.view().map(HomeMessage::Shows),
            Self::Movies(movies) => movies.view().map(HomeMessage::Movies),
            Self::Comments(_) => center(text("Comments"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            Self::Search(_) => center(text("Search"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            Self::Custom(_) => center(text("Custom"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }
}
