use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;

use super::{
    AudioId, DirectoryId, Media, SubtitleId, VideoInfoId, datetime_to_sql, image::Image, media::*,
    naivedate_to_sql,
};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovieId(pub(super) Uuid);

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

    /// Expects 'media_id' column name
    pub fn from_collection(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    pub(super) fn from_row_helper(column: &str, row: &Row<'_>) -> rusqlite::Result<Self> {
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
    pub path: String,
    pub poster: Option<Image>,
    pub backdrop: Option<String>,
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
    pub video_id: Option<VideoInfoId>,
    pub audio_id: Option<AudioId>,
    pub subtitle_id: Option<SubtitleId>,
    source: String,
    pub status: Status,
}

impl Movie {
    pub fn subtitle_maybe(row: &Row<'_>) -> rusqlite::Result<Option<SubtitleId>> {
        SubtitleId::from_row_maybe("subtitle_id", row)
    }

    pub fn video_maybe(row: &Row<'_>) -> rusqlite::Result<Option<VideoInfoId>> {
        VideoInfoId::from_row_maybe("video_id", row)
    }

    pub fn audio_maybe(row: &Row<'_>) -> rusqlite::Result<Option<AudioId>> {
        AudioId::from_row_maybe("audio_id", row)
    }

    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Movie> {
        let id = MovieId::from_row(row)?;
        let directory = DirectoryId::from_row(row)?;

        let path = row.get::<_, String>("path")?;

        let name = row.get::<_, String>("name")?;
        let original_name = row.get::<_, String>("original_name")?;

        let poster = {
            let poster = row.get::<_, Option<String>>("poster")?;

            if poster.is_some() {
                Some(Image::from_row(row, "poster_")?)
            } else {
                None
            }
        };

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

        let source = row.get::<_, String>("source")?;

        let video_id = Self::video_maybe(row)?;
        let subtitle_id = Self::subtitle_maybe(row)?;
        let audio_id = Self::audio_maybe(row)?;

        let status = Status::from_row(row)?;

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
            source,
            video_id,
            subtitle_id,
            audio_id,
            status,
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
            source: _source,
            video_id: _video,
            audio_id: _audio,
            subtitle_id: _sub,
            status,
        } = self;

        let id = ToSqlOutput::from(*id);
        let directory = ToSqlOutput::from(*directory);
        let path = ToSqlOutput::from(path.clone());

        let name = ToSqlOutput::from(name.clone());
        let original_name = ToSqlOutput::from(original_name.clone());

        let poster = poster
            .as_ref()
            .map(|poster| poster.path.display().to_string());
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
        let status = ToSqlOutput::from(*status);

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
            (":status", status),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO movie (id, directory, path, name, original_name,  poster, backdrop, tags, synopsis, release, created_at, watch_count, rating, progress, last_watched, duration, status) VALUES (:id, :directory, :path, :name, :original_name,  :poster, :backdrop, :tags, :synopsis, :release, :added, :watch_count, :rating, :progress, :last_watched, :duration, :status) ON CONFLICT(directory, path) DO UPDATE SET duration=:duration, status=:status";

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
        let status = Status::Tombstone;
        let sql = "UPDATE movie SET status=:status WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":status", ToSqlOutput::from(status)),
        ];

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
    pub fn mark_watched<'a>(id: MovieId, count: u32) -> Query<'a> {
        let sql = "UPDATE movie SET watch_count=:count, progress=1.0 WHERE id=:id";
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
    pub fn set_video<'a>(id: MovieId, video: VideoInfoId) -> Query<'a> {
        let sql = "UPDATE movie SET video_id=:video WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":video", ToSqlOutput::from(video)),
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
    pub fn set_audio<'a>(id: MovieId, audio: AudioId) -> Query<'a> {
        let sql = "UPDATE movie SET audio_id=:audio WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":audio", ToSqlOutput::from(audio)),
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
    pub fn set_subtitle<'a>(id: MovieId, subtitle: SubtitleId) -> Query<'a> {
        let sql = "UPDATE movie SET subtitle_id=:subtitle WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":subtitle", ToSqlOutput::from(subtitle)),
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
    pub fn set_status<'a>(id: MovieId, status: Status) -> Query<'a> {
        let sql = "UPDATE movie SET status=:status WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":status", ToSqlOutput::from(status)),
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
    pub fn set_path<'a>(id: MovieId, path: String) -> Query<'a> {
        let sql = "UPDATE movie SET path=:path WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":path", ToSqlOutput::from(path)),
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
    pub fn set_dir<'a>(id: MovieId, dir: DirectoryId) -> Query<'a> {
        let sql = "UPDATE movie SET directory=:dir WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":dir", ToSqlOutput::from(dir)),
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
            source: String::default(),
            video_id: None,
            audio_id: None,
            subtitle_id: None,
            status: Status::Normal,
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

    fn poster(&self) -> Option<&Image> {
        self.poster.as_ref()
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

    fn source(&self) -> &str {
        &self.source
    }

    fn status(&self) -> Status {
        self.status
    }
}
