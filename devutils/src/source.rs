use core::{Context, Result, variants};
use registry::{
    db::Query,
    models::{EpisodeId, ItemId, MovieId, SeasonId, ShowId, WishId, WishKind},
};
use reqwest::{
    Client, ClientBuilder,
    header::{ACCEPT, HeaderMap},
};
use rusqlite::{
    Row,
    types::{ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
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
pub(crate) const IMAGE_SQL: &str = "INSERT INTO image (path) VALUES (:path) ON CONFLICT (path) DO UPDATE SET main=NULL, accent=NULL, generated=FALSE";

variants! {

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SourceSet {
    #[serde(rename="none")]
    None,
    #[serde(rename="tmdb")]
    Tmdb,
}
}

impl SourceSet {
    pub const NONE: &str = "none";
    pub const TMDB: &str = "tmdb";

    pub fn from_str(s: &str) -> Self {
        match s {
            Self::TMDB => Self::Tmdb,
            "Tmdb" => Self::Tmdb,
            "TMDB" => Self::Tmdb,
            "None" => Self::None,
            "NONE" => Self::None,
            Self::NONE => Self::None,
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

        let source = name.as_deref().map(Self::from_str).unwrap_or(Self::None);

        Ok(source)
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

    pub fn wish_request<'a>(
        &self,
        id: WishId,
        name: String,
        kind: WishKind,
    ) -> Option<(Query<'a>, String)> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::wish_request(id, name, kind),
        }
    }

    pub fn refetch<'a>(&self, id: impl Into<ItemId>) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::refetch(id),
        }
    }

    pub fn set_source_id<'a>(
        &self,
        id: impl Into<ItemId>,
        source_id: SourceId,
        top_level: bool,
    ) -> Option<Query<'a>> {
        match (self, source_id) {
            (Self::None, _) => None,

            (Self::Tmdb, SourceId::Tmdb(value)) => {
                if top_level {
                    Some(tmdb::TMDB::set_tmdb_id(id, value))
                } else {
                    Some(tmdb::TMDB::set_number(
                        id,
                        value.try_into().unwrap_or_default(),
                    ))
                }
            }
            (Self::Tmdb, _) => None,
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

    pub fn set_wish_source_id<'a>(&self, wish: WishId, id: SourceId) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::set_wish_id(id, wish),
        }
    }

    pub fn delete_wish<'a>(&self, wish: WishId) -> Option<Query<'a>> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::delete_wish(wish),
        }
    }

    pub fn source_id(&self, s: &str) -> Option<SourceId> {
        match self {
            Self::None => None,
            Self::Tmdb => tmdb::TMDB::source_id(s),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceId {
    None,
    Tmdb(u32),
}

impl SourceId {
    pub fn to_str(&self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Tmdb(id) => id.to_string(),
        }
    }
}

/// Guidelines for what's expected from sources
pub trait SourceImpl {
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

    fn wish_request<'a>(id: WishId, name: String, kind: WishKind) -> Option<(Query<'a>, String)>;

    fn refetch<'a>(id: impl Into<ItemId>) -> Option<Query<'a>>;

    fn delete<'a>(id: impl Into<ItemId>) -> Option<Query<'a>>;

    fn set_wish_id<'a>(id: SourceId, wish: WishId) -> Option<Query<'a>>;

    fn delete_wish<'a>(wish: WishId) -> Option<Query<'a>>;

    fn source_id(s: &str) -> Option<SourceId>;
}

pub(crate) fn poster_path<P: AsRef<Path>, Id: std::fmt::Display>(path: &P, id: &Id) -> PathBuf {
    poster(path, id, "jpg")
}

pub(crate) fn backdrop_path<P: AsRef<Path>, Id: std::fmt::Display>(path: &P, id: &Id) -> PathBuf {
    backdrop(path, id, "jpg")
}

pub fn poster<P: AsRef<Path>, Id: std::fmt::Display, E: std::fmt::Display>(
    path: &P,
    id: &Id,
    extension: E,
) -> PathBuf {
    path.as_ref().join(format!("{id}_poster.{extension}"))
}

pub fn backdrop<P: AsRef<Path>, Id: std::fmt::Display, E: std::fmt::Display>(
    path: &P,
    id: &Id,
    extension: E,
) -> PathBuf {
    path.as_ref().join(format!("{id}_backdrop.{extension}"))
}

fn insert_wish_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: WishId,
    poster: Option<String>,
) -> Result<()> {
    let Some(poster) = poster else { return Ok(()) };

    let sql = "UPDATE wish SET poster=:poster WHERE id=:id";
    let mut statement = db.prepare_cached(sql)?;

    let poster_sql = ToSqlOutput::Borrowed(ValueRef::Text(poster.as_bytes()));
    statement
        .execute(&[(":id", &ToSqlOutput::from(id)), (":poster", &poster_sql)])
        .with_context(|| format!("Insert wish {id} poster {poster}"))?;

    db.execute(IMAGE_SQL, &[(":path", &poster_sql)])
        .with_context(|| format!("Insert image {poster}"))?;

    Ok(())
}

fn insert_movie_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: MovieId,
    poster: Option<String>,
    backdrop: Option<String>,
) -> Result<()> {
    if let Some(poster) = poster {
        let sql =
            "UPDATE movie SET poster=:poster, generate_poster=FALSE, fetched=TRUE WHERE id=:id";
        let poster_sql = ToSqlOutput::Borrowed(ValueRef::Text(poster.as_bytes()));

        db.execute(
            sql,
            &[(":id", &ToSqlOutput::from(id)), (":poster", &poster_sql)],
        )
        .with_context(|| format!("Insert movie {id} poster {poster}"))?;

        db.execute(IMAGE_SQL, &[(":path", &poster_sql)])
            .with_context(|| format!("Insert image {poster}"))?;
    }

    if let Some(backdrop) = backdrop {
        let sql = "UPDATE movie SET backdrop=:backdrop WHERE id=:id";
        let backdrop_sql = ToSqlOutput::Borrowed(ValueRef::Text(backdrop.as_bytes()));

        db.execute(
            sql,
            &[
                (":id", &ToSqlOutput::from(id)),
                (":backdrop", &backdrop_sql),
            ],
        )
        .with_context(|| format!("Insert movie {id} backdrop {backdrop}"))?;
    }

    Ok(())
}

fn insert_show_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: ShowId,
    poster: Option<String>,
    backdrop: Option<String>,
) -> Result<()> {
    if let Some(poster) = poster {
        let sql = "UPDATE tv_show SET poster=:poster WHERE id=:id";
        let poster_sql = ToSqlOutput::Borrowed(ValueRef::Text(poster.as_bytes()));

        db.execute(
            sql,
            &[(":id", &ToSqlOutput::from(id)), (":poster", &poster_sql)],
        )
        .with_context(|| format!("Insert show {id} poster {poster}"))?;

        db.execute(IMAGE_SQL, &[(":path", &poster_sql)])
            .with_context(|| format!("Insert image {poster}"))?;
    }

    if let Some(backdrop) = backdrop {
        let sql = "UPDATE tv_show SET backdrop=:backdrop WHERE id=:id";
        let backdrop_sql = ToSqlOutput::Borrowed(ValueRef::Text(backdrop.as_bytes()));

        db.execute(
            sql,
            &[
                (":id", &ToSqlOutput::from(id)),
                (":backdrop", &backdrop_sql),
            ],
        )
        .with_context(|| format!("Insert show {id} backdrop {backdrop}"))?;
    }

    Ok(())
}

fn insert_season_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: SeasonId,
    poster: Option<String>,
) -> Result<()> {
    let Some(poster) = poster else { return Ok(()) };

    let sql = "UPDATE season SET poster=:poster WHERE id=:id";
    let mut statement = db.prepare_cached(sql)?;

    let poster_sql = ToSqlOutput::Borrowed(ValueRef::Text(poster.as_bytes()));

    statement
        .execute(&[(":id", &ToSqlOutput::from(id)), (":poster", &poster_sql)])
        .with_context(|| format!("Insert season {id} poster {poster}"))?;

    db.execute(IMAGE_SQL, &[(":path", &poster_sql)])
        .with_context(|| format!("Insert image {poster}"))?;

    Ok(())
}

fn insert_episode_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: EpisodeId,
    poster: Option<String>,
) -> Result<()> {
    let Some(poster) = poster else { return Ok(()) };

    let sql = "UPDATE episode SET poster=:poster, generate_poster=FALSE, fetched=TRUE WHERE id=:id";
    let mut statement = db.prepare_cached(sql)?;

    let poster_sql = ToSqlOutput::Borrowed(ValueRef::Text(poster.as_bytes()));
    statement
        .execute(&[(":id", &ToSqlOutput::from(id)), (":poster", &poster_sql)])
        .with_context(|| format!("Insert episode {id} poster {poster}"))?;

    db.execute(IMAGE_SQL, &[(":path", &poster_sql)])
        .with_context(|| format!("Insert image {poster}"))?;

    Ok(())
}

pub fn insert_media_image(
    db: &impl Deref<Target = rusqlite::Connection>,
    id: impl Into<ItemId>,
    poster: Option<String>,
    backdrop: Option<String>,
) -> Result<()> {
    let id = id.into();

    match id {
        ItemId::Movie(id) => insert_movie_image(db, id, poster, backdrop)
            .with_context(|| format!("Inserting movie {id} images")),
        ItemId::Show(id) => insert_show_image(db, id, poster, backdrop)
            .with_context(|| format!("Inserting show {id} images")),
        ItemId::Season(id) => insert_season_image(db, id, poster)
            .with_context(|| format!("Inserting season {id} images")),
        ItemId::Episode(id) => insert_episode_image(db, id, poster)
            .with_context(|| format!("Inserting episode {id} images")),
    }
}
