use super::{VideoId, datetime_to_sql};
use crate::db::{Operation, Query, Table};
use chrono::{DateTime, Local};
use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommentId(Uuid);

impl CommentId {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        row.get::<_, String>("id")
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<CommentId> for ToSqlOutput<'_> {
    fn from(value: CommentId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for CommentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: CommentId,
    pub content: String,
    added: DateTime<Local>,
    pub kind: VideoId,
    pub timestamp: Option<u64>,
    pub removed: bool,
}

impl Comment {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = CommentId::from_row(row)?;
        let content = row.get::<_, String>("content")?;
        let kind = VideoId::from_comment(row)?;

        let timestamp = row.get::<_, Option<u64>>("timestamp")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let removed = row.get::<_, bool>("removed")?;

        Ok(Self {
            id,
            content,
            added,
            kind,
            timestamp,
            removed,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            content,
            added,
            kind,
            timestamp,
            removed,
        } = self;

        let id = ToSqlOutput::from(*id);
        let content = ToSqlOutput::from(content.clone());

        let added = datetime_to_sql(added);

        let timestamp = timestamp.map(|timestamp| timestamp as i64);
        let timestamp = ToSqlOutput::Owned(Value::from(timestamp));
        let removed = ToSqlOutput::from(*removed);

        let (kind, media) = {
            (
                ToSqlOutput::from(kind.name_str().to_owned()),
                ToSqlOutput::from(*kind),
            )
        };

        vec![
            (":id", id),
            (":content", content),
            (":created_at", added),
            (":media_id", media),
            (":media_type", kind),
            (":timestamp", timestamp),
            (":removed", removed),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO comment (id, content, created_at, media_type, media_id, timestamp, removed) VALUES (:id, :content, :created_at, :media_type, :media_id, :timestamp, :removed) ON CONFLICT(id) DO UPDATE SET content=:content, timestamp=:timestamp, removed=:removed";

        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Comment,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove<'a>(id: CommentId) -> Query<'a> {
        let sql = "UPDATE comment SET removed=TRUE WHERE id=:id";
        let params = [(":id", ToSqlOutput::from(id))];

        Query {
            id: id.0,
            table: Table::Comment,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_content<'a>(id: CommentId, content: String) -> Query<'a> {
        let sql = "UPDATE comment SET content=:content WHERE id=:id";

        let params = [
            (":id", ToSqlOutput::from(id)),
            (":content", ToSqlOutput::from(content)),
        ];

        Query {
            id: id.0,
            table: Table::Comment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_timestamp<'a>(id: CommentId, timestamp: Option<u64>) -> Query<'a> {
        let sql = "UPDATE comment SET timestamp=:timestamp WHERE id=:id";
        let timestamp = timestamp.map(|timestamp| timestamp as i64);
        let timestamp = ToSqlOutput::Owned(Value::from(timestamp));

        let params = [
            (":id", ToSqlOutput::from(id)),
            (":timestamp", ToSqlOutput::from(timestamp)),
        ];

        Query {
            id: id.0,
            table: Table::Comment,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(content: String, timestamp: Option<u64>, kind: VideoId) -> (Self, Query<'a>) {
        let added = Local::now();
        let id = CommentId(Uuid::now_v7());

        let new = Self {
            id,
            content,
            timestamp,
            kind,
            added,
            removed: false,
        };

        let query = new.insert();

        (new, query)
    }
}
