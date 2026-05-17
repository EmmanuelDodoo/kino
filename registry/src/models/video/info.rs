use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Result, Row};
use uuid::Uuid;

use super::{EpisodeId, MovieId, VideoId};
use crate::db::{Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VideoInfoId(Uuid);

impl From<VideoInfoId> for ToSqlOutput<'_> {
    fn from(value: VideoInfoId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

impl std::fmt::Display for VideoInfoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl VideoInfoId {
    /// Expects relevant column name as "id"
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        row.get::<_, String>("id")
            .map(|id| Self(Uuid::try_parse(&id).unwrap()))
    }

    pub(crate) fn from_row_maybe(column: &str, row: &Row<'_>) -> rusqlite::Result<Option<Self>> {
        row.get::<_, Option<String>>(column)
            .map(|id| id.map(|id| Self(Uuid::try_parse(&id).unwrap())))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub id: VideoInfoId,
    pub stream: u32,
    pub video: VideoId,
    pub tag: Option<String>,
    pub codec: Option<String>,
    pub bitrate: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub framerate: f32,
    pub interlaced: bool,

    /// Display Aspect Ratio
    pub dar_num: u32,
    pub dar_denom: u32,
}

impl VideoInfo {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = VideoInfoId::from_row(row)?;
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

        let tag = row.get::<_, Option<String>>("tag")?;
        let codec = row.get::<_, Option<String>>("codec")?;
        let bitrate = row.get::<_, u32>("bitrate")?;
        let width = row.get::<_, u32>("width")?;
        let height = row.get::<_, u32>("height")?;
        let depth = row.get::<_, u32>("depth")?;

        let framerate = row.get::<_, f32>("framerate")?;
        let interlaced = row.get::<_, bool>("interlaced")?;

        let dar_num = row.get::<_, u32>("dar_num")?;
        let dar_denom = row.get::<_, u32>("dar_denom")?;

        let stream = row.get::<_, u32>("stream")?;

        Ok(Self {
            id,
            stream,
            video,
            tag,
            codec,
            bitrate,
            width,
            height,
            depth,
            framerate,
            interlaced,
            dar_num,
            dar_denom,
        })
    }

    pub fn new(
        video: impl Into<VideoId>,
        tag: Option<String>,
        codec: Option<String>,
        bitrate: u32,
        width: u32,
        height: u32,
        depth: u32,
        framerate: f32,
        interlaced: bool,
        dar_num: u32,
        dar_denom: u32,
        stream: u32,
    ) -> Self {
        Self {
            id: VideoInfoId(Uuid::now_v7()),
            video: video.into(),
            tag,
            codec,
            bitrate,
            width,
            height,
            depth,
            framerate,
            interlaced,
            dar_num,
            dar_denom,
            stream,
        }
    }

    pub fn resolution(&self) -> String {
        match (self.width, self.height) {
            (7680, 4320) => "8K".to_owned(),
            (3840, 2160) => "4K".to_owned(),
            (4096, 2160) => "4K (DCI)".to_owned(),
            (2048, 1080) => "2K (DCI)".to_owned(),
            (2560, 1440) => "1440p (QHD)".to_owned(),
            (1920, 1080) => "1080p (FHD)".to_owned(),
            (1280, 720) => "720p (HD)".to_owned(),
            (854, 480) => "480p (SD)".to_owned(),
            (640, 360) => "360p".to_owned(),
            (426, 240) => "240p".to_owned(),
            (_, height) => format!("{height}p"),
        }
    }

    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO video (id, media, media_type, tag, codec, bitrate, width, height, depth, framerate, interlaced, dar_num, dar_denom, stream) VALUES (:id, :media, :media_type,  :tag, :codec, :bitrate, :width, :height, :depth, :framerate, :interlaced, :dar_num, :dar_denom, :stream) ON CONFLICT DO NOTHING";

        let Self {
            id,
            video,
            tag,
            codec,
            bitrate,
            width,
            height,
            depth,
            framerate,
            interlaced,
            dar_num,
            dar_denom,
            stream,
        } = &self;

        let media_type = video.name_str().to_owned();

        let codec = codec
            .as_ref()
            .map(|codec| ToSqlOutput::from(codec.clone()))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let tag = tag
            .as_ref()
            .map(|tag| ToSqlOutput::from(tag.clone()))
            .unwrap_or(ToSqlOutput::Owned(Value::Null));

        let params = vec![
            (":id", ToSqlOutput::from(*id)),
            (":media", ToSqlOutput::from(*video)),
            (":media_type", ToSqlOutput::from(media_type)),
            (":tag", tag),
            (":codec", codec),
            (":bitrate", ToSqlOutput::from(*bitrate)),
            (":width", ToSqlOutput::from(*width)),
            (":height", ToSqlOutput::from(*height)),
            (":depth", ToSqlOutput::from(*depth)),
            (":framerate", ToSqlOutput::from(*framerate)),
            (":interlaced", ToSqlOutput::from(*interlaced)),
            (":dar_num", ToSqlOutput::from(*dar_num)),
            (":dar_denom", ToSqlOutput::from(*dar_denom)),
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
