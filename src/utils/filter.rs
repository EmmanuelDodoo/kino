use chrono::Datelike;

use crate::models::Media;
pub use search::SearchFilter;
use std::fmt::{self, Display};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FilterMode {
    #[default]
    And,
    Or,
}

impl FilterMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::And => Self::Or,
            Self::Or => Self::And,
        }
    }

    /// Compares two conditions using self.
    pub fn compare(&self, x: bool, y: bool) -> bool {
        match self {
            Self::Or => x | y,
            Self::And => x && y,
        }
    }

    /// Like `compare` but with multiple values
    pub fn compare_many(&self, conditions: &[bool]) -> bool {
        let init = matches!(self, Self::And);

        conditions
            .iter()
            .fold(init, |acc, curr| self.compare(acc, *curr))
    }
}

impl Display for FilterMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Or => "OR",
                Self::And => "AND",
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Comp {
    Less,
    #[default]
    Equal,
    Greater,
}

impl Comp {
    #[allow(dead_code)]
    pub const ALL: [Self; 3] = [Self::Less, Self::Equal, Self::Greater];

    pub fn icon(&self) -> char {
        use super::icons::{CHEV_LEFT, CHEV_RIGHT, EQUALS};
        match self {
            Self::Equal => EQUALS,
            Self::Greater => CHEV_LEFT,
            Self::Less => CHEV_RIGHT,
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::Less => Self::Equal,
            Self::Equal => Self::Greater,
            Self::Greater => Self::Less,
        }
    }

    pub fn compare<T: PartialEq + PartialOrd>(&self, x: T, y: T) -> bool {
        match self {
            Self::Less => x < y,
            Self::Equal => x == y,
            Self::Greater => x > y,
        }
    }
}

impl Display for Comp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Less => "<",
                Self::Greater => ">",
                Self::Equal => "=",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ProgressKind {
    #[default]
    Any,
    Zero,
    TwentyFive,
    Fifty,
    SeventyFive,
    Complete,
}

impl ProgressKind {
    pub const ALL: [Self; 6] = [
        Self::Any,
        Self::Zero,
        Self::TwentyFive,
        Self::Fifty,
        Self::SeventyFive,
        Self::Complete,
    ];
}

impl Display for ProgressKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Any => "Any",
                Self::Zero => "0%",
                Self::TwentyFive => "25%",
                Self::Fifty => "50%",
                Self::SeventyFive => "75%",
                Self::Complete => "100%",
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Progress {
    pub kind: ProgressKind,
    pub comp: Comp,
}

impl Progress {
    pub fn is_any(&self) -> bool {
        matches!(self.kind, ProgressKind::Any)
    }

    pub fn compare(&self, value: f32) -> bool {
        let Some(comp) = self.f32() else {
            return true;
        };

        self.comp.compare(value, comp)
    }

    pub fn f32(&self) -> Option<f32> {
        use ProgressKind::*;

        match self.kind {
            Any => None,
            Zero => Some(0.0),
            TwentyFive => Some(0.25),
            Fifty => Some(0.5),
            SeventyFive => Some(0.75),
            Complete => Some(1.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RatingKind {
    #[default]
    Any,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl RatingKind {
    pub const ALL: [Self; 6] = [
        Self::Any,
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
    ];
}

impl Display for RatingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Any => "Any".to_string(),
                Self::One => 1.to_string(),
                Self::Two => 2.to_string(),
                Self::Three => 3.to_string(),
                Self::Four => 4.to_string(),
                Self::Five => 5.to_string(),
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rating {
    pub kind: RatingKind,
    pub comp: Comp,
}

impl Rating {
    pub fn is_any(&self) -> bool {
        matches!(self.kind, RatingKind::Any)
    }

    pub fn compare(&self, value: f32) -> bool {
        let Some(comp) = self.u8() else { return true };

        self.comp.compare(value, comp as f32)
    }

    pub fn u8(&self) -> Option<u8> {
        use RatingKind::*;

        match self.kind {
            Any => None,
            One => Some(1),
            Two => Some(2),
            Three => Some(3),
            Four => Some(4),
            Five => Some(5),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comments {
    pub number: u32,
    pub comp: Comp,
}

impl Comments {
    pub fn compare(&self, value: u32) -> bool {
        self.comp.compare(value, self.number)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Release {
    pub year: i32,
    pub comp: Comp,
}

impl Release {
    pub fn compare(&self, value: i32) -> bool {
        self.comp.compare(value, self.year)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration {
    pub secs: u64,
    pub comp: Comp,
}

impl Duration {
    pub fn compare(&self, value: u64) -> bool {
        self.comp.compare(value, self.secs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Filter {
    pub progress: Progress,
    pub rating: Rating,
    pub comments: Option<Comments>,
    pub release: Option<Release>,
    pub duration: Option<Duration>,
    pub mode: FilterMode,
}

impl Filter {
    pub fn new(mode: FilterMode) -> Self {
        Self {
            progress: Progress::default(),
            rating: Rating::default(),
            comments: None,
            release: None,
            duration: None,
            mode,
        }
    }

    pub fn none() -> Self {
        Self::new(FilterMode::default())
    }

    pub fn is_any(&self) -> bool {
        self.progress.is_any()
            && self.rating.is_any()
            && self.comments.is_none()
            && self.release.is_none()
            && self.duration.is_none()
    }

    /// Resets all filters keeping the mode intact
    pub fn clear(&mut self) {
        self.progress = Progress::default();
        self.rating = Rating::default();
        self.comments = None;
        self.release = None;
        self.duration = None;
    }

    pub fn query(&self, prefix: Option<&str>) -> Option<String> {
        if self.is_any() {
            return None;
        }

        let Self {
            progress,
            rating,
            comments,
            release,
            duration,
            mode,
        } = *self;

        let prefix = prefix
            .map(|prefix| format!("{prefix}."))
            .unwrap_or_default();

        let progress = progress
            .f32()
            .map(|value| format!("{prefix}progress {} {value:.02}", progress.comp));
        let rating = rating
            .u8()
            .map(|value| format!("{prefix}rating {} {value:.02}", rating.comp));
        let comments = comments.map(|comments| {
            format!(
                "{prefix}comment_count {} {}",
                comments.comp, comments.number
            )
        });
        let release =
            release.map(|release| format!("{prefix}release {} {}", release.comp, release.year));
        let duration = duration
            .map(|duration| format!("{prefix}duration {} {}", duration.comp, duration.secs));

        let query = [progress, rating, comments, release, duration]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Some(query.join(&format!(" {} ", mode)))
    }

    pub fn filter<T: Media>(&self, media: &T) -> bool {
        // Compiler error when new field is added
        let Filter {
            progress,
            rating,
            comments,
            release,
            duration,
            mode,
        } = *self;

        let progress = progress.compare(media.progress());
        let rating = rating.compare(media.rating().unwrap_or_default());
        let comments = comments
            .map(|comments| comments.compare(media.comments()))
            .unwrap_or_else(|| matches!(self.mode, FilterMode::And));
        let release = release
            .map(|release| release.compare(media.release().year()))
            .unwrap_or_else(|| matches!(self.mode, FilterMode::And));
        let duration = duration
            .map(|duration| duration.compare(media.duration()))
            .unwrap_or_else(|| matches!(self.mode, FilterMode::And));

        mode.compare_many(&[progress, rating, comments, release, duration])
    }
}

pub mod comments {
    use super::{Comp, Duration, FilterMode};
    use chrono::{DateTime, Local};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Added {
        pub time: DateTime<Local>,
        pub comp: Comp,
    }

    pub type Timestamp = Duration;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Filter {
        pub added: Option<Added>,
        pub timestamp: Option<Timestamp>,
        pub mode: FilterMode,
    }

    impl Filter {
        pub fn new(mode: FilterMode) -> Self {
            Self {
                added: None,
                timestamp: None,
                mode,
            }
        }

        pub fn is_any(&self) -> bool {
            self.added.is_none() && self.timestamp.is_none()
        }

        pub fn clear(&mut self) {
            self.added = None;
            self.timestamp = None;
        }

        pub fn query(&self, prefix: Option<&str>) -> Option<String> {
            if self.is_any() {
                return None;
            }

            let Self {
                added,
                timestamp,
                mode,
            } = *self;

            let prefix = prefix
                .map(|prefix| format!("{prefix}."))
                .unwrap_or_default();

            let added = added.map(|added| {
                let str_date = added
                    .time
                    .with_timezone(&chrono::Utc)
                    .format("%F %T%.f%:z")
                    .to_string();

                format!("{prefix}created_at {} \"{}\"", added.comp, str_date)
            });

            let timestamp = timestamp.map(|timestamp| {
                format!(
                    "{prefix}episode_timestamp {} {}",
                    timestamp.comp, timestamp.secs
                )
            });

            let query = [added, timestamp].into_iter().flatten().collect::<Vec<_>>();

            Some(query.join(&format!(" {mode} ")))
        }
    }
}

pub mod search {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum SearchFilter {
        Movie,
        Show,
        Season,
        Episode,
    }

    impl SearchFilter {
        pub fn new(s: &str) -> Option<Self> {
            let new = match s {
                "movie" => Self::Movie,
                "show" => Self::Show,
                "season" => Self::Season,
                "episode" => Self::Episode,
                _ => return None,
            };

            Some(new)
        }

        pub fn to_str(&self) -> &'static str {
            match self {
                Self::Movie => "movie",
                Self::Show => "show",
                Self::Season => "season",
                Self::Episode => "episode",
            }
        }

        pub fn query(&self) -> String {
            let kind = self.to_str();

            format!("AND media_type = '{kind}'")
        }
    }
}
