use iced::{
    Element, Length, Subscription, Task,
    time::Instant,
    widget::{center, text},
};

use super::HomeMessage;
use super::movies::{Movies, MoviesMessage};
use crate::utils::{Filter, Sort, ViewType};

#[derive(Debug, Clone, PartialEq)]
pub enum PageUpdate {
    Layout(ViewType),
    Sort(Sort),
    Filters(Filter),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageKind {
    Shows,
    Movies,
    Comments,
    Search,
    Custom,
}

#[derive(Debug, Clone)]
pub enum Page {
    Shows(()),
    Movies(Movies),
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
        match self {
            Self::Shows(_) => true,
            _ => false,
        }
    }

    pub fn is_movies(&self) -> bool {
        match self {
            Self::Movies(_) => true,
            _ => false,
        }
    }

    pub fn is_comments(&self) -> bool {
        match self {
            Self::Comments(_) => true,
            _ => false,
        }
    }

    pub fn is_custom(&self) -> bool {
        match self {
            Self::Custom(_) => true,
            _ => false,
        }
    }

    pub fn movies_update(&mut self, message: MoviesMessage, now: Instant) -> Task<MoviesMessage> {
        match self {
            Self::Movies(movies) => movies.update(message, now),
            _ => Task::none(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Movies(movies) => movies.name(),
            _ => todo!(),
        }
    }

    /// Returns true if the collection can go to a previous page
    pub fn can_back(&self) -> bool {
        match self {
            Self::Movies(movies) => movies.can_back(),
            _ => todo!(),
        }
    }

    /// Returns true if the collection can go to a next page
    pub fn can_forward(&self) -> bool {
        match self {
            Self::Movies(movies) => movies.can_forward(),
            _ => todo!(),
        }
    }

    /// Navigates to the previous page of the collection.
    /// Returning `false` causes the entire collection to be navigated past.
    pub fn back(&mut self) -> bool {
        match self {
            Self::Movies(_) => false,
            _ => todo!(),
        }
    }

    /// Navigates to the next page of the collection.
    /// Returning `false` causes the entire collection to be navigated past.
    pub fn forward(&mut self) -> bool {
        match self {
            Self::Movies(_) => false,
            _ => todo!(),
        }
    }

    pub fn page_update(&mut self, update: PageUpdate, now: Instant) {
        match self {
            Self::Movies(movies) => movies.page_update(update, now),
            _ => todo!(),
        }
    }

    pub fn subscription(&self) -> Subscription<HomeMessage> {
        match self {
            Self::Movies(movies) => movies.subscription().map(HomeMessage::Movies),
            _ => todo!(),
        }
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {
        match self {
            Self::Shows(_) => center(text("Shows"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
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
