use super::{CollectionId, datetime_to_sql};
use crate::db::{Database, Operation, Query, Table};
use crate::variants;
use chrono::{DateTime, Local, NaiveDate};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef};
use rusqlite::{Result, Row};
use uuid::Uuid;

variants! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Media {
        Movies,
        Shows,
        Seasons,
        Episodes,
    }
}

impl Media {
    pub const TAGS: &[Self] = &[Self::Movies, Self::Shows];

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "movie" => Some(Self::Movies),
            "show" => Some(Self::Shows),
            "season" => Some(Self::Seasons),
            "episode" => Some(Self::Episodes),
            _ => None,
        }
    }

    pub fn to_table(&self) -> &str {
        match self {
            Media::Shows => "tv_show",
            Media::Movies => "movie",
            Media::Seasons => "season",
            Media::Episodes => "episode",
        }
    }
}

impl FromSql for Media {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| Media::from_str(s).ok_or(FromSqlError::InvalidType))
    }
}

impl<'a> From<Media> for ToSqlOutput<'a> {
    fn from(value: Media) -> Self {
        ToSqlOutput::from(value.to_string())
    }
}

impl std::fmt::Display for Media {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Shows => "show",
                Self::Movies => "movie",
                Self::Seasons => "season",
                Self::Episodes => "episode",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InsertId(Uuid);

impl InsertId {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        Self::from_row_heler("id", row)
    }

    pub fn from_row_heler(column: &'static str, row: &Row<'_>) -> Result<Self> {
        row.get::<_, String>(column)
            .map(|id| InsertId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<InsertId> for ToSqlOutput<'_> {
    fn from(value: InsertId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

variants! {
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Comparison {
    Less,
    LE,
    #[default]
    Equal,
    NE,
    Greater,
    GE,
}
}

impl Comparison {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "<" => Some(Self::Less),
            "<=" => Some(Self::LE),
            "=" => Some(Self::Equal),
            ">" => Some(Self::Greater),
            ">=" => Some(Self::GE),
            "!=" => Some(Self::NE),
            _ => None,
        }
    }

    pub fn not(&self) -> Self {
        match self {
            Self::Less => Self::GE,
            Self::LE => Self::Greater,
            Self::Equal => Self::NE,
            Self::NE => Self::Equal,
            Self::Greater => Self::LE,
            Self::GE => Self::Less,
        }
    }
}

impl std::fmt::Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Less => "<",
                Self::LE => "<=",
                Self::Equal => "=",
                Self::NE => "!=",
                Self::Greater => ">",
                Self::GE => ">=",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Logic {
    pub name: Option<(bool, String)>,
    pub synopsis: Option<(bool, String)>,
    pub tags: Option<(bool, String)>,
    pub last_watched: Option<(Comparison, DateTime<Local>)>,
    pub duration: Option<(Comparison, u64)>,
    pub progress: Option<(Comparison, f32)>,
    pub watch_count: Option<(Comparison, u32)>,
    pub release: Option<(Comparison, NaiveDate)>,
    pub rating: Option<(Comparison, f32)>,
    pub comment: Option<(Comparison, u32)>,
}

impl Logic {
    const SEP: &str = ":::";

    fn is_some(&self) -> bool {
        let Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        } = self;

        name.is_some()
            || synopsis.is_some()
            || tags.is_some()
            || last_watched.is_some()
            || duration.is_some()
            || progress.is_some()
            || watch_count.is_some()
            || release.is_some()
            || rating.is_some()
            || comment.is_some()
    }

    fn query(&self, prefix: &str) -> Option<String> {
        if !self.is_some() {
            return Some(" TRUE ".to_owned());
        }

        let Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        } = self;

        let params = [
            name.as_ref().map(|(not, pattern)| {
                format!(
                    "{prefix}.name {}LIKE '%{pattern}%'",
                    if *not { "NOT " } else { "" }
                )
            }),
            synopsis.as_ref().map(|(not, pattern)| {
                format!(
                    "{prefix}.synopsis {}LIKE '%{pattern}%'",
                    if *not { "NOT " } else { "" }
                )
            }),
            tags.as_ref().map(|(not, pattern)| {
                format!(
                    "{prefix}.tags {}LIKE '%{pattern}%'",
                    if *not { "NOT " } else { "" }
                )
            }),
            last_watched.as_ref().map(|(comparison, date)| {
                format!(
                    "{prefix}.last_watched {comparison} '{}'",
                    date.with_timezone(&chrono::Utc)
                        .format("%F %T%.f%:z")
                        .to_string()
                )
            }),
            duration
                .as_ref()
                .map(|(comparison, duration)| format!("{prefix}.duration {comparison} {duration}")),
            progress.as_ref().map(|(comparison, progress)| {
                format!("{prefix}.progress {comparison} {progress:.2}")
            }),
            watch_count
                .as_ref()
                .map(|(comparison, count)| format!("{prefix}.watch_count {comparison} {count}")),
            release.as_ref().map(|(comparison, date)| {
                format!(
                    "{prefix}.release {comparison} '{}'",
                    date.format("%F").to_string()
                )
            }),
            rating
                .as_ref()
                .map(|(comparison, rating)| format!("{prefix}.rating {comparison} {rating:.2}")),
            comment.as_ref().map(|(comparison, comment)| {
                format!("{prefix}.comment_count {comparison} {comment}")
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if params.is_empty() {
            None
        } else {
            Some(params.join(" AND "))
        }
    }

    pub fn not(&self) -> Self {
        let Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        } = self;

        let name = name
            .as_ref()
            .map(|(comp, pattern)| (!*comp, pattern.clone()));
        let synopsis = synopsis
            .as_ref()
            .map(|(comp, pattern)| (!*comp, pattern.clone()));
        let tags = tags
            .as_ref()
            .map(|(comp, pattern)| (!*comp, pattern.clone()));

        let last_watched = last_watched
            .as_ref()
            .map(|(comp, date)| (comp.not(), date.clone()));

        let duration = duration.map(|(comp, date)| (comp.not(), date));

        let progress = progress.map(|(comp, progress)| (comp.not(), progress));

        let watch_count = watch_count.map(|(comp, count)| (comp.not(), count));

        let release = release.map(|(comp, release)| (comp.not(), release));

        let rating = rating.map(|(comp, rating)| (comp.not(), rating));

        let comment = comment.map(|(comp, comment)| (comp.not(), comment));

        Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        }
    }
}

impl std::fmt::Display for Logic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        } = self;

        let params = [
            name.as_ref()
                .map(|(comp, pattern)| format!("name-{comp}-{pattern}")),
            synopsis
                .as_ref()
                .map(|(comp, pattern)| format!("synopsis-{comp}-{pattern}")),
            tags.as_ref()
                .map(|(comp, pattern)| format!("tags-{comp}-{pattern}")),
            last_watched.as_ref().map(|(comparison, date)| {
                format!(
                    "last_watched-{comparison}-{}",
                    date.with_timezone(&chrono::Utc)
                        .format("%F %T%.f%:z")
                        .to_string()
                )
            }),
            duration
                .as_ref()
                .map(|(comparison, duration)| format!("duration-{comparison}-{duration}")),
            progress
                .as_ref()
                .map(|(comparison, progress)| format!("progress-{comparison}-{progress:.2}")),
            watch_count
                .as_ref()
                .map(|(comparison, count)| format!("watch_count-{comparison}-{count}")),
            release.as_ref().map(|(comparison, release)| {
                format!("release-{comparison}-{}", release.format("%F").to_string())
            }),
            rating
                .as_ref()
                .map(|(comparison, rating)| format!("rating-{comparison}-{rating:.2}")),
            comment
                .as_ref()
                .map(|(comparison, comment)| format!("comment-{comparison}-{comment}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        write!(f, "{}", params.join(Self::SEP))
    }
}

impl<'a> From<Logic> for ToSqlOutput<'a> {
    fn from(value: Logic) -> Self {
        if value.is_some() {
            ToSqlOutput::from(value.to_string())
        } else {
            ToSqlOutput::Owned(Value::Null)
        }
    }
}

impl FromSql for Logic {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        if matches!(value, ValueRef::Null) {
            return Ok(Logic::default());
        }

        let value = value.as_str()?;
        value
            .parse::<Logic>()
            .map_err(|_| FromSqlError::InvalidType)
    }
}

impl std::str::FromStr for Logic {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let params = s.split(Self::SEP);

        let mut name = None;
        let mut synopsis = None;
        let mut tags = None;
        let mut last_watched = None;
        let mut duration = None;
        let mut progress = None;
        let mut watch_count = None;
        let mut release = None;
        let mut rating = None;
        let mut comment = None;

        for param in params {
            let mut fields = param.split("-");
            let field = fields.next();

            match field {
                Some("name") => {
                    if name.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(|comp| comp.parse::<bool>().ok())
                    else {
                        return Err(());
                    };

                    let Some(pattern) = fields.next().map(ToOwned::to_owned) else {
                        return Err(());
                    };

                    name = Some((comparison, pattern));
                }
                Some("synopsis") => {
                    if synopsis.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(|comp| comp.parse::<bool>().ok())
                    else {
                        return Err(());
                    };

                    let Some(pattern) = fields.next().map(ToOwned::to_owned) else {
                        return Err(());
                    };

                    synopsis = Some((comparison, pattern));
                }
                Some("tags") => {
                    if tags.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(|comp| comp.parse::<bool>().ok())
                    else {
                        return Err(());
                    };

                    let Some(pattern) = fields.next().map(ToOwned::to_owned) else {
                        return Err(());
                    };

                    tags = Some((comparison, pattern));
                }
                Some("last_watched") => {
                    if last_watched.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(date) = fields
                        .next()
                        .and_then(|value| DateTime::parse_from_str(value, "%F %T%.f%:z").ok())
                    else {
                        return Err(());
                    };

                    last_watched = Some((comparison, date.into()));
                }
                Some("duration") => {
                    if duration.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields
                        .next()
                        .and_then(|duration| duration.parse::<u64>().ok())
                    else {
                        return Err(());
                    };

                    duration = Some((comparison, value))
                }
                Some("progress") => {
                    if progress.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields
                        .next()
                        .and_then(|progress| progress.parse::<f32>().ok())
                    else {
                        return Err(());
                    };

                    progress = Some((comparison, value))
                }
                Some("watch_count") => {
                    if watch_count.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields.next().and_then(|count| count.parse::<u32>().ok())
                    else {
                        return Err(());
                    };

                    watch_count = Some((comparison, value))
                }
                Some("release") => {
                    if release.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields
                        .next()
                        .and_then(|release| NaiveDate::parse_from_str(release, "%F").ok())
                    else {
                        return Err(());
                    };

                    release = Some((comparison, value))
                }
                Some("rating") => {
                    if rating.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields.next().and_then(|value| value.parse::<f32>().ok())
                    else {
                        return Err(());
                    };

                    rating = Some((comparison, value))
                }
                Some("comment") => {
                    if comment.is_some() {
                        return Err(());
                    }

                    let Some(comparison) = fields.next().and_then(Comparison::from_str) else {
                        return Err(());
                    };

                    let Some(value) = fields.next().and_then(|value| value.parse::<u32>().ok())
                    else {
                        return Err(());
                    };

                    comment = Some((comparison, value))
                }
                _ => return Err(()),
            }
        }

        Ok(Self {
            name,
            synopsis,
            tags,
            last_watched,
            duration,
            progress,
            watch_count,
            release,
            rating,
            comment,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertTrigger {
    pub id: InsertId,
    pub collection: CollectionId,
    pub name: String,
    trigger_name: String,
    created_at: DateTime<Local>,
    pub logic: Logic,
    pub media: Media,
}

impl InsertTrigger {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = InsertId::from_row(row)?;
        let collection = CollectionId::from_member(row)?;

        let name = row.get::<_, String>("name")?;
        let trigger_name = row.get::<_, String>("trigger_name")?;
        let logic = row.get::<_, Logic>("logic")?;

        let created_at = row.get::<_, DateTime<Local>>("created_at")?;

        let media = row.get::<_, Media>("media_type")?;

        Ok(Self {
            id,
            collection,
            name,
            trigger_name,
            created_at,
            logic,
            media,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            collection,
            name,
            trigger_name,
            created_at,
            logic,
            media,
        } = self;

        let id = ToSqlOutput::from(*id);
        let name = ToSqlOutput::from(name.clone());
        let trigger_name = ToSqlOutput::from(trigger_name.clone());
        let collection = ToSqlOutput::from(*collection);

        let logic = ToSqlOutput::from(logic.clone());
        let media = ToSqlOutput::from(*media);

        let created_at = datetime_to_sql(created_at);

        vec![
            (":id", id),
            (":name", name),
            (":collection", collection),
            (":trigger_name", trigger_name),
            (":logic", logic),
            (":media", media),
            (":created_at", created_at),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO collection_inserts (id, collection_id, name, trigger_name, created_at, logic, media_type) VALUES (:id, :collection, :name, :trigger_name, :created_at, :logic, :media) ON CONFLICT(id) DO UPDATE SET name=:name,  logic=:logic, media_type=:media";
        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::InsertTrigger,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove(self, db: &Database) -> Result<()> {
        let sql = format!(
            "
            DROP TRIGGER IF EXISTS '{}_insert';
            DROP TRIGGER IF EXISTS '{}_insert_update';
            DELETE FROM collection_inserts WHERE id='{}';
            ",
            self.trigger_name, self.trigger_name, self.id.0
        );

        db.execute_batch(&sql)
    }

    pub fn query(&self, prefix: &str) -> String {
        let logic = self
            .logic
            .query(prefix)
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();

        let table = self.media.to_table();

        format!(
            "INSERT INTO collection_item (collection_id, media_type, media_id) SELECT '{}', '{}', {prefix}.id FROM {table} {logic} ON CONFLICT(collection_id, media_type, media_id) DO UPDATE SET created_at=CURRENT_TIMESTAMP",
            self.collection.0, self.media,
        )
    }

    pub fn save(&self, db: &Database) -> Result<()> {
        if !self.logic.is_some() {
            return Ok(())
        }

        let query = self.query("NEW");
        let table = self.media.to_table();

        let insert = format!(
            "DROP TRIGGER IF EXISTS '{}_insert';
            CREATE TRIGGER '{}_insert' AFTER INSERT ON {} 
                BEGIN 
                    {query};
                END;
            ",
            self.trigger_name, self.trigger_name, table,
        );

        let update = format!(
            "DROP TRIGGER IF EXISTS '{}_insert_update';
            CREATE TRIGGER '{}_insert_update' AFTER UPDATE ON {} 
                BEGIN 
                    {query};
                END;
            ",
            self.trigger_name, self.trigger_name, table,
        );

        let sql = format!("{insert} {update}");

        db.execute_batch(&sql)
    }

    pub fn run_on_existing(&self, db: &mut Database) -> Result<()> {
        if !self.logic.is_some() {
            return Ok(())
        }
        let trans = db.transaction()?;

        let table = self.media.to_table();

        let sql = self.query(table);

        trans.execute(&sql, [])?;

        trans.commit()
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE collection_inserts SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::InsertTrigger,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        collection: CollectionId,
        name: impl Into<String>,
        logic: Logic,
        media: Media,
    ) -> Self {
        let id = InsertId(Uuid::now_v7());

        let trigger_name = format!("{}_{}", id.0, media);

        Self {
            id,
            collection,
            name: name.into(),
            trigger_name,
            created_at: Local::now(),
            media,
            logic,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeleteId(Uuid);

impl DeleteId {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        Self::from_row_heler("id", row)
    }

    pub fn from_row_heler(column: &'static str, row: &Row<'_>) -> Result<Self> {
        row.get::<_, String>(column)
            .map(|id| DeleteId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<DeleteId> for ToSqlOutput<'_> {
    fn from(value: DeleteId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteTrigger {
    pub id: DeleteId,
    pub collection: CollectionId,
    trigger_name: String,
    pub name: String,
    created_at: DateTime<Local>,
    pub logic: Logic,
    pub media: Media,
}

impl DeleteTrigger {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = DeleteId::from_row(row)?;
        let collection = CollectionId::from_member(row)?;

        let name = row.get::<_, String>("name")?;
        let trigger_name = row.get::<_, String>("trigger_name")?;
        let logic = row.get::<_, Logic>("logic")?;

        let created_at = row.get::<_, DateTime<Local>>("created_at")?;

        let media = row.get::<_, Media>("media_type")?;

        Ok(Self {
            id,
            collection,
            name,
            trigger_name,
            created_at,
            logic,
            media,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            collection,
            name,
            trigger_name,
            created_at,
            logic,
            media,
        } = self;

        let id = ToSqlOutput::from(*id);
        let name = ToSqlOutput::from(name.clone());
        let trigger_name = ToSqlOutput::from(trigger_name.clone());
        let collection = ToSqlOutput::from(*collection);

        let logic = ToSqlOutput::from(logic.clone());
        let media = ToSqlOutput::from(*media);

        let created_at = datetime_to_sql(created_at);

        vec![
            (":id", id),
            (":name", name),
            (":collection", collection),
            (":trigger_name", trigger_name),
            (":logic", logic),
            (":media", media),
            (":created_at", created_at),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO collection_deletes (id, collection_id, name, trigger_name, created_at, logic, media_type) VALUES (:id, :collection, :name, :trigger_name, :created_at, :logic, :media) ON CONFLICT(id) DO UPDATE SET name=:name, logic=:logic, media_type=:media";
        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::DeleteTrigger,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove(self, db: &Database) -> Result<()> {
        let sql = format!(
            "
            DROP TRIGGER IF EXISTS '{}_delete';
            DELETE FROM collection_deletes WHERE id='{}';
            ",
            self.trigger_name, self.id.0
        );

        db.execute_batch(&sql)
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE collection_deletes SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::DeleteTrigger,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    fn query(&self) -> String {
        let table = self.media.to_table();

        let logic = self
            .logic
            .query(table)
            .map(|query| format!(" AND {query}"))
            .unwrap_or_default();

        format!(
            "
                DELETE FROM collection_item
                WHERE collection_item.collection_id = '{}' AND collection_item.media_type = '{}'
                AND EXISTS (
                    SELECT 1 FROM {table}
                    WHERE {table}.id = collection_item.media_id {logic}
                )
            ",
            self.collection.0, self.media,
        )
    }

    pub fn save(&self, db: &Database) -> Result<()> {
        if !self.logic.is_some() {
            return Ok(())
        }
        let table = self.media.to_table();

        let query = self.query();

        let sql = format!(
            "DROP TRIGGER IF EXISTS '{}_delete';
            CREATE TRIGGER '{}_delete' AFTER UPDATE ON {table}
            BEGIN
                {query};
            END;
            ",
            self.trigger_name, self.trigger_name,
        );

        db.execute_batch(&sql)
    }

    pub fn run_on_existing(&self, db: &mut Database) -> Result<()> {
        if !self.logic.is_some() {
            return Ok(())
        }
        let trans = db.transaction()?;

        let sql = self.query();

        trans.execute(&sql, [])?;

        trans.commit()
    }

    pub fn new<'a>(
        collection: CollectionId,
        name: impl Into<String>,
        logic: Logic,
        media: Media,
    ) -> Self {
        let id = DeleteId(Uuid::now_v7());

        let trigger_name = format!("{}_{}", id.0, media);

        Self {
            id,
            collection,
            name: name.into(),
            trigger_name,
            created_at: Local::now(),
            media,
            logic,
        }
    }
}
