use crate::db::Database;
use crate::models::{EpisodeId, MovieId, SeasonId, ShowId};
use reqwest::{
    Client, ClientBuilder,
    header::{ACCEPT, HeaderMap},
};
use serde::Deserialize;
use std::ops::Deref;
use std::path::Path;
use std::sync::LazyLock;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time;

use rusqlite::types::{ToSqlOutput, ValueRef};

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".try_into().unwrap());

    ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Cannot build request client")
});

#[derive(Deserialize, Debug, Clone)]
struct ImageConfig {
    base_url: String,
    backdrop_sizes: Vec<String>,
    poster_sizes: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBId {
    id: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct Genres {
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBMovie {
    id: u32,
    backdrop_path: String,
    genres: Vec<Genres>,
    overview: String,
    poster_path: String,
    release_date: String,
    title: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBShow {
    id: u32,
    backdrop_path: String,
    genres: Vec<Genres>,
    overview: String,
    poster_path: String,
    name: String,
    first_air_date: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBSeason {
    id: u32,
    air_date: String,
    name: String,
    overview: String,
    poster_path: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBEpisode {
    id: u32,
    air_date: String,
    name: String,
    overview: String,
    still_path: String,
    episode_number: u32,
}

async fn get_config(auth: &str) -> reqwest::Result<ImageConfig> {
    #[derive(Deserialize)]
    struct Response {
        images: ImageConfig,
    }

    let response: Response = CLIENT
        .get("https://api.themoviedb.org/3/configuration")
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())?
        .json()
        .await?;

    Ok(response.images)
}

async fn search_item(auth: &str, snippet: impl AsRef<str>, movie: bool) -> Option<TMDBId> {
    #[derive(Deserialize)]
    struct Response {
        results: Vec<TMDBId>,
    }

    let item = if movie { "movie" } else { "tv" };

    let response: Response = CLIENT
        .get(format!("https://api.themoviedb.org/3/search/{item}"))
        .query(&[("query", snippet.as_ref())])
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| {
            tracing::error!(
                "Search fetch error on {}.\n Error {error}",
                snippet.as_ref()
            )
        })
        .ok()?
        .json()
        .await
        .inspect_err(|error| {
            tracing::error!(
                "Search fetch error on {}.\n Error {error}",
                snippet.as_ref()
            )
        })
        .ok()?;

    response.results.first().cloned()
}

async fn get_movie(auth: &str, id: TMDBId) -> Option<TMDBMovie> {
    let response: TMDBMovie = CLIENT
        .get(format!("https://api.themoviedb.org/3/movie/{}", id.id))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| tracing::error!("Get movie error on {}.\n Error {error}", id.id))
        .ok()?
        .json()
        .await
        .inspect_err(|error| tracing::error!("Get movie error on {}.\n Error {error}", id.id))
        .ok()?;

    Some(response)
}

async fn get_show(auth: &str, id: TMDBId) -> Option<TMDBShow> {
    let response: TMDBShow = CLIENT
        .get(format!("https://api.themoviedb.org/3/tv/{}", id.id))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| tracing::error!("Get show error on {}.\n Error {error}", id.id))
        .ok()?
        .json()
        .await
        .inspect_err(|error| tracing::error!("Get show error on {}.\n Error {error}", id.id))
        .ok()?;

    Some(response)
}

async fn get_season(auth: &str, show: TMDBId, number: u32) -> Option<TMDBSeason> {
    let response: TMDBSeason = CLIENT
        .get(format!(
            "https://api.themoviedb.org/3/tv/{}/season/{number}",
            show.id
        ))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| tracing::error!("Get season error.\n Error {error}"))
        .ok()?
        .json()
        .await
        .inspect_err(|error| tracing::error!("Get season error.\n Error {error}"))
        .ok()?;

    Some(response)
}

async fn get_episode(auth: &str, show: TMDBId, season: u32, number: u32) -> Option<TMDBEpisode> {
    let response: TMDBEpisode = CLIENT
        .get(format!(
            "https://api.themoviedb.org/3/tv/{}/season/{season}/episode/{number}",
            show.id
        ))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| tracing::error!("Get episode error.\n Error {error}"))
        .ok()?
        .json()
        .await
        .inspect_err(|error| tracing::error!("Get episode error.\n Error {error}"))
        .ok()?;

    Some(response)
}

async fn download(
    auth: &str,
    config: &ImageConfig,
    image: &str,
    poster: bool,
    path: impl AsRef<Path>,
) -> bool {
    let size = if poster {
        let len = config.poster_sizes.len();
        config
            .poster_sizes
            .get(len.saturating_sub(2))
            .map(|size| size.deref())
            .unwrap_or("original")
    } else {
        let len = config.backdrop_sizes.len();
        config
            .backdrop_sizes
            .get(len.saturating_sub(2))
            .map(|size| size.deref())
            .unwrap_or("original")
    };

    let url = format!("{}{size}/{image}", config.base_url);

    img_download(auth, url, path).await
}

async fn img_download(auth: &str, url: String, path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    let bytes = CLIENT
        .get(url)
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| {
            tracing::error!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok();

    let Some(bytes) = bytes else {
        return false;
    };

    let Some(bytes) = bytes
        .bytes()
        .await
        .inspect_err(|error| {
            tracing::error!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
    else {
        return false;
    };

    let Some(file) = File::create(path)
        .await
        .inspect_err(|error| {
            tracing::error!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
    else {
        return false;
    };
    let mut writer = BufWriter::new(file);

    if writer
        .write(bytes.deref())
        .await
        .inspect_err(|error| {
            tracing::error!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
        .is_none()
    {
        return false;
    };
    if writer
        .flush()
        .await
        .inspect_err(|error| {
            tracing::error!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
        .is_none()
    {
        return false;
    };

    true
}

pub async fn fetcher(
    mut auth_rx: mpsc::Receiver<String>,
    db: impl AsRef<Path>,
    images_path: impl AsRef<Path>,
    auth: String,
    interval: std::time::Duration,
) {
    let mut db = match Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            tracing::error!("fetcher Db Error \n{error}");
            return;
        }
    };

    let mut auth = auth;
    let mut image_config = get_config(&auth)
        .await
        .inspect_err(|error| {
            tracing::error!("\nGetting image config with auth {auth} failed. \nError{error}\n",)
        })
        .ok();

    loop {
        if !auth_rx.is_empty()
            && let Some(new_auth) = auth_rx.recv().await
        {
            auth = new_auth;
            image_config = get_config(&auth)
                .await
                .inspect_err(|error| {
                    tracing::error!(
                        "\nGetting image config with auth {auth} failed. \nError{error}\n",
                    )
                })
                .ok();
        }

        let Some(image_config) = image_config.as_ref() else {
            time::sleep(interval).await;
            continue;
        };

        movies::handle_movies(&auth, &mut db, image_config, 40, &images_path).await;
        shows::handle_shows(&auth, &mut db, image_config, 40, &images_path).await;
        seasons::handle_seasons(&auth, &mut db, image_config, 20, &images_path).await;
        episodes::handle_episodes(&auth, &mut db, image_config, 20, &images_path).await;

        time::sleep(interval).await;
    }
}

mod movies {
    use super::*;

    struct PendingData {
        id: MovieId,
        name: String,
    }

    struct PendingImage {
        id: MovieId,
        poster: String,
        backdrop: String,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        let sql = "SELECT movie.id, movie.name FROM movie WHERE movie.tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = MovieId::from_row(row)?;
                let name = row.get::<_, String>("name")?;
                Ok(PendingData { id, name })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        let sql = "SELECT movie.id, movie.poster, movie.backdrop FROM movie WHERE movie.tmdb_id IS NOT NULL AND NOT movie.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = MovieId::from_row(row)?;
                let poster = row.get::<_, String>("poster")?;
                let backdrop = row.get::<_, String>("backdrop")?;
                Ok(PendingImage {
                    id,
                    poster,
                    backdrop,
                })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        movies: Vec<(MovieId, TMDBMovie)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE movie SET backdrop=:backdrop, poster=:poster, tmdb_id=:tmdb_id, tags=:tags, synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (movie, data) in movies {
            let tags = data
                .genres
                .iter()
                .map(|genre| genre.name.clone())
                .collect::<Vec<_>>();

            let tags = tags.join(", ");

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(data.id)),
                    (":id", &ToSqlOutput::from(movie)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (":overview", &ToSqlOutput::from(data.overview)),
                    (":release", &ToSqlOutput::from(data.release_date)),
                    (":name", &ToSqlOutput::from(data.title)),
                    (":poster", &ToSqlOutput::from(data.poster_path)),
                    (":backdrop", &ToSqlOutput::from(data.backdrop_path)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(MovieId, String, String)>,
    ) -> rusqlite::Result<usize> {
        use rusqlite::types::ToSqlOutput;

        let trans = db.transaction()?;
        let sql =
            "UPDATE movie SET  poster=:poster, backdrop=:backdrop, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &ToSqlOutput::from(poster)),
                    (":backdrop", &ToSqlOutput::from(backdrop)),
                    (":fetched", &ToSqlOutput::from(true)),
                ],
            )?;
        }

        trans.commit()?;

        Ok(rows)
    }

    pub async fn handle_movies(
        auth: &str,
        db: &mut Database,
        image_config: &ImageConfig,
        limit: u8,
        images_path: impl AsRef<Path>,
    ) {
        let images_path = images_path.as_ref();

        if let Ok(pending) = fetch_data(db, limit).inspect_err(|error| tracing::error!("{error}")) {
            let mut data = Vec::with_capacity(pending.len());

            for movie in pending {
                let Some(id) = search_item(auth, &movie.name, true).await else {
                    continue;
                };

                if let Some(res) = get_movie(auth, id).await {
                    data.push((movie.id, res))
                };
            }

            if let Err(error) = insert_data(db, data) {
                tracing::error!("{error}");
            }
        };

        if let Ok(pending) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(pending.len());

            for movie in pending {
                let poster_path = images_path.join(format!("{}_poster.jpg", movie.id));
                let poster = download(auth, image_config, &movie.poster, true, &poster_path).await;

                let backdrop_path = images_path.join(format!("{}_backdrop.jpg", movie.id));
                let backdrop =
                    download(auth, image_config, &movie.backdrop, false, &backdrop_path).await;

                if poster && backdrop {
                    images.push((
                        movie.id,
                        poster_path.display().to_string(),
                        backdrop_path.display().to_string(),
                    ))
                }
            }

            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}

mod shows {
    use super::*;

    struct PendingData {
        id: ShowId,
        name: String,
    }

    struct PendingImage {
        id: ShowId,
        poster: String,
        backdrop: String,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        let sql = "SELECT tv_show.id, tv_show.name FROM tv_show WHERE tv_show.tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = ShowId::from_row(row)?;
                let name = row.get::<_, String>("name")?;
                Ok(PendingData { id, name })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        let sql = "SELECT tv_show.id, tv_show.poster, tv_show.backdrop FROM tv_show WHERE tv_show.tmdb_id IS NOT NULL AND NOT tv_show.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = ShowId::from_row(row)?;
                let poster = row.get::<_, String>("poster")?;
                let backdrop = row.get::<_, String>("backdrop")?;
                Ok(PendingImage {
                    id,
                    poster,
                    backdrop,
                })
            })?
            .collect()
    }

    fn insert_data(db: &mut Database, shows: Vec<(ShowId, TMDBShow)>) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE tv_show SET backdrop=:backdrop, poster=:poster, tmdb_id=:tmdb_id, tags=:tags, synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (show, data) in shows {
            let tags = data
                .genres
                .iter()
                .map(|genre| genre.name.clone())
                .collect::<Vec<_>>();

            let tags = tags.join(", ");

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(data.id)),
                    (":id", &ToSqlOutput::from(show)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (":overview", &ToSqlOutput::from(data.overview)),
                    (":release", &ToSqlOutput::from(data.first_air_date)),
                    (":name", &ToSqlOutput::from(data.name)),
                    (":poster", &ToSqlOutput::from(data.poster_path)),
                    (":backdrop", &ToSqlOutput::from(data.backdrop_path)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(ShowId, String, String)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql =
            "UPDATE tv_show SET  poster=:poster, backdrop=:backdrop, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &ToSqlOutput::from(poster)),
                    (":backdrop", &ToSqlOutput::from(backdrop)),
                    (":fetched", &ToSqlOutput::from(true)),
                ],
            )?;
        }

        trans.commit()?;

        Ok(rows)
    }

    pub async fn handle_shows(
        auth: &str,
        db: &mut Database,
        image_config: &ImageConfig,
        limit: u8,
        images_path: impl AsRef<Path>,
    ) {
        let images_path = images_path.as_ref();

        if let Ok(pending) = fetch_data(db, limit).inspect_err(|error| tracing::error!("{error}")) {
            let mut data = Vec::with_capacity(pending.len());

            for show in pending {
                let Some(id) = search_item(auth, &show.name, false).await else {
                    continue;
                };

                if let Some(res) = get_show(auth, id).await {
                    data.push((show.id, res))
                };
            }

            if let Err(error) = insert_data(db, data) {
                tracing::error!("{error}");
            }
        };

        if let Ok(pending) = fetch_image(db, limit) {
            let mut images = Vec::with_capacity(pending.len());

            for show in pending {
                let poster_path = images_path.join(format!("{}_poster.jpg", show.id));
                let poster = download(auth, image_config, &show.poster, true, &poster_path).await;

                let backdrop_path = images_path.join(format!("{}_backdrop.jpg", show.id));
                let backdrop =
                    download(auth, image_config, &show.backdrop, false, &backdrop_path).await;
                if poster && backdrop {
                    images.push((
                        show.id,
                        poster_path.display().to_string(),
                        backdrop_path.display().to_string(),
                    ))
                }
            }

            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}

mod seasons {
    use super::*;

    struct PendingData {
        id: SeasonId,
        show: TMDBId,
        number: u32,
    }

    struct PendingImage {
        id: SeasonId,
        poster: String,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        let sql = "SELECT season.id, season.season_number, tv_show.tmdb_id AS show_tmdb_id FROM season INNER JOIN tv_show ON season.show_id=tv_show.id AND tv_show.tmdb_id NOT NULL AND season.tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = SeasonId::from_row(row)?;
                let show = row.get::<_, u32>("show_tmdb_id")?;
                let show = TMDBId { id: show };
                let number = row.get::<_, u32>("season_number")?;
                Ok(PendingData { show, id, number })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        let sql = "SELECT season.id, season.poster FROM season WHERE season.tmdb_id IS NOT NULL AND NOT season.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = SeasonId::from_row(row)?;
                let poster = row.get::<_, String>("poster")?;
                Ok(PendingImage { id, poster })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        seasons: Vec<(SeasonId, TMDBSeason)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE season SET tmdb_id=:tmdb_id, poster=:poster, synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (season, data) in seasons {
            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(data.id)),
                    (":id", &ToSqlOutput::from(season)),
                    (":overview", &ToSqlOutput::from(data.overview)),
                    (":release", &ToSqlOutput::from(data.air_date)),
                    (":name", &ToSqlOutput::from(data.name)),
                    (":poster", &ToSqlOutput::from(data.poster_path)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(SeasonId, String)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE season SET poster=:poster, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":fetched", &ToSqlOutput::from(true)),
                    (":poster", &ToSqlOutput::from(poster)),
                ],
            )?;
        }

        trans.commit()?;

        Ok(rows)
    }

    pub async fn handle_seasons(
        auth: &str,
        db: &mut Database,
        image_config: &ImageConfig,
        limit: u8,
        images_path: impl AsRef<Path>,
    ) {
        let images_path = images_path.as_ref();
        if let Ok(seasons) = fetch_data(db, limit).inspect_err(|error| tracing::error!("{error}")) {
            let mut data = Vec::with_capacity(seasons.len());

            for season in seasons {
                if let Some(res) = get_season(auth, season.show, season.number).await {
                    data.push((season.id, res))
                }
            }

            if let Err(error) = insert_data(db, data) {
                tracing::error!("{error}");
            }
        };

        if let Ok(seasons) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(seasons.len());

            for season in seasons {
                let poster_path = images_path.join(format!("{}_poster.jpg", season.id));
                let poster = download(auth, image_config, &season.poster, true, &poster_path).await;

                if poster {
                    images.push((season.id, poster_path.display().to_string()))
                }
            }

            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}

mod episodes {
    use super::*;

    struct PendingData {
        id: EpisodeId,
        show: TMDBId,
        season: u32,
        number: u32,
    }

    struct PendingImage {
        id: EpisodeId,
        poster: String,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        let sql = "SELECT id, episode_number, season_number, show_tmdb_id FROM get_episode_data WHERE show_tmdb_id NOT NULL AND tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = EpisodeId::from_row(row)?;
                let show = row.get::<_, u32>("show_tmdb_id")?;
                let show = TMDBId { id: show };
                let season = row.get::<_, u32>("season_number")?;
                let number = row.get::<_, u32>("episode_number")?;
                Ok(PendingData {
                    show,
                    id,
                    season,
                    number,
                })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        let sql = "SELECT episode.id, episode.poster FROM episode WHERE episode.tmdb_id IS NOT NULL AND NOT episode.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = EpisodeId::from_row(row)?;
                let poster = row.get::<_, String>("poster")?;
                Ok(PendingImage { id, poster })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        episodes: Vec<(EpisodeId, TMDBEpisode)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE episode SET tmdb_id=:tmdb_id, poster=:poster, synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (episode, data) in episodes {
            let name = format!("{:02} {}", data.episode_number, data.name);
            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(data.id)),
                    (":id", &ToSqlOutput::from(episode)),
                    (":overview", &ToSqlOutput::from(data.overview)),
                    (":release", &ToSqlOutput::from(data.air_date)),
                    (":name", &ToSqlOutput::from(name)),
                    (":poster", &ToSqlOutput::from(data.still_path)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(EpisodeId, String)>,
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE episode SET poster=:poster, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":fetched", &ToSqlOutput::from(true)),
                    (":poster", &ToSqlOutput::from(poster)),
                ],
            )?;
        }

        trans.commit()?;

        Ok(rows)
    }

    pub async fn handle_episodes(
        auth: &str,
        db: &mut Database,
        image_config: &ImageConfig,
        limit: u8,
        images_path: impl AsRef<Path>,
    ) {
        let images_path = images_path.as_ref();
        if let Ok(episodes) = fetch_data(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut data = Vec::with_capacity(episodes.len());

            for episode in episodes {
                if let Some(res) =
                    get_episode(auth, episode.show, episode.season, episode.number).await
                {
                    data.push((episode.id, res))
                }
            }

            if let Err(error) = insert_data(db, data) {
                tracing::error!("{error}");
                return;
            }
        };

        if let Ok(episodes) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(episodes.len());

            for episode in episodes {
                let poster_path = images_path.join(format!("{}_poster.jpg", episode.id));
                let poster =
                    download(auth, image_config, &episode.poster, true, &poster_path).await;

                if poster {
                    images.push((episode.id, poster_path.display().to_string()))
                }
            }
            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}
