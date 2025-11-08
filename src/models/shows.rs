#![allow(dead_code)]
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{ToSqlOutput, Value};
use std::path::PathBuf;
use uuid::Uuid;

use super::{DirectoryId, Media, datetime_to_sql, naivedate_to_sql};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShowId(pub(super) Uuid);

impl From<ShowId> for ToSqlOutput<'_> {
    fn from(value: ShowId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeasonId(pub(super) Uuid);

impl From<SeasonId> for ToSqlOutput<'_> {
    fn from(value: SeasonId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpisodeId(pub(super) Uuid);

impl From<EpisodeId> for ToSqlOutput<'_> {
    fn from(value: EpisodeId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ECommentId(Uuid);

impl From<ECommentId> for ToSqlOutput<'_> {
    fn from(value: ECommentId) -> Self {
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
    backdrop: Option<String>,
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
        let id = row.get::<_, String>("id")?;
        let id = ShowId(Uuid::try_parse(&id).unwrap());
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
        let recent_season = row
            .get::<_, Option<String>>("recent_season")?
            .map(|season| SeasonId(Uuid::try_parse(&season).unwrap()));

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
        let sql = "INSERT INTO tv_show (id, directory, path, name, original_name, poster, backdrop, tags, synopsis, release, created_at) VALUES (:id, :directory, :path, :name, :original_name, :poster, :backdrop, :tags, :synopsis, :release, :added)";

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

    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        directory: DirectoryId,
        path: String,
        name: String,
        original_name: String,
        backdrop: Option<String>,
        poster: Option<String>,
        tags: Vec<String>,
        synopsis: String,
        release: NaiveDate,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

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
            seasons: 0,
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

    pub fn dummy<'a>(directory: DirectoryId) -> (Self, Query<'a>) {
        let path = "dummy_show".to_owned();
        let name = "Testing dummy".to_owned();
        let backdrop = None;
        let poster = None;
        let tags = ["test", "hope", "growth"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        let synopsis = "A test dummy which is partially adequate".to_owned();
        let release = NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d").unwrap();

        Self::new(
            directory,
            path,
            name.clone(),
            name,
            backdrop,
            poster,
            tags,
            synopsis,
            release,
        )
    }

    pub fn testing() -> Self {
        let path = "dummy_show".to_owned();
        let name = "Testing dummy".to_owned();
        let backdrop = Some("assets/show2.png".into());
        let poster = Some("assets/show.png".into());
        let tags = vec!["Comedy", "Space", "Engineers"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        let  synopsis= "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned();
        let release = NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d").unwrap();

        Self::new(
            DirectoryId(Uuid::now_v7()),
            path,
            name.clone(),
            name,
            backdrop,
            poster,
            tags,
            synopsis,
            release,
        )
        .0
    }

    pub fn testing1() -> Self {
        let path = "dummy_show".to_owned();
        let name = "Testing dummy".to_owned();
        let backdrop = Some("assets/show3.png".into());
        let poster = Some("assets/show1.png".into());
        let tags = vec!["Drama", "Mystery", "Sci-Fi", "Fantasy"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        let synopsis= "When they were boys, Sam and Dean Winchester lost their mother to a mysterious and demonic supernatural force. Subsequently, their father raised them to be soldiers. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned();
        let release = NaiveDate::parse_from_str("2015-09-05", "%Y-%m-%d").unwrap();

        Self::new(
            DirectoryId(Uuid::now_v7()),
            path,
            name.clone(),
            name,
            backdrop,
            poster,
            tags,
            synopsis,
            release,
        )
        .0
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

    fn synopsis(&self) -> &str {
        &self.synopsis
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
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
    pub number: u16,
    progress: f32,
    rating: Option<f32>,
    last_watched: Option<DateTime<Local>>,
    pub recent_episode: Option<EpisodeId>,
    duration: u64,
    comments: u32,
}

impl Season {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        // let id = SeasonId(row.get::<_, Uuid>("id")?);
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let id = row.get::<_, String>("id")?;
        let id = SeasonId(Uuid::try_parse(&id).unwrap());

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
        let show = ShowId(Uuid::try_parse(&row.get::<_, String>("show_id")?).unwrap());

        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let recent_episode = row
            .get::<_, Option<String>>("recent_episode")?
            .map(|episode| EpisodeId(Uuid::try_parse(&episode).unwrap()));

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
            number,
            progress,
            rating,
            last_watched,
            recent_episode,
            duration,
            comments,
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
            number,
            progress,
            rating,
            last_watched,
            recent_episode,
            duration,
            comments,
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
            (":number", number),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO season (id, show_id, name, original_name, path, poster, synopsis,release, created_at, watch_count, episode_count, rating, progress, last_watched, recent_episode, duration, comment_count, season_number) VALUES (:id, :show, :name, :original_name, :path, :poster, :synopsis, :release, :added, :watch_count, :episodes, :rating, :progress, :last_watched, :recent_episode, :duration, :comments, :number)";

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

    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        show: &Show,
        name: String,
        original_name: String,
        path: String,
        poster: Option<String>,
        synopsis: String,
        release: NaiveDate,
        season_number: u16,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: SeasonId(Uuid::now_v7()),
            show: show.id,
            name,
            original_name,
            path,
            backdrop: show.backdrop.clone(),
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
            number: season_number,
        };

        let query = new.insert();

        (new, query)
    }

    pub fn dummy<'a>(show: &Show) -> (Self, Query<'a>) {
        let name = "Test Season".to_owned();
        let path = "Test Season 1".to_owned();
        let poster = None;
        let synopsis = "A dummy season for testing purposes only".to_owned();
        let release = NaiveDate::parse_from_str("2019-09-05", "%Y-%m-%d").unwrap();
        let season_number = 5;

        Self::new(
            show,
            name.clone(),
            name,
            path,
            poster,
            synopsis,
            release,
            season_number,
        )
    }

    pub fn testing() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;
        let release = NaiveDate::parse_from_str("2011-03-05", "%Y-%m-%d").unwrap();

        let added = Local::now();

        Self {
            show: ShowId(Uuid::now_v7()),
            path: "Dummy 1".into(),
            id: SeasonId(Uuid::now_v7()),
            name: "1 Season".into(),
            original_name: "1 Season".into(),
            number: 6,
            episodes: 20,
            recent_episode: Some(EpisodeId(Uuid::now_v7())),
            last_watched: None,
            duration,
            rating: Some(2.1),
            progress: 0.5,
            poster: Some("assets/show.png".into()),
            backdrop: Some("assets/show2.png".into()),
            release,
            added,
            comments: 20,
            watch_count: 28,
            synopsis: "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned(),
        }
    }

    pub fn testing2() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;
        let release = NaiveDate::parse_from_str("2021-03-05", "%Y-%m-%d").unwrap();

        let added = Local::now();

        let last_watched = DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z").unwrap();
        let last_watched = Some(last_watched.into());

        Self {
            show: ShowId(Uuid::now_v7()),
            recent_episode: None,
            path: "Someting".into(),
            id: SeasonId(Uuid::now_v7()),
            name: "2 Season".into(),
            original_name: "2 Season".into(),
            number: 1,
            episodes: 20,
            last_watched,
            duration,
            rating: Some(4.99),
            progress: 0.5,
            poster: Some("assets/show1.png".into()),
            backdrop: Some("assets/show3.png".into()),
            release,
            added,
            watch_count: 6,
            comments: 420,
            synopsis: "When they were boys, Sam. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned(),

        }
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

    fn synopsis(&self) -> &str {
        &self.synopsis
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: EpisodeId,
    name: String,
    original_name: String,
    path: String,
    pub full_path: PathBuf,
    poster: Option<String>,
    // Join from show
    backdrop: Option<String>,
    synopsis: String,
    release: NaiveDate,
    added: DateTime<Local>,
    watch_count: u32,
    pub show: ShowId,
    pub season: SeasonId,
    pub number: u16,
    progress: f32,
    rating: Option<f32>,
    last_watched: Option<DateTime<Local>>,
    duration: u64,
    comments: u32,
}

impl Episode {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        // let id = SeasonId(row.get::<_, Uuid>("id")?);
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        let id = row.get::<_, String>("id")?;
        let id = EpisodeId(Uuid::try_parse(&id).unwrap());

        let path = row.get::<_, String>("path")?;

        let name = row.get::<_, String>("name")?;
        let original_name = row.get::<_, String>("original_name")?;
        let poster = row.get::<_, Option<String>>("poster")?;

        let backdrop = row.get::<_, Option<String>>("backdrop")?;
        let synopsis = row.get::<_, String>("synopsis")?;

        let release = row.get::<_, NaiveDate>("release")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let watch_count = row.get::<_, u32>("watch_count")?;

        let number = row.get::<_, u16>("episode_number")?;

        let progress = row.get::<_, f32>("progress")?;

        let rating = row.get::<_, Option<f32>>("rating")?;

        let last_watched = row.get::<_, Option<DateTime<Local>>>("last_watched")?;

        // todo
        let show = ShowId(Uuid::try_parse(&row.get::<_, String>("show_id")?).unwrap());
        let season = SeasonId(Uuid::try_parse(&row.get::<_, String>("season_id")?).unwrap());

        let duration = row.get::<_, u64>("duration")?;

        let comments = row.get::<_, u32>("comment_count")?;

        let full_path: PathBuf = {
            let directory = row.get::<_, String>("directory_path")?;
            let show = row.get::<_, String>("show_path")?;
            let season = row.get::<_, String>("season_path")?;
            [&directory, &show, &season, &path].iter().collect()
        };

        Ok(Self {
            id,
            name,
            original_name,
            path,
            full_path,
            poster,
            backdrop,
            synopsis,
            release,
            added,
            watch_count,
            show,
            season,
            number,
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
            path,
            full_path: _full_path,
            poster,
            backdrop: _backdrop,
            synopsis,
            release,
            added,
            watch_count,
            show: _show,
            season,
            number,
            progress,
            rating,
            last_watched,
            duration,
            comments,
        } = self;

        let id = ToSqlOutput::from(*id);
        let season = ToSqlOutput::from(*season);
        let path = ToSqlOutput::from(path.clone());

        let name = ToSqlOutput::from(name.clone());
        let original_name = ToSqlOutput::from(original_name.clone());
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
            (":number", number),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO episode (id, season_id, name, original_name, path, poster, synopsis, release, created_at, watch_count, rating, progress, last_watched, duration, comment_count, episode_number) VALUES (:id, :season, :name, :original_name, :path, :poster, :synopsis, :release, :added, :watch_count, :rating, :progress, :last_watched, :duration, :comments, :number)";

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
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELTE FROM episode WHERE id=:id";
        let id = ToSqlOutput::from(self.id);
        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE episode SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_rating<'a>(&mut self, rating: f32) -> Query<'a> {
        assert!(rating > 0.0 && rating <= 5.0, "Episode rating out of range");
        self.rating = Some(rating);

        let sql = "UPDATE episode SET rating=:rating WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":rating", ToSqlOutput::from(rating)),
        ];

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_watch_count<'a>(&mut self, count: u32) -> Query<'a> {
        self.watch_count = count;

        let sql = "UPDATE episode SET watch_count=:count WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":count", ToSqlOutput::from(count)),
        ];

        Query {
            id: self.id.0,
            table: Table::Episode,
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

        let sql = "UPDATE episode SET progress=:progress WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":progress", ToSqlOutput::from(progress)),
        ];

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_last_watched<'a>(&mut self, watched: DateTime<Local>) -> Query<'a> {
        self.last_watched = Some(watched);

        let sql = "UPDATE episode SET last_watched=:watched WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":watched", datetime_to_sql(&watched)),
        ];

        Query {
            id: self.id.0,
            table: Table::Episode,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        season: &Season,
        name: String,
        original_name: String,
        path: String,
        full_path: PathBuf,
        poster: Option<String>,
        backdrop: Option<String>,
        synopsis: String,
        release: NaiveDate,
        duration: u64,
        episode_number: u16,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: EpisodeId(Uuid::now_v7()),
            name,
            original_name,
            path,
            full_path,
            season: season.id,
            show: season.show,
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
            number: episode_number,
        };

        let query = new.insert();

        (new, query)
    }

    pub fn dummy<'a>(season: &Season) -> (Self, Query<'a>) {
        let name = "Test Episode".to_owned();
        let path = "Test Episode.mkv".to_owned();
        let full_path = PathBuf::from("test_full_path.mkv");
        let poster = None;
        let backdrop = None;
        let synopsis = "A dummy episode for testing purposes only".to_owned();
        let release = NaiveDate::parse_from_str("2022-09-02", "%Y-%m-%d").unwrap();
        let episode_number = 12;
        let duration = 3872;

        Self::new(
            season,
            name.clone(),
            name,
            path,
            full_path,
            poster,
            backdrop,
            synopsis,
            release,
            duration,
            episode_number,
        )
    }

    pub fn testing() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;
        let added = Local::now();

        let release = NaiveDate::parse_from_str("2011-09-05", "%Y-%m-%d").unwrap();

        let path = "assets/test3.mp4";
        let full_path = PathBuf::from(path);

        Self {
            show: ShowId(Uuid::now_v7()),
            season: SeasonId(Uuid::now_v7()),
            last_watched: None,
            id: EpisodeId(Uuid::now_v7()),
            name: "Episode 33".into(),
            original_name: "The Big Bang Theory".into(),
            path: path.into(),
            full_path,
            number: 1,
            duration,
            rating: Some(3.0),
            progress: 0.5,
            poster: Some("assets/show.png".into()),
            backdrop: Some("assets/show2.png".into()),
            release,
            added,
            watch_count: 12,
            comments: 20,
            synopsis: "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned(),
        }
    }

    pub fn testing2() -> Self {
        let duration = (crate::utils::rand_u32() as usize) as u64;
        let release = NaiveDate::parse_from_str("2019-09-05", "%Y-%m-%d").unwrap();

        let added = Local::now();

        let path = "assets/test1.mp4";
        let full_path = PathBuf::from(path);
        let last_watched = DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z").unwrap();
        let last_watched = Some(last_watched.into());

        Self {
            show: ShowId(Uuid::now_v7()),
            id: EpisodeId(Uuid::now_v7()),
            season: SeasonId(Uuid::now_v7()),
            original_name: "Supernatural".into(),
            name: "4 Episode".into(),
            path: path.into(),
            last_watched,
            full_path,
            number:6,
            duration,
            rating: Some(2.0),
            progress: 0.5,
            poster: Some("assets/show1.png".into()),
            backdrop: Some("assets/show3.png".into()),
            release,
            watch_count: 0,
            added,
            comments: 420,
            synopsis: "When they were boys, Sam. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned(),

        }
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

    fn synopsis(&self) -> &str {
        &self.synopsis
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
    }
}

#[derive(Debug, Clone)]
pub struct EComment {
    pub id: ECommentId,
    pub added: DateTime<Local>,
    pub content: String,
    pub episode: EpisodeId,
    pub timestamp: Option<u64>,
}

impl EComment {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        //todo
        let id = row.get::<_, String>("id")?;
        let id = ECommentId(Uuid::try_parse(&id).unwrap());

        let episode = row.get::<_, String>("episode_id")?;
        let episode = EpisodeId(Uuid::try_parse(&episode).unwrap());

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let content = row.get::<_, String>("content")?;

        let timestamp = row.get::<_, Option<u64>>("episode_timestamp")?;

        Ok(Self {
            id,
            added,
            content,
            episode,
            timestamp,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            added,
            content,
            episode,
            timestamp,
        } = self;

        let id = ToSqlOutput::from(*id);
        let added = datetime_to_sql(added);
        let content = ToSqlOutput::from(content.clone());

        let episode = ToSqlOutput::from(*episode);
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
            (":episode", episode),
            (":timestamp", timestamp),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO episode_comment (id, created_at, content, episode_id, episode_timestamp) VALUES (:id, :added, :content, :episode, :timestamp)";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::EComment,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM episode_comment WHERE id=:id";
        let id = ToSqlOutput::from(self.id);
        let params = [(":id", id)];

        Query {
            id: self.id.0,
            table: Table::EComment,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_content<'a>(&mut self, content: String) -> Query<'a> {
        self.content = content.clone();

        let sql = "UPDATE episode_comment SET content=:content WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":content", ToSqlOutput::from(content)),
        ];

        Query {
            id: self.id.0,
            table: Table::EComment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_timestamp<'a>(&mut self, timestamp: Option<u64>) -> Query<'a> {
        self.timestamp = timestamp;

        let sql = "UPDATE episode_comment SET episode_timestamp=:timestamp WHERE id=:id";

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
            table: Table::EComment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        episode: EpisodeId,
        content: String,
        timestamp: Option<u64>,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: ECommentId(Uuid::now_v7()),
            added,
            content,
            timestamp,
            episode,
        };

        let query = new.insert();

        (new, query)
    }

    pub(super) fn dummy<'a>(episode: EpisodeId) -> (Self, Query<'a>) {
        let content = "A dummy episode comment".to_owned();
        let timestamp = None;

        Self::new(episode, content, timestamp)
    }
}
