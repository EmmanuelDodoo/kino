use chrono::{DateTime, Local, NaiveDate};
use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

use super::{datetime_to_sql, image::Image, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WishId(pub(super) Uuid);

impl From<WishId> for ToSqlOutput<'_> {
    fn from(value: WishId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for WishId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl WishId {
    /// Expects relevant column name as "id"
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        Self::from_row_helper("id", row)
    }

    /// Expects 'media_id' column name
    pub fn from_collection(row: &Row<'_>) -> Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    pub(super) fn from_row_helper(column: &str, row: &Row<'_>) -> Result<Self> {
        row.get::<_, String>(column)
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }
}

#[derive(Debug, Clone)]
pub enum WishKind {
    Movie {
        duration: u64,
        tags: Vec<String>,
    },
    Show {
        tags: Vec<String>,
        seasons: u16,
    },
    Season {
        number: u16,
        episodes: u16,
    },
    Episode {
        season: u16,
        number: u16,
        duration: u64,
    },
}

impl WishKind {
    pub fn movie() -> Self {
        Self::Movie {
            duration: 0,
            tags: Vec::with_capacity(0),
        }
    }

    pub fn show() -> Self {
        Self::Show {
            tags: Vec::with_capacity(0),
            seasons: 0,
        }
    }

    pub fn season(number: u16) -> Self {
        Self::Season {
            number,
            episodes: 0,
        }
    }

    pub fn episode(season: u16, number: u16) -> Self {
        Self::Episode {
            season,
            number,
            duration: 0,
        }
    }

    fn from_row(row: &Row<'_>) -> Result<Self> {
        let media_type = row.get_ref("media_type")?.as_str()?;

        match media_type {
            "movie" => {
                let duration = row.get::<_, u64>("duration")?;
                let tags = row.get_ref("tags")?.as_str_or_null()?;

                let tags = tags
                    .map(|tags| tags.split(",").map(ToOwned::to_owned).collect::<Vec<_>>())
                    .unwrap_or_default();

                Ok(Self::Movie { duration, tags })
            }
            "show" => {
                let seasons = row.get::<_, u16>("count")?;
                let tags = row.get_ref("tags")?.as_str_or_null()?;

                let tags = tags
                    .map(|tags| tags.split(",").map(ToOwned::to_owned).collect::<Vec<_>>())
                    .unwrap_or_default();

                Ok(Self::Show { tags, seasons })
            }
            "season" => {
                let episodes = row.get::<_, u16>("count")?;
                let number = row.get::<_, u16>("season_number")?;
                Ok(Self::Season { number, episodes })
            }

            "episode" => {
                let number = row.get::<_, u16>("episode_number")?;
                let season = row.get::<_, u16>("season_number")?;
                let duration = row.get::<_, u64>("duration")?;

                Ok(Self::Episode {
                    season,
                    number,
                    duration,
                })
            }
            _ => unreachable!("Invalid wish media"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wish {
    pub id: WishId,
    pub kind: WishKind,
    pub name: String,
    pub added: DateTime<Local>,
    pub poster: Option<Image>,
    pub synopsis: String,
    pub release: NaiveDate,
    pub rating: Option<f32>,
    pub completed: bool,
    pub source: String,
}

impl Wish {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = WishId::from_row(row)?;
        let kind = WishKind::from_row(row)?;

        let name = row.get::<_, String>("name")?;
        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let synopsis = row.get::<_, String>("synopsis")?;

        let poster = {
            let poster = row.get::<_, Option<String>>("poster")?;

            if poster.is_some() {
                Some(Image::from_row(row, "poster_")?)
            } else {
                None
            }
        };
        let release = row.get::<_, NaiveDate>("release")?;
        let rating = row.get::<_, Option<f32>>("rating")?;

        let completed = row.get::<_, bool>("completed")?;

        let source = row.get::<_, String>("source")?;

        Ok(Self {
            id,
            kind,
            name,
            added,
            poster,
            synopsis,
            release,
            rating,
            completed,
            source,
        })
    }

    pub fn insert<'a>(&self) -> Query<'a> {
        let Self {
            id,
            kind,
            name,
            added,
            poster: _poster,
            synopsis,
            release,
            rating,
            completed,
            source: _source,
        } = self;

        let sql = "INSERT into wish (id, media_type, name, created_at, synopsis, release, rating, completed, season_number, episode_number) VALUES (:id, :media_type, :name, :created_at, :synopsis, :release, :rating, :completed,   :season_number, :episode_number) ON CONFLICT DO NOTHING";

        let id = ToSqlOutput::from(*id);
        let name = ToSqlOutput::from(name.clone());
        let added = datetime_to_sql(added);
        let synopsis = ToSqlOutput::from(synopsis.clone());
        let release = naivedate_to_sql(release);
        let rating = ToSqlOutput::Owned(Value::from(*rating));
        let completed = ToSqlOutput::from(*completed);

        let null = ToSqlOutput::from(0);
        let (kind, season_number, episode_number) = match kind {
            WishKind::Movie {
                duration: _duration,
                tags: _tags,
            } => {
                let kind = ToSqlOutput::from("movie".to_owned());

                (kind, null.clone(), null)
            }
            WishKind::Show {
                tags: _tags,
                seasons: _seasons,
            } => {
                let kind = ToSqlOutput::from("show".to_owned());
                (kind, null.clone(), null)
            }
            WishKind::Season {
                number,
                episodes: _episodes,
            } => {
                let number = ToSqlOutput::from(*number);
                let kind = ToSqlOutput::from("season".to_owned());

                (kind, number, null)
            }
            WishKind::Episode {
                season,
                number,
                duration: _duration,
            } => {
                let number = ToSqlOutput::from(*number);
                let season = ToSqlOutput::from(*season);
                let kind = ToSqlOutput::from("episode".to_owned());

                (kind, season, number)
            }
        };

        let params = vec![
            (":id", id),
            (":media_type", kind),
            (":name", name),
            (":created_at", added),
            (":synopsis", synopsis),
            (":release", release),
            (":rating", rating),
            (":completed", completed),
            (":season_number", season_number),
            (":episode_number", episode_number),
        ];

        Query {
            id: self.id.0,
            table: Table::Wishlist,
            op: Operation::Insert,
            sql,
            params,
        }
    }

    pub fn update<'a>(id: WishId, name: String, kind: WishKind, source: String) -> Query<'a> {
        let sql = "UPDATE wish SET name=:name, media_type=:media_type, season_number=:season_number, episode_number=:episode_number, source=:source WHERE id=:id";
        let null = ToSqlOutput::from(0);

        let (kind, season, episode) = match kind {
            WishKind::Movie { .. } => {
                (ToSqlOutput::from(String::from("movie")), null.clone(), null)
            }
            WishKind::Show { .. } => (ToSqlOutput::from(String::from("show")), null.clone(), null),
            WishKind::Season { number, .. } => (
                ToSqlOutput::from(String::from("season")),
                ToSqlOutput::from(number),
                null,
            ),

            WishKind::Episode { season, number, .. } => (
                ToSqlOutput::from(String::from("episode")),
                ToSqlOutput::from(season),
                ToSqlOutput::from(number),
            ),
        };

        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":name", ToSqlOutput::from(name)),
            (":media_type", kind),
            (":season_number", season),
            (":episode_number", episode),
            (":source", ToSqlOutput::from(source)),
        ];

        Query {
            id: id.0,
            table: Table::Wishlist,
            op: Operation::Update,
            sql,
            params,
        }
    }

    pub fn set_completion<'a>(id: WishId, completed: bool) -> Query<'a> {
        let sql = "UPDATE wish SET completed=:completed WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":completed", ToSqlOutput::from(completed)),
        ];

        Query {
            id: id.0,
            table: Table::Wishlist,
            op: Operation::Update,
            sql,
            params,
        }
    }

    pub fn set_source_request<'a>(id: WishId, source: String, request_id: String) -> Query<'a> {
        let sql = "UPDATE wish SET source=:source, request=:request WHERE id=:id";

        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":source", ToSqlOutput::from(source)),
            (":request", ToSqlOutput::from(request_id)),
        ];

        Query {
            id: id.0,
            table: Table::Wishlist,
            op: Operation::Insert,
            sql,
            params,
        }
    }

    pub fn delete<'a>(id: WishId) -> Query<'a> {
        let sql = "DELETE FROM wish WHERE id=:id";
        let params = vec![(":id", ToSqlOutput::from(id))];

        Query {
            id: id.0,
            table: Table::Wishlist,
            op: Operation::Delete,
            sql,
            params,
        }
    }

    pub fn release_year(&self) -> String {
        use chrono::Datelike;

        self.release.year().to_string()
    }

    pub fn new(name: String, kind: WishKind, source: String) -> Self {
        let release = NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap();

        Self {
            id: WishId(Uuid::now_v7()),
            kind,
            name,
            added: Local::now(),
            poster: None,
            synopsis: String::default(),
            release,
            rating: None,
            completed: false,
            source,
        }
    }
}
