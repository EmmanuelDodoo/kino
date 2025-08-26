use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct ShowId(usize);

#[derive(Debug, Clone)]
pub struct Show {
    pub id: ShowId,
    pub name: String,
    pub seasons: u16,
    pub episodes: u16,
    pub last_played: Option<SeasonId>,
    pub rating: u8,
    pub progress: f32,
    pub duration: u64,
    pub release: DateTime<Local>,
    pub added: DateTime<Local>,
    pub recent: DateTime<Local>,
    pub synapsis: String,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub tags: Vec<String>,
    pub comments: u32,
    pub sub_comments: u32,
    pub watch_count: u32,
}

impl Show {
    pub fn testing(id: usize) -> Self {
        let duration = ((id / 2) * utils::rand_u32() as usize) as u64;
        let datetime = Local::now();
        let local = datetime.timezone();
        let release = local
            .timestamp_opt(1656003592, 0)
            .earliest()
            .expect("Couldn't make a local datetime");

        let added = local
            .timestamp_opt(1704704607, 0)
            .earliest()
            .expect("Added datetime");

        let recent = local
            .timestamp_opt(1756260617, 0)
            .earliest()
            .expect("Recent Datetime");

        Self {
            id: ShowId(id),
            name: format!("The Big Bang Theory {id}"),
            seasons: 12,
            episodes: 240,
            last_played: None,
            duration,
            rating: 4,
            progress: 0.75,
            poster: Some("assets/show.png".into()),
            release,
            added,
            recent, 
            comments: 20,
            sub_comments: 4,
            synapsis: "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned(),
            tags: vec!["Comedy"].into_iter().map(ToOwned::to_owned).collect(),
            backdrop: Some("assets/show2.png".into()),
            watch_count: 37


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
            id: ShowId(id),
            name: format!("Supernatural {id}"),
            seasons: 15,
            episodes: 300,
            last_played: Some(SeasonId(1)),
            duration,
            rating: 5,
            progress: 0.5,
            poster: Some("assets/show1.png".into()),
            release,
            added,
            recent, 
            comments: 420,
            sub_comments: 42,
            synapsis: "When they were boys, Sam and Dean Winchester lost their mother to a mysterious and demonic supernatural force. Subsequently, their father raised them to be soldiers. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned(),
            tags: vec!["Drama", "Mystery", "Sci-Fi", "Fantasy"].into_iter().map(ToOwned::to_owned).collect(),
            backdrop: Some("assets/show3.png".into()),
            watch_count: 73,


        }

    }
}

impl Media for Show {
    type Id = ShowId;


    fn name(&self) -> &str{
        &self.name
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn poster(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn backdrop(&self) -> Option<&str> {
        self.backdrop.as_deref()
    }

    fn synapsis(&self) -> &str {
        &self.synapsis
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct SeasonId(usize);

#[derive(Debug, Clone)]
pub struct Season{
    pub id: SeasonId,
    pub show_id: ShowId,
    pub name: String,
    pub show_name: String,
    pub number: u16,
    pub episodes: u16,
    pub last_played: Option<EpisodeId>,
    pub rating: u8,
    pub progress: f32,
    pub duration: u64,
    pub release: DateTime<Local>,
    pub added: DateTime<Local>,
    pub recent: DateTime<Local>,
    pub synapsis: String,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub comments: u32,
    pub sub_comments: u32,
    pub watch_count: u32,
}

impl Season {
    pub fn testing(id: usize) -> Self {
        let duration = ((id / 2) * utils::rand_u32() as usize) as u64;
        let datetime = Local::now();
        let local = datetime.timezone();
        let release = local
            .timestamp_opt(1656003592, 0)
            .earliest()
            .expect("Couldn't make a local datetime");

        let added = local
            .timestamp_opt(1704704607, 0)
            .earliest()
            .expect("Added datetime");

        let recent = local
            .timestamp_opt(1756260617, 0)
            .earliest()
            .expect("Recent Datetime");

        Self {
            show_id: ShowId(id),
            id: SeasonId(id),
            show_name: format!("The Big Bang Theory {id}"),
            name: format!("Season {id}"),
            number: id as u16,
            episodes: 20,
            last_played: Some(EpisodeId(0)),
            duration,
            rating: 2,
            progress: 0.5,
            poster: Some("assets/show.png".into()),
            backdrop: Some("assets/show2.png".into()),
            release,
            added,
            recent, 
            comments: 20,
            watch_count: 28,
            sub_comments: 4,
            synapsis: "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned(),
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
            show_id: ShowId(id),
            id: SeasonId(id),
            show_name: format!("Supernatural {id}"),
            name: format!("Season {id}"),
            number: id as u16,
            episodes: 20,
            last_played: None,
            duration,
            rating: 5,
            progress: 0.5,
            poster: Some("assets/show1.png".into()),
            backdrop: Some("assets/show3.png".into()),
            release,
            added,
            watch_count: 6,
            recent, 
            comments: 420,
            sub_comments: 42,
            synapsis: "When they were boys, Sam. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned(),

        }

    }
}

impl Media for Season {
    type Id = SeasonId;

    fn name(&self) -> &str{
        &self.name
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn poster(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn backdrop(&self) -> Option<&str> {
        None
    }

    fn synapsis(&self) -> &str {
        &self.synapsis
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct EpisodeId(usize);

#[derive(Debug, Clone)]
pub struct Episode {
    pub id: EpisodeId,
    // todo: Ideally should be in the form, `Episode number: Episode name`
    pub name: String,
    pub number: u16,
    pub show_id: ShowId,
    pub show_name: String,
    pub season_id: SeasonId,
    pub rating: u8,
    pub progress: f32,
    pub duration: u64,
    pub release: DateTime<Local>,
    pub added: DateTime<Local>,
    pub recent: DateTime<Local>,
    pub synapsis: String,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub comments: u32,
    pub watch_count: u32,
}

impl Episode {
    pub fn testing(id: usize) -> Self {
        let duration = ((id / 2) * utils::rand_u32() as usize) as u64;
        let datetime = Local::now();
        let local = datetime.timezone();
        let release = local
            .timestamp_opt(1656003592, 0)
            .earliest()
            .expect("Couldn't make a local datetime");

        let added = local
            .timestamp_opt(1704704607, 0)
            .earliest()
            .expect("Added datetime");

        let recent = local
            .timestamp_opt(1756260617, 0)
            .earliest()
            .expect("Recent Datetime");

        Self {
            show_id: ShowId(id),
            id: EpisodeId(id),
            season_id: SeasonId(id),
            show_name: format!("The Big Bang Theory {id}"),
            name: format!("Episode {id}"),
            number: id as u16,
            duration,
            rating: 2,
            progress: 0.5,
            poster: Some("assets/show.png".into()),
            backdrop: Some("assets/show2.png".into()),
            release,
            added,
            recent, 
            watch_count: 12,
            comments: 20,
            synapsis: "Physicists Leonard and Sheldon find their nerd-centric social circle with pals Howard and Raj expanding when aspiring actress Penny moves in next door.".to_owned(),
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
            show_id: ShowId(id),
            id: EpisodeId(id),
            season_id: SeasonId(id),
            show_name: format!("Supernatural {id}"),
            name: format!("Episode {id}"),
            number: id as u16,
            duration,
            rating: 5,
            progress: 0.5,
            poster: Some("assets/show1.png".into()),
            backdrop: Some("assets/show3.png".into()),
            release,
            watch_count: 0,
            added,
            recent, 
            comments: 420,
            synapsis: "When they were boys, Sam. He taught them about the paranormal evil that lives in the dark corners and on the back roads of America ... and he taught them how to kill it. Now, the Winchester brothers crisscross the country in their '67 Chevy Impala, battling every kind of supernatural threat they encounter along the way.".to_owned(),

        }

    }
}

impl Media for Episode {
    type Id = EpisodeId;

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Self::Id {
        self.id
    }

    fn poster(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn backdrop(&self) -> Option<&str> {
        None
    }

    fn synapsis(&self) -> &str {
        &self.synapsis
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
