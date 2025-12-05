use crate::db::Database;
use crate::models::{EpisodeId, MovieId, SeasonId, ShowId};
use reqwest::{
    Client, ClientBuilder, Request,
    header::{ACCEPT, AUTHORIZATION, HeaderMap},
};
use serde::Deserialize;
use std::ops::Deref;
use std::path::Path;
use std::sync::LazyLock;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time;

use rusqlite::{
    Result, Row,
    types::{ToSqlOutput, Value, ValueRef},
};

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
    secure_base_url: String,
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
    season_number: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBEpisode {
    air_date: String,
    id: u32,
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
            eprintln!(
                "Search fetch error on {}.\n Error {error}",
                snippet.as_ref()
            )
        })
        .ok()?
        .json()
        .await
        .inspect_err(|error| {
            eprintln!(
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
        .inspect_err(|error| eprintln!("Get movie error on {}.\n Error {error}", id.id))
        .ok()?
        .json()
        .await
        .inspect_err(|error| eprintln!("Get movie error on {}.\n Error {error}", id.id))
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
        .inspect_err(|error| eprintln!("Get show error on {}.\n Error {error}", id.id))
        .ok()?
        .json()
        .await
        .inspect_err(|error| eprintln!("Get show error on {}.\n Error {error}", id.id))
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
        .inspect_err(|error| eprintln!("Get season error.\n Error {error}"))
        .ok()?
        .json()
        .await
        .inspect_err(|error| eprintln!("Get season error.\n Error {error}"))
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
        .inspect_err(|error| eprintln!("Get episode error.\n Error {error}"))
        .ok()?
        .json()
        .await
        .inspect_err(|error| eprintln!("Get episode error.\n Error {error}"))
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
            eprintln!(
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
            eprintln!(
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
            eprintln!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
    else {
        return false;
    };
    let mut writer = BufWriter::new(file);

    if let None = writer
        .write(bytes.deref())
        .await
        .inspect_err(|error| {
            eprintln!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
    {
        return false;
    };
    if let None = writer
        .flush()
        .await
        .inspect_err(|error| {
            eprintln!(
                "Image download error on {}: Error \n{error}",
                path.display()
            )
        })
        .ok()
    {
        return false;
    };

    true
}

pub async fn fetcher(
    mut auth_rx: mpsc::Receiver<String>,
    db: impl AsRef<Path>,
    images_path: impl AsRef<Path>,
    interval: std::time::Duration,
) {
    let mut db = match Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("fetcher Db Error \n{error}");
            return;
        }
    };

    let mut auth = String::default();
    let mut image_config = None;

    loop {
        if !auth_rx.is_empty() {
            if let Some(new_auth) = auth_rx.recv().await {
                auth = new_auth;
                image_config = get_config(&auth)
                    .await
                    .inspect_err(|error| {
                        eprintln!(
                            "\nGetting image config with auth {auth} failed. \nError{error}\n",
                        )
                    })
                    .ok();
            }
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

    struct PendingMovie {
        id: MovieId,
        name: String,
    }

    fn fetch(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingMovie>> {
        let sql = "SELECT movie.id, movie.name FROM movie WHERE movie.tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = MovieId::from_row(row)?;
                let name = row.get::<_, String>("name")?;
                Ok(PendingMovie { id, name })
            })?
            .collect()
    }

    fn insert_data(db: &mut Database, movies: &[(MovieId, TMDBMovie)]) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE movie SET tmdb_id=:tmdb_id, tags=:tags, synopsis=:overview, release=:release, name=:name WHERE id=:id";
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
                    (":id", &ToSqlOutput::from(*movie)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (
                        ":overview",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.overview)),
                    ),
                    (
                        ":release",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.release_date)),
                    ),
                    (
                        ":name",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.title)),
                    ),
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
        use rusqlite::types::{ToSqlOutput, Value, ValueRef};

        let trans = db.transaction()?;
        let sql = "UPDATE movie SET  poster=:poster, backdrop=:backdrop WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &ToSqlOutput::from(poster)),
                    (":backdrop", &ToSqlOutput::from(backdrop)),
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
        let Ok(movies) = fetch(db, limit).inspect_err(|error| eprintln!("{error}")) else {
            return;
        };

        let mut data = Vec::with_capacity(movies.len());

        for movie in movies {
            let Some(id) = search_item(auth, &movie.name, true).await else {
                continue;
            };

            if let Some(res) = get_movie(auth, id).await {
                data.push((movie.id, res))
            };
        }

        if let Err(error) = insert_data(db, &data) {
            eprintln!("{error}");
            return;
        }

        let mut images = Vec::with_capacity(data.len());

        for (movie, data) in data {
            let poster_path = images_path.join(format!("{movie}_poster.jpg"));
            let poster = download(auth, image_config, &data.poster_path, true, &poster_path).await;

            let backdrop_path = images_path.join(format!("{movie}_backdrop.jpg"));
            let backdrop = download(
                auth,
                image_config,
                &data.backdrop_path,
                false,
                &backdrop_path,
            )
            .await;

            if poster && backdrop {
                images.push((
                    movie,
                    poster_path.display().to_string(),
                    backdrop_path.display().to_string(),
                ))
            }
        }

        if let Err(error) = insert_images(db, images) {
            eprintln!("{error}");
            return;
        }
    }
}

mod shows {
    use super::*;

    struct PendingShow {
        id: ShowId,
        name: String,
    }

    fn fetch(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingShow>> {
        let sql = "SELECT tv_show.id, tv_show.name FROM tv_show WHERE tv_show.tmdb_id IS NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = ShowId::from_row(row)?;
                let name = row.get::<_, String>("name")?;
                Ok(PendingShow { id, name })
            })?
            .collect()
    }

    fn insert_data(db: &mut Database, shows: &[(ShowId, TMDBShow)]) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE tv_show SET tmdb_id=:tmdb_id, tags=:tags, synopsis=:overview, release=:release, name=:name WHERE id=:id";
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
                    (":id", &ToSqlOutput::from(*show)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (
                        ":overview",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.overview)),
                    ),
                    (
                        ":release",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.first_air_date)),
                    ),
                    (":name", &ToSqlOutput::Borrowed(ValueRef::from(&*data.name))),
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
        let sql = "UPDATE tv_show SET  poster=:poster, backdrop=:backdrop WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &ToSqlOutput::from(poster)),
                    (":backdrop", &ToSqlOutput::from(backdrop)),
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
        let Ok(shows) = fetch(db, limit).inspect_err(|error| eprintln!("{error}")) else {
            return;
        };

        let mut data = Vec::with_capacity(shows.len());

        for show in shows {
            let Some(id) = search_item(auth, &show.name, false).await else {
                continue;
            };

            if let Some(res) = get_show(auth, id).await {
                data.push((show.id, res))
            };
        }

        if let Err(error) = insert_data(db, &data) {
            eprintln!("{error}");
            return;
        }

        let mut images = Vec::with_capacity(data.len());

        for (show, data) in data {
            let poster_path = images_path.join(format!("{show}_poster.jpg"));
            let poster = download(auth, image_config, &data.poster_path, true, &poster_path).await;

            let backdrop_path = images_path.join(format!("{show}_backdrop.jpg"));
            let backdrop = download(
                auth,
                image_config,
                &data.backdrop_path,
                false,
                &backdrop_path,
            )
            .await;

            if poster && backdrop {
                images.push((
                    show,
                    poster_path.display().to_string(),
                    backdrop_path.display().to_string(),
                ))
            }
        }

        if let Err(error) = insert_images(db, images) {
            eprintln!("{error}");
            return;
        }
    }
}

mod seasons {
    use super::*;

    struct PendingSeason {
        id: SeasonId,
        show: TMDBId,
        number: u32,
    }

    fn fetch(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingSeason>> {
        let sql = "SELECT season.id, season.season_number, tv_show.tmdb_id AS tmdb_id FROM season INNER JOIN tv_show ON season.show_id=tv_show.id AND tv_show.tmdb_id NOT NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = SeasonId::from_row(row)?;
                let show = row.get::<_, u32>("tmdb_id")?;
                let show = TMDBId { id: show };
                let number = row.get::<_, u32>("season_number")?;
                Ok(PendingSeason { show, id, number })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        seasons: &[(SeasonId, TMDBSeason)],
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql = "UPDATE season SET synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (season, data) in seasons {
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(*season)),
                    (
                        ":overview",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.overview)),
                    ),
                    (
                        ":release",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.air_date)),
                    ),
                    (":name", &ToSqlOutput::Borrowed(ValueRef::from(&*data.name))),
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
        let Ok(seasons) = fetch(db, limit).inspect_err(|error| eprintln!("{error}")) else {
            return;
        };

        let mut data = Vec::with_capacity(seasons.len());

        for season in seasons {
            if let Some(res) = get_season(auth, season.show, season.number).await {
                data.push((season.id, res))
            }
        }

        if let Err(error) = insert_data(db, &data) {
            eprintln!("{error}");
            return;
        }

        let mut images = Vec::with_capacity(data.len());

        for (season, data) in data {
            let poster_path = images_path.join(format!("{season}_poster.jpg"));
            let poster = download(auth, image_config, &data.poster_path, true, &poster_path).await;

            if poster {
                images.push((season, poster_path.display().to_string()))
            }
        }

        if let Err(error) = insert_images(db, images) {
            eprintln!("{error}");
            return;
        }
    }
}

mod episodes {
    use super::*;

    struct PendingEpisode {
        id: EpisodeId,
        show: TMDBId,
        season: u32,
        number: u32,
    }

    fn fetch(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingEpisode>> {
        let sql = "SELECT id, episode_number, season_number, tmdb_id FROM get_episode_data WHERE tmdb_id NOT NULL AND NOT fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = EpisodeId::from_row(row)?;
                let show = row.get::<_, u32>("tmdb_id")?;
                let show = TMDBId { id: show };
                let season = row.get::<_, u32>("season_number")?;
                let number = row.get::<_, u32>("episode_number")?;
                Ok(PendingEpisode {
                    show,
                    id,
                    season,
                    number,
                })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        episodes: &[(EpisodeId, TMDBEpisode)],
    ) -> rusqlite::Result<usize> {
        let trans = db.transaction()?;
        let sql =
            "UPDATE episode SET synopsis=:overview, release=:release, name=:name WHERE id=:id";
        let mut rows = 0;

        for (episode, data) in episodes {
            let name = format!("{}. {}", data.episode_number, data.name);
            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(*episode)),
                    (
                        ":overview",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.overview)),
                    ),
                    (
                        ":release",
                        &ToSqlOutput::Borrowed(ValueRef::from(&*data.air_date)),
                    ),
                    (":name", &ToSqlOutput::from(name)),
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
        let Ok(episodes) = fetch(db, limit).inspect_err(|error| eprintln!("{error}")) else {
            return;
        };

        let mut data = Vec::with_capacity(episodes.len());

        for episode in episodes {
            if let Some(res) = get_episode(auth, episode.show, episode.season, episode.number).await
            {
                data.push((episode.id, res))
            }
        }

        if let Err(error) = insert_data(db, &data) {
            eprintln!("{error}");
            return;
        }

        let mut images = Vec::with_capacity(data.len());

        for (episode, data) in data {
            let poster_path = images_path.join(format!("{episode}_poster.jpg"));
            let poster = download(auth, image_config, &data.still_path, true, &poster_path).await;

            if poster {
                images.push((episode, poster_path.display().to_string()))
            }
        }

        if let Err(error) = insert_images(db, images) {
            eprintln!("{error}");
            return;
        }
    }
}
