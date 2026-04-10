use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

use super::{EpisodeId, MovieId, VideoId};
use crate::db::{Operation, Query, Table};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubtitleId(Uuid);

impl From<SubtitleId> for ToSqlOutput<'_> {
    fn from(value: SubtitleId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for SubtitleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SubtitleId {
    /// Expects relevant column name as "id"
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>("id")
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }

    pub(super) fn from_row_maybe(column: &str, row: &Row<'_>) -> rusqlite::Result<Option<Self>> {
        row.get::<_, Option<String>>(column)
            .map(|id| id.map(|id| Self(Uuid::try_parse(&id).unwrap())))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubtitleKind {
    Embedded,
    Loaded { path: PathBuf, removed: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subtitle {
    pub id: SubtitleId,
    pub video: VideoId,
    pub kind: SubtitleKind,
    pub title: String,
    pub lang: String,
    pub offset: f32,
}

impl Subtitle {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = SubtitleId::from_row(row)?;
        let video = row.get::<_, String>("media_type")?;

        let video = match video.as_str() {
            "movie" => {
                let id = MovieId::from_row_helper("video", row)?;
                VideoId::from(id)
            }
            "episode" => {
                let id = EpisodeId::from_row_helper("video", row)?;
                VideoId::from(id)
            }
            _ => unreachable!("stored invalid subtitle media"),
        };

        let kind = row.get::<_, String>("kind")?;
        let kind = match kind.as_str() {
            "embedded" => SubtitleKind::Embedded,
            "loaded" => {
                let path = row.get::<_, String>("path")?;
                let removed = row.get::<_, bool>("removed")?;

                SubtitleKind::Loaded {
                    path: PathBuf::from(path),
                    removed,
                }
            }
            _ => unreachable!("stored invalid subtitle kind"),
        };

        let title = row.get::<_, String>("title")?;
        let lang = row.get::<_, String>("lang")?;

        let offset = row.get::<_, f32>("sub_offset")?;

        Ok(Self {
            id,
            video,
            kind,
            title,
            lang,
            offset,
        })
    }

    fn new(
        video: impl Into<VideoId>,
        kind: SubtitleKind,
        title: impl Into<String>,
        lang: impl Into<String>,
    ) -> Self {
        Self {
            id: SubtitleId(Uuid::now_v7()),
            kind,
            video: video.into(),
            title: title.into(),
            lang: lang.into(),
            offset: 0.0,
        }
    }

    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO subtitle (id, video, media_type, kind, path, title, lang, removed, sub_offset) VALUES (:id, :video, :media_type, :kind, :path, :title, :lang, :removed, :sub_offset) ON CONFLICT(id) DO UPDATE SET sub_offset=:sub_offset";

        let Self {
            id,
            video,
            kind,
            title,
            lang,
            offset,
        } = &self;

        let media_type = video.name_str().to_owned();

        let (kind, path, removed) = match kind {
            SubtitleKind::Embedded => (
                ToSqlOutput::from("embedded".to_owned()),
                ToSqlOutput::Owned(Value::Null),
                ToSqlOutput::from(false),
            ),
            SubtitleKind::Loaded { path, removed } => {
                let path = path.display().to_string();
                (
                    ToSqlOutput::from("loaded".to_owned()),
                    ToSqlOutput::from(path),
                    ToSqlOutput::from(*removed),
                )
            }
        };

        let params = vec![
            (":id", ToSqlOutput::from(*id)),
            (":video", ToSqlOutput::from(*video)),
            (":media_type", ToSqlOutput::from(media_type)),
            (":kind", kind),
            (":path", path),
            (":removed", removed),
            (":title", ToSqlOutput::from(title.clone())),
            (":lang", ToSqlOutput::from(lang.clone())),
            (":sub_offset", ToSqlOutput::from(*offset)),
        ];

        Query {
            id: self.id.0,
            table: Table::Subtitle,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    pub fn new_loaded(video: impl Into<VideoId>, path: impl Into<PathBuf>) -> Self {
        Self::new(
            video,
            SubtitleKind::Loaded {
                path: path.into(),
                removed: false,
            },
            "unknown",
            "unknown",
        )
    }

    pub fn new_embedded(
        video: impl Into<VideoId>,
        title: impl Into<String>,
        lang: impl Into<String>,
    ) -> Self {
        Self::new(video, SubtitleKind::Embedded, title, lang)
    }
}
