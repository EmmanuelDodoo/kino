use registry::db::Database;
use registry::models::{EpisodeId, MovieId, SeasonId, ShowId};
use reqwest::{
    Client, ClientBuilder,
    header::{ACCEPT, HeaderMap},
};
use rusqlite::types::Value;
use serde::Deserialize;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time;

use rusqlite::types::ToSqlOutput;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".try_into().unwrap());

    ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Cannot build request client")
});

pub(super) const POSTER_SNIPPET: &str = "_poster.jpg";
pub(super) const BACKDROP_SNIPPET: &str = "_backdrop.jpg";
pub(super) const IMAGE_SQL: &str = "INSERT INTO image (path) VALUES (:path) ON CONFLICT (path) DO UPDATE SET main=NULL, accent=NULL, generated=FALSE";

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
    backdrop_path: Option<String>,
    genres: Vec<Genres>,
    overview: Option<String>,
    poster_path: Option<String>,
    release_date: Option<String>,
    vote_average: f64,
    title: String,
    runtime: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBShow {
    id: u32,
    backdrop_path: Option<String>,
    genres: Vec<Genres>,
    overview: Option<String>,
    poster_path: Option<String>,
    name: String,
    vote_average: f64,
    first_air_date: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBSeason {
    id: u32,
    air_date: Option<String>,
    overview: Option<String>,
    vote_average: f64,
    poster_path: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBEpisode {
    id: u32,
    air_date: Option<String>,
    name: String,
    overview: Option<String>,
    still_path: Option<String>,
    vote_average: f64,
    episode_number: u32,
    runtime: u32,
}

async fn get_config(auth: &str) -> reqwest::Result<ImageConfig> {
    tracing::debug!("fetching image configuration");
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
    let snippet = snippet.as_ref();
    tracing::debug!("Searching for media item: {snippet}");

    #[derive(Deserialize)]
    struct Response {
        results: Vec<TMDBId>,
    }

    let item = if movie { "movie" } else { "tv" };

    let response: Response = CLIENT
        .get(format!("https://api.themoviedb.org/3/search/{item}"))
        .query(&[("query", snippet)])
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .inspect_err(|error| tracing::error!("Search fetch error on {}.\n Error {error}", snippet))
        .ok()?
        .json()
        .await
        .inspect_err(|error| tracing::error!("Search fetch error on {}.\n Error {error}", snippet))
        .ok()?;

    response.results.first().cloned()
}

async fn get_movie(auth: &str, id: TMDBId) -> Option<TMDBMovie> {
    tracing::debug!("Fetching movie {}", id.id);
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
    tracing::debug!("Fetching show {}", id.id);
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
    tracing::debug!("Fetching show {} season {number}", show.id);
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
    tracing::debug!("Fetching show {} season {season} episode {number}", show.id);
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
    tracing::debug!("Downloading image at {}", path.as_ref().display());

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
    db: impl AsRef<Path>,
    mut auth_rx: mpsc::Receiver<String>,
    auth: String,
    mut rating_rx: mpsc::Receiver<bool>,
    rating: bool,
    images_path: impl AsRef<Path>,
    interval: std::time::Duration,
) {
    tracing::debug!("Starting up Fetcher instance");
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

    let mut rating = rating;

    loop {
        if !auth_rx.is_empty()
            && let Some(new_auth) = auth_rx.recv().await
        {
            tracing::debug!("New API token received");
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

        if !rating_rx.is_empty()
            && let Some(new_rating) = rating_rx.recv().await
        {
            tracing::debug!("New TMDB rating option received");
            rating = new_rating
        }

        let Some(image_config) = image_config.as_ref() else {
            time::sleep(interval).await;
            continue;
        };

        movies::handle_movies(&auth, &mut db, image_config, 40, &images_path, rating).await;
        shows::handle_shows(&auth, &mut db, image_config, 40, &images_path, rating).await;
        seasons::handle_seasons(&auth, &mut db, image_config, 20, &images_path, rating).await;
        episodes::handle_episodes(&auth, &mut db, image_config, 20, &images_path, rating).await;

        time::sleep(interval).await;
    }
}

pub fn poster_path<P: AsRef<Path>, Id: std::fmt::Display>(path: P, id: Id) -> PathBuf {
    path.as_ref().join(format!("{}{POSTER_SNIPPET}", id))
}

pub fn backdrop_path<P: AsRef<Path>, Id: std::fmt::Display>(path: P, id: Id) -> PathBuf {
    path.as_ref().join(format!("{}{BACKDROP_SNIPPET}", id))
}

mod movies {

    use super::*;

    struct UserPending {
        id: MovieId,
        user_tmdb_id: TMDBId,
    }

    struct PendingData {
        id: MovieId,
        name: String,
    }

    struct PendingImage {
        id: MovieId,
        poster: Option<String>,
        backdrop: Option<String>,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        tracing::debug!("Querying pending movie data");
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

    fn fetch_user_pending(db: &Database, limit: u8) -> rusqlite::Result<Vec<UserPending>> {
        tracing::debug!("Querying user pending movie data");
        let sql = "SELECT movie.id, movie.user_tmdb_id FROM movie WHERE movie.user_tmdb_id IS NOT NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = MovieId::from_row(row)?;
                let user_tmdb_id = TMDBId {
                    id: row.get::<_, u32>("user_tmdb_id")?,
                };
                Ok(UserPending { id, user_tmdb_id })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        tracing::debug!("Querying pending movie images");
        let sql = "SELECT movie.id, movie.poster, movie.backdrop FROM movie WHERE movie.tmdb_id IS NOT NULL AND NOT movie.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = MovieId::from_row(row)?;
                let poster = row.get::<_, Option<String>>("poster")?;
                let backdrop = row.get::<_, Option<String>>("backdrop")?;
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
        rating: bool,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} movie data", movies.len());
        let trans = db.transaction()?;
        let sql = "UPDATE movie SET backdrop=:backdrop, poster=:poster, tmdb_id=:tmdb_id, user_tmdb_id=NULL, tags=:tags, duration=:duration, synopsis=:overview, release=:release, name=:name, rating=:rating WHERE id=:id";
        let mut rows = 0;

        for (movie, data) in movies {
            let TMDBMovie {
                id,
                backdrop_path,
                genres,
                overview,
                poster_path,
                release_date,
                vote_average,
                title,
                runtime,
            } = data;

            let tags = genres
                .iter()
                .map(|genre| genre.name.clone())
                .collect::<Vec<_>>();

            let tags = tags.join(", ");

            let rating_value = (vote_average / 10.0) * 5.0;
            let rating = if rating {
                &ToSqlOutput::from(rating_value)
            } else {
                &ToSqlOutput::Owned(rusqlite::types::Value::Null)
            };

            let duration = runtime * 60;
            let overview = overview.unwrap_or("<empty synopsis>".to_owned());
            let release_date = release_date.unwrap_or("1970-01-01".to_owned());
            let poster_path = poster_path
                .map(ToSqlOutput::from)
                .unwrap_or(ToSqlOutput::Owned(Value::Null));

            let backdrop_path = match backdrop_path {
                Some(path) => ToSqlOutput::from(path),
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(id)),
                    (":id", &ToSqlOutput::from(movie)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (":overview", &ToSqlOutput::from(overview)),
                    (":release", &ToSqlOutput::from(release_date)),
                    (":name", &ToSqlOutput::from(title)),
                    (":poster", &poster_path),
                    (":backdrop", &backdrop_path),
                    (":rating", rating),
                    (":duration", &ToSqlOutput::from(duration)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(MovieId, Option<String>, Option<String>)>,
    ) -> rusqlite::Result<usize> {
        use rusqlite::types::ToSqlOutput;

        tracing::debug!("Inserting {} movie images", images.len());
        let trans = db.transaction()?;
        let sql = "UPDATE movie SET  poster=:poster, backdrop=:backdrop, fetched=:fetched, generate_poster=:generate_poster WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            let poster = match poster {
                Some(poster) => {
                    let poster = ToSqlOutput::from(poster);
                    if let Err(error) = trans.execute(IMAGE_SQL, &[(":path", &poster)]) {
                        tracing::error!("Could not insert into image table. Error\n{error}")
                    };

                    poster
                }
                None => ToSqlOutput::Owned(Value::Null),
            };

            let backdrop = match backdrop {
                Some(path) => ToSqlOutput::from(path),
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &poster),
                    (":backdrop", &backdrop),
                    (":fetched", &ToSqlOutput::from(true)),
                    (":generate_poster", &ToSqlOutput::from(false)),
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
        rating: bool,
    ) {
        tracing::debug!("Handling movies fetching");
        let images_path = images_path.as_ref();

        if let Ok(pending) =
            fetch_user_pending(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut data = Vec::with_capacity(pending.len());

            for movie in pending {
                if let Some(res) = get_movie(auth, movie.user_tmdb_id).await {
                    data.push((movie.id, res))
                }
            }

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
            }
        }

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

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
            }
        };

        if let Ok(pending) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(pending.len());

            for movie in pending {
                let poster = match &movie.poster {
                    Some(poster) => {
                        let poster_path = poster_path(images_path, movie.id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let backdrop = match &movie.backdrop {
                    Some(backdrop) => {
                        let backdrop_path = backdrop_path(images_path, movie.id);
                        let backdrop =
                            download(auth, image_config, backdrop, false, &backdrop_path).await;

                        if backdrop {
                            Some(backdrop_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                images.push((movie.id, poster, backdrop));
            }

            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}

mod shows {
    use super::*;

    struct UserPending {
        id: ShowId,
        user_tmdb_id: TMDBId,
    }

    struct PendingData {
        id: ShowId,
        name: String,
    }

    struct PendingImage {
        id: ShowId,
        poster: Option<String>,
        backdrop: Option<String>,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        tracing::debug!("Querying pending show data");
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

    fn fetch_user_pending(db: &Database, limit: u8) -> rusqlite::Result<Vec<UserPending>> {
        tracing::debug!("Querying user pending show data");
        let sql = "SELECT tv_show.id, tv_show.user_tmdb_id FROM tv_show WHERE tv_show.user_tmdb_id IS NOT NULL LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = ShowId::from_row(row)?;
                let user_tmdb_id = TMDBId {
                    id: row.get::<_, u32>("user_tmdb_id")?,
                };
                Ok(UserPending { id, user_tmdb_id })
            })?
            .collect()
    }

    fn fetch_image(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingImage>> {
        tracing::debug!("Querying pending show images");
        let sql = "SELECT tv_show.id, tv_show.poster, tv_show.backdrop FROM tv_show WHERE tv_show.tmdb_id IS NOT NULL AND NOT tv_show.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = ShowId::from_row(row)?;
                let poster = row.get::<_, Option<String>>("poster")?;
                let backdrop = row.get::<_, Option<String>>("backdrop")?;
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
        shows: Vec<(ShowId, TMDBShow)>,
        rating: bool,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} show data", shows.len());
        let trans = db.transaction()?;
        let sql = "UPDATE tv_show SET backdrop=:backdrop, poster=:poster, tmdb_id=:tmdb_id, user_tmdb_id=NULL, tags=:tags, synopsis=:overview, release=:release, name=:name, rating=:rating WHERE id=:id";
        let mut rows = 0;

        for (show, data) in shows {
            let TMDBShow {
                id,
                backdrop_path,
                genres,
                overview,
                poster_path,
                name,
                vote_average,
                first_air_date,
            } = data;

            let tags = genres
                .iter()
                .map(|genre| genre.name.clone())
                .collect::<Vec<_>>();

            let tags = tags.join(", ");
            let rating_value = (vote_average / 10.0) * 5.0;
            let rating = if rating {
                &ToSqlOutput::from(rating_value)
            } else {
                &ToSqlOutput::Owned(rusqlite::types::Value::Null)
            };
            let overview = overview.unwrap_or("<empty synopsis>".to_owned());
            let first_air_date = first_air_date.unwrap_or("1970-01-01".to_owned());
            let poster_path = poster_path
                .map(ToSqlOutput::from)
                .unwrap_or(ToSqlOutput::Owned(Value::Null));

            let backdrop_path = match backdrop_path {
                Some(path) => ToSqlOutput::from(path),
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(id)),
                    (":id", &ToSqlOutput::from(show)),
                    (":tags", &ToSqlOutput::from(tags)),
                    (":overview", &ToSqlOutput::from(overview)),
                    (":release", &ToSqlOutput::from(first_air_date)),
                    (":name", &ToSqlOutput::from(name)),
                    (":poster", &poster_path),
                    (":backdrop", &backdrop_path),
                    (":rating", rating),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(ShowId, Option<String>, Option<String>)>,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} show images", images.len());
        let trans = db.transaction()?;
        let sql =
            "UPDATE tv_show SET  poster=:poster, backdrop=:backdrop, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster, backdrop) in images {
            let poster = match poster {
                Some(poster) => {
                    let poster = ToSqlOutput::from(poster);
                    if let Err(error) = trans.execute(IMAGE_SQL, &[(":path", &poster)]) {
                        tracing::error!("Could not insert into image table. Error\n{error}")
                    };

                    poster
                }
                None => ToSqlOutput::Owned(Value::Null),
            };

            let backdrop = match backdrop {
                Some(path) => ToSqlOutput::from(path),
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":poster", &poster),
                    (":backdrop", &backdrop),
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
        rating: bool,
    ) {
        tracing::debug!("Handling show fetching");
        let images_path = images_path.as_ref();

        if let Ok(pending) =
            fetch_user_pending(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut data = Vec::with_capacity(pending.len());

            for show in pending {
                if let Some(res) = get_show(auth, show.user_tmdb_id).await {
                    data.push((show.id, res))
                };
            }

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
            }
        }

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

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
            }
        };

        if let Ok(pending) = fetch_image(db, limit) {
            let mut images = Vec::with_capacity(pending.len());

            for show in pending {
                let poster = match &show.poster {
                    Some(poster) => {
                        let poster_path = poster_path(images_path, show.id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let backdrop = match &show.backdrop {
                    Some(backdrop) => {
                        let backdrop_path = backdrop_path(images_path, show.id);
                        let backdrop =
                            download(auth, image_config, backdrop, false, &backdrop_path).await;

                        if backdrop {
                            Some(backdrop_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                images.push((show.id, poster, backdrop));
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
        poster: Option<String>,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        tracing::debug!("Querying pending season data");
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
        tracing::debug!("Querying pending season images");
        let sql = "SELECT season.id, season.poster FROM season WHERE season.tmdb_id IS NOT NULL AND NOT season.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = SeasonId::from_row(row)?;
                let poster = row.get::<_, Option<String>>("poster")?;
                Ok(PendingImage { id, poster })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        seasons: Vec<(SeasonId, TMDBSeason)>,
        rating: bool,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} season data", seasons.len());
        let trans = db.transaction()?;
        let sql = "UPDATE season SET tmdb_id=:tmdb_id, poster=:poster, synopsis=:overview, release=:release, rating=:rating WHERE id=:id";
        let mut rows = 0;

        for (season, data) in seasons {
            let TMDBSeason {
                id,
                air_date,
                overview,
                vote_average,
                poster_path,
            } = data;

            let rating_value = (vote_average / 10.0) * 5.0;
            let rating = if rating {
                &ToSqlOutput::from(rating_value)
            } else {
                &ToSqlOutput::Owned(rusqlite::types::Value::Null)
            };

            let overview = overview.unwrap_or("<empty synopsis>".to_owned());
            let air_date = air_date.unwrap_or("1970-01-01".to_owned());
            let poster_path = poster_path
                .map(ToSqlOutput::from)
                .unwrap_or(ToSqlOutput::Owned(Value::Null));

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(id)),
                    (":id", &ToSqlOutput::from(season)),
                    (":overview", &ToSqlOutput::from(overview)),
                    (":release", &ToSqlOutput::from(air_date)),
                    (":poster", &poster_path),
                    (":rating", rating),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(SeasonId, Option<String>)>,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} season images", images.len());
        let trans = db.transaction()?;
        let sql = "UPDATE season SET poster=:poster, fetched=:fetched WHERE id=:id";
        let mut rows = 0;

        for (id, poster) in images {
            let poster = match poster {
                Some(poster) => {
                    let poster = ToSqlOutput::from(poster);
                    if let Err(error) = trans.execute(IMAGE_SQL, &[(":path", &poster)]) {
                        tracing::error!("Could not insert into image table. Error\n{error}")
                    };

                    poster
                }
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":fetched", &ToSqlOutput::from(true)),
                    (":poster", &poster),
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
        rating: bool,
    ) {
        tracing::debug!("Handling season fethcing");
        let images_path = images_path.as_ref();
        if let Ok(seasons) = fetch_data(db, limit).inspect_err(|error| tracing::error!("{error}")) {
            let mut data = Vec::with_capacity(seasons.len());

            for season in seasons {
                if let Some(res) = get_season(auth, season.show, season.number).await {
                    data.push((season.id, res))
                }
            }

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
            }
        };

        if let Ok(seasons) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(seasons.len());

            for season in seasons {
                let poster = match &season.poster {
                    Some(poster) => {
                        let poster_path = poster_path(images_path, season.id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                images.push((season.id, poster));
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
        poster: Option<String>,
    }

    fn fetch_data(db: &Database, limit: u8) -> rusqlite::Result<Vec<PendingData>> {
        tracing::debug!("Querying pending episode data");
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
        tracing::debug!("Querying pending episode images");
        let sql = "SELECT episode.id, episode.poster FROM episode WHERE episode.tmdb_id IS NOT NULL AND NOT episode.fetched LIMIT :limit";

        let mut statement = db.prepare_cached(sql)?;

        statement
            .query_map(&[(":limit", &ToSqlOutput::from(limit))], |row| {
                let id = EpisodeId::from_row(row)?;
                let poster = row.get::<_, Option<String>>("poster")?;
                Ok(PendingImage { id, poster })
            })?
            .collect()
    }

    fn insert_data(
        db: &mut Database,
        episodes: Vec<(EpisodeId, TMDBEpisode)>,
        rating: bool,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} episode data", episodes.len());
        let trans = db.transaction()?;
        let sql = "UPDATE episode SET tmdb_id=:tmdb_id, poster=:poster, synopsis=:overview, duration=:duration, release=:release, name=:name, rating=:rating WHERE id=:id";
        let mut rows = 0;

        for (episode, data) in episodes {
            let TMDBEpisode {
                id,
                air_date,
                name,
                overview,
                still_path,
                vote_average,
                episode_number,
                runtime,
            } = data;

            let name = format!("{:02} {}", episode_number, name);
            let rating_value = (vote_average / 10.0) * 5.0;
            let rating = if rating {
                &ToSqlOutput::from(rating_value)
            } else {
                &ToSqlOutput::Owned(rusqlite::types::Value::Null)
            };
            let overview = overview.unwrap_or("<empty synopsis>".to_owned());
            let air_date = air_date.unwrap_or("1970-01-01".to_owned());

            let duration = runtime * 60;

            let still_path = still_path
                .map(ToSqlOutput::from)
                .unwrap_or(ToSqlOutput::Owned(Value::Null));

            rows += trans.execute(
                sql,
                &[
                    (":tmdb_id", &ToSqlOutput::from(id)),
                    (":id", &ToSqlOutput::from(episode)),
                    (":overview", &ToSqlOutput::from(overview)),
                    (":release", &ToSqlOutput::from(air_date)),
                    (":name", &ToSqlOutput::from(name)),
                    (":poster", &still_path),
                    (":rating", rating),
                    (":duration", &ToSqlOutput::from(duration)),
                ],
            )?
        }

        trans.commit()?;

        Ok(rows)
    }

    fn insert_images(
        db: &mut Database,
        images: Vec<(EpisodeId, Option<String>)>,
    ) -> rusqlite::Result<usize> {
        tracing::debug!("Inserting {} episode images", images.len());
        let trans = db.transaction()?;
        let sql = "UPDATE episode SET poster=:poster, fetched=:fetched, generate_poster=:generate_poster WHERE id=:id";
        let mut rows = 0;

        for (id, poster) in images {
            let poster = match poster {
                Some(poster) => {
                    let poster = ToSqlOutput::from(poster);
                    if let Err(error) = trans.execute(IMAGE_SQL, &[(":path", &poster)]) {
                        tracing::error!("Could not insert into image table. Error\n{error}")
                    };

                    poster
                }
                None => ToSqlOutput::Owned(Value::Null),
            };

            rows += trans.execute(
                sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":fetched", &ToSqlOutput::from(true)),
                    (":poster", &poster),
                    (":generate_poster", &ToSqlOutput::from(false)),
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
        rating: bool,
    ) {
        tracing::debug!("Handling episode fetching");
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

            if let Err(error) = insert_data(db, data, rating) {
                tracing::error!("{error}");
                return;
            }
        };

        if let Ok(episodes) = fetch_image(db, limit).inspect_err(|error| tracing::error!("{error}"))
        {
            let mut images = Vec::with_capacity(episodes.len());

            for episode in episodes {
                let poster = match &episode.poster {
                    Some(poster) => {
                        let poster_path = poster_path(images_path, episode.id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                images.push((episode.id, poster));
            }
            if let Err(error) = insert_images(db, images) {
                tracing::error!("{error}");
            }
        }
    }
}
