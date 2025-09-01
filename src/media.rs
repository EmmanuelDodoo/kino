use crate::utils;
use chrono::{DateTime, Datelike, Local,  TimeZone};

pub mod show;
pub use show::*;

pub trait Media{
    type Id: Copy + Clone + std::hash::Hash + PartialEq + Eq;

    fn name(&self) -> &str;

    fn id(&self) -> Self::Id;

    fn duration(&self) -> u64;

    fn added(&self) -> DateTime<Local>;

    fn release(&self) -> DateTime<Local>;

    fn recent(&self) -> DateTime<Local>;

    fn progress(&self) -> f32;

    fn watch_count(&self) -> u32;

    fn rating(&self) -> u8;

    fn comments(&self) -> u32;

    fn synapsis(&self) -> &str;

    fn poster(&self) -> Option<&str>;

    fn backdrop(&self) -> Option<&str>;

    fn progress_icon(&self) -> char{
        use utils::icons::*;

         match self.progress() {
            ..0.15 => PROGRESS_10,
            0.15..0.3 => PROGRESS_20,
            0.3..0.5 => PROGRESS_40,
            0.5..0.7 => PROGRESS_60,
            0.7..0.85 => PROGRESS_80,
            x if x < 1.0 => PROGRESS_90,
            _ => PROGRESS_100,
        }

    }

    fn release_year(&self) -> String {
        self.release().year().to_string()
    }

    fn release_my(&self) -> String {
        let release = self.release();
        let year = release.year();
        let month = match release.month0() {
            0 => "Jan",
            1 => "Feb",
            2 => "Mar",
            3 => "Apr",
            4 => "May",
            5 => "Jun",
            6 => "Jul",
            7 => "Aug",
            8 => "Sep",
            9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => unreachable!(),
        };

        format!("{month}, {year}")
    }

    fn added_my(&self) -> String {
        let added = self.added();
        let year = added.year();
        let month = match added.month0() {
            0 => "Jan",
            1 => "Feb",
            2 => "Mar",
            3 => "Apr",
            4 => "May",
            5 => "Jun",
            6 => "Jul",
            7 => "Aug",
            8 => "Sep",
            9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => unreachable!(),
        };

        format!("{month}, {year}")
    }

    fn added_full(&self) -> String {
        //todo
        self.added().to_string()
    }

    /// Duration in `(hrs) hours (mins) minutes` format.
    fn duration_full(&self) -> String {
        let duration = self.duration();
        let hrs = duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs}:")
        } else {
            String::default()
        };

        let mins = (duration % 3600) / 60;
        let secs = (duration % 3600) % 60;

        format!("{hrs}{mins:02}:{secs:02}")
    }

    /// Duration in the `(hrs)h (mins)m` format.
    fn duration_short(&self) -> String {
        let duration = self.duration();
        let hrs = duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs}h")
        } else {
            String::default()
        };

        let mins = (duration % 3600) / 60;
        let mins = if mins > 0 {
            format!("{mins}m")
        } else {
            String::default()
        };

        format!("{hrs} {mins}")
    }

    fn recent_short(&self) -> String {
        let recent = self.recent();
        let day = recent.day();
        let year = recent.year();
        let month = match recent.month0() {
            0 => "Jan",
            1 => "Feb",
            2 => "Mar",
            3 => "Apr",
            4 => "May",
            5 => "Jun",
            6 => "Jul",
            7 => "Aug",
            8 => "Sep",
            9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => unreachable!(),
        };


        format!("{month} {day}, {year}")
    }

    fn recent_long(&self) -> String {
        // todo
        self.recent().to_string()
    }
}

impl<T: Media + ?Sized> Media for &T{
    type Id = T::Id;

    fn name(&self) -> &str {
        (*self).name()
    }

    fn id(&self) -> Self::Id {
        (*self).id()
    }

    fn duration(&self) -> u64 {
        (*self).duration()
    }

    fn synapsis(&self) -> &str {
        (*self).synapsis()
    }

    fn poster(&self) -> Option<&str> {
        (*self).poster()
    }

    fn backdrop(&self) -> Option<&str> {
        (*self).backdrop()
    }

    fn added(&self) -> DateTime<Local> {
        (*self).added()
    }

    fn release(&self) -> DateTime<Local> {
        (*self).release()
    }

    fn recent(&self) -> DateTime<Local> {
        (*self).recent()
    }

    fn progress(&self) -> f32{
        (*self).progress()
    }

    fn watch_count(&self) -> u32 {
        (*self).watch_count()
    }

    fn rating(&self) -> u8{
        (*self).rating()
    }

    fn comments(&self) -> u32{
        (*self).comments()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct MovieId(usize);

#[derive(Debug, Clone)]
pub struct Movie {
    pub id: MovieId,
    pub name: String,
    pub duration: u64,
    pub rating: u8,
    pub progress: f32,
    pub poster: Option<String>,
    pub release: DateTime<Local>,
    pub added: DateTime<Local>,
    pub recent: DateTime<Local>,
    pub comments: u32,
    pub watch_count: u32,
    pub synapsis: String,
    pub tags: Vec<String>,
    pub backdrop: Option<String>,
}

impl Movie {
    pub fn testing(id: usize) -> Self {
        let duration = (id * utils::rand_u32() as usize) as u64;
        let datetime = Local::now();
        let local = datetime.timezone();
        let release = local
            .timestamp_opt(1460719892, 0)
            .earliest()
            .expect("Couldn't make a local datetime");

        let added = local
            .timestamp_opt(1671316859, 0)
            .earliest()
            .expect("Added datetime");

        let recent = local
            .timestamp_opt(1766011259, 0)
            .earliest()
            .expect("Recent Datetime");

        Self {
            id: MovieId(id),
            name: format!("Fantastic Beasts And Where To Find Them {id}"),
            duration,
            rating: 3,
            progress: 0.35,
            poster: Some("assets/fantastic.png".into()),
            release,
            added ,
            recent,
            comments: 69,
            watch_count: 57,
            synapsis: "In 1926, Newt Scamander arrives at the Magical Congress of the United States of America with a magically expanded briefcase, which houses a number of dangerous creatures and their habitats. When the creatures escape from the briefcase, it sends the American wizarding authorities after Newt, and threatens to strain even further the state of magical and non-magical relations.".to_owned(),
            tags: vec!["tag-1".into(), "tag-2".into(), "tag-team".into()],
            backdrop: Some("assets/test.jpg".into())

        }
    }

    pub fn testing2(id: usize) -> Self {
        let duration = ((id / 2) * utils::rand_u32() as usize) as u64;
        let datetime = Local::now();
        let local = datetime.timezone();
        let release = local
            .timestamp_opt(1636803092, 0)
            .earliest()
            .expect("Couldn't make a local datetime");

        let added = local
            .timestamp_opt(1724724617, 0)
            .earliest()
            .expect("Added datetime");

        let recent = local
            .timestamp_opt(1756260617, 0)
            .earliest()
            .expect("Recent Datetime");

        Self {
            id: MovieId(id),
            name: format!("Ready Player One {id}"),
            duration,
            rating: 1,
            progress: 0.95,
            poster: Some("assets/ready.png".into()),
            release,
            added,
            recent, 
            comments: 420,
            watch_count: 1,
            synapsis: "When the creator of a popular video game system dies, a virtual contest is created to compete for his fortune.".to_owned(),
            tags: vec!["Adventure", "Action", "Science Fiction"].into_iter().map(ToOwned::to_owned).collect(),
            backdrop: Some("assets/player1.jpg".into()),

        }
    }

}

impl Media for Movie {
    type Id = MovieId;

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn synapsis(&self) -> &str {
        &self.synapsis
    }

    fn poster(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn backdrop(&self) -> Option<&str> {
        self.backdrop.as_deref()
    }

    fn duration(&self) -> u64 {
        self.duration
    }

    fn added(&self) -> DateTime<Local> {
        self.added
    }

    fn release(&self) -> DateTime<Local> {
        self.release
    }

    fn recent(&self) -> DateTime<Local> {
        self.recent
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn watch_count(&self) -> u32 {
        self.watch_count
    }

    fn rating(&self) -> u8 {
        self.rating
    }

    fn comments(&self) -> u32 {
       self.comments
    }
}
