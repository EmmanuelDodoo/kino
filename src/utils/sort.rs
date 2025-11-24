use super::H7;
use crate::models::Media;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sort {
    count: u8,
    sorts: [Option<u8>; SORTS],
}

impl Sort {
    pub fn new() -> Self {
        Self {
            count: 0,
            sorts: [None; SORTS],
        }
    }

    pub fn new_with_name() -> Self {
        let mut new = Self::new();
        new.push(SortKind::Name);

        new
    }

    pub fn release() -> Self {
        let mut new = Self::new();
        new.push(SortKind::Release);

        new
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.sorts = [None; SORTS];
    }

    pub fn query(&self, prefix: Option<&str>) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let prefix = prefix
            .map(|prefix| format!("{prefix}."))
            .unwrap_or_default();

        let column = |kind: SortKind| {
            let name = match kind {
                SortKind::Name => "name",
                SortKind::Watch => "watch_count",
                SortKind::Added => "created_at",
                SortKind::Rating => "rating",
                SortKind::Recent => "last_watched",
                SortKind::Release => "release",
                SortKind::Duration => "duration",
                SortKind::Progress => "progress",
                SortKind::Comments => "comment_count",
            };

            format!("{prefix}{name}")
        };

        let sorts = self.prepare().map(column).collect::<Vec<_>>();

        Some(sorts.join(", "))
    }

    pub fn sort<T: Media>(&self, x: &T, y: &T) -> std::cmp::Ordering {
        let sorts = self.prepare();

        for kind in sorts {
            let ord = kind.cmp(x, y);

            if !matches!(ord, std::cmp::Ordering::Equal) {
                return ord;
            }
        }

        std::cmp::Ordering::Equal
    }

    fn prepare(&self) -> impl Iterator<Item = SortKind> {
        let mut sorts = self
            .sorts
            .into_iter()
            .enumerate()
            .filter_map(|(idx, position)| {
                position.map(|position| (SortKind::from_usize(idx), position))
            })
            .collect::<Vec<_>>();

        sorts.sort_by(|(_, x), (_, y)| x.cmp(y));

        sorts.into_iter().map(|(kind, _)| kind)
    }

    pub fn push(&mut self, kind: SortKind) {
        self.sorts[kind as usize] = Some(self.count);
        self.count += 1;
    }

    pub fn remove(&mut self, kind: SortKind) {
        let Some(old) = self.sorts[kind as usize].take() else {
            return;
        };

        self.count = self.count.saturating_sub(1);
        for kind in &mut self.sorts {
            let Some(idx) = kind else {
                continue;
            };

            if *idx > old {
                *idx -= 1
            }
        }
    }

    pub fn reverse(&mut self) {
        let max = self.count.saturating_sub(1);

        for kind in &mut self.sorts {
            let Some(idx) = kind else { continue };

            *idx = max - *idx;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn position(&self, kind: SortKind) -> Option<usize> {
        self.sorts[kind as usize].map(|pos| pos as usize)
    }
}

impl Default for Sort {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortKind {
    Name = 0,
    Duration = 1,
    Progress = 2,
    Rating = 3,
    Watch = 4,
    Release = 5,
    Comments = 6,
    Added = 7,
    Recent = 8,
    // Tags,
}

const SORTS: usize = 9;

impl SortKind {
    pub const VISIBLE: [SortKind; 8] = [
        Self::Name,
        Self::Duration,
        Self::Progress,
        Self::Rating,
        Self::Added,
        Self::Recent,
        Self::Watch,
        Self::Release,
    ];

    pub const HIDDEN: [SortKind; 0] = [];

    pub fn view(&self, order: Option<usize>) -> String {
        let order = order
            .map(|order| (order + 1).to_string())
            .unwrap_or_default();

        format!("{self} {}", order)
    }

    pub fn cmp<T: Media>(&self, x: &T, y: &T) -> std::cmp::Ordering {
        match self {
            Self::Name => {
                alphanumeric_sort::compare_str(x.name().to_lowercase(), y.name().to_lowercase())
            }
            Self::Duration => x.duration().cmp(&y.duration()),
            Self::Added => x.added().cmp(&y.added()),
            Self::Rating => match (x.rating(), y.rating()) {
                (Some(x), Some(y)) => x.total_cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
            Self::Recent => x.recent().cmp(&y.recent()),
            Self::Release => x.release().cmp(&y.release()),
            Self::Progress => x.progress().total_cmp(&y.progress()),
            Self::Comments => x.comments().cmp(&y.comments()),
            Self::Watch => x.watch_count().cmp(&y.watch_count()),
        }
    }

    fn from_usize(idx: usize) -> Self {
        match idx {
            0 => Self::Name,
            1 => Self::Duration,
            2 => Self::Progress,
            3 => Self::Rating,
            4 => Self::Watch,
            5 => Self::Release,
            6 => Self::Comments,
            7 => Self::Added,
            8 => Self::Recent,
            _ => unreachable!("Sort Kind invalid idx"),
        }
    }
}

impl std::fmt::Display for SortKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name => "Name",
                Self::Duration => "Duration",
                Self::Progress => "Progress",
                Self::Rating => "Rating",
                // Self::Tags => "Tags",
                Self::Release => "Release",
                Self::Comments => "Comments",
                Self::Added => "Date Added",
                Self::Recent => "Recent",
                Self::Watch => "Watch Count",
            }
        )
    }
}

pub mod comments {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Kind {
        Added = 0,
        Episode = 1,
        Timestamp = 2,
    }

    const KINDS: usize = 3;

    impl Kind {
        fn from_usize(idx: usize) -> Self {
            match idx {
                0 => Self::Added,
                1 => Self::Episode,
                2 => Self::Timestamp,
                _ => unreachable!("Invalid Comment sort idx"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Sort {
        count: u8,
        sorts: [Option<u8>; KINDS],
    }

    impl Default for Sort {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Sort {
        pub fn new() -> Self {
            Self {
                count: 0,
                sorts: [None; KINDS],
            }
        }

        pub fn query(&self, prefix: Option<&str>) -> Option<String> {
            if self.is_empty() {
                return None;
            }

            let prefix = prefix
                .map(|prefix| format!("{prefix}."))
                .unwrap_or_default();

            let column = |kind: Kind| {
                let name = match kind {
                    Kind::Added => "created_at",
                    Kind::Timestamp => "episode_timestamp",
                    Kind::Episode => "episode_id",
                };

                format!("{prefix}{name}")
            };

            let sort = self.prepare().map(column).collect::<Vec<_>>();

            Some(sort.join(", "))
        }

        fn prepare(&self) -> impl Iterator<Item = Kind> {
            let mut sorts = self
                .sorts
                .into_iter()
                .enumerate()
                .filter_map(|(idx, position)| {
                    position.map(|position| (Kind::from_usize(idx), position))
                })
                .collect::<Vec<_>>();

            sorts.sort_by(|(_, x), (_, y)| x.cmp(y));

            sorts.into_iter().map(|(kind, _)| kind)
        }

        pub fn clear(&mut self) {
            self.count = 0;
            self.sorts = [None; KINDS];
        }

        pub fn push(&mut self, kind: Kind) {
            self.sorts[kind as usize] = Some(self.count);
            self.count += 1;
        }

        pub fn remove(&mut self, kind: Kind) {
            let Some(old) = self.sorts[kind as usize].take() else {
                return;
            };

            self.count = self.count.saturating_sub(1);
            for kind in &mut self.sorts {
                let Some(idx) = kind else {
                    continue;
                };

                if *idx > old {
                    *idx -= 1
                }
            }
        }

        pub fn reverse(&mut self) {
            let max = self.count.saturating_sub(1);

            for kind in &mut self.sorts {
                let Some(idx) = kind else { continue };

                *idx = max - *idx;
            }
        }

        pub fn is_empty(&self) -> bool {
            self.count == 0
        }

        pub fn position(&self, kind: Kind) -> Option<usize> {
            self.sorts[kind as usize].map(|pos| pos as usize)
        }
    }
}
