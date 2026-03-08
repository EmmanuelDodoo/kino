use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;

use super::{DirectoryId, Media, datetime_to_sql, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovieId(Uuid);

impl From<MovieId> for ToSqlOutput<'_> {
    fn from(value: MovieId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for MovieId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl MovieId {
    /// Expects relevant column name as "id"
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("id", row)
    }

    pub fn from_collection(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    fn from_row_helper(column: &str, row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>(column)
            .map(|id| MovieId(Uuid::try_parse(&id).unwrap()))
    }
}

#[derive(Debug, Clone)]
pub struct Movie {
    pub id: MovieId,
    name: String,
    original_name: String,
    pub directory: DirectoryId,
    path: String,
    poster: Option<String>,
    backdrop: Option<String>,
    pub tags: Vec<String>,
    synopsis: String,
    release: NaiveDate,
    added: DateTime<Local>,
    watch_count: u32,
    rating: Option<f32>,
    progress: f32,
    last_watched: Option<DateTime<Local>>,
    duration: u64,
    comments: u32,
}

impl Movie {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Movie> {
        let id = MovieId::from_row(row)?;
        let directory = row.get::<_, String>("directory")?;
        let directory = DirectoryId(Uuid::try_parse(&directory).unwrap());

        let path = row.get::<_, String>("path")?;

        let name = row.get::<_, String>("name")?;
        let original_name = row.get::<_, String>("original_name")?;
        let poster = row.get::<_, Option<String>>("poster")?;
        let backdrop = row.get::<_, Option<String>>("backdrop")?;
        let tags = {
            let tags = row.get::<_, Option<String>>("tags")?;
            tags.map(|tags| {
                tags.split(",")
                    .map(ToOwned::to_owned)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
        };
        let synopsis = row.get::<_, String>("synopsis")?;

        let release = row.get::<_, NaiveDate>("release")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let watch_count = row.get::<_, u32>("watch_count")?;
        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;
        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;

        Ok(Self {
            id,
            directory,
            name,
            original_name,
            path,
            poster,
            backdrop,
            tags,
            synopsis,
            release,
            added,
            watch_count,
            progress,
            rating,
            last_watched,
            duration,
            comments,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            name,
            original_name,
            directory,
            path,
            poster,
            backdrop,
            tags,
            synopsis,
            release,
            added,
            watch_count,
            rating,
            progress,
            last_watched,
            duration,
            comments: _comments,
        } = self;

        let id = ToSqlOutput::from(*id);
        let directory = ToSqlOutput::from(*directory);
        let path = ToSqlOutput::from(path.clone());

        let name = ToSqlOutput::from(name.clone());
        let original_name = ToSqlOutput::from(original_name.clone());
        let poster = ToSqlOutput::Owned(Value::from(poster.clone()));
        let backdrop = ToSqlOutput::Owned(Value::from(backdrop.clone()));
        let tags = ToSqlOutput::from(tags.join(","));
        let synopsis = ToSqlOutput::from(synopsis.clone());
        let release = naivedate_to_sql(release);
        let added = datetime_to_sql(added);
        let watch_count = ToSqlOutput::from(*watch_count);

        let rating = ToSqlOutput::Owned(Value::from(*rating));
        let progress = ToSqlOutput::from(*progress);
        let last_watched = last_watched
            .map(|date| datetime_to_sql(&date))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));
        let duration = i64::try_from(*duration).expect("duration cannot be expressed as i64");
        let duration = ToSqlOutput::from(duration);

        vec![
            (":id", id),
            (":directory", directory),
            (":path", path),
            (":name", name),
            (":original_name", original_name),
            (":poster", poster),
            (":backdrop", backdrop),
            (":tags", tags),
            (":synopsis", synopsis),
            (":release", release),
            (":added", added),
            (":watch_count", watch_count),
            (":rating", rating),
            (":progress", progress),
            (":last_watched", last_watched),
            (":duration", duration),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO movie (id, directory, path, name, original_name, poster, backdrop, tags, synopsis, release, created_at, watch_count, rating, progress, last_watched, duration) VALUES (:id, :directory, :path, :name, :original_name, :poster, :backdrop, :tags, :synopsis, :release, :added, :watch_count, :rating, :progress, :last_watched, :duration) ON CONFLICT(directory, path) DO NOTHING";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove<'a>(id: MovieId) -> Query<'a> {
        let sql = "UPDATE movie SET removed=TRUE WHERE id=:id";
        let params = [(":id", ToSqlOutput::from(id))];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(id: MovieId, name: String) -> Query<'a> {
        let sql = "UPDATE movie SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":name", ToSqlOutput::from(name)),
        ];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_synopsis<'a>(id: MovieId, synopsis: String) -> Query<'a> {
        let sql = "UPDATE movie SET synopsis=:synopsis WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":synopsis", ToSqlOutput::from(synopsis)),
        ];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params,
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn refetch<'a>(id: MovieId) -> Query<'a> {
        let sql = "UPDATE movie SET tmdb_id=NULL, poster=NULL, backdrop=NULL, fetched=FALSE, generate_poster=TRUE WHERE id=:id";
        let params = [(":id", ToSqlOutput::from(id))];

        Query {
            id: id.0,
            table: Table::Movies,
            op: Operation::Update,
            sql,
            params: params.to_vec(),
        }
    }

    #[must_use]
    pub fn set_watch_count<'a>(id: MovieId, count: u32) -> Query<'a> {
        let sql = "UPDATE movie SET watch_count=:count WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":count", ToSqlOutput::from(count)),
        ];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(id: MovieId, rating: f32) -> Query<'a> {
        debug_assert!((0.0..=5.0).contains(&rating), "Movie rating out of range");

        let sql = "UPDATE movie SET rating=:rating WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_tmdb_id<'a>(id: MovieId, tmdb_id: u32) -> Query<'a> {
        let sql = "UPDATE movie SET user_tmdb_id=:tmdb_id, tmdb_id=NULL, poster=NULL, backdrop=NULL, fetched=FALSE WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":tmdb_id", ToSqlOutput::from(tmdb_id)),
        ];

        Query {
            id: id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        directory: DirectoryId,
        path: String,
        name: String,
        original_name: String,
        duration: u64,
    ) -> (Self, Query<'a>) {
        let added = Local::now();
        let backdrop = None;
        let poster = None;
        let tags = vec![];
        let synopsis = String::default();
        let release = NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap();

        let new = Self {
            id: MovieId(Uuid::now_v7()),
            directory,
            path,
            name,
            original_name,
            backdrop,
            poster,
            tags,
            synopsis,
            release,
            added,
            watch_count: 0,
            progress: 0.0,
            rating: None,
            last_watched: None,
            duration,
            comments: 0,
        };

        let query = new.insert();

        (new, query)
    }
}

impl Media for Movie {
    type Id = MovieId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn added(&self) -> DateTime<Local> {
        self.added
    }

    fn recent(&self) -> Option<DateTime<Local>> {
        self.last_watched
    }

    fn rating(&self) -> Option<f32> {
        self.rating
    }

    fn poster(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn backdrop(&self) -> Option<&str> {
        self.backdrop.as_deref()
    }

    fn release(&self) -> NaiveDate {
        self.release
    }

    fn duration(&self) -> u64 {
        self.duration
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn comments(&self) -> u32 {
        self.comments
    }

    fn synopsis(&self) -> &str {
        &self.synopsis
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
    }
}
