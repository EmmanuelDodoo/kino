use super::H7;
use crate::media::Media;

#[derive(Debug, Clone, PartialEq)]
pub struct Sort {
    kinds: Vec<SortKind>,
}

impl Sort {
    pub fn clear(&mut self) {
        self.kinds.clear();
    }

    pub fn sort<T: Media>(&self, x: &T, y: &T) -> std::cmp::Ordering {
        for kind in self.kinds.iter() {
            let ord = match kind {
                SortKind::Name => alphanumeric_sort::compare_str(x.name(), y.name()),
                SortKind::Duration => x.duration().cmp(&y.duration()),
                SortKind::Added => x.added().cmp(&y.added()),
                SortKind::Rating => x.rating().cmp(&y.rating()),
                SortKind::Recent => x.recent().cmp(&y.recent()),
                SortKind::Release => x.release().cmp(&y.release()),
                SortKind::Progress => x.progress().total_cmp(&y.progress()),
                SortKind::Comments => x.comments().cmp(&y.comments()),
                SortKind::Watch => x.watch_count().cmp(&y.watch_count()),
            };

            if !matches!(ord, std::cmp::Ordering::Equal) {
                return ord;
            }
        }

        std::cmp::Ordering::Equal
    }

    pub fn push(&mut self, kind: SortKind) {
        self.kinds.push(kind);
    }

    pub fn remove(&mut self, kind: SortKind) {
        self.kinds.retain(|own| *own != kind);
    }

    pub fn reverse(&mut self) {
        self.kinds.reverse();
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SortKind> {
        self.kinds.iter()
    }
}

impl Default for Sort {
    fn default() -> Self {
        Sort {
            kinds: vec![SortKind::Name],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortKind {
    Name,
    Duration,
    Progress,
    Rating,
    // Tags,
    Watch,
    Release,
    Comments,
    Added,
    Recent,
}

impl SortKind {
    pub const VISIBLE: [SortKind; 6] = [
        Self::Name,
        Self::Duration,
        Self::Progress,
        Self::Rating,
        Self::Added,
        Self::Recent,
    ];

    pub const HIDDEN: [SortKind; 3] = [Self::Release, Self::Comments, Self::Watch];

    pub fn view<'a, Message, A, R>(
        &'a self,
        on_add: A,
        on_remove: R,
        order: Option<usize>,
    ) -> iced::Element<'a, Message>
    where
        Message: Clone + 'a,
        A: Fn(SortKind) -> Message + 'a,
        R: Fn(SortKind) -> Message + 'a,
    {
        use iced::{
            Border,
            widget::{button, text},
        };

        let enable = order.is_none();
        let msg = if enable {
            on_add(*self)
        } else {
            on_remove(*self)
        };

        let order = order
            .map(|order| (order + 1).to_string())
            .unwrap_or_default();
        let content = text(format!("{self} {}", order)).size(H7);
        // let content = row!(content).spacing(2.0).align_y(Vertical::Center);

        button(content)
            .on_press(msg)
            .style(move |theme, status| {
                let default = if enable {
                    button::background(theme, status)
                } else {
                    button::secondary(theme, status)
                };
                let border = Border::default().width(2.0).rounded(5.0);

                button::Style { border, ..default }
            })
            .into()
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
