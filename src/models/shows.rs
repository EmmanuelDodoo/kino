use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;

use super::{DirectoryId, Media, SeasonId, datetime_to_sql, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShowId(Uuid);

impl ShowId {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("id", row)
    }

    pub fn from_collection(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("media_id", row)
    }

    pub fn from_child(row: &Row<'_>) -> rusqlite::Result<Self> {
        Self::from_row_helper("show_id", row)
    }

    fn from_row_helper(column: &str, row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>(column)
            .map(|id| ShowId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<ShowId> for ToSqlOutput<'_> {
    fn from(value: ShowId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Show {
    pub id: ShowId,
    pub directory: DirectoryId,
    path: String,
    name: String,
    original_name: String,
    poster: Option<String>,
    pub(super) backdrop: Option<String>,
    pub tags: Vec<String>,
    synopsis: String,
    release: NaiveDate,
    added: DateTime<Local>,
    watch_count: u32,
    pub seasons: u16,
    rating: Option<f32>,
    progress: f32,
    last_watched: Option<DateTime<Local>>,
    pub recent_season: Option<SeasonId>,
    duration: u64,
    comments: u32,
}

impl Show {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        // let id = ShowId(row.get::<_, Uuid>("id")?);
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let id = ShowId::from_row(row)?;
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

        let seasons = row.get::<_, u16>("season_count")?;

        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;

        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        // let recent_season = row.get::<_, Option<Uuid>>("recent_season")?.map(SeasonId);
        let recent_season = SeasonId::from_recents(row)?;

        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;

        Ok(Self {
            id,
            directory,
            path,
            name,
            original_name,
            poster,
            backdrop,
            tags,
            synopsis,
            release,
            added,
            watch_count,
            seasons,
            rating,
            progress,
            last_watched,
            recent_season,
            duration,
            comments,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            directory,
            path,
            name,
            original_name,
            poster,
            backdrop,
            tags,
            synopsis,
            release,
            added,
            watch_count: _watch,
            seasons: _seasons,
            rating: _rating,
            progress: _progress,
            last_watched: _last_watched,
            recent_season: _recent,
            duration: _duration,
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
        ]
    }

    fn update_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let id = ToSqlOutput::from(self.id);
        let name = ToSqlOutput::from(self.name.clone());
        let poster = ToSqlOutput::Owned(Value::from(self.poster.clone()));
        let backdrop = ToSqlOutput::Owned(Value::from(self.backdrop.clone()));

        vec![
            (":id", id),
            (":name", name),
            (":poster", poster),
            (":backdrop", backdrop),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO tv_show (id, directory, path, name, original_name, poster, backdrop, tags, synopsis, release, created_at) VALUES (:id, :directory, :path, :name, :original_name, :poster, :backdrop, :tags, :synopsis, :release, :added) ON CONFLICT(directory, path) DO NOTHING";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Show,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM tv_show WHERE id=:id";

        let id = ToSqlOutput::from(self.id);

        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::Show,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    fn update<'a>(&self) -> Query<'a> {
        let sql = "UPDATE tv_show SET name=:name, poster=:poster, backdrop=:backdrop WHERE id=:id";
        let params = self.update_params();

        Query {
            id: self.id.0,
            table: Table::Show,
            sql,
            params,
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE tv_show SET name=:name WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::Show,
            sql,
            params,
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(&mut self, rating: f32) -> Query<'a> {
        debug_assert!((0.0..=5.0).contains(&rating), "Show rating out of range");
        self.rating = Some(rating);

        let sql = "UPDATE tv_show SET rating=:rating WHERE id=:id";
        let params = vec![
            (":id", ToSqlOutput::from(self.id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: self.id.0,
            table: Table::Show,
            sql,
            params,
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        directory: DirectoryId,
        path: String,
        name: String,
        original_name: String,
        seasons: u16,
    ) -> (Self, Query<'a>) {
        let added = Local::now();
        let backdrop = None;
        let poster = None;
        let tags = vec![];
        let synopsis = String::default();
        let release = NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap();

        let new = Self {
            id: ShowId(Uuid::now_v7()),
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
            seasons,
            rating: None,
            progress: 0.0,
            last_watched: None,
            recent_season: None,
            duration: 0,
            comments: 0,
        };

        let query = new.insert();
        (new, query)
    }
}

impl Media for Show {
    type Id = ShowId;

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
