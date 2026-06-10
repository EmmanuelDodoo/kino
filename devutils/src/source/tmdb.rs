use core::error::{Context, ContextLog, Error, Log, Result, bail};
use registry::db::Query;
use registry::models::sources::tmdb::{Media, Request, RequestId, Status, WishType};
use registry::models::{EpisodeId, ItemId, MovieId, SeasonId, ShowId, WishId, wish};
use rusqlite::Connection;
use rusqlite::types::{ToSqlOutput, Value};
use serde::Deserialize;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time;

use super::{
    CLIENT, SourceId, SourceImpl, backdrop_path, insert_episode_image, insert_movie_image,
    insert_season_image, insert_show_image, insert_wish_image, poster_path,
};

#[derive(Debug)]
pub struct TMDB {}

impl TMDB {
    pub fn set_tmdb_id<'a>(id: impl Into<ItemId>, tmdb_id: u32) -> Query<'a> {
        Request::update_tmdb_id(id, tmdb_id)
    }

    pub fn set_number<'a>(id: impl Into<ItemId>, number: u16) -> Query<'a> {
        Request::update_number(id, number)
    }
}

impl SourceImpl for TMDB {
    type Id<'a> = RequestId;

    fn id<'a>(row: &rusqlite::Row<'_>, column: &str) -> rusqlite::Result<Self::Id<'a>> {
        RequestId::from_row_helper(row, column)
    }

    fn id_from_str<'a>(s: &str) -> Self::Id<'a> {
        RequestId::from_str(s)
    }

    fn movie_request<'a>(id: MovieId, name: String) -> Option<(Query<'a>, String)> {
        let media = Media::new_movie(id, name);
        let request = Request::new(media);

        Some((request.insert(), request.id.to_string()))
    }

    fn show_request<'a>(id: ShowId, name: String) -> Option<(Query<'a>, String)> {
        let media = Media::new_show(id, name);
        let request = Request::new(media);

        Some((request.insert(), request.id.to_string()))
    }

    fn season_request<'a>(
        id: SeasonId,
        parent: Self::Id<'a>,
        number: u16,
    ) -> Option<(Query<'a>, String)> {
        let media = Media::new_season(id, parent, number);
        let request = Request::new(media);

        Some((request.insert(), request.id.to_string()))
    }

    fn episode_request<'a>(
        id: EpisodeId,
        parent: Self::Id<'a>,
        season: u16,
        number: u16,
    ) -> Option<(Query<'a>, String)> {
        let media = Media::new_episode(id, parent, season, number);
        let request = Request::new(media);

        Some((request.insert(), request.id.to_string()))
    }

    fn wish_request<'a>(
        id: WishId,
        name: String,
        kind: wish::WishKind,
    ) -> Option<(Query<'a>, String)> {
        let kind = match kind {
            wish::WishKind::Movie { .. } => WishType::Movie,
            wish::WishKind::Show { .. } => WishType::Show,
            wish::WishKind::Season { number, .. } => WishType::Season(number),
            wish::WishKind::Episode { season, number, .. } => WishType::Episode { season, number },
        };

        let media = Media::new_wish(id, name, kind);
        let request = Request::new(media);

        Some((request.insert(), request.id.to_string()))
    }

    fn refetch<'a>(id: impl Into<ItemId>) -> Option<Query<'a>> {
        Some(Request::refetch(id))
    }

    fn delete<'a>(id: impl Into<ItemId>) -> Option<Query<'a>> {
        Some(Request::delete(id))
    }

    fn set_wish_id<'a>(id: SourceId, wish: WishId) -> Option<Query<'a>> {
        match id {
            SourceId::Tmdb(tmdb) => Some(Request::update_wish_tmdb(wish, tmdb)),
            _ => None,
        }
    }

    fn delete_wish<'a>(wish: WishId) -> Option<Query<'a>> {
        Some(Request::delete_wish(wish))
    }

    fn source_id(s: &str) -> Option<SourceId> {
        s.parse::<u32>().ok().map(SourceId::Tmdb)
    }

    fn season_sync<'a>(id: &'a str, parent: &'a str) -> Option<Query<'a>> {
        Some(Request::season_sync(id, parent))
    }

    fn episode_sync<'a>(id: &'a str, parent: &'a str) -> Option<Query<'a>> {
        Some(Request::episode_sync(id, parent))
    }
}

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
    backdrop_path: Option<String>,
    genres: Vec<Genres>,
    overview: Option<String>,
    poster_path: Option<String>,
    name: String,
    vote_average: f64,
    first_air_date: Option<String>,
    number_of_seasons: Option<u16>,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBSeason {
    air_date: Option<String>,
    overview: Option<String>,
    vote_average: f64,
    poster_path: Option<String>,
    episodes: Vec<TMDBDummyEpisode>,
}

#[derive(Deserialize, Debug, Clone)]
struct TMDBDummyEpisode {}

#[derive(Deserialize, Debug, Clone)]
struct TMDBEpisode {
    air_date: Option<String>,
    name: String,
    overview: Option<String>,
    still_path: Option<String>,
    vote_average: f64,
    episode_number: u32,
    runtime: Option<u32>,
}

async fn get_config(auth: &str) -> reqwest::Result<ImageConfig> {
    tracing::debug!("fetching TMDB image configuration");
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
    tracing::debug!("Searching for TMDB media item: {snippet}");

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
        .ctx_log(format!(
            "TMDB search error on {snippet}. Failed to send request"
        ))?
        .json()
        .await
        .ctx_log(format!("TMDB search json conversion error on {snippet}"))?;

    response.results.first().cloned()
}

async fn get_movie(auth: &str, id: TMDBId) -> Option<TMDBMovie> {
    tracing::debug!("Fetching TMDB movie {}", id.id);
    let response: TMDBMovie = CLIENT
        .get(format!("https://api.themoviedb.org/3/movie/{}", id.id))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .ctx_log(format!(
            "Get TMDB movie error on {}. Failed to send request",
            id.id
        ))?
        .json()
        .await
        .ctx_log(format!("Get TMDB movie json conversion error on {}", id.id))?;

    Some(response)
}

async fn get_show(auth: &str, id: TMDBId) -> Option<TMDBShow> {
    tracing::debug!("Fetching TMDB show {}", id.id);
    let response: TMDBShow = CLIENT
        .get(format!("https://api.themoviedb.org/3/tv/{}", id.id))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .ctx_log(format!(
            "Get TMDB show error on {}. Failed to send request",
            id.id
        ))?
        .json()
        .await
        .ctx_log(format!("Get TMDB show json conversion error on {}", id.id))?;

    Some(response)
}

async fn get_season(auth: &str, show: TMDBId, number: u32) -> Option<TMDBSeason> {
    tracing::debug!("Fetching TMDB show {} season {number}", show.id);
    let response: TMDBSeason = CLIENT
        .get(format!(
            "https://api.themoviedb.org/3/tv/{}/season/{number}",
            show.id
        ))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .ctx_log(format!(
            "Get TMDB season error on show: {} season: {}. Failed to send request",
            show.id, number
        ))?
        .json()
        .await
        .ctx_log(format!(
            "Get TMDB season json conversion error on show: {} season: {}",
            show.id, number
        ))?;

    Some(response)
}

async fn get_episode(auth: &str, show: TMDBId, season: u32, number: u32) -> Option<TMDBEpisode> {
    tracing::debug!(
        "Fetching TMDB show {} season {season} episode {number}",
        show.id
    );

    let response: TMDBEpisode = CLIENT
        .get(format!(
                "https://api.themoviedb.org/3/tv/{}/season/{season}/episode/{number}",
                show.id
        ))
        .bearer_auth(auth)
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .ctx_log(format!(
                "Get TMDB episode error on show: {} season: {season} episode: {number}. Failed to send request",
                show.id,
        ))?
        .json()
        .await
        .ctx_log(format!(
                "Get TMDB episode json conversion error on show: {} season: {season} episode: {number}",
                show.id,
        ))?;

    Some(response)
}

async fn download(
    auth: &str,
    config: &ImageConfig,
    image: &str,
    poster: bool,
    path: impl AsRef<Path>,
) -> bool {
    tracing::debug!("Downloading TMDB image at {}", path.as_ref().display());

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
        .ctx_log(format!(
            "TMDB image download error on {}: Failed to send request",
            path.display()
        ));

    let Some(bytes) = bytes else {
        return false;
    };

    let Some(bytes) = bytes.bytes().await.ctx_log(format!(
        "TMDB image download error on {}. Failed to read image bytes",
        path.display()
    )) else {
        return false;
    };

    let Some(file) = File::create(path).await.ctx_log(format!(
        "TMDB image download error on {}. Failed to create output file",
        path.display()
    )) else {
        return false;
    };
    let mut writer = BufWriter::new(file);

    if writer
        .write(bytes.deref())
        .await
        .ctx_log(format!(
            "TMDB image download error on {}. Failed to write to output file ",
            path.display()
        ))
        .is_none()
    {
        return false;
    };
    if writer
        .flush()
        .await
        .ctx_log(format!(
            "TMDB image download error on {}. Failed to flush to output file ",
            path.display()
        ))
        .is_none()
    {
        return false;
    };

    true
}

fn get_requests(
    db: &impl Deref<Target = Connection>,
    retry_limit: u8,
    limit: u16,
) -> rusqlite::Result<Vec<Request>> {
    let searching = Status::Searching as u8;
    let done = Status::Done as u8;

    let sql = "SELECT * from tmdb WHERE (status >= :searching AND status < :done) AND retry < :retry LIMIT :limit";

    let mut statement = db.prepare_cached(sql)?;

    statement
        .query_map(
            &[
                (":searching", &ToSqlOutput::from(searching)),
                (":done", &ToSqlOutput::from(done)),
                (":retry", &ToSqlOutput::from(retry_limit)),
                (":limit", &ToSqlOutput::from(limit)),
            ],
            Request::from_row,
        )?
        .collect()
}

pub async fn run(
    db: impl AsRef<Path>,
    mut auth_rx: mpsc::Receiver<String>,
    auth: String,
    mut rating_rx: mpsc::Receiver<bool>,
    rating: bool,
    images_path: impl AsRef<Path>,
    interval: Duration,
) {
    tracing::debug!("Starting up TMDB fetcher instance");
    let Some(mut db) =
        registry::db::Database::open(db).ctx_log(format!("TMDB fetcher DB opening error"))
    else {
        return;
    };

    let img_config = async |auth: &str| {
        if auth.trim().is_empty() {
            None
        } else {
            get_config(&auth).await.ctx_log(format!(
                "Getting TMDB image config with auth: {auth} failed"
            ))
        }
    };

    let mut auth = auth;
    let mut image_config = img_config(&auth).await;

    let mut rating = rating;
    let retry_limit = 5;
    let limit = 50;

    loop {
        if !auth_rx.is_empty()
            && let Some(new_auth) = auth_rx.recv().await
        {
            tracing::debug!("New TMDB API token received");
            auth = new_auth;
            image_config = img_config(&auth).await;
        }

        if auth.is_empty() {
            time::sleep(interval).await;
            continue;
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

        let requests = get_requests(&db, retry_limit, limit)
            .ctx_log(format!("Failed to get TMDB requests"))
            .unwrap_or_default();

        for mut request in requests {
            if let Err(error) = execute(
                &mut db,
                &image_config,
                &images_path,
                &auth,
                retry_limit,
                rating,
                3,
                &mut request,
            )
            .await
            .with_context(|| format!("TMDB request execution error on retry {}", request.retry))
            {
                tracing::error!("{error:#}");
                request.retry += 1;
            };

            request.update().execute(&db).log_ctx("TMDB request update");
        }

        time::sleep(interval).await;
    }
}

async fn execute(
    db: &mut impl Deref<Target = Connection>,
    image_config: &ImageConfig,
    images_path: impl AsRef<Path>,
    auth: &str,
    retry_limit: u8,
    rating: bool,
    depth: u8,
    request: &mut Request,
) -> Result<(), Error> {
    if depth == 0 {
        return Ok(());
    }

    // todo: Not quite right. As it stands, the retry will be increased again after the return
    // probably better to just return a () or something
    if request.retry >= retry_limit {
        bail!(
            "TMDB Retry limit reached at {} for {}",
            request.retry,
            request.id
        );
    }

    match request.status {
        Status::Waiting | Status::Done => return Ok(()),
        Status::Searching => match &request.media {
            Media::Movie { name, .. } => {
                let Some(search): Option<TMDBId> = search_item(auth, name, true).await else {
                    bail!("Could not find TMDB movie {name} request: {}", request.id)
                };
                request.tmdb_id = Some(search.id);
                request.status = Status::Data;
            }
            Media::Show { name, .. } => {
                let Some(search): Option<TMDBId> = search_item(auth, name, false).await else {
                    bail!("Could not find TMDB show {name} request: {}", request.id)
                };
                request.tmdb_id = Some(search.id);
                request.status = Status::Data;
            }
            Media::Wish {
                id: _id,
                name,
                kind,
            } => match kind {
                WishType::Movie => {
                    let Some(search): Option<TMDBId> = search_item(auth, name, true).await else {
                        bail!(
                            "Could not find TMDB wish movie {name} request: {}",
                            request.id
                        )
                    };

                    request.tmdb_id = Some(search.id);
                    request.status = Status::Data;
                }

                WishType::Show | WishType::Season(_) | WishType::Episode { .. } => {
                    let Some(search): Option<TMDBId> = search_item(auth, name, false).await else {
                        bail!(
                            "Could not find TMDB wish show {name} request: {}",
                            request.id
                        )
                    };

                    request.tmdb_id = Some(search.id);
                    request.status = Status::Data;
                }
            },
            _ => {
                bail!("Cannot TMDB search season/episode")
            }
        },
        Status::Data => {
            let Some(tmdb) = request.tmdb_id.map(|id| TMDBId { id }) else {
                bail!(
                    "Tried to get data with null TMDB ID. Request {}",
                    request.id
                );
            };

            match &mut request.media {
                Media::Movie { id, backdrop, name } => {
                    let Some(movie): Option<TMDBMovie> = get_movie(auth, tmdb).await else {
                        bail!(
                            "Could not get TMDB movie {name} data. Request: {}",
                            request.id
                        );
                    };

                    insert_movie(db, request.id, *id, &movie, rating).with_context(|| {
                        format!("Inserting TMDB movie {name}. Request: {}", request.id)
                    })?;
                    request.status = Status::Image;
                    request.poster = movie.poster_path;
                    *backdrop = movie.backdrop_path;
                }
                Media::Show { id, backdrop, name } => {
                    let Some(show): Option<TMDBShow> = get_show(auth, tmdb).await else {
                        bail!(
                            "Could not get TMDB show {name} data. Request: {}",
                            request.id
                        );
                    };

                    insert_show(db, request.id, *id, &show, rating).with_context(|| {
                        format!("Inserting TMDB show {name}. Request: {}", request.id)
                    })?;
                    request.status = Status::Image;
                    request.poster = show.poster_path;
                    *backdrop = show.backdrop_path;
                }
                Media::Season { id, number, .. } => {
                    let Some(season): Option<TMDBSeason> =
                        get_season(auth, tmdb, *number as u32).await
                    else {
                        bail!(
                            "Could not get TMDB season {number} data. Request: {}",
                            request.id
                        );
                    };

                    insert_season(db, request.id, *id, &season, rating).with_context(|| {
                        format!("Inserting TMDB season {number}. Request: {}", request.id)
                    })?;
                    request.status = Status::Image;
                    request.poster = season.poster_path;
                }
                Media::Episode {
                    id, season, number, ..
                } => {
                    let Some(episode): Option<TMDBEpisode> =
                        get_episode(auth, tmdb, *season as u32, *number as u32).await
                    else {
                        bail!(
                            "Could not get TMDB episode {number} data. Request: {}",
                            request.id
                        );
                    };

                    insert_episode(db, request.id, *id, &episode, rating).with_context(|| {
                        format!("Inserting TMDB season {number}. Request: {}", request.id)
                    })?;
                    request.status = Status::Image;
                    request.poster = episode.still_path;
                }
                Media::Wish { id, name, kind } => match kind {
                    WishType::Movie => {
                        let Some(movie): Option<TMDBMovie> = get_movie(auth, tmdb).await else {
                            bail!(
                                "Could not get TMDB wish movie {name} data. Request: {}",
                                request.id
                            );
                        };

                        let TMDBMovie {
                            backdrop_path: _backdrop,
                            genres,
                            overview,
                            poster_path,
                            release_date,
                            vote_average,
                            title,
                            runtime,
                        } = movie;

                        let duration = (runtime * 60) as u64;
                        let tags = genres
                            .into_iter()
                            .map(|genre| genre.name)
                            .collect::<Vec<_>>();

                        let kind = wish::WishKind::Movie { duration, tags };

                        insert_wish(
                            db,
                            request.id,
                            *id,
                            title,
                            overview,
                            release_date,
                            vote_average,
                            kind,
                        )
                        .with_context(|| {
                            format!("Inserting TMDB wish movie {name}. Request: {}", request.id)
                        })?;

                        request.status = Status::Image;
                        request.poster = poster_path;
                    }
                    WishType::Show => {
                        let Some(show): Option<TMDBShow> = get_show(auth, tmdb).await else {
                            bail!(
                                "Could not get TMDB wish show {name} data. Request: {}",
                                request.id
                            );
                        };

                        let TMDBShow {
                            backdrop_path: _backdrop,
                            genres,
                            overview,
                            poster_path,
                            name: title,
                            vote_average,
                            first_air_date,
                            number_of_seasons,
                        } = show;

                        let tags = genres
                            .into_iter()
                            .map(|genre| genre.name)
                            .collect::<Vec<_>>();

                        let kind = wish::WishKind::Show {
                            tags,
                            seasons: number_of_seasons.unwrap_or_default(),
                        };

                        insert_wish(
                            db,
                            request.id,
                            *id,
                            title,
                            overview,
                            first_air_date,
                            vote_average,
                            kind,
                        )
                        .with_context(|| {
                            format!("Inserting TMDB wish show {name}. Request: {}", request.id)
                        })?;

                        request.status = Status::Image;
                        request.poster = poster_path;
                    }
                    WishType::Season(number) => {
                        let Some(season): Option<TMDBSeason> =
                            get_season(auth, tmdb, *number as u32).await
                        else {
                            bail!(
                                "Could not get TMDB wish season {number} data. Request: {}",
                                request.id
                            );
                        };

                        let TMDBSeason {
                            air_date,
                            overview,
                            vote_average,
                            poster_path,
                            episodes,
                        } = season;

                        let kind = wish::WishKind::Season {
                            number: *number,
                            episodes: episodes.len() as u16,
                        };

                        insert_wish(
                            db,
                            request.id,
                            *id,
                            name.clone(),
                            overview,
                            air_date,
                            vote_average,
                            kind,
                        )
                        .with_context(|| {
                            format!(
                                "Inserting TMDB wish season {number}. Request: {}",
                                request.id
                            )
                        })?;
                        request.status = Status::Image;
                        request.poster = poster_path;
                    }
                    WishType::Episode { season, number } => {
                        let Some(episode): Option<TMDBEpisode> =
                            get_episode(auth, tmdb, *season as u32, *number as u32).await
                        else {
                            bail!(
                                "Could not get TMDB wish episode {number} data. Request: {}",
                                request.id
                            );
                        };

                        let TMDBEpisode {
                            air_date,
                            name: _name,
                            overview,
                            still_path,
                            vote_average,
                            episode_number: _unused,
                            runtime,
                        } = episode;

                        let duration = runtime
                            .map(|runtime| (runtime * 60) as u64)
                            .unwrap_or_default();

                        let kind = wish::WishKind::Episode {
                            season: *season,
                            number: *number,
                            duration,
                        };
                        insert_wish(
                            db,
                            request.id,
                            *id,
                            name.clone(),
                            overview,
                            air_date,
                            vote_average,
                            kind,
                        )
                        .with_context(|| {
                            format!(
                                "Inserting TMDB wish episode {number}. Request: {}",
                                request.id
                            )
                        })?;
                        request.status = Status::Image;
                        request.poster = still_path;
                    }
                },
            }
        }
        Status::Image => match &request.media {
            Media::Movie { id, backdrop, name } => {
                let poster = match &request.poster {
                    Some(poster) => {
                        let poster_path = poster_path(&images_path, id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let backdrop = match &backdrop {
                    Some(backdrop) => {
                        let backdrop_path = backdrop_path(&images_path, id);
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

                if poster.is_none() && backdrop.is_none() {
                    bail!(
                        "Could not download movie {name} images. Request: {}",
                        request.id
                    );
                }

                insert_movie_image(db, *id, poster, backdrop).with_context(|| {
                    format!("Inserting TMDB movie {name} image. Request: {}", request.id)
                })?;
                request.status = Status::Done;
            }
            Media::Show { id, backdrop, name } => {
                let poster = match &request.poster {
                    Some(poster) => {
                        let poster_path = poster_path(&images_path, id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let backdrop = match &backdrop {
                    Some(backdrop) => {
                        let backdrop_path = backdrop_path(&images_path, id);
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

                if poster.is_none() && backdrop.is_none() {
                    bail!(
                        "Could not download show {name} images. Request: {}",
                        request.id
                    );
                }

                insert_show_image(db, *id, poster, backdrop).with_context(|| {
                    format!("Inserting TMDB show {name} image. Request: {}", request.id)
                })?;
                request.status = Status::Done;
            }
            Media::Season { id, number, .. } => {
                let poster = match &request.poster {
                    Some(poster) => {
                        let poster_path = poster_path(&images_path, id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                if poster.is_none() {
                    bail!(
                        "Could not download TMDB season {number} poster. Request: {}",
                        request.id
                    )
                }

                insert_season_image(db, *id, poster).with_context(|| {
                    format!(
                        "Inserting TMDB season {number} image. Request: {}",
                        request.id
                    )
                })?;
                request.status = Status::Done;
            }
            Media::Episode { id, number, .. } => {
                let poster = match &request.poster {
                    Some(poster) => {
                        let poster_path = poster_path(&images_path, id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                if poster.is_none() {
                    bail!(
                        "Could not download episode {number} poster. Request: {}",
                        request.id
                    )
                }

                insert_episode_image(db, *id, poster).with_context(|| {
                    format!(
                        "Inserting TMDB episode {number} image. Request: {}",
                        request.id
                    )
                })?;
                request.status = Status::Done;
            }
            Media::Wish { id, .. } => {
                let poster = match &request.poster {
                    Some(poster) => {
                        let poster_path = poster_path(&images_path, id);
                        let poster = download(auth, image_config, poster, true, &poster_path).await;

                        if poster {
                            Some(poster_path.display().to_string())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                if poster.is_none() {
                    bail!(
                        "Could not download TMDB wish poster. Request: {}",
                        request.id
                    )
                }

                insert_wish_image(db, *id, poster).with_context(|| {
                    format!("Inserting TMDB wish image. Request: {}", request.id)
                })?;

                request.status = Status::Done;
            }
        },
    }

    request.retry = 0;
    Box::pin(execute(
        db,
        image_config,
        images_path,
        auth,
        retry_limit,
        rating,
        depth.saturating_sub(1),
        request,
    ))
    .await
}

fn insert_movie(
    db: &impl Deref<Target = Connection>,
    request: RequestId,
    id: MovieId,
    movie: &TMDBMovie,
    rating: bool,
) -> rusqlite::Result<()> {
    let sql = "UPDATE movie SET tags=:tags, duration=:duration, synopsis=:overview, release=:release, name=:name, rating=:rating WHERE id=:id AND request=:request";

    let mut statement = db.prepare_cached(sql)?;

    let TMDBMovie {
        genres,
        overview,
        backdrop_path: _backdrop,
        poster_path: _poster,
        release_date,
        vote_average,
        title,
        runtime,
    } = movie;

    let tags = genres
        .iter()
        .map(|genre| genre.name.as_str())
        .collect::<Vec<_>>();

    let tags = tags.join(", ");

    let rating_value = (vote_average / 10.0) * 5.0;
    let rating = if rating {
        &ToSqlOutput::from(rating_value)
    } else {
        &ToSqlOutput::Owned(Value::Null)
    };

    let duration = runtime * 60;
    let overview = overview.as_deref().unwrap_or("<empty synopsis>");
    let release_date = release_date.as_deref().unwrap_or("1970-01-01");

    statement.execute(&[
        (":id", &ToSqlOutput::from(id)),
        (":request", &ToSqlOutput::from(request)),
        (":tags", &ToSqlOutput::from(tags)),
        (":overview", &ToSqlOutput::from(overview)),
        (":release", &ToSqlOutput::from(release_date)),
        (":name", &ToSqlOutput::from(title.as_str())),
        (":rating", rating),
        (":duration", &ToSqlOutput::from(duration)),
    ])?;

    Ok(())
}

fn insert_wish(
    db: &impl Deref<Target = Connection>,
    request: RequestId,
    id: WishId,
    name: String,
    synopsis: Option<String>,
    release: Option<String>,
    rating: f64,
    wsh: wish::WishKind,
) -> rusqlite::Result<()> {
    let sql = "UPDATE wish SET name=:name, synopsis=:synopsis, rating=:rating, release=:release, tags=:tags, duration=:duration, count=:count WHERE id=:id AND request=:request";

    let mut statement = db.prepare_cached(sql)?;

    let release = release.unwrap_or("1970-01-01".to_owned());
    let synopsis = synopsis.unwrap_or("<empty synopsis>".to_owned());
    let rating = (rating / 10.0) * 5.0;

    let name = ToSqlOutput::from(name);
    let synopsis = ToSqlOutput::from(synopsis);
    let release = ToSqlOutput::from(release);
    let rating = ToSqlOutput::from(rating);
    let null = ToSqlOutput::Owned(Value::Null);
    let zero = ToSqlOutput::from(0);

    let (duration, tags, count) = match wsh {
        wish::WishKind::Movie { duration, tags } => {
            let duration = i64::try_from(duration).expect("duration cannot be expressed as i64");
            let duration = ToSqlOutput::from(duration);
            let tags = ToSqlOutput::from(tags.join(","));
            (duration, tags, zero)
        }
        wish::WishKind::Show { tags, seasons } => {
            let tags = ToSqlOutput::from(tags.join(","));
            let seasons = ToSqlOutput::from(seasons);

            (zero, tags, seasons)
        }
        wish::WishKind::Season { episodes, .. } => {
            let episodes = ToSqlOutput::from(episodes);

            (zero, null, episodes)
        }
        wish::WishKind::Episode { duration, .. } => {
            let duration = i64::try_from(duration).expect("duration cannot be expressed as i64");
            let duration = ToSqlOutput::from(duration);

            (duration, null, zero)
        }
    };

    statement.execute(&[
        (":id", &ToSqlOutput::from(id)),
        (":request", &ToSqlOutput::from(request)),
        (":name", &name),
        (":release", &release),
        (":synopsis", &synopsis),
        (":rating", &rating),
        (":duration", &duration),
        (":tags", &tags),
        (":count", &count),
    ])?;

    Ok(())
}

fn insert_show(
    db: &impl Deref<Target = Connection>,
    request: RequestId,
    id: ShowId,
    show: &TMDBShow,
    rating: bool,
) -> rusqlite::Result<()> {
    let sql = "UPDATE tv_show SET  tags=:tags, synopsis=:overview, release=:release, name=:name, rating=:rating WHERE id=:id AND request=:request";
    let mut statement = db.prepare_cached(sql)?;

    let TMDBShow {
        genres,
        overview,
        name,
        vote_average,
        first_air_date,
        poster_path: _poster,
        backdrop_path: _backdrop,
        number_of_seasons: _seasons,
    } = show;

    let tags = genres
        .iter()
        .map(|genre| genre.name.as_str())
        .collect::<Vec<_>>();

    let tags = tags.join(", ");
    let rating_value = (vote_average / 10.0) * 5.0;
    let rating = if rating {
        &ToSqlOutput::from(rating_value)
    } else {
        &ToSqlOutput::Owned(Value::Null)
    };
    let overview = overview.as_deref().unwrap_or("<empty synopsis>");
    let first_air_date = first_air_date.as_deref().unwrap_or("1970-01-01");

    statement.execute(&[
        (":id", &ToSqlOutput::from(id)),
        (":request", &ToSqlOutput::from(request)),
        (":tags", &ToSqlOutput::from(tags)),
        (":overview", &ToSqlOutput::from(overview)),
        (":release", &ToSqlOutput::from(first_air_date)),
        (":name", &ToSqlOutput::from(name.as_str())),
        (":rating", rating),
    ])?;

    Ok(())
}

fn insert_season(
    db: &impl Deref<Target = Connection>,
    request: RequestId,
    id: SeasonId,
    season: &TMDBSeason,
    rating: bool,
) -> rusqlite::Result<()> {
    let sql = "UPDATE season SET  synopsis=:overview, release=:release, rating=:rating WHERE id=:id AND request=:request";
    let mut statement = db.prepare_cached(sql)?;

    let TMDBSeason {
        air_date,
        overview,
        vote_average,
        poster_path: _poster,
        episodes: _episodes,
    } = season;

    let rating_value = (vote_average / 10.0) * 5.0;
    let rating = if rating {
        &ToSqlOutput::from(rating_value)
    } else {
        &ToSqlOutput::Owned(Value::Null)
    };

    let overview = overview.as_deref().unwrap_or("<empty synopsis>");
    let air_date = air_date.as_deref().unwrap_or("1970-01-01");

    statement.execute(&[
        (":id", &ToSqlOutput::from(id)),
        (":request", &ToSqlOutput::from(request)),
        (":overview", &ToSqlOutput::from(overview)),
        (":release", &ToSqlOutput::from(air_date)),
        (":rating", rating),
    ])?;

    Ok(())
}

fn insert_episode(
    db: &impl Deref<Target = Connection>,
    request: RequestId,
    id: EpisodeId,
    episode: &TMDBEpisode,
    rating: bool,
) -> rusqlite::Result<()> {
    let sql = "UPDATE episode SET  synopsis=:overview, duration=:duration, release=:release, name=:name, rating=:rating, episode_number=:episode_number WHERE id=:id AND request=:request";
    let mut statement = db.prepare_cached(sql)?;

    let TMDBEpisode {
        air_date,
        name,
        overview,
        vote_average,
        episode_number,
        runtime,
        still_path: _still,
    } = episode;

    let name = format!("{:02} {}", episode_number, name);
    let rating_value = (vote_average / 10.0) * 5.0;
    let rating = if rating {
        &ToSqlOutput::from(rating_value)
    } else {
        &ToSqlOutput::Owned(Value::Null)
    };
    let overview = overview.as_deref().unwrap_or("<empty synopsis>");
    let air_date = air_date.as_deref().unwrap_or("1970-01-01");

    let duration = runtime.unwrap_or_default() * 60;

    statement.execute(&[
        (":id", &ToSqlOutput::from(id)),
        (":request", &ToSqlOutput::from(request)),
        (":overview", &ToSqlOutput::from(overview)),
        (":release", &ToSqlOutput::from(air_date)),
        (":name", &ToSqlOutput::from(name)),
        (":rating", rating),
        (":duration", &ToSqlOutput::from(duration)),
        (":episode_number", &ToSqlOutput::from(*episode_number)),
    ])?;

    Ok(())
}
