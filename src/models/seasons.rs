use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;

use super::{EpisodeId, Media, ShowId, datetime_to_sql, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeasonId(Uuid);

impl std::fmt::Display for SeasonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SeasonId {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("id", row)
    }

    pub fn from_collection(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    pub fn from_recents(row: &Row<'_>) -> rusqlite::Result<Option<Self>> {
        Ok(row
            .get::<_, Option<String>>("recent_season")?
            .map(|id| SeasonId(Uuid::try_parse(&id).unwrap())))
    }

    pub fn from_child(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("season_id", row)
    }

    fn from_row_helper(column: &str, row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>(column)
            .map(|id| SeasonId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<SeasonId> for ToSqlOutput<'_> {
    fn from(value: SeasonId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Season {
    pub id: SeasonId,
    name: String,
    original_name: String,
    path: String,
    poster: Option<String>,
    // Join from show
    backdrop: Option<String>,
    synopsis: String,
    release: NaiveDate,
    added: DateTime<Local>,
    watch_count: u32,
    pub show: ShowId,
    pub episodes: u16,
    progress: f32,
    rating: Option<f32>,
    last_watched: Option<DateTime<Local>>,
    pub recent_episode: Option<EpisodeId>,
    duration: u64,
    comments: u32,
    pub number: u16,
}

impl Season {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        // let id = SeasonId(row.get::<_, Uuid>("id")?);
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let id = SeasonId::from_row(row)?;

        let path = row.get::<_, String>("path")?;

        let name = row.get::<_, String>("name")?;
        let original_name = row.get::<_, String>("original_name")?;
        let poster = row.get::<_, Option<String>>("poster")?;
        let backdrop = row.get::<_, Option<String>>("backdrop")?;
        let synopsis = row.get::<_, String>("synopsis")?;

        let release = row.get::<_, NaiveDate>("release")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let watch_count = row.get::<_, u32>("watch_count")?;

        let episodes = row.get::<_, u16>("episode_count")?;

        let number = row.get::<_, u16>("season_number")?;

        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;

        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let show = ShowId::from_child(row)?;

        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let recent_episode = EpisodeId::from_recents(row)?;

        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;

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
            show,
            episodes,
            progress,
            rating,
            last_watched,
            recent_episode,
            duration,
            comments,
            number,
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
            show,
            episodes,
            progress,
            rating,
            last_watched,
            recent_episode,
            duration,
            comments,
            number,
        } = self;

        let id = ToSqlOutput::from(*id);
        let show = ToSqlOutput::from(*show);
        let path = ToSqlOutput::from(path.clone());

        let name = ToSqlOutput::from(name.clone());
        let original_name = ToSqlOutput::from(original_name.clone());
        let poster = ToSqlOutput::Owned(Value::from(poster.clone()));

        let synopsis = ToSqlOutput::from(synopsis.clone());
        let release = naivedate_to_sql(release);
        let added = datetime_to_sql(added);
        let watch_count = ToSqlOutput::from(*watch_count);
        let episodes = ToSqlOutput::from(*episodes);
        let number = ToSqlOutput::from(*number);

        let rating = ToSqlOutput::Owned(Value::from(*rating));
        let progress = ToSqlOutput::from(*progress);
        let last_watched = last_watched
            .map(|date| datetime_to_sql(&date))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let duration = i64::try_from(*duration).expect("duration cannot be expressed as i64");
        let duration = ToSqlOutput::from(duration);
        let comments = ToSqlOutput::from(*comments);

        let recent_episode =
            recent_episode.map_or(ToSqlOutput::Owned(Value::Null), ToSqlOutput::from);

        vec![
            (":id", id),
            (":show", show),
            (":name", name),
            (":original_name", original_name),
            (":path", path),
            (":poster", poster),
            (":synopsis", synopsis),
            (":release", release),
            (":added", added),
            (":watch_count", watch_count),
            (":episodes", episodes),
            (":rating", rating),
            (":progress", progress),
            (":last_watched", last_watched),
            (":recent_episode", recent_episode),
            (":duration", duration),
            (":comments", comments),
            (":season_number", number),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO season (id, show_id, name, original_name, path, poster, synopsis,release, created_at, watch_count, episode_count, rating, progress, last_watched, recent_episode, duration, comment_count, season_number) VALUES (:id, :show, :name, :original_name, :path, :poster, :synopsis, :release, :added, :watch_count, :episodes, :rating, :progress, :last_watched, :recent_episode, :duration, :comments, :season_number) ON CONFLICT(show_id, path) DO NOTHING";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Season,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM season WHERE id=:id";
        let id = ToSqlOutput::from(self.id);
        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::Season,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE season SET name=:name WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::Season,
            sql,
            params,
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(&mut self, rating: f32) -> Query<'a> {
        debug_assert!((0.0..=5.0).contains(&rating), "Season rating out of range");
        self.rating = Some(rating);

        let sql = "UPDATE season SET rating=:rating WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(self.id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: self.id.0,
            table: Table::Season,
            sql,
            params,
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        show: ShowId,
        name: String,
        path: String,
        number: Option<u16>,
    ) -> (Self, Query<'a>) {
        let added = Local::now();
        let backdrop = None;
        let poster = None;
        let synopsis = String::default();
        let release = NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap();
        let original_name = name.clone();

        let new = Self {
            id: SeasonId(Uuid::now_v7()),
            show,
            name,
            original_name,
            path,
            backdrop,
            poster,
            synopsis,
            added,
            release,
            duration: 0,
            watch_count: 0,
            episodes: 0,
            rating: None,
            progress: 0.0,
            last_watched: None,
            recent_episode: None,
            comments: 0,
            number: number.unwrap_or_default(),
        };

        let query = new.insert();

        (new, query)
    }
}

impl Media for Season {
    type Id = SeasonId;

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
