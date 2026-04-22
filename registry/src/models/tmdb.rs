use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

use super::{EpisodeId, ItemId, MovieId, SeasonId, ShowId};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestId(Uuid);

impl From<RequestId> for ToSqlOutput<'_> {
    fn from(value: RequestId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl RequestId {
    pub fn from_row_maybe(row: &Row<'_>, column: &str) -> rusqlite::Result<Option<Self>> {
        row.get::<_, Option<String>>(column)
            .map(|id| id.map(|id| Self(Uuid::try_parse(&id).unwrap())))
    }

    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper(row, "id")
    }

    fn from_parent(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper(row, "parent")
    }

    pub fn from_row_helper(row: &Row<'_>, column: &str) -> Result<Self> {
        row.get::<_, String>(column)
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }

    pub fn from_str(s: &str) -> Self {
        Self(Uuid::try_parse(s).expect("Infallible Uuid conversion"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Waiting = 0,
    Searching = 1,
    Data = 2,
    Image = 3,
    Done = 4,
}

impl Status {
    fn from_row(row: &Row<'_>) -> Result<Self> {
        let status = row.get::<_, u8>("status")?;

        Ok(match status {
            0 => Self::Waiting,
            1 => Self::Searching,
            2 => Self::Data,
            3 => Self::Image,
            4 => Self::Done,
            _ => unreachable!("Invalid tmdb status"),
        })
    }
}

impl From<Status> for ToSqlOutput<'_> {
    fn from(value: Status) -> Self {
        let code = value as u8;

        ToSqlOutput::from(code)
    }
}

#[derive(Debug, Clone)]
pub enum Media {
    Movie {
        id: MovieId,
        name: String,
        backdrop: Option<String>,
    },
    Show {
        id: ShowId,
        name: String,
        backdrop: Option<String>,
    },
    Season {
        id: SeasonId,
        parent: RequestId,
        number: u16,
    },
    Episode {
        id: EpisodeId,
        parent: RequestId,
        season: u16,
        number: u16,
    },
}

impl Media {
    fn from_row(row: &Row<'_>) -> Result<Self> {
        let media_type = row.get::<_, String>("media_type")?;

        match media_type.as_str() {
            "movie" => {
                let id = MovieId::from_collection(&row)?;
                let name = row.get::<_, String>("name")?;
                let backdrop = row.get::<_, Option<String>>("backdrop")?;
                Ok(Self::Movie { id, name, backdrop })
            }
            "show" => {
                let id = ShowId::from_collection(&row)?;
                let name = row.get::<_, String>("name")?;
                let backdrop = row.get::<_, Option<String>>("backdrop")?;

                Ok(Self::Show { id, name, backdrop })
            }
            "season" => {
                let id = SeasonId::from_collection(&row)?;
                let parent = RequestId::from_parent(row)?;
                let number = row.get::<_, u16>("number")?;

                Ok(Self::Season { id, parent, number })
            }
            "episode" => {
                let id = EpisodeId::from_collection(&row)?;
                let parent = RequestId::from_parent(row)?;
                let number = row.get::<_, u16>("number")?;
                let season = row
                    .get::<_, String>("name")?
                    .parse::<u16>()
                    .expect("Episode season number should be stored within name column");

                Ok(Self::Episode {
                    id,
                    parent,
                    season,
                    number,
                })
            }
            _ => unreachable!("Invalid tmdb request media"),
        }
    }

    pub fn new_movie(id: MovieId, name: String) -> Self {
        Self::Movie {
            id,
            name,
            backdrop: None,
        }
    }

    pub fn new_show(id: ShowId, name: String) -> Self {
        Self::Show {
            id,
            name,
            backdrop: None,
        }
    }

    pub fn new_season(id: SeasonId, parent: RequestId, number: u16) -> Self {
        Self::Season { id, parent, number }
    }

    pub fn new_episode(id: EpisodeId, parent: RequestId, season: u16, number: u16) -> Self {
        Self::Episode {
            id,
            parent,
            season,
            number,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub media: Media,
    pub tmdb_id: Option<u32>,
    pub status: Status,
    pub retry: u8,
    pub poster: Option<String>,
}

impl Request {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = RequestId::from_row(row)?;
        let tmdb_id = row.get::<_, Option<u32>>("tmdb_id")?;
        let media = Media::from_row(row)?;

        let status = Status::from_row(row)?;

        let retry = row.get::<_, u8>("retry")?;
        let poster = row.get::<_, Option<String>>("poster")?;

        Ok(Self {
            id,
            tmdb_id,
            media,
            status,
            retry,
            poster,
        })
    }

    pub fn insert<'a>(&self) -> Query<'a> {
        let Self {
            id,
            media,
            tmdb_id,
            status,
            retry,
            poster,
        } = self;

        let sql = "INSERT INTO tmdb (id, tmdb_id, media_type, media_id, status, retry, poster, parent, name, number, backdrop) VALUES (:id, :tmdb_id, :media_type, :media_id, :status, :retry, :poster, :parent, :name, :number, :backdrop)";

        let null = ToSqlOutput::Owned(Value::Null);

        let id = ToSqlOutput::from(*id);
        let tmdb_id = tmdb_id
            .map(|tmdb_id| ToSqlOutput::from(tmdb_id))
            .unwrap_or(null.clone());
        let status = ToSqlOutput::from(*status);
        let retry = ToSqlOutput::from(*retry);
        let poster = poster
            .clone()
            .map(|poster| ToSqlOutput::from(poster))
            .unwrap_or(null.clone());

        let (media_type, media_id, name, backdrop, parent, number) = match media {
            Media::Movie { id, name, backdrop } => {
                let media = ToSqlOutput::from("movie".to_owned());
                let id = ToSqlOutput::from(*id);
                let name = ToSqlOutput::from(name.clone());
                let backdrop = backdrop
                    .clone()
                    .map(|backdrop| ToSqlOutput::from(backdrop))
                    .unwrap_or(null.clone());

                (media, id, name, backdrop, null.clone(), null.clone())
            }
            Media::Show { id, name, backdrop } => {
                let media = ToSqlOutput::from("show".to_owned());
                let id = ToSqlOutput::from(*id);
                let name = ToSqlOutput::from(name.clone());
                let backdrop = backdrop
                    .clone()
                    .map(|backdrop| ToSqlOutput::from(backdrop))
                    .unwrap_or(null.clone());

                (media, id, name, backdrop, null.clone(), null.clone())
            }
            Media::Season { id, parent, number } => {
                let media = ToSqlOutput::from("season".to_owned());
                let id = ToSqlOutput::from(*id);
                let parent = ToSqlOutput::from(*parent);
                let number = ToSqlOutput::from(*number);

                (media, id, null.clone(), null.clone(), parent, number)
            }
            Media::Episode {
                id,
                parent,
                season,
                number,
            } => {
                let media = ToSqlOutput::from("episode".to_owned());
                let id = ToSqlOutput::from(*id);
                let parent = ToSqlOutput::from(*parent);
                let number = ToSqlOutput::from(*number);
                let season = ToSqlOutput::from(*season);

                (media, id, season, null.clone(), parent, number)
            }
        };

        let params = vec![
            (":id", id),
            (":tmdb_id", tmdb_id),
            (":media_type", media_type),
            (":media_id", media_id),
            (":status", status),
            (":retry", retry),
            (":poster", poster),
            (":backdrop", backdrop),
            (":parent", parent),
            (":name", name),
            (":number", number),
        ];

        Query {
            id: self.id.0,
            table: Table::TMDBRequest,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    pub fn update<'a>(&self) -> Query<'a> {
        let Self {
            id,
            media,
            tmdb_id,
            status,
            retry,
            poster,
        } = self;

        let sql = "UPDATE tmdb SET tmdb_id=:tmdb_id, status=:status, retry=:retry, poster=:poster, backdrop=:backdrop WHERE id=:id";

        let null = ToSqlOutput::Owned(Value::Null);

        let id = ToSqlOutput::from(*id);
        let tmdb_id = tmdb_id
            .map(|tmdb_id| ToSqlOutput::from(tmdb_id))
            .unwrap_or(null.clone());
        let status = ToSqlOutput::from(*status);
        let retry = ToSqlOutput::from(*retry);
        let poster = poster
            .clone()
            .map(|poster| ToSqlOutput::from(poster))
            .unwrap_or(null.clone());

        let backdrop = match media {
            Media::Movie { backdrop, .. } => {
                let backdrop = backdrop
                    .clone()
                    .map(|backdrop| ToSqlOutput::from(backdrop))
                    .unwrap_or(null.clone());

                backdrop
            }
            Media::Show { backdrop, .. } => {
                let backdrop = backdrop
                    .clone()
                    .map(|backdrop| ToSqlOutput::from(backdrop))
                    .unwrap_or(null.clone());

                backdrop
            }
            Media::Season { .. } | Media::Episode { .. } => null,
        };

        let params = vec![
            (":id", id),
            (":tmdb_id", tmdb_id),
            (":status", status),
            (":retry", retry),
            (":poster", poster),
            (":backdrop", backdrop),
        ];

        Query {
            id: self.id.0,
            table: Table::TMDBRequest,
            sql,
            params,
            op: Operation::Update,
        }
    }

    pub fn refetch<'a>(id: impl Into<ItemId>) -> Query<'a> {
        let id = id.into();

        let status = if matches!(id, ItemId::Movie(_) | ItemId::Show(_)) {
            Status::Searching
        } else {
            Status::Data
        };

        let sql = "UPDATE tmdb SET status=:status WHERE media_id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":status", ToSqlOutput::from(status)),
        ];

        Query {
            id: id.inner(),
            table: Table::TMDBRequest,
            op: Operation::Update,
            sql,
            params: params.to_vec(),
        }
    }

    pub fn update_tmdb_id<'a>(id: impl Into<ItemId>, tmdb_id: u32) -> Query<'a> {
        let id = id.into();

        let status = Status::Data;
        let sql = "UPDATE tmdb SET status=:status, tmdb_id=:tmdb_id WHERE media_id=:id";

        let params = [
            (":id", ToSqlOutput::from(id)),
            (":status", ToSqlOutput::from(status)),
            (":tmdb_id", ToSqlOutput::from(tmdb_id)),
        ];

        Query {
            id: id.inner(),
            table: Table::TMDBRequest,
            op: Operation::Update,
            sql,
            params: params.to_vec(),
        }
    }

    pub fn update_number<'a>(id: impl Into<ItemId>, number: u16) -> Query<'a> {
        let id = id.into();

        let status = Status::Data;
        let sql = "UPDATE tmdb SET status=:status, number=:number WHERE media_id=:id";

        let params = [
            (":id", ToSqlOutput::from(id)),
            (":status", ToSqlOutput::from(status)),
            (":number", ToSqlOutput::from(number)),
        ];

        Query {
            id: id.inner(),
            table: Table::TMDBRequest,
            op: Operation::Update,
            sql,
            params: params.to_vec(),
        }
    }

    pub fn delete<'a>(id: impl Into<ItemId>) -> Query<'a> {
        let id = id.into();

        let sql = "DELETE FROM tmdb WHERE media_id=:id";

        let params = [(":media_id", ToSqlOutput::from(id))];

        Query {
            id: id.inner(),
            table: Table::TMDBRequest,
            op: Operation::Delete,
            sql,
            params: params.to_vec(),
        }
    }

    pub fn new(media: Media) -> Self {
        let status = if matches!(media, Media::Movie { .. } | Media::Show { .. }) {
            Status::Searching
        } else {
            Status::Waiting
        };

        Self {
            id: RequestId(Uuid::now_v7()),
            media,
            status,
            tmdb_id: None,
            retry: 0,
            poster: None,
        }
    }
}
