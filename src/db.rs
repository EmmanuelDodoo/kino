// #![allow(unused_imports, dead_code)]
use crate::models::{
    Collection, CollectionId, Directory, DirectoryId, EComment, ECommentId, Episode, EpisodeId,
    MComment, MCommentId, Movie, MovieId, Season, SeasonId, Show, ShowId,
    collection::{self, Item},
};

use crate::filter::{self, Filter};
use crate::sort::{self, Sort};

use chrono::{DateTime, Local};
use rusqlite::{
    Connection, Result, Row, Transaction, params_from_iter,
    types::{ToSqlOutput, Value},
};
use std::ops::Deref;
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl std::ops::Deref for Database {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl std::ops::DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

impl Database {
    const MOVIE_QUERY: &str = "SELECT movie.*, directory.path as directory_path FROM movie INNER JOIN directory ON movie.directory=directory.id";

    const SHOW_QUERY: &str = "SELECT tv_show.*, directory.path as directory_path FROM tv_show INNER JOIN directory ON tv_show.directory=directory.id";

    const SEASON_QUERY: &str = "SELECT season.*, tv_show.backdrop FROM season INNER JOIN tv_show ON season.show_id=tv_show.id";

    const EPISODE_QUERY: &str = "SELECT * FROM get_episode_data";

    const COLLECTION_QUERY: &str = "SELECT * FROM get_collection";

    pub fn get_directories(&self) -> rusqlite::Result<Vec<Directory>> {
        let sql = "SELECT * FROM directory ORDER BY active";

        let mut statement = self.prepare_cached(sql)?;

        statement.query_map([], Directory::from_row)?.collect()
    }

    pub fn get_directory(&self, id: DirectoryId) -> rusqlite::Result<Directory> {
        let mut statement = self.prepare_cached("SELECT * FROM directory WHERE id=:id")?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Directory::from_row)
    }

    pub fn get_movies(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
    ) -> rusqlite::Result<Vec<Movie>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} {filter} {sort} LIMIT :limit OFFSET :offset",
            Self::MOVIE_QUERY
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":limit", &limit), (":offset", &offset)], Movie::from_row)?
            .collect()
    }

    pub fn get_movie(&self, id: MovieId) -> rusqlite::Result<Movie> {
        let sql = format!("{} WHERE movie.id=:id", Self::MOVIE_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Movie::from_row)
    }

    pub fn get_show(&self, id: ShowId) -> rusqlite::Result<Show> {
        let mut statement = self.prepare_cached("SELECT * FROM tv_show WHERE id=:id")?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Show::from_row)
    }

    pub fn get_shows(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
    ) -> rusqlite::Result<Vec<Show>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} {} {} LIMIT :limit OFFSET :offset",
            Self::SHOW_QUERY,
            filter,
            sort
        );

        let mut statement = self.prepare_cached(&sql)?;
        statement
            .query_map(&[(":limit", &limit), (":offset", &offset)], Show::from_row)?
            .collect()
    }

    pub fn get_show_seasons(
        &self,
        show: ShowId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
    ) -> rusqlite::Result<Vec<Season>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(Some("season"))
            .map(|query| format!("AND ( {query} )"))
            .unwrap_or_default();
        let sort = sort
            .query(Some("season"))
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} WHERE season.show_id=:show {filter} {sort} LIMIT :limit OFFSET :offset",
            Self::SEASON_QUERY,
        );

        let mut statement = self.prepare_cached(&sql)?;
        statement
            .query_map(
                &[
                    (":show", &ToSqlOutput::from(show)),
                    (":limit", &ToSqlOutput::from(limit)),
                    (":offset", &ToSqlOutput::from(offset)),
                ],
                Season::from_row,
            )?
            .collect()
    }

    pub fn get_show_season(&self, id: SeasonId, show: ShowId) -> rusqlite::Result<Season> {
        let sql = format!(
            "{} WHERE season.show_id=:show AND season.id=:id",
            Self::SEASON_QUERY
        );
        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(
            &[
                (":id", &ToSqlOutput::from(id)),
                (":show", &ToSqlOutput::from(show)),
            ],
            Season::from_row,
        )
    }

    pub fn get_season(&self, id: SeasonId) -> rusqlite::Result<Season> {
        let sql = format!("{} WHERE season.id=:id", Self::SEASON_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Season::from_row)
    }

    pub fn get_season_episodes(
        &self,
        season: SeasonId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
    ) -> rusqlite::Result<Vec<Episode>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("AND ( {query} )"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} WHERE season_id=:season {filter} {sort} LIMIT :limit OFFSET :offset",
            Self::EPISODE_QUERY,
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[
                    (":season", &ToSqlOutput::from(season)),
                    (":limit", &ToSqlOutput::from(limit)),
                    (":offset", &ToSqlOutput::from(offset)),
                ],
                Episode::from_row,
            )?
            .collect()
    }

    pub fn get_season_episode(&self, id: EpisodeId, season: SeasonId) -> rusqlite::Result<Episode> {
        let sql = format!("{} WHERE season_id=:season AND id=:id", Self::EPISODE_QUERY);
        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(
            &[
                (":id", &ToSqlOutput::from(id)),
                (":season", &ToSqlOutput::from(season)),
            ],
            Episode::from_row,
        )
    }

    pub fn get_episode(&self, id: EpisodeId) -> rusqlite::Result<Episode> {
        let sql = format!("{} WHERE id=:id", Self::EPISODE_QUERY);
        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Episode::from_row)
    }

    pub fn get_ecomments(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
    ) -> rusqlite::Result<Vec<EComment>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql =
            format!("SELECT * FROM episode_comment {filter} {sort} LIMIT :limit OFFSET :offset");

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[(":limit", &limit), (":offset", &offset)],
                EComment::from_row,
            )?
            .collect()
    }

    pub fn get_episode_comments(
        &self,
        episode: EpisodeId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
    ) -> rusqlite::Result<Vec<EComment>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(Some("episode_comment"))
            .map(|query| format!("AND ( {query} )"))
            .unwrap_or_default();
        let sort = sort
            .query(Some("episode_comment"))
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "SELECT * FROM episode_comment WHERE episode_comment.episode_id=:episode {filter} {sort} LIMIT :limit OFFSET :offset"
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[
                    (":episode", &ToSqlOutput::from(episode)),
                    (":limit", &ToSqlOutput::from(limit)),
                    (":offset", &ToSqlOutput::from(offset)),
                ],
                EComment::from_row,
            )?
            .collect()
    }

    pub fn get_episode_comment(
        &self,
        id: ECommentId,
        episode: EpisodeId,
    ) -> rusqlite::Result<EComment> {
        let sql = "SELECT * FROM episode_comment WHERE episode_comment.episode_id=:episode AND episode_comment.id=:id ";

        let mut statement = self.prepare_cached(sql)?;

        statement.query_row(
            &[
                (":id", &ToSqlOutput::from(id)),
                (":episode", &ToSqlOutput::from(episode)),
            ],
            EComment::from_row,
        )
    }

    pub fn get_mcomments(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
    ) -> rusqlite::Result<Vec<MComment>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql =
            format!("SELECT * FROM movie_comment {filter} {sort} LIMIT :limit OFFSET :offset");

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[(":limit", &limit), (":offset", &offset)],
                MComment::from_row,
            )?
            .collect()
    }

    pub fn get_movie_comments(
        &self,
        movie: MovieId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
    ) -> rusqlite::Result<Vec<MComment>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(Some("movie_comment"))
            .map(|query| format!("AND ( {query} )"))
            .unwrap_or_default();
        let sort = sort
            .query(Some("movie_comment"))
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "SELECT * FROM movie_comment WHERE movie_comment.movie_id=:movie {filter} {sort} LIMIT :limit OFFSET :offset"
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[
                    (":movie", &ToSqlOutput::from(movie)),
                    (":limit", &ToSqlOutput::from(limit)),
                    (":offset", &ToSqlOutput::from(offset)),
                ],
                MComment::from_row,
            )?
            .collect()
    }

    pub fn get_movie_comment(&self, id: MCommentId, movie: MovieId) -> rusqlite::Result<MComment> {
        let sql = "SELECT * FROM movie_comment WHERE movie_comment.movie_id=:movie AND movie_comment.id=:id ";

        let mut statement = self.prepare_cached(sql)?;

        statement.query_row(
            &[
                (":id", &ToSqlOutput::from(id)),
                (":movie", &ToSqlOutput::from(movie)),
            ],
            MComment::from_row,
        )
    }

    pub fn get_collections(&self, sort: collection::Sort) -> rusqlite::Result<Vec<Collection>> {
        let sql = format!("{} ORDER BY {}", Self::COLLECTION_QUERY, sort.query());

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_map([], Collection::from_row)?.collect()
    }

    pub fn get_collection(&self, id: CollectionId) -> rusqlite::Result<Collection> {
        let sql = format!("{} WHERE id=:id", Self::COLLECTION_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], Collection::from_row)
    }

    #[allow(clippy::type_complexity)]
    pub fn get_collection_items(
        &self,
        collection: CollectionId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
    ) -> rusqlite::Result<(Vec<Movie>, Vec<Show>, Vec<Season>, Vec<Episode>)> {
        use collection::{Item, ItemId};

        let repeat = |count: usize| -> String {
            assert_ne!(count, 0);
            let mut s = "?,".repeat(count);
            // Remove trailing comma
            s.pop();
            s
        };

        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let sql = "SELECT * FROM collection_item WHERE collection_id=:collection";

        let mut movies = vec![];
        let mut shows = vec![];
        let mut seasons = vec![];
        let mut episodes = vec![];

        let mut ids = self.prepare_cached(sql)?;
        let mut ids = ids.query(&[(":collection", &ToSqlOutput::from(collection))])?;

        while let Some(row) = ids.next()? {
            match ItemId::from_row(row)? {
                ItemId::Movie(movie) => movies.push(ToSqlOutput::from(movie)),
                ItemId::Show(show) => shows.push(ToSqlOutput::from(show)),
                ItemId::Season(season) => seasons.push(ToSqlOutput::from(season)),
                ItemId::Episode(episode) => episodes.push(ToSqlOutput::from(episode)),
            }
        }

        let movies = if !movies.is_empty() {
            let vars = repeat(movies.len());

            let filter = filter
                .query(Some("movie"))
                .map(|query| format!("AND {query}"))
                .unwrap_or_default();
            let sort = sort
                .query(Some("movie"))
                .map(|query| format!("ORDER BY {query}"))
                .unwrap_or_default();
            let sql = format!(
                "{} WHERE movie.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::MOVIE_QUERY,
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(movies),  Movie::from_row)?
                .collect::<rusqlite::Result<Vec<Movie>>>()?
        } else {
            vec![]
        };

        let shows = if !shows.is_empty() {
            let filter = filter
                .query(Some("tv_show"))
                .map(|query| format!("AND {query}"))
                .unwrap_or_default();
            let sort = sort
                .query(Some("tv_show"))
                .map(|query| format!("ORDER BY {query}"))
                .unwrap_or_default();

            let vars = repeat(shows.len());
            let sql = format!(
                "{} WHERE tv_show.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::SHOW_QUERY
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(shows),  Show::from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            vec![]
        };

        let seasons = if !seasons.is_empty() {
            let filter = filter
                .query(Some("season"))
                .map(|query| format!("AND {query}"))
                .unwrap_or_default();
            let sort = sort
                .query(Some("season"))
                .map(|query| format!("ORDER BY {query}"))
                .unwrap_or_default();

            let vars = repeat(seasons.len());
            let sql = format!(
                "{} WHERE season.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::SEASON_QUERY
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(seasons),  Season::from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            vec![]
        };

        let episodes = if !episodes.is_empty() {
            let filter = filter
                .query(None)
                .map(|query| format!("AND {query}"))
                .unwrap_or_default();
            let sort = sort
                .query(None)
                .map(|query| format!("ORDER BY {query}"))
                .unwrap_or_default();

            let vars = repeat(episodes.len());

            let sql = format!(
                "{} WHERE id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::EPISODE_QUERY
            );

            let mut statement = self.prepare_cached(&sql)?;

            statement
                .query_map(params_from_iter(episodes),  Episode::from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            vec![]
        };

        Ok((movies, shows, seasons, episodes))
    }

    pub(super) fn open_test_db() -> rusqlite::Result<Database> {
        let schema = include_str!("../schema.sql");

        let conn = rusqlite::Connection::open_in_memory()?;

        conn.execute_batch(schema)?;

        let dummy = include_str!("../dummy.txt");

        conn.execute_batch(dummy)?;

        Ok(Database { conn })
    }

    pub fn open(path: &str) -> rusqlite::Result<Database> {
        let conn = rusqlite::Connection::open(path)?;

        Ok(Database { conn })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Table {
    Directory,
    Movies,
    Show,
    Season,
    Episode,
    EComment,
    MComment,
    Collection,
    CollectionItem,
    WatchList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Insert = 0,
    Update = 1,
    Delete = 2,
}

#[derive(Debug, PartialEq)]
pub struct Query<'a> {
    pub id: Uuid,
    pub table: Table,
    pub op: Operation,
    pub sql: &'a str,
    pub params: Vec<(&'a str, ToSqlOutput<'a>)>,
}

impl<'a> Query<'a> {
    pub fn execute(self, db: &impl Deref<Target = Connection>) -> Result<Success, Failure<'a>> {
        let Self {
            sql,
            params,
            id,
            table,
            op,
        } = self;

        match db.prepare_cached(sql) {
            Ok(mut statement) => match statement.execute(params.as_slice()).map(|_| ()) {
                Ok(_) => Ok(Success { id, table, op }),
                Err(error) => Err(Failure {
                    query: Query {
                        id,
                        table,
                        sql,
                        params,
                        op,
                    },
                    error: Box::new(error),
                }),
            },
            Err(error) => Err(Failure {
                query: Query {
                    id,
                    table,
                    sql,
                    params,
                    op,
                },
                error: Box::new(error),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Success {
    pub id: Uuid,
    pub table: Table,
    pub op: Operation,
}

#[derive(Debug, PartialEq)]
pub struct Failure<'a> {
    pub query: Query<'a>,
    pub error: Box<rusqlite::Error>,
}

#[derive(Debug, PartialEq)]
pub struct Batch<'a> {
    directories: Vec<Query<'a>>,
    movies: Vec<Query<'a>>,
    shows: Vec<Query<'a>>,
    seasons: Vec<Query<'a>>,
    episodes: Vec<Query<'a>>,
    ecomments: Vec<Query<'a>>,
    mcomments: Vec<Query<'a>>,
    collections: Vec<Query<'a>>,
    collection_items: Vec<Query<'a>>,
    watch_list: Vec<Query<'a>>,
}

impl<'a> Batch<'a> {
    pub fn new() -> Self {
        Self {
            directories: Vec::default(),
            movies: Vec::default(),
            shows: Vec::default(),
            seasons: Vec::default(),
            episodes: Vec::default(),
            ecomments: Vec::default(),
            mcomments: Vec::default(),
            collections: Vec::default(),
            collection_items: Vec::default(),
            watch_list: Vec::default(),
        }
    }

    pub fn push(&mut self, query: Query<'a>) {
        match query.table {
            Table::Directory => self.directories.push(query),
            Table::Movies => self.movies.push(query),
            Table::Show => self.shows.push(query),
            Table::Season => self.seasons.push(query),
            Table::Episode => self.episodes.push(query),
            Table::EComment => self.ecomments.push(query),
            Table::MComment => self.mcomments.push(query),
            Table::Collection => self.collections.push(query),
            Table::CollectionItem => self.collection_items.push(query),
            Table::WatchList => self.watch_list.push(query),
        }
    }

    pub fn push_many(&mut self, query: impl IntoIterator<Item = Query<'a>>) {
        for query in query.into_iter() {
            self.push(query)
        }
    }

    fn sort(&mut self) {
        let Self {
            directories,
            movies,
            shows,
            seasons,
            episodes,
            ecomments,
            mcomments,
            collections,
            collection_items,
            watch_list,
        } = self;

        directories.sort_by(|x, y| x.op.cmp(&y.op));
        movies.sort_by(|x, y| x.op.cmp(&y.op));
        shows.sort_by(|x, y| x.op.cmp(&y.op));
        seasons.sort_by(|x, y| x.op.cmp(&y.op));
        episodes.sort_by(|x, y| x.op.cmp(&y.op));
        ecomments.sort_by(|x, y| x.op.cmp(&y.op));
        mcomments.sort_by(|x, y| x.op.cmp(&y.op));
        collections.sort_by(|x, y| x.op.cmp(&y.op));
        collection_items.sort_by(|x, y| x.op.cmp(&y.op));
        watch_list.sort_by(|x, y| x.op.cmp(&y.op));
    }

    pub fn execute(mut self, db: &mut Database) -> Result<BatchResult<'a>, BatchError<'a>> {
        self.sort();

        let Self {
            directories,
            movies,
            shows,
            seasons,
            episodes,
            ecomments,
            mcomments,
            collections,
            collection_items,
            watch_list,
        } = self;

        let mut failures = vec![];
        let mut successes = vec![];

        let directories_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    residue: Box::new(Batch {
                        directories,
                        movies,
                        shows,
                        seasons,
                        episodes,
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                    error: Box::new(error),
                });
            }
        };

        for directory in directories {
            match directory.execute(&directories_tx) {
                Ok(success) => successes.push(success),
                Err(failure) => failures.push(failure),
            };
        }

        if let Err(error) = directories_tx.commit() {
            return Err(BatchError {
                residue: Box::new(Batch {
                    directories: vec![],
                    movies,
                    shows,
                    seasons,
                    episodes,
                    ecomments,
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
                error: Box::new(error),
            });
        }

        let movies_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies,
                        shows,
                        seasons,
                        episodes,
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                    error: Box::new(error),
                });
            }
        };

        for movie in movies {
            match movie.execute(&movies_tx) {
                Ok(success) => successes.push(success),
                Err(failure) => failures.push(failure),
            };
        }

        if let Err(error) = movies_tx.commit() {
            return Err(BatchError {
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows,
                    seasons,
                    episodes,
                    ecomments,
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
                error: Box::new(error),
            });
        }

        let shows_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows,
                        seasons,
                        episodes,
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for show in shows {
            match show.execute(&shows_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            };
        }

        if let Err(error) = shows_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons,
                    episodes,
                    ecomments,
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let seasons_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons,
                        episodes,
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for season in seasons {
            match season.execute(&seasons_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            };
        }

        if let Err(error) = seasons_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes,
                    ecomments,
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let episodes_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes,
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for episodes in episodes {
            match episodes.execute(&episodes_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            };
        }

        if let Err(error) = episodes_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments,
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let ecomments_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes: vec![],
                        ecomments,
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for ecomment in ecomments {
            match ecomment.execute(&ecomments_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            };
        }

        if let Err(error) = ecomments_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments: vec![],
                    mcomments,
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let mcomments_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes: vec![],
                        ecomments: vec![],
                        mcomments,
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for mcomment in mcomments {
            match mcomment.execute(&mcomments_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            }
        }

        if let Err(error) = mcomments_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments: vec![],
                    mcomments: vec![],
                    collections,
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let collections_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes: vec![],
                        ecomments: vec![],
                        mcomments: vec![],
                        collections,
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for collection in collections {
            match collection.execute(&collections_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            }
        }

        if let Err(error) = collections_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments: vec![],
                    mcomments: vec![],
                    collections: vec![],
                    collection_items,
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let citem_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes: vec![],
                        ecomments: vec![],
                        mcomments: vec![],
                        collections: vec![],
                        collection_items,
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for item in collection_items {
            match item.execute(&citem_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            }
        }

        if let Err(error) = citem_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments: vec![],
                    mcomments: vec![],
                    collections: vec![],
                    collection_items: vec![],
                    watch_list,
                }),
                successes,
                failures,
            });
        };

        let watch_list_tx = match db.transaction() {
            Ok(trans) => trans,
            Err(error) => {
                return Err(BatchError {
                    error: Box::new(error),
                    residue: Box::new(Batch {
                        directories: vec![],
                        movies: vec![],
                        shows: vec![],
                        seasons: vec![],
                        episodes: vec![],
                        ecomments: vec![],
                        mcomments: vec![],
                        collections: vec![],
                        collection_items: vec![],
                        watch_list,
                    }),
                    successes,
                    failures,
                });
            }
        };

        for list in watch_list {
            match list.execute(&watch_list_tx) {
                Ok(succ) => successes.push(succ),
                Err(fail) => failures.push(fail),
            }
        }

        if let Err(error) = watch_list_tx.commit() {
            return Err(BatchError {
                error: Box::new(error),
                residue: Box::new(Batch {
                    directories: vec![],
                    movies: vec![],
                    shows: vec![],
                    seasons: vec![],
                    episodes: vec![],
                    ecomments: vec![],
                    mcomments: vec![],
                    collections: vec![],
                    collection_items: vec![],
                    watch_list: vec![],
                }),
                successes,
                failures,
            });
        };

        Ok(BatchResult {
            successes,
            failures,
        })
    }
}

#[derive(Debug)]
/// The accumulated results of executing a [`Batch`].
pub struct BatchResult<'a> {
    pub successes: Vec<Success>,
    pub failures: Vec<Failure<'a>>,
}

impl BatchResult<'_> {
    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }

    pub fn has_successes(&self) -> bool {
        !self.successes.is_empty()
    }
}

#[derive(Debug)]
pub struct BatchError<'a> {
    pub residue: Box<Batch<'a>>,
    pub successes: Vec<Success>,
    pub failures: Vec<Failure<'a>>,
    pub error: Box<rusqlite::Error>,
}
