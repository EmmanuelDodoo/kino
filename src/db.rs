use crate::models::{
    CollectionId, Directory, DirectoryId, EComment, ECommentId, EpisodeId, MComment, MCommentId,
    MovieId, SearchItem, SeasonId, ShowId,
    collection::{self, ItemId, Items},
};

use crate::filter::{self, Filter, search::SearchFilter};
use crate::sort::{self, Sort};

use rusqlite::{Connection, Result, Row, ToSql, params_from_iter, types::ToSqlOutput};
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::Path;
use uuid::Uuid;

struct Migration {
    version: u64,
    sql: &'static str,
}

#[rustfmt::skip]
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 0,
        sql: ""
    },
    Migration {
        version: 1,
        sql: include_str!("../resources/db/migrations/1.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../resources/db/migrations/2.sql"),
    },
    Migration {
        version: 3,
        sql: include_str!("../resources/db/migrations/3.sql"),
    },
];

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
    const MOVIE_QUERY: &str = "SELECT directory.id as directory_id,  directory.path as directory_path, CASE WHEN (NOT movie.fetched) AND movie.generate_poster THEN NULL ELSE movie.poster END AS poster, CASE WHEN NOT movie.fetched THEN NULL ELSE movie.backdrop END AS backdrop, movie.*  FROM movie INNER JOIN directory ON movie.directory=directory.id";

    const SHOW_QUERY: &str = "SELECT directory.id as directory_id, directory.path as directory_path, CASE WHEN NOT tv_show.fetched THEN NULL ELSE tv_show.poster END AS poster, CASE WHEN NOT tv_show.fetched THEN NULL ELSE tv_show.backdrop END AS backdrop, tv_show.* FROM tv_show INNER JOIN directory ON tv_show.directory=directory.id";

    const SEASON_QUERY: &str = "SELECT  tv_show.backdrop, CASE WHEN NOT season.fetched THEN NULL ELSE season.poster END AS poster, season.* FROM season INNER JOIN tv_show ON season.show_id=tv_show.id";

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

    pub fn get_dir_movies<T>(
        &self,
        dir: DirectoryId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = format!("{} WHERE directory_id=:dir", Self::MOVIE_QUERY);
        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":dir", &ToSqlOutput::from(dir))], map)?
            .collect()
    }

    pub fn get_dir_shows<T>(
        &self,
        dir: DirectoryId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = format!("{} WHERE directory_id=:dir", Self::SHOW_QUERY);
        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":dir", &ToSqlOutput::from(dir))], map)?
            .collect()
    }

    pub fn get_movies<T>(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("AND {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} WHERE NOT removed {filter} {sort} LIMIT :limit OFFSET :offset",
            Self::MOVIE_QUERY
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":limit", &limit), (":offset", &offset)], map)?
            .collect()
    }

    pub fn get_movie<T>(
        &self,
        id: MovieId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let sql = format!("{} WHERE movie.id=:id", Self::MOVIE_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], map)
    }

    pub fn get_show<T>(
        &self,
        id: ShowId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let mut statement = self.prepare_cached("SELECT * FROM tv_show WHERE id=:id")?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], map)
    }

    pub fn get_shows<T>(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let filter = filter
            .query(None)
            .map(|query| format!("AND {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query}"))
            .unwrap_or_default();

        let sql = format!(
            "{} WHERE not removed {filter} {sort} LIMIT :limit OFFSET :offset",
            Self::SHOW_QUERY,
        );

        let mut statement = self.prepare_cached(&sql)?;
        statement
            .query_map(&[(":limit", &limit), (":offset", &offset)], map)?
            .collect()
    }

    pub fn get_show_seasons<T>(
        &self,
        show: ShowId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
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
            "{} WHERE NOT season.removed AND season.show_id=:show {filter} {sort} LIMIT :limit OFFSET :offset",
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
                map,
            )?
            .collect()
    }

    pub fn get_show_seasons_removed<T>(
        &self,
        show: ShowId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = format!("{} WHERE season.show_id=:show", Self::SEASON_QUERY,);

        let mut statement = self.prepare_cached(&sql)?;
        statement
            .query_map(&[(":show", &ToSqlOutput::from(show))], map)?
            .collect()
    }

    pub fn get_season<T>(
        &self,
        id: SeasonId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let sql = format!("{} WHERE season.id=:id", Self::SEASON_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], map)
    }

    pub fn get_season_episodes<T>(
        &self,
        season: SeasonId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
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
            "{} WHERE NOT removed AND season_id=:season {filter} {sort} LIMIT :limit OFFSET :offset",
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
                map,
            )?
            .collect()
    }

    pub fn get_season_episodes_removed<T>(
        &self,
        season: SeasonId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = format!("{} WHERE season_id=:season", Self::EPISODE_QUERY,);

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":season", &ToSqlOutput::from(season))], map)?
            .collect()
    }

    pub fn get_episode<T>(
        &self,
        id: EpisodeId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let sql = format!("{} WHERE id=:id", Self::EPISODE_QUERY);
        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], map)
    }

    pub fn get_ecomments<T>(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
        map: fn(EComment) -> T,
    ) -> rusqlite::Result<Vec<T>> {
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
            .query_map(&[(":limit", &limit), (":offset", &offset)], |row| {
                EComment::from_row(row).map(map)
            })?
            .collect()
    }

    pub fn get_episode_comments<T>(
        &self,
        episode: EpisodeId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
        map: fn(EComment) -> T,
    ) -> rusqlite::Result<Vec<T>> {
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
                |row| EComment::from_row(row).map(map),
            )?
            .collect()
    }

    pub fn get_episode_comment<T>(
        &self,
        id: ECommentId,
        map: fn(EComment) -> T,
    ) -> rusqlite::Result<T> {
        let sql = "SELECT * FROM episode_comment WHERE episode_comment.id=:id ";

        let mut statement = self.prepare_cached(sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], |row| {
            EComment::from_row(row).map(map)
        })
    }

    pub fn get_mcomments<T>(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
        map: fn(MComment) -> T,
    ) -> rusqlite::Result<Vec<T>> {
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
            .query_map(&[(":limit", &limit), (":offset", &offset)], |row| {
                MComment::from_row(row).map(map)
            })?
            .collect()
    }

    pub fn get_movie_comments<T>(
        &self,
        movie: MovieId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: filter::comments::Filter,
        sort: sort::comments::Sort,
        map: fn(MComment) -> T,
    ) -> rusqlite::Result<Vec<T>> {
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
                |row| MComment::from_row(row).map(map),
            )?
            .collect()
    }

    pub fn get_movie_comment<T>(
        &self,
        id: MCommentId,
        map: fn(MComment) -> T,
    ) -> rusqlite::Result<T> {
        let sql = "SELECT * FROM movie_comment WHERE movie_comment.id=:id ";

        let mut statement = self.prepare_cached(sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], |row| {
            MComment::from_row(row).map(map)
        })
    }

    pub fn get_collections<T>(
        &self,
        sort: collection::Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = format!(
            "{} WHERE NOT removed ORDER BY {}",
            Self::COLLECTION_QUERY,
            sort.query()
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_map([], map)?.collect()
    }

    pub fn get_memberships<T>(
        &self,
        collections: Vec<CollectionId>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: collection::Sort,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        if collections.is_empty() {
            return Ok(vec![]);
        }

        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let vars = repeat(collections.len());

        let sql = format!(
            "SELECT * FROM collection WHERE collection.id IN ({vars}) AND NOT collection.removed ORDER BY {} LIMIT {limit} OFFSET {offset}",
            sort.query()
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                params_from_iter(collections.into_iter().map(ToSqlOutput::from)),
                map,
            )?
            .collect()
    }

    pub fn get_collection<T>(
        &self,
        id: CollectionId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let sql = format!("{} WHERE id=:id", Self::COLLECTION_QUERY);

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row(&[(":id", &ToSqlOutput::from(id))], map)
    }

    pub fn get_collection_inserts<T>(
        &self,
        id: CollectionId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = "SELECT * FROM collection_inserts WHERE collection_id=:id";

        let mut statement = self.prepare_cached(sql)?;

        statement
            .query_map(&[(":id", &ToSqlOutput::from(id))], map)?
            .collect()
    }

    pub fn get_collection_deletes<T>(
        &self,
        id: CollectionId,
        map: fn(&Row<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<Vec<T>> {
        let sql = "SELECT * FROM collection_deletes WHERE collection_id=:id";

        let mut statement = self.prepare_cached(sql)?;

        statement
            .query_map(&[(":id", &ToSqlOutput::from(id))], map)?
            .collect()
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub fn get_collection_members<M, S, A, E>(
        &self,
        collection: CollectionId,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Filter,
        sort: Sort,
        movie_map: fn(&Row<'_>) -> rusqlite::Result<M>,
        show_map: fn(&Row<'_>) -> rusqlite::Result<S>,
        season_map: fn(&Row<'_>) -> rusqlite::Result<A>,
        episode_map: fn(&Row<'_>) -> rusqlite::Result<E>,
    ) -> rusqlite::Result<(Vec<M>, Vec<S>, Vec<A>, Vec<E>)> {
        let limit = limit.unwrap_or(-1);
        let offset = offset.unwrap_or(-1);

        let mut movies = vec![];
        let mut shows = vec![];
        let mut seasons = vec![];
        let mut episodes = vec![];

        let items = self.get_collection_items(collection)?;

        for item in items {
            match item {
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
                "{} WHERE NOT removed AND movie.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::MOVIE_QUERY,
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(movies), movie_map)?
                .collect::<rusqlite::Result<Vec<_>>>()?
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
                "{} WHERE NOT removed AND tv_show.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::SHOW_QUERY
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(shows), show_map)?
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
                "{} WHERE NOT season.removed AND season.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::SEASON_QUERY
            );
            let mut statement = self.prepare_cached(&sql)?;
            statement
                .query_map(params_from_iter(seasons), season_map)?
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
                "{} WHERE NOT removed AND id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
                Self::EPISODE_QUERY
            );

            let mut statement = self.prepare_cached(&sql)?;

            statement
                .query_map(params_from_iter(episodes), episode_map)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            vec![]
        };

        Ok((movies, shows, seasons, episodes))
    }

    pub fn get_collection_items(&self, collection: CollectionId) -> rusqlite::Result<Vec<ItemId>> {
        let sql = "SELECT * FROM collection_item WHERE collection_id=:collection";

        let mut ids = self.prepare_cached(sql)?;

        ids.query_map(
            &[(":collection", &ToSqlOutput::from(collection))],
            ItemId::from_row,
        )?
        .collect()
    }

    pub fn remove_collection(&self, collection: CollectionId) -> rusqlite::Result<usize> {
        let sql = "UPDATE collection SET removed=TRUE WHERE id=:id";

        let mut statement = self.prepare_cached(sql)?;

        statement.execute(&[(":id", &ToSqlOutput::from(collection))])
    }

    pub fn remove_collection_items(
        &self,
        collection: CollectionId,
        items: Items,
    ) -> rusqlite::Result<usize> {
        let cond = match items {
            Items::All => "",
            Items::Shows => "AND media_type = 'show'",
            Items::Movies => "AND media_type = 'movie'",
            Items::Seasons => "AND media_type = 'season'",
            Items::Episodes => "AND media_type = 'episode'",
        };

        let sql = format!("DELETE FROM collection_item WHERE collection_id=:id {cond}");

        let mut statement = self.prepare_cached(&sql)?;

        statement.execute(&[(":id", &ToSqlOutput::from(collection))])
    }

    pub fn insert_collection_items(
        &mut self,
        collection: CollectionId,
        items: Vec<ItemId>,
    ) -> rusqlite::Result<usize> {
        if items.is_empty() {
            return Ok(0);
        }

        let trans = self.transaction()?;

        let mut vars = "(?, ?, ?),".repeat(items.len());
        vars.pop();

        let sql = format!(
            "INSERT OR IGNORE INTO collection_item (collection_id, media_type, media_id) VALUES {vars}"
        );

        let mut params = vec![];

        for item in items {
            params.push(ToSqlOutput::from(collection));
            params.push(ToSqlOutput::from(item.name_str()));
            params.push(ToSqlOutput::from(item));
        }

        let params = params
            .iter()
            .map(|param| param as &dyn ToSql)
            .collect::<Vec<_>>();

        let rows = trans.execute(&sql, params.as_slice())?;

        trans.commit()?;

        Ok(rows)
    }

    pub fn toggle_membership(
        &mut self,
        item: ItemId,
        collections: Vec<(CollectionId, bool)>,
    ) -> rusqlite::Result<bool> {
        if collections.is_empty() {
            return Ok(false);
        }

        let (inserts, deletes): (Vec<(CollectionId, bool)>, Vec<_>) =
            collections.iter().partition(|(_, insert)| *insert);

        let trans = self.transaction()?;

        let kind = item.name_str();

        let mut rows = 0;

        if !inserts.is_empty() {
            let mut vars = "(?, ?, ?),".repeat(inserts.len());
            // Remove trailing comma
            vars.pop();

            let insert = format!(
                "INSERT OR IGNORE INTO collection_item (collection_id, media_type, media_id) VALUES {vars} "
            );

            let mut params = vec![];

            for (insert, _) in inserts {
                params.push(ToSqlOutput::from(insert));
                params.push(ToSqlOutput::from(kind));
                params.push(ToSqlOutput::from(item));
            }

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += trans.execute(&insert, params.as_slice())?;
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!(
                "DELETE FROM collection_item WHERE collection_id IN ({vars}) AND media_type='{kind}' AND media_id=?"
            );

            let params = deletes
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .chain(std::iter::once(ToSqlOutput::from(item)))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += trans.execute(&delete, params.as_slice())?;
        }

        trans.commit()?;

        Ok(rows > 0)
    }

    pub fn toggle_directories(
        &mut self,
        directories: Vec<(Directory, Operation)>,
    ) -> rusqlite::Result<bool> {
        if directories.is_empty() {
            return Ok(false);
        }

        let mut inserts = vec![];
        let mut updates = vec![];
        let mut deletes = vec![];

        for (dir, operation) in directories {
            match operation {
                Operation::Insert => inserts.push(dir),
                Operation::Update => updates.push(dir),
                Operation::Delete => deletes.push(dir),
            }
        }

        let trans = self.transaction()?;

        let mut rows = 0;

        if !inserts.is_empty() {
            let mut vars = "(?, ?, ?, ?),".repeat(inserts.len());
            // Remove trailing comma
            vars.pop();

            let insert = format!(
                "INSERT OR IGNORE INTO directory (id, path, active, media_type) VALUES {vars}"
            );

            let mut params = vec![];

            for dir in inserts {
                params.push(ToSqlOutput::from(dir.id));
                params.push(ToSqlOutput::from(dir.path));
                params.push(ToSqlOutput::from(dir.active));
                params.push(ToSqlOutput::from(dir.media_type));
            }

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += trans.execute(&insert, params.as_slice())?;
        }

        if !updates.is_empty() {
            let vars = repeat(updates.len());
            let sql = "UPDATE OR IGNORE directory SET path=:path, active=:active, media_type=:type WHERE id=:id";

            let mut params = Vec::with_capacity(updates.len());
            let delete_movies = format!("DELETE FROM movie where directory in ({vars})");
            let delete_shows = format!("DELETE FROM tv_show where directory in ({vars})");

            for dir in updates {
                params.push(ToSqlOutput::from(dir.id));
                rows += trans.execute(
                    sql,
                    &[
                        (":id", &ToSqlOutput::from(dir.id)),
                        (":path", &ToSqlOutput::from(dir.path)),
                        (":active", &ToSqlOutput::from(dir.active)),
                        (":type", &ToSqlOutput::from(dir.media_type)),
                    ],
                )?;
            }

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += trans.execute(&delete_movies, params.as_slice())?;
            rows += trans.execute(&delete_shows, params.as_slice())?;
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!("DELETE FROM directory WHERE id IN ({vars})");

            let params = deletes
                .into_iter()
                .map(|dir| ToSqlOutput::from(dir.id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += trans.execute(&delete, params.as_slice())?;
        }

        trans.commit()?;

        Ok(rows > 0)
    }

    pub fn search<T>(
        &self,
        term: String,
        filter: Option<SearchFilter>,
        limit: Option<i32>,
        map: fn(SearchItem) -> T,
    ) -> rusqlite::Result<Vec<T>> {
        let limit = limit.unwrap_or(-1);
        let filter = filter.map(|filter| filter.query()).unwrap_or_default();

        let sql = format!(
            "SELECT
            i.media_type,
            i.media_id,
            i.poster,
            f.name,
            snippet(media_fts, 1, '***', '***', '...', 16) as snippet,
            f.tags
            FROM media_fts f
            INNER JOIN media_fts_index i on f.rowid = i.rowid
            WHERE media_fts MATCH :term {filter} AND NOT i.removed
            ORDER BY rank
            LIMIT {limit}"
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(&[(":term", &ToSqlOutput::from(term))], |row| {
                SearchItem::from_row(row).map(map)
            })?
            .collect()
    }

    pub fn get_item_membership_ids(&self, item: ItemId) -> rusqlite::Result<Vec<CollectionId>> {
        let kind = item.name_str();
        let sql = format!(
            "SELECT collection_item.collection_id FROM collection_item WHERE media_type='{kind}' AND media_id=:id"
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement
            .query_map(
                &[(":id", &ToSqlOutput::from(item))],
                CollectionId::from_member,
            )?
            .collect()
    }

    pub fn get_random(&self) -> rusqlite::Result<ItemId> {
        use rand::{seq::SliceRandom, thread_rng};

        let mut rng = thread_rng();
        let media = [0, 1, 2, 3].choose(&mut rng).unwrap();

        let (getter, media, removed) = match media {
            0 => (Self::MOVIE_QUERY, "movie", "removed"),
            1 => (Self::SHOW_QUERY, "show", "removed"),
            2 => (Self::SEASON_QUERY, "season", "season.removed"),
            3 => (Self::EPISODE_QUERY, "episode", "removed"),
            _ => unreachable!(),
        };

        let table = match media {
            "show" => "tv_show.".to_owned(),
            "episode" => "".to_owned(),
            _ => format!("{media}."),
        };

        let sql = format!(
            "{getter} WHERE NOT {removed} AND {table}progress < 0.85 ORDER BY RANDOM() * (6 - {table}rating) LIMIT 1",
        );

        let mut statement = self.prepare_cached(&sql)?;

        statement.query_row([], |row| ItemId::from_random(row, media))
    }

    pub fn last_watched_episode<'a>(
        &self,
        id: EpisodeId,
        last_watched: ToSqlOutput<'a>,
    ) -> rusqlite::Result<()> {
        let sql = "UPDATE episode SET last_watched=:last_watched WHERE episode.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let _ = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":last_watched", &last_watched),
        ])?;

        Ok(())
    }

    pub fn update_episode_stats(
        &self,
        id: EpisodeId,
        watch_count: u32,
        progress: f32,
        duration: u64,
        subtitle_uri: Option<String>,
    ) -> rusqlite::Result<()> {
        let sql = "UPDATE episode SET watch_count=:watch_count, duration=:duration, progress=:progress, subtitle_uri=:subtitle_uri, last_watched=CURRENT_TIMESTAMP WHERE episode.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let subtitle_uri = subtitle_uri
            .to_sql()
            .expect("Option<String> ToSqlOutput conversion always successful");

        let _ = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":watch_count", &ToSqlOutput::from(watch_count)),
            (":progress", &ToSqlOutput::from(progress)),
            (":duration", &ToSqlOutput::from(duration as isize)),
            (":subtitle_uri", &subtitle_uri),
        ])?;

        Ok(())
    }

    pub fn update_movie_stats(
        &self,
        id: MovieId,
        watch_count: u32,
        progress: f32,
        duration: u64,
        subtitle_uri: Option<String>,
    ) -> rusqlite::Result<usize> {
        let sql = "UPDATE movie SET watch_count=:watch_count, duration=:duration, progress=:progress, subtitle_uri=:subtitle_uri, last_watched=CURRENT_TIMESTAMP WHERE movie.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let subtitle_uri = subtitle_uri
            .to_sql()
            .expect("Option<String> ToSqlOutput conversion always successful");

        let rows = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":watch_count", &ToSqlOutput::from(watch_count)),
            (":progress", &ToSqlOutput::from(progress)),
            (":duration", &ToSqlOutput::from(duration as isize)),
            (":subtitle_uri", &subtitle_uri),
        ])?;

        Ok(rows)
    }

    pub fn last_watched_movie<'a>(
        &self,
        id: MovieId,
        last_watched: ToSqlOutput<'a>,
    ) -> rusqlite::Result<()> {
        let sql = "UPDATE movie SET last_watched=:last_watched WHERE movie.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let _ = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":last_watched", &last_watched),
        ])?;

        Ok(())
    }

    pub fn last_scan<'a>(
        &self,
        id: DirectoryId,
        last_scan: ToSqlOutput<'a>,
    ) -> rusqlite::Result<usize> {
        let sql = "UPDATE directory SET last_scan=:last_scan WHERE directory.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":last_watched", &last_scan),
        ])
    }

    pub fn last_scans<'a>(
        &self,
        ids: Vec<DirectoryId>,
        last_scan: ToSqlOutput<'a>,
    ) -> rusqlite::Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let vars = repeat(ids.len());
        let sql = format!("UPDATE directory SET last_scan=? WHERE id IN ({vars})");

        let mut statement = self.prepare_cached(&sql)?;

        let params = std::iter::once(last_scan)
            .chain(ids.into_iter().map(ToSqlOutput::from))
            .collect::<Vec<_>>();

        let params = params
            .iter()
            .map(|param| param as &dyn ToSql)
            .collect::<Vec<_>>();

        statement.execute(params.as_slice())
    }

    pub fn insert_remove_movies(&self, movies: Vec<(MovieId, bool)>) -> rusqlite::Result<usize> {
        if movies.is_empty() {
            return Ok(0);
        }

        let mut rows = 0;
        let (inserts, deletes): (Vec<(MovieId, bool)>, Vec<_>) =
            movies.into_iter().partition(|(_, insert)| *insert);

        if !inserts.is_empty() {
            let vars = repeat(inserts.len());

            let insert = format!("UPDATE movie SET removed=FALSE WHERE id IN ({vars})");

            let params = inserts
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&insert, params.as_slice())?
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!("UPDATE movie SET removed=TRUE WHERE id IN ({vars})");

            let params = deletes
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&delete, params.as_slice())?
        }

        Ok(rows)
    }

    pub fn insert_remove_shows(&self, shows: Vec<(ShowId, bool)>) -> rusqlite::Result<usize> {
        if shows.is_empty() {
            return Ok(0);
        }

        let mut rows = 0;
        let (inserts, deletes): (Vec<(ShowId, bool)>, Vec<_>) =
            shows.into_iter().partition(|(_, insert)| *insert);

        if !inserts.is_empty() {
            let vars = repeat(inserts.len());

            let insert = format!("UPDATE tv_show SET removed=FALSE WHERE id IN ({vars})");

            let params = inserts
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&insert, params.as_slice())?
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!("UPDATE tv_show SET removed=TRUE WHERE id IN ({vars})");

            let params = deletes
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&delete, params.as_slice())?
        }

        Ok(rows)
    }

    pub fn insert_remove_seasons(&self, seasons: Vec<(SeasonId, bool)>) -> rusqlite::Result<usize> {
        if seasons.is_empty() {
            return Ok(0);
        }

        let mut rows = 0;
        let (inserts, deletes): (Vec<(SeasonId, bool)>, Vec<_>) =
            seasons.into_iter().partition(|(_, insert)| *insert);

        if !inserts.is_empty() {
            let vars = repeat(inserts.len());

            let insert = format!("UPDATE season SET removed=FALSE WHERE id IN ({vars})");

            let params = inserts
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&insert, params.as_slice())?
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!("UPDATE season SET removed=TRUE WHERE id IN ({vars})");

            let params = deletes
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&delete, params.as_slice())?
        }

        Ok(rows)
    }

    pub fn insert_remove_episodes(
        &self,
        episodes: Vec<(EpisodeId, bool)>,
    ) -> rusqlite::Result<usize> {
        if episodes.is_empty() {
            return Ok(0);
        }

        let mut rows = 0;
        let (inserts, deletes): (Vec<(EpisodeId, bool)>, Vec<_>) =
            episodes.into_iter().partition(|(_, insert)| *insert);

        if !inserts.is_empty() {
            let vars = repeat(inserts.len());

            let insert = format!("UPDATE episode SET removed=FALSE WHERE id IN ({vars})");

            let params = inserts
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&insert, params.as_slice())?
        }

        if !deletes.is_empty() {
            let vars = repeat(deletes.len());

            let delete = format!("UPDATE episode SET removed=TRUE WHERE id IN ({vars})");

            let params = deletes
                .into_iter()
                .map(|(id, _)| ToSqlOutput::from(id))
                .collect::<Vec<_>>();

            let params = params
                .iter()
                .map(|param| param as &dyn ToSql)
                .collect::<Vec<_>>();

            rows += self.execute(&delete, params.as_slice())?
        }

        Ok(rows)
    }

    pub fn open_with_dummies(
        path: impl AsRef<Path>,
        dummies: impl AsRef<Path>,
    ) -> crate::error::Result<Database> {
        let exists = path.as_ref().try_exists()?;
        let conn = Database::open(path)?;

        if !exists {
            tracing::debug!("writing DB schema");
            let schema = include_str!("../resources/db/schema.sql");
            conn.execute_batch(schema)?;

            tracing::debug!("writing DB dummies");
            let dummies = read_to_string(dummies)?;
            conn.execute_batch(&dummies)?;
            Ok(conn)
        } else {
            Ok(apply_migration(conn)?)
        }
    }

    pub fn open_with_schema(db: impl AsRef<Path>) -> crate::error::Result<Database> {
        let exists = db.as_ref().try_exists()?;
        let conn = Database::open(db)?;

        if !exists {
            tracing::debug!("writing DB schema");
            let schema = include_str!("../resources/db/schema.sql");
            conn.execute_batch(schema)?;
            Ok(conn)
        } else {
            Ok(apply_migration(conn)?)
        }
    }

    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Database> {
        tracing::debug!("Opening DB connection");
        let conn = rusqlite::Connection::open(path)?;

        Ok(Database { conn })
    }
}

fn apply_migration(mut db: Database) -> rusqlite::Result<Database> {
    tracing::debug!("Initiating DB migration application");
    let curr_version: u64 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let mut new_version = curr_version;
    let trans = db.transaction()?;

    for migration in MIGRATIONS {
        if migration.version <= curr_version {
            continue;
        }

        tracing::debug!("Applying DB migration at version {}", migration.version);
        trans.execute_batch(migration.sql)?;
        new_version = migration.version;
    }

    trans.execute(&format!("PRAGMA user_version = {new_version}"), [])?;
    trans.commit()?;

    tracing::debug!("Ending DB migration application");
    Ok(db)
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
    InsertTrigger,
    DeleteTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Insert = 0,
    Update = 1,
    Delete = 2,
}

#[derive(Debug, Clone, PartialEq)]
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
            Ok(mut statement) => match statement.execute(params.as_slice()) {
                Ok(rows) => Ok(Success {
                    id,
                    table,
                    op,
                    rows,
                }),
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
    pub rows: usize,
}

impl Success {
    pub fn log(&self) {
        tracing::debug!(
            "Success {:?} on {:?} with {} rows changed",
            self.op,
            self.table,
            self.rows
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Failure<'a> {
    pub query: Query<'a>,
    pub error: Box<rusqlite::Error>,
}

impl<'a> Failure<'a> {
    pub fn log(&self) {
        tracing::warn!("Failed {:?} on {:?}", self.query.op, self.query.table)
    }
}

#[derive(Debug)]
/// The accumulated results of executing a [`Batch`].
pub struct BatchResult<'a> {
    pub successes: Vec<Success>,
    pub failures: Vec<Failure<'a>>,
}

impl BatchResult<'_> {
    pub fn empty() -> Self {
        Self {
            successes: vec![],
            failures: vec![],
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.successes.extend(other.successes);
        self.failures.extend(other.failures);
    }

    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }

    pub fn has_successes(&self) -> bool {
        !self.successes.is_empty()
    }

    pub fn log(&self) {
        for success in &self.successes {
            success.log()
        }

        for failed in &self.failures {
            failed.log()
        }
    }
}

fn repeat(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}
