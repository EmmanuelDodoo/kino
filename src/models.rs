use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use uuid::Uuid;

pub mod collection;
pub mod episodes;
pub mod movies;
pub mod seasons;
pub mod shows;

pub use collection::ItemId;
pub use collection::{Collection, CollectionId, CollectionView, SimpleCollection};
pub use episodes::*;
pub use movies::*;
pub use seasons::*;
pub use shows::*;

use crate::db::{Operation, Query, Table};

pub trait Media {
    type Id: Copy + Clone + std::hash::Hash + PartialEq + Eq;

    fn name(&self) -> &str;

    fn id(&self) -> Self::Id;

    fn duration(&self) -> u64;

    fn added(&self) -> DateTime<Local>;

    fn release(&self) -> NaiveDate;

    fn recent(&self) -> Option<DateTime<Local>>;

    fn progress(&self) -> f32;

    fn watch_count(&self) -> u32;

    fn rating(&self) -> Option<f32>;

    fn comments(&self) -> u32;

    fn synopsis(&self) -> &str;

    fn poster(&self) -> Option<&str>;

    fn backdrop(&self) -> Option<&str>;

    fn release_year(&self) -> String {
        use chrono::Datelike;

        self.release().year().to_string()
    }

    fn release_my(&self) -> String {
        let release = self.release();

        format!("{}", release.format("%b, %Y"))
    }

    fn added_my(&self) -> String {
        let added = self.added();

        format!("{}", added.format("%b, %Y"))
    }

    /// Duration in `(hrs) hours (mins) minutes` format.
    fn duration_full(&self) -> String {
        let duration = self.duration();

        if duration < 60 {
            return format!("{duration} seconds");
        }

        let hrs = duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs}:")
        } else {
            String::default()
        };

        let mins = (duration % 3600) / 60;
        let secs = (duration % 3600) % 60;

        format!("{hrs}{mins:02}:{secs:02}")
    }

    /// Duration in the `(hrs)h (mins)m` format.
    fn duration_short(&self) -> String {
        let duration = self.duration();

        if duration < 60 {
            return format!("{duration:02}s");
        }

        let hrs = duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs}h")
        } else {
            String::default()
        };

        let mins = (duration % 3600) / 60;
        let mins = if mins > 0 {
            format!("{mins}m")
        } else {
            String::default()
        };

        format!("{hrs} {mins}")
    }

    fn recent_short(&self) -> Option<String> {
        let recent = self.recent();

        recent.map(|recent| format!("{}", recent.format("%b %d, %Y")))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum MediaType {
    Movies,
    Shows,
}

impl MediaType {
    pub const ALL: [Self; 2] = [Self::Movies, Self::Shows];

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "movies" => Some(Self::Movies),
            "shows" => Some(Self::Shows),
            _ => None,
        }
    }
}

impl FromSql for MediaType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| MediaType::from_str(s).ok_or(FromSqlError::InvalidType))
    }
}

impl<'a> From<MediaType> for ToSqlOutput<'a> {
    fn from(value: MediaType) -> Self {
        ToSqlOutput::from(value.to_string())
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Shows => "shows",
                Self::Movies => "movies",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectoryId(Uuid);

impl From<DirectoryId> for ToSqlOutput<'_> {
    fn from(value: DirectoryId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    pub id: DirectoryId,
    pub path: String,
    pub active: bool,
    pub media_type: MediaType,
    pub last_scan: DateTime<Local>,
}

impl Directory {
    pub(super) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = row.get::<_, String>("id")?;
        let id = DirectoryId(Uuid::try_parse(&id).unwrap());
        let path = row.get::<_, String>("path")?;
        let media_type = row.get::<_, MediaType>("media_type")?;
        let active = row.get::<_, bool>("active")?;
        let last_scan = row.get::<_, DateTime<Local>>("last_scan")?;

        Ok(Self {
            id,
            path,
            media_type,
            active,
            last_scan,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            path,
            active,
            media_type,
            last_scan,
        } = self;

        let id = ToSqlOutput::from(*id);

        let path = ToSqlOutput::from(path.clone());
        let active = ToSqlOutput::from(*active);
        let media_type = ToSqlOutput::from(*media_type);
        let last_scan = datetime_to_sql(last_scan);

        vec![
            (":id", id),
            (":path", path),
            (":active", active),
            (":media_type", media_type),
            (":last_scan", last_scan),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO directory (id, path, active, media_type, last_scan) VALUES (:id, :path, :active, :media_type, :last_scan)";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Directory,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM directory WHERE id=:id";

        let id = ToSqlOutput::from(self.id);

        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::Directory,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_path<'a>(&mut self, path: String) -> Query<'a> {
        self.path = path.clone();

        let sql = "UPDATE directory SET path=:path WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":path", ToSqlOutput::from(path)),
        ];

        Query {
            id: self.id.0,
            table: Table::Directory,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_active<'a>(&mut self, active: bool) -> Query<'a> {
        self.active = active;

        let sql = "UPDATE directory SET active=:active WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":active", ToSqlOutput::from(active)),
        ];

        Query {
            id: self.id.0,
            table: Table::Directory,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_last_scan<'a>(&mut self, last_scan: DateTime<Local>) -> Query<'a> {
        self.last_scan = last_scan;

        let sql = "UPDATE directory SET last_scan=:last_scan WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":last_scan", datetime_to_sql(&last_scan)),
        ];

        Query {
            id: self.id.0,
            table: Table::Directory,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new(path: String, media_type: MediaType, active: bool) -> Self {
        let last_scan = Local::now();

        Self {
            id: DirectoryId(Uuid::now_v7()),
            media_type,
            path,
            active,
            last_scan,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub id: ItemId,
    pub name: String,
    pub snippet: String,
    pub tags: Vec<String>,
    pub poster: Option<String>,
}

impl SearchItem {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = ItemId::from_row(row)?;

        let name = row.get::<_, String>("name")?;

        let snippet = row.get::<_, String>("snippet")?;

        let poster = row.get::<_, Option<String>>("poster")?;

        let tags = row.get::<_, Option<String>>("tags")?;
        let tags = {
            tags.map(|tags| {
                tags.split(",")
                    .map(ToOwned::to_owned)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
        };

        Ok(Self {
            id,
            name,
            snippet,
            tags,
            poster,
        })
    }
}

fn naivedate_to_sql<'a>(date: &NaiveDate) -> ToSqlOutput<'a> {
    let date_str = date.format("%F").to_string();
    ToSqlOutput::from(date_str)
}

pub fn datetime_to_sql<'a>(datetime: &DateTime<Local>) -> ToSqlOutput<'a> {
    let str_date = datetime
        .with_timezone(&chrono::Utc)
        .format("%F %T%.f%:z")
        .to_string();

    ToSqlOutput::from(str_date)
}
