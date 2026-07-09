use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use std::path::PathBuf;
use uuid::Uuid;

pub mod collection;
pub mod comment;
pub mod episodes;
pub mod image;
pub mod movies;
pub mod seasons;
pub mod shows;
pub mod sources;
pub mod video;
pub mod wish;
pub use media::Media;

pub use collection::ItemId;
pub use collection::{Collection, CollectionId, CollectionView, SimpleCollection};
pub use comment::*;
pub use episodes::*;
use image::Image;
pub use movies::*;
pub use seasons::*;
pub use shows::*;
pub use video::*;
pub use wish::*;

use crate::db::{Operation, Query, Table};

pub fn humanize_datetime(from: DateTime<Local>, to: DateTime<Local>) -> String {
    use std::time::Duration;
    const SEC: u64 = 1;
    const MIN: u64 = 60;
    const HOUR: u64 = MIN * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = WEEK * 4;
    const YEAR: u64 = 365 * DAY;

    let duration = to.signed_duration_since(from);

    let Ok(duration) = duration.to_std() else {
        return "??".to_owned();
    };

    let max = Duration::from_secs(YEAR);
    let min = Duration::from_secs(SEC);

    if duration > max || duration < min {
        return from.format("%b %d, %Y").to_string();
    };

    let format = |value: u64, name: &str| {
        let single = value <= 1;

        format!("About {value} {name}{} ago", if single { "" } else { "s" })
    };

    match duration.as_secs() {
        d if d < SEC => unreachable!(),
        secs if secs < MIN => format(secs, "second"),
        mins if mins < HOUR => {
            let mins = mins / MIN;

            format(mins, "minute")
        }
        hours if hours < DAY => {
            let hours = hours / HOUR;

            format(hours, "hour")
        }
        days if days < WEEK => {
            let days = days / DAY;

            format(days, "day")
        }
        weeks if weeks < MONTH => {
            let weeks = weeks / WEEK;

            format(weeks, "week")
        }
        months if months < YEAR => {
            let months = months / MONTH;

            format(months, "month")
        }
        years => {
            let years = years / YEAR;

            format(years, "year")
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum MediaType {
    Movies,
    Shows,
}

impl MediaType {
    pub const ALL: [Self; 2] = [Self::Movies, Self::Shows];

    pub fn parse_str(s: &str) -> Option<Self> {
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
            .and_then(|s| MediaType::parse_str(s).ok_or(FromSqlError::InvalidType))
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

impl DirectoryId {
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>("directory")
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<DirectoryId> for ToSqlOutput<'_> {
    fn from(value: DirectoryId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for DirectoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    pub id: DirectoryId,
    pub path: PathBuf,
    pub name: String,
    pub active: bool,
    pub media_type: MediaType,
    pub last_scan: DateTime<Local>,
    pub source: String,
}

impl Directory {
    pub(super) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = row.get::<_, String>("id")?;
        let id = DirectoryId(Uuid::try_parse(&id).unwrap());

        let path = row.get::<_, String>("path")?;
        let path = PathBuf::from(path);

        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown directory")
            .to_owned();

        let media_type = row.get::<_, MediaType>("media_type")?;
        let active = row.get::<_, bool>("active")?;
        let last_scan = row.get::<_, DateTime<Local>>("last_scan")?;
        let source = row.get::<_, String>("source")?;

        Ok(Self {
            id,
            path,
            name,
            media_type,
            active,
            last_scan,
            source,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            path,
            name: _unused,
            active,
            media_type,
            last_scan,
            source,
        } = self;

        let id = ToSqlOutput::from(*id);

        let path = ToSqlOutput::from(path.display().to_string());
        let active = ToSqlOutput::from(*active);
        let media_type = ToSqlOutput::from(*media_type);
        let last_scan = datetime_to_sql(last_scan);
        let source = ToSqlOutput::from(source.clone());

        vec![
            (":id", id),
            (":path", path),
            (":active", active),
            (":media_type", media_type),
            (":last_scan", last_scan),
            (":source", source),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO directory (id, path, active, media_type, last_scan, source) VALUES (:id, :path, :active, :media_type, :last_scan, :source)";

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
        self.path = PathBuf::from(&path);

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

    pub fn new(path: String, media_type: MediaType, active: bool, source: String) -> Self {
        let last_scan = Local::now();
        let path = PathBuf::from(path);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown directory")
            .to_owned();

        Self {
            id: DirectoryId(Uuid::now_v7()),
            media_type,
            path,
            name,
            active,
            last_scan,
            source,
        }
    }

    pub fn is_movie(&self) -> bool {
        matches!(self.media_type, MediaType::Movies)
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

/// Duration in `(hrs) hours (mins) minutes` format.
pub fn duration_full(duration: u64) -> String {
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

pub mod media {
    use super::*;
    use rusqlite::Result;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Status {
        Normal = 0,
        Tombstone = 1,
        Archived = 2,
    }

    impl Status {
        pub fn from_row(row: &Row<'_>) -> Result<Self> {
            let status = row.get::<_, u8>("status")?;

            Ok(match status {
                0 => Self::Normal,
                1 => Self::Tombstone,
                2 => Self::Archived,
                _ => Self::Normal,
            })
        }
    }

    impl std::fmt::Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", *self as u8)
        }
    }

    impl From<Status> for ToSqlOutput<'_> {
        fn from(value: Status) -> Self {
            let code = value as u8;

            ToSqlOutput::from(code)
        }
    }

    pub trait Media {
        type Id: Copy + Clone + std::hash::Hash + PartialEq + Eq + Send;

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

        fn poster(&self) -> Option<&Image>;

        fn backdrop(&self) -> Option<&str>;

        fn source(&self) -> &str;

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

        fn added_humaized(&self) -> String {
            humanize_datetime(self.added(), Local::now())
        }

        /// Duration in `(hrs) hours (mins) minutes` format.
        fn duration_full(&self) -> String {
            duration_full(self.duration())
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

        fn recent_humanized(&self) -> Option<String> {
            self.recent()
                .map(|recent| humanize_datetime(recent, Local::now()))
        }

        fn status(&self) -> Status;
    }
}
