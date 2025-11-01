use iced::Task;

use super::HomeMessage;
use super::collection::{CollectionMessage, CollectionPage};
use super::collections::{Collections, CollectionsMessage};
use super::episode::{EpisodePage, EpisodePageMessage};
use super::movie::{MoviePage, MoviePageMessage};
use super::movies::{Movies, MoviesMessage};
use super::season::{SeasonPage, SeasonPageMessage};
use super::series::{ShowPage, ShowPageMessage};
use super::shows::{TvShows, TvShowsMessage};
use crate::models::{CollectionId, EpisodeId, MovieId, SeasonId, ShowId};
use crate::utils::{Filter, Layout, Sort};

#[derive(Debug, Clone, PartialEq)]
pub struct PageUpdate {
    pub layout: Layout,
    pub sort: Sort,
    pub filters: Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum PageKind {
    Shows,
    Movies,
    Collections,
    Comments,
    Episode(EpisodeId),
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Collection(CollectionId),
}

#[derive(Debug, Clone)]
pub enum Page {
    Shows(TvShows),
    Movies(Movies),
    Comments(()),
    Collections(Collections),
    Collection {
        collection: CollectionPage,
        id: CollectionId,
    },
    Episode {
        page: EpisodePage,
        id: EpisodeId,
    },
    Movie {
        page: MoviePage,
        id: MovieId,
    },
    Season {
        page: SeasonPage,
        id: SeasonId,
    },
    Show {
        page: ShowPage,
        id: ShowId,
    },
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

    pub fn is_collections(&self) -> bool {
        matches!(self, Self::Collections(_))
    }

    pub fn is_collection(&self, id: &CollectionId) -> bool {
        match self {
            Self::Collection { id: own, .. } => own == id,
            _ => false,
        }
    }

    pub fn movies_update(&mut self, message: MoviesMessage) -> Option<HomeMessage> {
        match self {
            Self::Movies(movies) => movies.update(message),
            _ => None,
        }
    }

    pub fn shows_update(&mut self, message: TvShowsMessage) -> Option<HomeMessage> {
        match self {
            Self::Shows(shows) => shows.update(message),
            _ => None,
        }
    }

    pub fn collection_update(&mut self, message: CollectionMessage) -> Option<HomeMessage> {
        match self {
            Self::Collection { id, collection } if message.id == *id => collection.update(message),
            _ => None,
        }
    }

    pub fn movie_update(&mut self, message: MoviePageMessage) -> Option<HomeMessage> {
        match self {
            Self::Movie { page, id } if message.id == *id => page.update(message),
            _ => None,
        }
    }

    pub fn episode_update(&mut self, message: EpisodePageMessage) -> Option<HomeMessage> {
        match self {
            Self::Episode { page, id } if message.id == *id => page.update(message),
            _ => None,
        }
    }

    pub fn season_update(&mut self, message: SeasonPageMessage) -> Option<HomeMessage> {
        match self {
            Self::Season { page, id } if message.id == *id => page.update(message),
            _ => None,
        }
    }

    pub fn show_update(&mut self, message: ShowPageMessage) -> Option<HomeMessage> {
        match self {
            Self::Show { page, id } if message.id == *id => page.update(message),
            _ => None,
        }
    }

    pub fn collections_update(&mut self, message: CollectionsMessage) -> Option<HomeMessage> {
        match self {
            Self::Collections(collections) => collections.update(message),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Movies(movies) => movies.name(),
            Self::Shows(shows) => shows.name(),
            Self::Collections(collections) => collections.name(),
            Self::Collection { collection, .. } => collection.name(),
            Self::Movie { page, .. } => page.name(),
            Self::Episode { page, .. } => page.name(),
            Self::Season { page, .. } => page.name(),
            Self::Show { page, .. } => page.name(),
            _ => todo!(),
        }
    }

    pub fn show_tools(&self) -> bool {
        match self {
            Self::Movies(_) => true,
            Self::Shows(_) => true,
            Self::Collections(_) => false,
            Self::Collection { collection, .. } => collection.show_tools(),
            Self::Episode { page, .. } => page.show_tools(),
            Self::Movie { page, .. } => page.show_tools(),
            Self::Season { page, .. } => page.show_tools(),
            Self::Show { page, .. } => page.show_tools(),
            _ => todo!(),
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self {
            Self::Movies(page) => page.update_scroll(),
            Self::Shows(page) => page.update_scroll(),
            Self::Collections(page) => page.update_scroll(),
            Self::Collection { collection, .. } => collection.update_scroll(),
            Self::Show { page, .. } => page.update_scroll(),
            Self::Season { page, .. } => page.update_scroll(),
            Self::Movie { .. } | Self::Episode { .. } => Task::none(),
            _ => todo!(),
        }
    }

    pub fn page_update(&mut self, update: PageUpdate) {
        match self {
            Self::Movies(movies) => movies.page_update(update),
            Self::Shows(shows) => shows.page_update(update),
            Self::Collections(collections) => collections.page_update(update),
            Self::Collection { collection, .. } => collection.page_update(update),
            Self::Show { page, .. } => page.page_update(update),
            Self::Season { page, .. } => page.page_update(update),
            Self::Episode { .. } | Self::Movie { .. } => {}
            _ => todo!(),
        }
    }
}
