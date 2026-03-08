use super::{EpisodeId, ItemId, MovieId};
use rusqlite::Row;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoId {
    Movie(MovieId),
    Episode(EpisodeId),
}

impl VideoId {
    pub fn from_episode(row: &Row<'_>) -> rusqlite::Result<Self> {
        EpisodeId::from_row(row).map(Self::Episode)
    }

    pub fn from_movie(row: &Row<'_>) -> rusqlite::Result<Self> {
        MovieId::from_row(row).map(Self::Movie)
    }

    pub fn from_comment(row: &Row<'_>) -> rusqlite::Result<Self> {
        let kind = row.get::<_, String>("media_type")?;

        match kind.as_str() {
            "movie" => MovieId::from_collection(row).map(Self::Movie),
            "episode" => EpisodeId::from_collection(row).map(Self::Episode),
            _ => unreachable!("stored invalid comment media"),
        }
    }

    pub fn name_str(&self) -> &'static str {
        match self {
            Self::Movie(_) => "movie",
            Self::Episode(_) => "episode",
        }
    }
}

impl From<VideoId> for ItemId {
    fn from(value: VideoId) -> Self {
        match value {
            VideoId::Movie(id) => ItemId::Movie(id),
            VideoId::Episode(id) => ItemId::Episode(id),
        }
    }
}

impl std::fmt::Display for VideoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Movie(id) => id.fmt(f),
            Self::Episode(id) => id.fmt(f),
        }
    }
}

impl From<VideoId> for rusqlite::types::ToSqlOutput<'_> {
    fn from(value: VideoId) -> Self {
        match value {
            VideoId::Movie(id) => id.into(),
            VideoId::Episode(id) => id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Video {
    pub id: VideoId,
    pub name: String,
    pub path: PathBuf,
    pub progress: f32,
    pub duration: u64,
    pub watch_count: u32,
    pub subtitle_uri: Option<PathBuf>,
    pub generate_poster: bool,
}

impl Video {
    pub fn from_episode(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let id = VideoId::from_episode(row)?;

        let full_path: PathBuf = {
            let path = row.get::<_, String>("path")?;
            let directory = row.get::<_, String>("directory_path")?;
            let show = row.get::<_, String>("show_path")?;
            let season = row.get::<_, String>("season_path")?;
            [&directory, &show, &season, &path].iter().collect()
        };

        let name = {
            let fetched = row.get::<_, bool>("fetched")?;
            let show = row.get::<_, String>("show_name")?;
            let season = row.get::<_, u16>("season_number")?;

            if fetched {
                let name = row.get::<_, String>("name")?;
                format!("{show} - S{season:02}E{name}")
            } else {
                let number = row.get::<_, u16>("episode_number")?;
                format!("{show} - S{season:02}E{number:02}")
            }
        };

        Self::new(row, id, full_path, name)
    }

    pub fn from_movie(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let id = VideoId::from_movie(row)?;
        let name = row.get::<_, String>("name")?;

        let full_path: PathBuf = {
            let path = row.get::<_, String>("path")?;
            let directory = row.get::<_, String>("directory_path")?;
            [&directory, &path].iter().collect()
        };

        Self::new(row, id, full_path, name)
    }

    fn new(
        row: &rusqlite::Row<'_>,
        id: VideoId,
        path: PathBuf,
        name: String,
    ) -> rusqlite::Result<Self> {
        let progress = row.get::<_, f32>("progress")?;
        let duration = row.get::<_, u64>("duration")?;
        let watch_count = row.get::<_, u32>("watch_count")?;
        let subtitle = row.get::<_, Option<String>>("subtitle_uri")?;
        let subtitle_uri = subtitle.map(PathBuf::from);
        let generate_poster = row.get::<_, bool>("generate_poster")?;
        let fetched = row.get::<_, bool>("fetched")?;

        Ok(Self {
            id,
            name,
            path,
            progress,
            duration,
            watch_count,
            subtitle_uri,
            generate_poster: generate_poster && !fetched,
        })
    }

    pub fn progress(&mut self, progress: f32) {
        assert!((0.0..1.0).contains(&progress), "Progress out of bounds");
        self.progress = progress;
    }
}
