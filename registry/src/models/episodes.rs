use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;

use super::{
    AudioId, Media, SeasonId, SubtitleId, VideoInfoId, datetime_to_sql, image::Image,
    naivedate_to_sql,
};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpisodeId(pub(super) Uuid);

impl std::fmt::Display for EpisodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl EpisodeId {
    /// Expects relevant column name as "id"
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("id", row)
    }

    pub fn from_recents(row: &Row<'_>) -> rusqlite::Result<Option<Self>> {
        Ok(row
            .get::<_, Option<String>>("recent_episode")?
            .map(|id| EpisodeId(Uuid::try_parse(&id).unwrap())))
    }

    /// Expects 'media_id' column name
    pub fn from_collection(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    pub(super) fn from_row_helper(column: &str, row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>(column)
            .map(|id| EpisodeId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<EpisodeId> for ToSqlOutput<'_> {
    fn from(value: EpisodeId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: EpisodeId,
    name: String,
    original_name: String,
    pub show_name: String,
    pub season_number: u16,
    path: String,
    pub poster: Option<Image>,
    // Join from show
    pub backdrop: Option<String>,
    synopsis: String,
    release: NaiveDate,
    added: DateTime<Local>,
    watch_count: u32,
    pub season: SeasonId,
    progress: f32,
    rating: Option<f32>,
    last_watched: Option<DateTime<Local>>,
    duration: u64,
    comments: u32,
    pub number: u16,
    source: String,
    pub video_id: Option<VideoInfoId>,
    pub audio_id: Option<AudioId>,
    pub subtitle_id: Option<SubtitleId>,
}

impl Episode {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = EpisodeId::from_row(row)?;

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
        let synopsis = row.get::<_, String>("synopsis")?;

        let release = row.get::<_, NaiveDate>("release")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let watch_count = row.get::<_, u32>("watch_count")?;

        let number = row.get::<_, u16>("episode_number")?;

        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;

        let season = SeasonId::from_child(row)?;

        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;
        let source = row.get::<_, String>("source")?;

        let video_id = VideoInfoId::from_row_maybe("video_id", row)?;
        let subtitle_id = SubtitleId::from_row_maybe("subtitle_id", row)?;
        let audio_id = AudioId::from_row_maybe("audio_id", row)?;

        let show_name = row.get::<_, String>("show_name")?;
        let season_number = row.get::<_, u16>("season_number")?;

        Ok(Self {
            id,
            name,
            original_name,
            path,
            poster,
            backdrop,
            synopsis,
            release,
            added,
            watch_count,
            season,
            progress,
            rating,
            last_watched,
            duration,
            comments,
            number,
            source,
            video_id,
            subtitle_id,
            audio_id,
            show_name,
            season_number,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            name,
            original_name,
            path,
            poster,
            backdrop: _backdrop,
            synopsis,
            release,
            added,
            watch_count,
            season,
            progress,
            rating,
            last_watched,
            duration,
            comments,
            number,
            source: _source,
            video_id: _video,
            subtitle_id: _sub,
            audio_id: _audio,
            show_name: _show,
            season_number: _season_number,
        } = self;

        let id = ToSqlOutput::from(*id);
        let season = ToSqlOutput::from(*season);
        let path = ToSqlOutput::from(path.clone());

        let name = ToSqlOutput::from(name.clone());
        let original_name = ToSqlOutput::from(original_name.clone());

        let poster = poster
            .as_ref()
            .map(|poster| poster.path.display().to_string());
        let poster = ToSqlOutput::Owned(Value::from(poster.clone()));

        let synopsis = ToSqlOutput::from(synopsis.clone());
        let release = naivedate_to_sql(release);
        let added = datetime_to_sql(added);
        let watch_count = ToSqlOutput::from(*watch_count);
        let number = ToSqlOutput::from(*number);

        let rating = ToSqlOutput::Owned(Value::from(*rating));
        let progress = ToSqlOutput::from(*progress);
        let last_watched = last_watched
            .map(|date| datetime_to_sql(&date))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let duration = i64::try_from(*duration).expect("duration cannot be expressed as i64");
        let duration = ToSqlOutput::from(duration);
        let comments = ToSqlOutput::from(*comments);

        vec![
            (":id", id),
            (":season", season),
            (":name", name),
            (":original_name", original_name),
            (":path", path),
            (":poster", poster),
            (":synopsis", synopsis),
            (":release", release),
            (":added", added),
            (":watch_count", watch_count),
            (":rating", rating),
            (":progress", progress),
            (":last_watched", last_watched),
            (":duration", duration),
            (":comments", comments),
            (":episode_number", number),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO episode (id, season_id, name, original_name,  path, poster, synopsis, release, created_at, watch_count, rating, progress, last_watched, duration, comment_count, episode_number) VALUES (:id, :season, :name, :original_name, :path, :poster, :synopsis, :release, :added, :watch_count, :rating, :progress, :last_watched, :duration, :comments, :episode_number) ON CONFLICT(season_id, path) DO UPDATE SET removed=FALSE";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove<'a>(id: EpisodeId) -> Query<'a> {
        let sql = "UPDATE episode SET removed=TRUE WHERE id=:id";
        let params = [(":id", ToSqlOutput::from(id))];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(id: EpisodeId, name: String) -> Query<'a> {
        let sql = "UPDATE episode SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":name", ToSqlOutput::from(name)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_synopsis<'a>(id: EpisodeId, synopsis: String) -> Query<'a> {
        let sql = "UPDATE episode SET synopsis=:synopsis WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":synopsis", ToSqlOutput::from(synopsis)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params,
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(id: EpisodeId, rating: f32) -> Query<'a> {
        debug_assert!((0.0..=5.0).contains(&rating), "Episode rating out of range");

        let sql = "UPDATE episode SET rating=:rating WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn mark_watched<'a>(id: EpisodeId, count: u32) -> Query<'a> {
        let sql = "UPDATE episode SET watch_count=:count, progress=1.0 WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(id)),
            (":count", ToSqlOutput::from(count)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_video<'a>(id: EpisodeId, video: VideoInfoId) -> Query<'a> {
        let sql = "UPDATE episode SET video_id=:video WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":video", ToSqlOutput::from(video)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_audio<'a>(id: EpisodeId, audio: AudioId) -> Query<'a> {
        let sql = "UPDATE episode SET audio_id=:audio WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":audio", ToSqlOutput::from(audio)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_subtitle<'a>(id: EpisodeId, subtitle: SubtitleId) -> Query<'a> {
        let sql = "UPDATE episode SET subtitle_id=:subtitle WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(id)),
            (":subtitle", ToSqlOutput::from(subtitle)),
        ];

        Query {
            id: id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        season: SeasonId,
        name: String,
        original_name: String,
        path: String,
        duration: u64,
        number: u16,
    ) -> (Self, Query<'a>) {
        let added = Local::now();
        let backdrop = None;
        let poster = None;
        let synopsis = String::default();
        let release = NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap();

        let new = Self {
            id: EpisodeId(Uuid::now_v7()),
            name,
            original_name,
            path,
            season,
            backdrop,
            poster,
            synopsis,
            added,
            release,
            duration,
            watch_count: 0,
            rating: None,
            progress: 0.0,
            last_watched: None,
            comments: 0,
            number,
            video_id: None,
            audio_id: None,
            subtitle_id: None,
            source: String::default(),

            // Not saved within episode table so these values are okay
            show_name: String::default(),
            season_number: 0,
        };

        let query = new.insert();

        (new, query)
    }
}

impl Media for Episode {
    type Id = EpisodeId;

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
}
