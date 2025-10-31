use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use std::path::PathBuf;
use uuid::Uuid;

use super::{DirectoryId, Media, datetime_to_sql, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovieId(pub(super) Uuid);

impl From<MovieId> for ToSqlOutput<'_> {
    fn from(value: MovieId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MCommentId(Uuid);

impl From<MCommentId> for ToSqlOutput<'_> {
    fn from(value: MCommentId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Movie {
    pub id: MovieId,
    name: String,
    original_name: String,
    pub directory: DirectoryId,
    path: String,
    pub full_path: PathBuf,
    poster: Option<String>,
    backdrop: Option<String>,
    pub tags: Vec<String>,
    synapsis: String,
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
        // todo
        let id = row.get::<_, String>("id")?;
        let id = MovieId(Uuid::try_parse(&id).unwrap());
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
        let synapsis = row.get::<_, String>("synapsis")?;

        let release = row.get::<_, NaiveDate>("release")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let watch_count = row.get::<_, u32>("watch_count")?;
        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;
        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;

        let full_path: PathBuf = {
            let directory = row.get::<_, String>("directory_path")?;
            [&directory, &path].iter().collect()
        };

        Ok(Self {
            id,
            directory,
            name,
            original_name,
            path,
            full_path,
            poster,
            backdrop,
            tags,
            synapsis,
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
            full_path: _full_path,
            poster,
            backdrop,
            tags,
            synapsis,
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
        let synapsis = ToSqlOutput::from(synapsis.clone());
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
            (":synapsis", synapsis),
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
        let sql = "INSERT INTO movie (id, directory, path, name, original_name, poster, backdrop, tags, synapsis, release, created_at, watch_count, rating, progress, last_watched, duration) VALUES (:id, :directory, :path, :name, :original_name, :poster, :backdrop, :tags, :synapsis, :release, :added, :watch_count, :rating, :progress, :last_watched, :duration)";

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
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM movie WHERE id=:id";
        let params = [(":id", ToSqlOutput::from(self.id))];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE movie SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_watch_count<'a>(&mut self, count: u32) -> Query<'a> {
        self.watch_count = count;

        let sql = "UPDATE movie SET watch_count=:count WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":count", ToSqlOutput::from(count)),
        ];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(&mut self, rating: f32) -> Query<'a> {
        assert!(rating > 0.0 && rating <= 5.0, "Episode rating out of range");
        self.rating = Some(rating);

        let sql = "UPDATE movie SET rating=:rating WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_progress<'a>(&mut self, progress: f32) -> Query<'a> {
        assert!(
            (0.0..1.0).contains(&progress),
            "Episode progress out of range",
        );
        self.progress = progress;

        let sql = "UPDATE movie SET progress=:progress WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":progress", ToSqlOutput::from(progress)),
        ];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_last_watched<'a>(&mut self, watched: DateTime<Local>) -> Query<'a> {
        self.last_watched = Some(watched);

        let sql = "UPDATE movie SET last_watched=:watched WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":watched", datetime_to_sql(&watched)),
        ];

        Query {
            id: self.id.0,
            table: Table::Movies,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        directory: DirectoryId,
        path: String,
        full_path: PathBuf,
        name: String,
        original_name: String,
        backdrop: Option<String>,
        poster: Option<String>,
        tags: Vec<String>,
        synapsis: String,
        release: NaiveDate,
        duration: u64,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: MovieId(Uuid::now_v7()),
            directory,
            path,
            full_path,
            name,
            original_name,
            backdrop,
            poster,
            tags,
            synapsis,
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

    pub fn dummy<'a>(directory: DirectoryId) -> (Self, Query<'a>) {
        let path = "dummy_movie".to_owned();
        let full_path = ["movies", &path].iter().collect();
        let name = "Test Movie".to_owned();
        let backdrop = None;
        let poster = None;
        let tags = ["test", "hope", "growth"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        let synapsis = "A test dummy which is partially adequate".to_owned();
        let release = NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d").unwrap();
        let duration = 3600;

        Self::new(
            directory,
            path,
            full_path,
            name.clone(),
            name,
            backdrop,
            poster,
            tags,
            synapsis,
            release,
            duration,
        )
    }

    pub fn testing() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;

        let release = NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d").unwrap();

        let added = Local::now();

        let id = MovieId(Uuid::now_v7());

        let path = "~/Desktop/coding/Projects/kino/assets/test.mkv";
        let full_path = PathBuf::from(path);

        Self {
            id,
            directory: DirectoryId(Uuid::now_v7()),
            name: "Fantastic Beasts And Where To Find Them".to_owned(),
            original_name: "Fantastic Beasts And Where To Find Them".to_owned(),
            duration,
            path: path.to_owned(), full_path,
            rating: Some(3.85),
            progress: 0.35,
            poster: Some("assets/fantastic.png".into()),
            release,
            added ,
            last_watched: None,
            comments: 69,
            watch_count: 57,
            synapsis: "In 1926, Newt Scamander arrives at the Magical Congress of the United States of America with a magically expanded briefcase, which houses a number of dangerous creatures and their habitats. When the creatures escape from the briefcase, it sends the American wizarding authorities after Newt, and threatens to strain even further the state of magical and non-magical relations.".to_owned(),
            tags: vec!["tag-1".into(), "tag-2".into(), "tag-team".into()],
            backdrop: Some("assets/test.jpg".into())

        }
    }

    pub fn testing2() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;

        let added = Local::now();
        let release = NaiveDate::parse_from_str("2011-03-05", "%Y-%m-%d").unwrap();

        let last_watched = DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z").unwrap();
        let last_watched = Some(last_watched.into());

        let path = "~/Desktop/coding/Projects/kino/assets/test2.mp4";
        let full_path = PathBuf::from(path);

        Self {
            id: MovieId(Uuid::now_v7()),
            directory: DirectoryId(Uuid::now_v7()),
            name: "Ready Player One".to_owned(),
            original_name: "Ready Player One".to_owned(),
            path: path.to_owned(), full_path,
            duration,
            rating: Some(1.24),
            progress: 0.95,
            poster: Some("assets/ready.png".into()),
            last_watched,
            release,
            added,
            comments: 420,
            watch_count: 1,
            synapsis: "When the creator of a popular video game system dies, a virtual contest is created to compete for his fortune.".to_owned(),
            tags: vec!["Adventure", "Action", "Science Fiction"].into_iter().map(ToOwned::to_owned).collect(),
            backdrop: Some("assets/player1.jpg".into()),

        }
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

    fn path(&self) -> &str {
        &self.path
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

    fn synapsis(&self) -> &str {
        &self.synapsis
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
    }
}

#[derive(Debug, Clone)]
pub struct MComment {
    pub id: MCommentId,
    added: DateTime<Local>,
    content: String,
    movie: MovieId,
    timestamp: Option<u64>,
}

impl MComment {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        //todo
        let id = row.get::<_, String>("id")?;
        let id = MCommentId(Uuid::try_parse(&id).unwrap());

        let movie = row.get::<_, String>("movie_id")?;
        let movie = MovieId(Uuid::try_parse(&movie).unwrap());

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let content = row.get::<_, String>("content")?;

        let timestamp = row.get::<_, Option<u64>>("movie_timestamp")?;

        Ok(Self {
            id,
            added,
            content,
            movie,
            timestamp,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            added,
            content,
            movie,
            timestamp,
        } = self;

        let id = ToSqlOutput::from(*id);
        let added = datetime_to_sql(added);
        let content = ToSqlOutput::from(content.clone());

        let movie = ToSqlOutput::from(*movie);
        let timestamp = timestamp
            .map(|time| {
                ToSqlOutput::from(
                    i64::try_from(time).expect("timestamp cannot be expressed as i64"),
                )
            })
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        vec![
            (":id", id),
            (":added", added),
            (":content", content),
            (":movie", movie),
            (":timestamp", timestamp),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO movie_comment (id, created_at, content, movie_id, movie_timestamp) VALUES (:id, :added, :content, :movie, :timestamp)";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::MComment,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM movie_comment WHERE id=:id";
        let id = ToSqlOutput::from(self.id);
        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::MComment,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_content<'a>(&mut self, content: String) -> Query<'a> {
        self.content = content.clone();

        let sql = "UPDATE movie_comment SET content=:content WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":content", ToSqlOutput::from(content)),
        ];

        Query {
            id: self.id.0,
            table: Table::MComment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_timestamp<'a>(&mut self, timestamp: Option<u64>) -> Query<'a> {
        self.timestamp = timestamp;

        let sql = "UPDATE movie_comment SET movie_timestamp=:timestamp WHERE id=:id";

        let timestamp = timestamp
            .map(|time| {
                ToSqlOutput::from(
                    i64::try_from(time).expect("Timestamp cannot be expressed as i64"),
                )
            })
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":timestamp", timestamp),
        ];

        Query {
            id: self.id.0,
            table: Table::MComment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(movie: MovieId, content: String, timestamp: Option<u64>) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: MCommentId(Uuid::now_v7()),
            added,
            content,
            timestamp,
            movie,
        };

        let query = new.insert();

        (new, query)
    }

    pub fn dummy<'a>(movie: MovieId) -> (Self, Query<'a>) {
        let content = "A dummy movie comment".to_owned();
        let timestamp = None;

        Self::new(movie, content, timestamp)
    }
}
