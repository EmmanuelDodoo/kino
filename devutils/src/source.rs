use registry::{
    db::Query,
    models::{EpisodeId, ItemId, MovieId, SeasonId, ShowId},
};
use reqwest::{
    Client, ClientBuilder,
    header::{ACCEPT, HeaderMap},
};
use rusqlite::{
    Row,
    types::{ToSqlOutput, ValueRef},
};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub mod tmdb;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".try_into().unwrap());

    ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Cannot build request client")
});
const POSTER_SNIPPET: &str = "_poster.jpg";
const BACKDROP_SNIPPET: &str = "_backdrop.jpg";
pub(crate) const IMAGE_SQL: &str = "INSERT INTO image (path) VALUES (:path) ON CONFLICT (path) DO UPDATE SET main=NULL, accent=NULL, generated=FALSE";

#[derive(Debug, Clone, Copy)]
pub enum SourceSet {
    None,
    Tmdb,
}

impl SourceSet {
    const NONE: &str = "none";
    const TMDB: &str = "tmdb";

    pub fn from_str(s: &str) -> Self {
        match s {
            Self::TMDB => Self::Tmdb,
            _ => Self::None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::None => Self::NONE,
            Self::Tmdb => Self::TMDB,
        }
    }

    pub fn from_row(row: &Row<'_>, column: &str) -> rusqlite::Result<Self> {
        let name = row.get::<_, Option<String>>(column)?;

        match name.as_deref() {
            Some(Self::TMDB) => Ok(Self::Tmdb),
            _ => Ok(Self::None),
        }
    }

    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, other) => other,
            (s, Self::None) => s,
            (s, _) => s,
        }
    }

    pub fn movie_request<'a>(&self, id: MovieId, name: String) -> Option<(Query<'a>, String)> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::movie_request(id, name),
        }
    }

    pub fn show_request<'a>(&self, id: ShowId, name: String) -> Option<(Query<'a>, String)> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::show_request(id, name),
        }
    }

    pub fn season_request<'a>(
        &self,
        id: SeasonId,
        parent: &str,
        number: u16,
    ) -> Option<(Query<'a>, String)> {
        match self {
            Self::None => None,
            Self::Tmdb => {
                let parent = tmdb::TMDB::id_from_str(parent);
                tmdb::TMDB::season_request(id, parent, number)
            }
        }
    }

    pub fn episode_request<'a>(
        &self,
        id: EpisodeId,
        parent: &str,
        season: u16,
        number: u16,
    ) -> Option<(Query<'a>, String)> {
        match self {
            Self::None => None,
            Self::Tmdb => {
                let parent = tmdb::TMDB::id_from_str(parent);
                tmdb::TMDB::episode_request(id, parent, season, number)
            }
        }
    }

    pub fn refetch<'a>(&self, id: impl Into<ItemId>) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::refetch(id),
        }
    }

    pub fn set_tmdb_id<'a>(&self, id: impl Into<ItemId>, tmdb_id: u32) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => Some(tmdb::TMDB::set_tmdb_id(id, tmdb_id)),
        }
    }

    pub fn set_tmdb_number<'a>(&self, id: impl Into<ItemId>, number: u16) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => Some(tmdb::TMDB::set_number(id, number)),
        }
    }

    pub fn delete<'a>(&self, id: impl Into<ItemId>) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::delete(id),
        }
    }
}

impl From<SourceSet> for ToSqlOutput<'_> {
    fn from(value: SourceSet) -> Self {
        match value {
            SourceSet::None => ToSqlOutput::Borrowed(ValueRef::Text(SourceSet::NONE.as_bytes())),
            SourceSet::Tmdb => ToSqlOutput::Borrowed(ValueRef::Text(SourceSet::TMDB.as_bytes())),
        }
    }
}

/// Guidelines for what's expected from sources
pub trait Source {
    type Id<'a>: Clone + Copy + Into<rusqlite::types::ToSqlOutput<'a>>;

    fn id<'a>(row: &rusqlite::Row<'_>, column: &str) -> rusqlite::Result<Self::Id<'a>>;

    fn id_from_str<'a>(s: &str) -> Self::Id<'a>;

    fn movie_request<'a>(id: MovieId, name: String) -> Option<(Query<'a>, String)>;

    fn show_request<'a>(id: ShowId, name: String) -> Option<(Query<'a>, String)>;

    fn season_request<'a>(
        id: SeasonId,
        parent: Self::Id<'a>,
        number: u16,
    ) -> Option<(Query<'a>, String)>;

    fn episode_request<'a>(
        id: EpisodeId,
        parent: Self::Id<'a>,
        season: u16,
        number: u16,
    ) -> Option<(Query<'a>, String)>;

    fn refetch<'a>(id: impl Into<ItemId>) -> Option<Query<'a>>;

    fn delete<'a>(id: impl Into<ItemId>) -> Option<Query<'a>>;
}

impl Source for () {
    type Id<'a> = rusqlite::types::Null;

    fn id<'a>(_row: &rusqlite::Row<'_>, _column: &str) -> rusqlite::Result<Self::Id<'a>> {
        Ok(rusqlite::types::Null)
    }

    fn id_from_str<'a>(_s: &str) -> Self::Id<'a> {
        rusqlite::types::Null
    }

    fn movie_request<'a>(_id: MovieId, _name: String) -> Option<(Query<'a>, String)> {
        None
    }

    fn show_request<'a>(_id: ShowId, _name: String) -> Option<(Query<'a>, String)> {
        None
    }

    fn season_request<'a>(
        _id: SeasonId,
        _parent: Self::Id<'a>,
        _number: u16,
    ) -> Option<(Query<'a>, String)> {
        None
    }

    fn episode_request<'a>(
        _id: EpisodeId,
        _parent: Self::Id<'a>,
        _number: u16,
        _season: u16,
    ) -> Option<(Query<'a>, String)> {
        None
    }

    fn refetch<'a>(_id: impl Into<ItemId>) -> Option<Query<'a>> {
        None
    }

    fn delete<'a>(_id: impl Into<ItemId>) -> Option<Query<'a>> {
        None
    }
}

pub(crate) fn poster_path<P: AsRef<Path>, Id: std::fmt::Display>(path: &P, id: &Id) -> PathBuf {
    path.as_ref().join(format!("{}{POSTER_SNIPPET}", id))
}

pub(crate) fn backdrop_path<P: AsRef<Path>, Id: std::fmt::Display>(path: &P, id: &Id) -> PathBuf {
    path.as_ref().join(format!("{}{BACKDROP_SNIPPET}", id))
}
