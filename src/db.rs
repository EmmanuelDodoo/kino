use crate::models::{
    CollectionId, Directory, DirectoryId, EComment, ECommentId, EpisodeId, MComment, MCommentId,
    Movie, MovieId, SearchItem, SeasonId, Show, ShowId,
    collection::{self, ItemId},
};

use crate::filter::{self, Filter, search::SearchFilter};
use crate::sort::{self, Sort};

use rusqlite::{Connection, Result, Row, ToSql, params_from_iter, types::ToSqlOutput};
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
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
            .unwrap_or_default();

        let sql = format!(
            "{} {filter} {sort} LIMIT :limit OFFSET :offset",
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
            .map(|query| format!("WHERE {query}"))
            .unwrap_or_default();
        let sort = sort
            .query(None)
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
            .unwrap_or_default();

        let sql = format!(
            "{} {} {} LIMIT :limit OFFSET :offset",
            Self::SHOW_QUERY,
            filter,
            sort
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
                map,
            )?
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
                map,
            )?
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
            .map(|query| format!("ORDER BY {query} NULLS LAST"))
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
            "{} ORDER BY {} NULLS LAST",
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
            "SELECT * FROM collection WHERE collection.id IN ({vars}) ORDER BY {} NULLS LAST LIMIT {limit} OFFSET {offset}",
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
                .map(|query| format!("ORDER BY {query} NULLS LAST"))
                .unwrap_or_default();
            let sql = format!(
                "{} WHERE movie.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
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
                .map(|query| format!("ORDER BY {query} NULLS LAST"))
                .unwrap_or_default();

            let vars = repeat(shows.len());
            let sql = format!(
                "{} WHERE tv_show.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
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
                .map(|query| format!("ORDER BY {query} NULLS LAST"))
                .unwrap_or_default();

            let vars = repeat(seasons.len());
            let sql = format!(
                "{} WHERE season.id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
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
                .map(|query| format!("ORDER BY {query} NULLS LAST"))
                .unwrap_or_default();

            let vars = repeat(episodes.len());

            let sql = format!(
                "{} WHERE id IN ({vars}) {filter} {sort} LIMIT {limit} OFFSET {offset}",
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
        directories: Vec<(Directory, bool)>,
    ) -> rusqlite::Result<bool> {
        if directories.is_empty() {
            return Ok(false);
        }

        let mut inserts = vec![];
        let mut deletes = vec![];

        for (dir, insert) in directories {
            if insert {
                inserts.push(dir)
            } else {
                deletes.push(dir);
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
            WHERE media_fts MATCH :term {filter}
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

        let (getter, media) = match media {
            0 => (Self::MOVIE_QUERY, "movie"),
            1 => (Self::SHOW_QUERY, "show"),
            2 => (Self::SEASON_QUERY, "season"),
            3 => (Self::EPISODE_QUERY, "episode"),
            _ => unreachable!(),
        };

        let table = match media {
            "show" => "tv_show.".to_owned(),
            "episode" => "".to_owned(),
            _ => format!("{media}."),
        };

        let sql = format!(
            "{getter} WHERE {table}progress < 0.85 ORDER BY RANDOM() * (6 - {table}rating) LIMIT 1",
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
    ) -> rusqlite::Result<()> {
        let sql = "UPDATE episode SET watch_count=:watch_count, duration=:duration, progress=:progress WHERE episode.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let _ = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":watch_count", &ToSqlOutput::from(watch_count)),
            (":progress", &ToSqlOutput::from(progress)),
            (":duration", &ToSqlOutput::from(duration as isize)),
        ])?;

        Ok(())
    }

    pub fn update_movie_stats(
        &self,
        id: MovieId,
        watch_count: u32,
        progress: f32,
        duration: u64,
    ) -> rusqlite::Result<usize> {
        let sql = "UPDATE movie SET watch_count=:watch_count, duration=:duration, progress=:progress WHERE movie.id=:id";

        let mut statement = self.prepare_cached(sql)?;

        let rows = statement.execute(&[
            (":id", &ToSqlOutput::from(id)),
            (":watch_count", &ToSqlOutput::from(watch_count)),
            (":progress", &ToSqlOutput::from(progress)),
            (":duration", &ToSqlOutput::from(duration as isize)),
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

    pub fn open_test_db(path: impl AsRef<std::path::Path>) -> rusqlite::Result<Database> {
        let conn = Database::open(path)?;

        let schema = include_str!("../schema.sql");
        conn.execute_batch(schema)?;

        let dummy = include_str!("../dummy.txt");

        conn.execute_batch(dummy)?;

        Ok(conn)
    }

    pub fn open_with_schema(path: impl AsRef<std::path::Path>) -> rusqlite::Result<Database> {
        let conn = Database::open(path)?;
        let schema = include_str!("../schema.sql");
        conn.execute_batch(schema)?;

        Ok(conn)
    }

    pub fn open(path: impl AsRef<std::path::Path>) -> rusqlite::Result<Database> {
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
}

#[derive(Debug)]
pub struct BatchError<'a> {
    pub residue: Box<Batch<'a>>,
    pub successes: Vec<Success>,
    pub failures: Vec<Failure<'a>>,
    pub error: Box<rusqlite::Error>,
}

fn repeat(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}
