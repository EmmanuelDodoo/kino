use rand::{seq::IteratorRandom, thread_rng};
use registry::models::{ItemId, Video};

#[derive(Debug, Clone)]
pub struct Playlist {
    pub shuffle: bool,
    pub repeat: bool,
    pub origins: Vec<ItemId>,
    current: usize,
    items: Vec<Video>,
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

    pub fn new(items: impl Iterator<Item = Video>, origin: ItemId) -> Self {
        Self {
            repeat: false,
            shuffle: false,
            current: 0,
            items: items.collect(),
            origins: vec![origin],
        }
    }

    pub fn single(item: Video) -> Self {
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

    pub fn update_current(&mut self, update: &Video) {
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

    pub fn items(&self) -> impl Iterator<Item = (usize, &Video, bool)> {
        self.items
            .iter()
            .enumerate()
            .map(|(idx, item)| (idx, item, idx == self.current))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&Video> {
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

    pub fn next_peek(&self) -> Option<&Video> {
        if self.repeat && self.current == self.len().saturating_sub(1) {
            return self.items.first();
        }

        self.items.get(self.current + 1)
    }

    pub fn current(&self) -> Option<&Video> {
        self.items.get(self.current)
    }

    fn current_mut(&mut self) -> Option<&mut Video> {
        self.items.get_mut(self.current)
    }

    pub fn previous(&mut self) -> Option<&Video> {
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

    pub fn previous_peek(&self) -> Option<&Video> {
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
