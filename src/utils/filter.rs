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
            Self::Greater => CHEV_RIGHT,
            Self::Less => CHEV_LEFT,
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
            Self::Less => x > y,
            Self::Equal => x == y,
            Self::Greater => x < y,
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
        let comp = match self.kind {
            ProgressKind::Any => return true,
            ProgressKind::Zero => 0.,
            ProgressKind::TwentyFive => 0.25,
            ProgressKind::Fifty => 0.5,
            ProgressKind::SeventyFive => 0.75,
            ProgressKind::Complete => 1.0,
        };

        self.comp.compare(value, comp)
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

    pub fn compare(&self, value: u8) -> bool {
        let comp = match self.kind {
            RatingKind::Any => return true,
            RatingKind::One => 1,
            RatingKind::Two => 2,
            RatingKind::Three => 3,
            RatingKind::Four => 4,
            RatingKind::Five => 5,
        };

        self.comp.compare(value, comp)
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
    pub year: u16,
    pub comp: Comp,
}

impl Release {
    pub fn compare(&self, value: u16) -> bool {
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
}
