use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct VideoId(usize);

impl Eq for VideoId {}

#[derive(Debug, Clone)]
pub struct Video {
    pub id: VideoId,
    pub name: String,
    pub duration: u64,
    pub rating: u8,
    pub progress: f32,
    pub poster: Option<String>,
    pub release: u16,
    pub added: u64,
    pub recent: u64,
    pub comments: u32,
    pub watch_count: u32,
    pub synapsis: String,
    pub tags: Vec<String>,
    pub backdrop: Option<String>,
}

impl Video {
    pub fn testing(id: usize) -> Self {
        let duration = (id * utils::rand_u32() as usize) as u64;

        Self {
            id: VideoId(id),
            name: format!("Fantastic Beasts And Where To Find Them {id}"),
            duration,
            rating: 3,
            progress: 0.35,
            poster: Some("assets/test.png".into()),
            release: 2016,
            added: 1671316859,
            recent: 1766011259,
            comments: 69,
            watch_count: 57,
            synapsis: "In 1926, Newt Scamander arrives at the Magical Congress of the United States of America with a magically expanded briefcase, which houses a number of dangerous creatures and their habitats. When the creatures escape from the briefcase, it sends the American wizarding authorities after Newt, and threatens to strain even further the state of magical and non-magical relations.".to_owned(),
            tags: vec!["tag-1".into(), "tag-2".into(), "tag-team".into()],
            backdrop: Some("assets/player1.jpg".into()),

        }
    }

    pub fn testing2(id: usize) -> Self {
        let duration = ((id / 2) * utils::rand_u32() as usize) as u64;

        Self {
            id: VideoId(id),
            name: format!("Ready Player One {id}"),
            duration,
            rating: 1,
            progress: 0.95,
            poster: Some("assets/test.png".into()),
            release: 2016,
            added: 1671316859,
            recent: 1766011259,
            comments: 420,
            watch_count: 1,
            synapsis: "When the creator of a popular video game system dies, a virtual contest is created to compete for his fortune.".to_owned(),
            tags: vec!["Adventure", "Action", "Science Fiction"].into_iter().map(ToOwned::to_owned).collect(),
            backdrop: Some("assets/test.jpg".into())

        }
    }

    pub fn added_short(&self) -> String {
        //todo
        self.added.to_string()
    }

    pub fn added_full(&self) -> String {
        //todo
        self.added.to_string()
    }

    /// Just the year
    pub fn release_short(&self) -> String {
        //todo
        self.release.to_string()
    }

    pub fn release_full(&self) -> String {
        //todo
        self.release.to_string()
    }

    pub fn duration_full(&self) -> String {
        let hrs = self.duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs} hour{}", if hrs > 1 { "s" } else { "" })
        } else {
            String::default()
        };

        let mins = (self.duration % 3600) / 60;
        let mins = if mins > 0 {
            format!("{mins} min{}", if mins > 1 { "s" } else { "" })
        } else {
            String::default()
        };

        format!("{hrs} {mins}")
    }

    pub fn duration_short(&self) -> String {
        let hrs = self.duration / 3600;
        let hrs = if hrs > 0 {
            format!("{hrs}h")
        } else {
            String::default()
        };

        let mins = (self.duration % 3600) / 60;
        let mins = if mins > 0 {
            format!("{mins}m")
        } else {
            String::default()
        };

        format!("{hrs} {mins}")
    }

    pub fn recent_short(&self) -> String {
        // todo
        self.recent.to_string()
    }

    pub fn recent_long(&self) -> String {
        // todo
        self.recent.to_string()
    }
}
