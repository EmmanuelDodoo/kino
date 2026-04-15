use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

use super::{EpisodeId, MovieId, VideoId};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AudioId(Uuid);

impl From<AudioId> for ToSqlOutput<'_> {
    fn from(value: AudioId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for AudioId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AudioId {
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
pub struct Audio {
    pub id: AudioId,
    pub stream: u32,
    pub video: VideoId,
    pub codec: Option<String>,
    pub lang: Option<String>,
    pub channels: u32,
    pub sample_rate: u32,
    pub bitrate: u32,
    pub depth: u32,
}

impl Audio {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = AudioId::from_row(row)?;
        let video = row.get::<_, String>("media_type")?;

        let video = match video.as_str() {
            "movie" => {
                let id = MovieId::from_row_helper("media", row)?;
                VideoId::from(id)
            }
            "episode" => {
                let id = EpisodeId::from_row_helper("media", row)?;
                VideoId::from(id)
            }
            _ => unreachable!("stored invalid audio media"),
        };

        let codec = row.get::<_, Option<String>>("codec")?;
        let lang = row.get::<_, Option<String>>("lang")?;

        let channels = row.get::<_, u32>("channels")?;
        let sample_rate = row.get::<_, u32>("sample_rate")?;
        let bitrate = row.get::<_, u32>("bitrate")?;
        let depth = row.get::<_, u32>("depth")?;

        let stream = row.get::<_, u32>("stream")?;

        Ok(Self {
            id,
            stream,
            video,
            codec,
            lang,
            channels,
            sample_rate,
            bitrate,
            depth,
        })
    }

    pub fn new(
        video: impl Into<VideoId>,
        codec: Option<String>,
        lang: Option<String>,
        channels: u32,
        sample_rate: u32,
        bitrate: u32,
        depth: u32,
        stream: u32,
    ) -> Self {
        Self {
            id: AudioId(Uuid::now_v7()),
            video: video.into(),
            codec,
            lang,
            channels,
            sample_rate,
            bitrate,
            depth,
            stream,
        }
    }

    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO audio (id, media, media_type, codec, lang, channels, sample_rate, bitrate, depth, stream) VALUES (:id, :media, :media_type,  :codec, :lang, :channels, :sample_rate, :bitrate, :depth, :stream) ON CONFLICT DO NOTHING";

        let Self {
            id,
            video,
            codec,
            lang,
            channels,
            sample_rate,
            bitrate,
            depth,
            stream,
        } = &self;

        let media_type = video.name_str().to_owned();

        let codec = codec
            .as_ref()
            .map(|codec| ToSqlOutput::from(codec.clone()))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let lang = lang
            .as_ref()
            .map(|lang| ToSqlOutput::from(lang.clone()))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let params = vec![
            (":id", ToSqlOutput::from(*id)),
            (":media", ToSqlOutput::from(*video)),
            (":media_type", ToSqlOutput::from(media_type)),
            (":codec", codec),
            (":lang", lang),
            (":channels", ToSqlOutput::from(*channels)),
            (":sample_rate", ToSqlOutput::from(*sample_rate)),
            (":bitrate", ToSqlOutput::from(*bitrate)),
            (":depth", ToSqlOutput::from(*depth)),
            (":stream", ToSqlOutput::from(*stream)),
        ];

        Query {
            id: self.id.0,
            table: Table::Subtitle,
            sql,
            params,
            op: Operation::Insert,
        }
    }
}
