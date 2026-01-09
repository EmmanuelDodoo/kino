use crate::models::{EpisodeId, ItemId, MovieId};

use rand::{seq::IteratorRandom, thread_rng};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayId {
    Movie(MovieId),
    Episode(EpisodeId),
}

impl From<PlayId> for ItemId {
    fn from(value: PlayId) -> Self {
        match value {
            PlayId::Movie(id) => ItemId::Movie(id),
            PlayId::Episode(id) => ItemId::Episode(id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayItem {
    pub id: PlayId,
    pub name: String,
    pub path: PathBuf,
    pub progress: f32,
    pub duration: u64,
    pub watch_count: u32,
    pub subtitle_uri: Option<PathBuf>,
}

impl PlayItem {
    pub fn from_episode(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let id = EpisodeId::from_row(row)?;
        let id = PlayId::Episode(id);

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
        let id = MovieId::from_row(row)?;
        let id = PlayId::Movie(id);
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
        id: PlayId,
        path: PathBuf,
        name: String,
    ) -> rusqlite::Result<Self> {
        let progress = row.get::<_, f32>("progress")?;
        let duration = row.get::<_, u64>("duration")?;
        let watch_count = row.get::<_, u32>("watch_count")?;
        let subtitle = row.get::<_, Option<String>>("subtitle_uri")?;
        let subtitle_uri = subtitle.map(PathBuf::from);

        Ok(Self {
            id,
            name,
            path,
            progress,
            duration,
            watch_count,
            subtitle_uri,
        })
    }

    pub fn progress(&mut self, progress: f32) {
        assert!((0.0..1.0).contains(&progress), "Progress out of bounds");
        self.progress = progress;
    }
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub shuffle: bool,
    pub repeat: bool,
    pub origins: Vec<ItemId>,
    current: usize,
    items: Vec<PlayItem>,
}

impl Playlist {
    pub fn empty() -> Self {
        Self {
            repeat: false,
            shuffle: false,
            current: 0,
            items: vec![],
            origins: vec![],
        }
    }

    pub fn new(items: impl Iterator<Item = PlayItem>, origin: ItemId) -> Self {
        Self {
            repeat: false,
            shuffle: false,
            current: 0,
            items: items.collect(),
            origins: vec![origin],
        }
    }

    pub fn single(item: PlayItem) -> Self {
        Self {
            repeat: false,
            shuffle: false,
            current: 0,
            origins: vec![item.id.into()],
            items: vec![item],
        }
    }

    pub fn merge(mut self, mut other: Self, flip: bool) -> Self {
        let total = self.items.len() + other.items.len();
        let current = if flip {
            (self.items.len() + other.current).min(total.saturating_sub(1))
        } else {
            self.current
        };

        self.items.append(&mut other.items);

        self.origins.append(&mut other.origins);

        Self {
            shuffle: self.shuffle && other.shuffle,
            repeat: self.repeat && other.repeat,
            current,
            items: self.items,
            origins: self.origins,
        }
    }

    pub fn position(&mut self, position: usize) {
        self.current = position.min(self.items.len().saturating_sub(1));
    }

    pub fn set_current(&mut self, current: usize) -> bool {
        if self.current == current {
            return false;
        }

        self.current = current.min(self.items.len().saturating_sub(1));
        true
    }

    pub fn update_current(&mut self, update: &PlayItem) {
        if let Some(old) = self.current_mut()
            && old.id == update.id
        {
            old.duration = update.duration;
            old.progress = update.progress;
            old.watch_count = update.watch_count;
        }
    }

    pub fn repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    pub fn shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
    }

    pub fn items(&self) -> impl Iterator<Item = (usize, &PlayItem, bool)> {
        self.items
            .iter()
            .enumerate()
            .map(|(idx, item)| (idx, item, idx == self.current))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&PlayItem> {
        if self.shuffle && !self.is_empty() {
            let mut rng = thread_rng();
            let new = (0..self.items.len())
                .choose(&mut rng)
                .expect("Playlist shuffle");

            self.current = new;
            return self.current();
        }

        self.current = (self.current + 1).min(self.items.len());

        if self.repeat && self.current == self.items.len() {
            self.current = 0
        }

        self.current()
    }

    pub fn next_peek(&self) -> Option<&PlayItem> {
        if self.repeat && self.current == self.len().saturating_sub(1) {
            return self.items.first();
        }

        self.items.get(self.current + 1)
    }

    pub fn current(&self) -> Option<&PlayItem> {
        self.items.get(self.current)
    }

    fn current_mut(&mut self) -> Option<&mut PlayItem> {
        self.items.get_mut(self.current)
    }

    pub fn previous(&mut self) -> Option<&PlayItem> {
        if self.shuffle && !self.is_empty() {
            let mut rng = thread_rng();
            let new = (0..self.items.len())
                .choose(&mut rng)
                .expect("Playlist shuffle");

            self.current = new;
            return self.current();
        }

        self.current = self.current.saturating_sub(1);

        self.current()
    }

    pub fn previous_peek(&self) -> Option<&PlayItem> {
        if self.current == 0 {
            return None;
        };

        self.items.get(self.current - 1)
    }

    pub fn restart(&mut self) {
        self.current = 0;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn has_next(&self) -> bool {
        self.current < self.items.len().saturating_sub(1) || (!self.is_empty() && self.repeat)
    }

    pub fn has_previous(&self) -> bool {
        self.current != 0
    }
}
