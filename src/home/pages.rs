use iced::Task;

use super::HomeMessage;
use super::collection::{CollectionMessage, CollectionPage};
use super::collections::{Collections, CollectionsMessage};
use super::directory::{DirectoryMessage, DirectoryPage};
use super::episode::{EpisodePage, EpisodePageMessage};
use super::movie::{MoviePage, MoviePageMessage};
use super::movies::{Movies, MoviesMessage};
use super::season::{SeasonPage, SeasonPageMessage};
use super::series::{ShowPage, ShowPageMessage};
use super::shows::{TvShows, TvShowsMessage};
use super::wishlist::{Wishlist, WishlistMessage};
use registry::models::{CollectionId, DirectoryId, EpisodeId, ItemId, MovieId, SeasonId, ShowId};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum PageKind {
    Home,
    Shows,
    Movies,
    Collections,
    Wishlist,
    Episode(EpisodeId),
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Collection(CollectionId),
    Directory(DirectoryId),
}

impl From<ItemId> for PageKind {
    fn from(value: ItemId) -> Self {
        match value {
            ItemId::Movie(id) => Self::Movie(id),
            ItemId::Show(id) => Self::Show(id),
            ItemId::Season(id) => Self::Season(id),
            ItemId::Episode(id) => Self::Episode(id),
        }
    }
}

impl PageKind {
    pub fn is_home(self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn is_shows(self) -> bool {
        matches!(self, Self::Shows)
    }

    pub fn is_movies(self) -> bool {
        matches!(self, Self::Movies)
    }

    pub fn is_collections(self) -> bool {
        matches!(self, Self::Collections)
    }

    pub fn is_wishlist(self) -> bool {
        matches!(self, Self::Wishlist)
    }

    pub fn is_collection(&self, id: &CollectionId) -> bool {
        match self {
            Self::Collection(collection) => collection == id,
            _ => false,
        }
    }

    pub fn is_directory(&self, id: &DirectoryId) -> bool {
        match self {
            Self::Directory(dir) => dir == id,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Page {
    Home,
    Shows(TvShows),
    Movies(Movies),
    Collections(Collections),
    Wishlist(Wishlist),
    Directory {
        dir: DirectoryPage,
        id: DirectoryId,
    },
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
    pub fn kind(&self) -> PageKind {
        match self {
            Self::Home => PageKind::Home,
            Self::Shows(_) => PageKind::Shows,
            Self::Movies(_) => PageKind::Movies,
            Self::Collections(_) => PageKind::Collections,
            Self::Wishlist(_) => PageKind::Wishlist,
            Self::Episode { id, .. } => PageKind::Episode(*id),
            Self::Movie { id, .. } => PageKind::Movie(*id),
            Self::Season { id, .. } => PageKind::Season(*id),
            Self::Show { id, .. } => PageKind::Show(*id),
            Self::Collection { id, .. } => PageKind::Collection(*id),
            Self::Directory { id, .. } => PageKind::Directory(*id),
        }
    }

    pub fn goto_shows() -> PageKind {
        PageKind::Shows
    }

    pub fn goto_movies() -> PageKind {
        PageKind::Movies
    }

    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn is_shows(&self) -> bool {
        matches!(self, Self::Shows(_))
    }

    pub fn is_movies(&self) -> bool {
        matches!(self, Self::Movies(_))
    }

    pub fn is_collections(&self) -> bool {
        matches!(self, Self::Collections(_))
    }

    pub fn is_wishlist(&self) -> bool {
        matches!(self, Self::Wishlist(_))
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

    pub fn directory_update(&mut self, message: DirectoryMessage) -> Option<HomeMessage> {
        match self {
            Self::Directory { dir, id } if message.id == *id => dir.update(message),
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

    pub fn wishlist_update(&mut self, message: WishlistMessage) -> Option<HomeMessage> {
        match self {
            Self::Wishlist(wishlist) => wishlist.update(message),
            _ => None,
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
            Self::Directory { dir, .. } => dir.show_tools(),
            Self::Home => false,
            Self::Wishlist(_) => false,
        }
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        match self {
            Self::Movies(page) => page.update_scroll(),
            Self::Shows(page) => page.update_scroll(),
            Self::Collections(page) => page.update_scroll(),
            Self::Collection { collection, .. } => collection.update_scroll(),
            Self::Movie { page, .. } => page.update_scroll(),
            Self::Show { page, .. } => page.update_scroll(),
            Self::Season { page, .. } => page.update_scroll(),
            Self::Episode { page, .. } => page.update_scroll(),
            Self::Directory { dir, .. } => dir.update_scroll(),
            Self::Home => Task::none(),
            Self::Wishlist(wishlist) => wishlist.update_scroll(),
        }
    }
}
