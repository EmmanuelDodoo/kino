use crate::db::{BatchResult, Database};
use crate::error;
use crate::models::{
    Directory, DirectoryId, Episode, EpisodeId, MediaType, Movie, MovieId, Season, SeasonId, Show,
    ShowId,
};
use fancy_regex::Regex;
use gstreamer_pbutils::Discoverer;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};
use std::sync::LazyLock;

static CLEANER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-zA-Z\d]+").expect("Cannot create sanitizer regex"));
static MOVIE_REG1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)(?=\d{3,4}p)|^.*$").expect("Cannot create movie regex 1"));
static MOVIE_REG2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.*)(?=(\(|\[|\.)\d{4}(\)|\]|\.)(?!.*(\(|\[|\.)\d{4}(\)|\]|\.)))|^.*")
        .expect("Cannot create movie regex 2")
});
static SEASON_REG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?<=[s|S])\s?\d{1,3}|(?<=[^a-zA-Z][s|S][e|E][a|A][s|S]|[o|O][n|N])\s?\d{1,3}")
        .expect("Cannot create season regex")
});
static EPISODE_REG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?<=[^a-zA-Z][e|E])\s?\d{1,3}|(?<=[e|E][p|P][i|I][s|S]|[o|O][d|D][e|E])\s?\d{1,3}|(?<=\dx)\s?\d{1,3}")
        .expect("Cannot create episode regex")
});

#[rustfmt::skip]
const EXTENSIONS: [&str; 9] = [
    "avi",
    "flv",
    "mkv",
    "mov",
    "mp4", 
    "mpeg",
    "mpg",
    "webm",
    "wmv",
];

#[derive(Debug)]
struct Video {
    name: String,
    path: String,
    duration: u64,
}

#[derive(Debug)]
struct SeasonPrim {
    // Get number at end of this?
    path: String,
    episodes: Vec<Video>,
}

#[derive(Debug)]
struct ShowPrim {
    path: String,
    seasons: Vec<SeasonPrim>,
}

pub fn scan_dir<'a>(
    db: &str,
    dir: Directory,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
) -> Option<BatchResult<'a>> {
    tracing::info!("Scanning directory {}", dir.path);
    let discoverer = if discoverer {
        if let Err(error) = gstreamer::init().map_err(error::GStreamerError::Glib) {
            tracing::error!(
                "Scan directory gstreamer init error on {}. Error \n{error}",
                dir.path
            );
        };
        Discoverer::new(gstreamer::ClockTime::from_seconds(5))
            .inspect_err(|error| {
                tracing::error!(
                    "Scan directory discoverer error on {}. Error \n{error}",
                    dir.path
                )
            })
            .ok()
    } else {
        None
    };

    let mut db = match Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            tracing::error!("Scan directory error on {}. Error \n{error}", dir.path);
            return None;
        }
    };

    scan_dir_helper(&mut db, dir, discoverer.as_ref(), movie_depth, restore)
}

pub fn scan_dirs<'a>(
    db: impl AsRef<Path>,
    dirs: Vec<Directory>,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
) -> (Option<BatchResult<'a>>, Vec<DirectoryId>) {
    tracing::info!("Scanning {} directories", dirs.len());
    let discoverer = if discoverer {
        if let Err(error) = gstreamer::init().map_err(error::GStreamerError::Glib) {
            tracing::error!("Scan directories gstreamer init error. Error \n{error}");
        };
        Discoverer::new(gstreamer::ClockTime::from_seconds(5))
            .inspect_err(|error| {
                tracing::error!("Scan directories discoverer error. Error \n{error}",)
            })
            .ok()
    } else {
        None
    };

    let mut db = match Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            tracing::error!("Scan directories error. Error \n{error}");
            return (None, Vec::with_capacity(0));
        }
    };

    let mut result = BatchResult::empty();
    let mut scanned = vec![];

    for dir in dirs {
        let id = dir.id;
        match scan_dir_helper(&mut db, dir, discoverer.as_ref(), movie_depth, restore) {
            Some(res) => {
                scanned.push(id);
                result.merge(res);
            }
            None => continue,
        };
    }

    (Some(result), scanned)
}

pub fn scan_dir_helper<'a>(
    db: &mut Database,
    dir: Directory,
    discoverer: Option<&Discoverer>,
    movie_depth: u8,
    restore: bool,
) -> Option<BatchResult<'a>> {
    let mut successes = vec![];
    let mut failures = vec![];

    match dir.media_type {
        MediaType::Movies => {
            tracing::info!("Scanning movie directory {}", dir.path);
            if let Some(videos) = scan_video_dir(&dir.path, discoverer, movie_depth, None) {
                let mut scanned = std::collections::HashSet::with_capacity(videos.len());

                for movie in videos {
                    let name = process_name(&movie.name).unwrap_or(movie.name.clone());
                    let (_, query) =
                        Movie::new(dir.id, movie.path.clone(), name, movie.name, movie.duration);
                    match query.execute(db) {
                        Ok(succ) => {
                            scanned.insert(movie.path);
                            successes.push(succ)
                        }
                        Err(fail) => failures.push(fail),
                    };
                }

                struct DirMovie {
                    id: MovieId,
                    path: String,
                    tombstone: bool,
                }

                tracing::info!("Fetching Directory movies");
                if let Ok(dir_movies) = db
                    .get_dir_movies(dir.id, |row| {
                        let id = MovieId::from_row(row)?;
                        let path = row.get::<_, String>("path")?;
                        let tombstone = row.get::<_, bool>("removed")?;

                        Ok(DirMovie {
                            id,
                            path,
                            tombstone,
                        })
                    })
                    .inspect_err(|error| tracing::error!("{error}"))
                {
                    let movies = dir_movies
                        .into_iter()
                        .map(|movie| {
                            let scanned = scanned.contains(&movie.path);
                            #[allow(clippy::nonminimal_bool)]
                            let insert = (scanned && restore && movie.tombstone)
                                || (scanned && !movie.tombstone);
                            (movie.id, insert)
                        })
                        .collect();

                    tracing::info!("Performing movies insert/remove");
                    if let Err(error) = db.insert_remove_movies(movies) {
                        tracing::error!("{error}")
                    };
                };
            }
        }
        MediaType::Shows => {
            struct DirEpisode {
                id: EpisodeId,
                path: String,
                tombstone: bool,
            }

            struct DirSeason {
                id: SeasonId,
                path: String,
                tombstone: bool,
            }

            struct DirShow {
                id: ShowId,
                path: String,
                tombstone: bool,
            }

            tracing::info!("Scanning shows directory {}", dir.path);
            let shows = scan_shows(&dir.path, discoverer)?;
            let mut scanned_shows = std::collections::HashSet::with_capacity(shows.len());

            for show in shows {
                let ShowPrim { path, seasons } = show;
                let name = process_name(&path).unwrap_or(path.clone());
                let (show, query) = Show::new(
                    dir.id,
                    path.clone(),
                    name.clone(),
                    path.clone(),
                    seasons.len() as _,
                );

                let new = match query.execute(db) {
                    Ok(succ) => {
                        let modified = succ.rows > 0;
                        successes.push(succ);
                        scanned_shows.insert(path.clone());
                        modified
                    }
                    Err(error) => {
                        failures.push(error);
                        continue;
                    }
                };

                let show = if new {
                    show.id
                } else {
                    match get_existing_show(db, &dir, &path) {
                        Ok(id) => id,
                        Err(error) => {
                            tracing::error!("{error}");
                            continue;
                        }
                    }
                };

                let mut scanned_seasons = std::collections::HashSet::with_capacity(seasons.len());

                tracing::info!("Scanning {name} seasons");
                for season in seasons {
                    let SeasonPrim { path, episodes } = season;
                    let number = process_season(&path);
                    let name = match number {
                        Some(number) => format!("Season {number}"),
                        None => path.clone(),
                    };

                    let (season, query) = Season::new(show, name.clone(), path.clone(), number);

                    let new = match query.execute(db) {
                        Ok(succ) => {
                            let modified = succ.rows > 0;
                            successes.push(succ);
                            scanned_seasons.insert(path.clone());
                            modified
                        }
                        Err(error) => {
                            failures.push(error);
                            continue;
                        }
                    };

                    let season = if new {
                        season.id
                    } else {
                        match get_existing_season(db, show, &path) {
                            Ok(id) => id,
                            Err(error) => {
                                tracing::error!("{error}");
                                continue;
                            }
                        }
                    };

                    let mut scanned_episodes =
                        std::collections::HashSet::with_capacity(episodes.len());

                    tracing::info!("Scanning {name} episodes");
                    for episode in episodes {
                        let number = process_episode(&episode.path);
                        let name = match number {
                            Some(number) => format!("Episode {number}"),
                            None => episode.name.clone(),
                        };

                        let (_, query) = Episode::new(
                            season,
                            name,
                            episode.name,
                            episode.path.clone(),
                            episode.duration,
                            number,
                        );
                        match query.execute(db) {
                            Ok(succ) => {
                                scanned_episodes.insert(episode.path);
                                successes.push(succ)
                            }
                            Err(fail) => failures.push(fail),
                        }
                    }

                    tracing::info!("Fetching season episodes");
                    if let Ok(dir_episodes) = db
                        .get_season_episodes_removed(season, |row| {
                            let id = EpisodeId::from_row(row)?;
                            let path = row.get::<_, String>("path")?;
                            let tombstone = row.get::<_, bool>("removed")?;

                            Ok(DirEpisode {
                                id,
                                path,
                                tombstone,
                            })
                        })
                        .inspect_err(|error| tracing::error!("{error}"))
                    {
                        let episodes = dir_episodes
                            .into_iter()
                            .map(|episode| {
                                let scanned = scanned_episodes.contains(&episode.path);
                                #[allow(clippy::nonminimal_bool)]
                                let insert = (scanned && restore && episode.tombstone)
                                    || (scanned && !episode.tombstone);
                                (episode.id, insert)
                            })
                            .collect();

                        tracing::info!("Performing episodes insert/remove");
                        if let Err(error) = db.insert_remove_episodes(episodes) {
                            tracing::error!("{error}")
                        };
                    };
                }

                tracing::info!("Fetching show seasons");
                if let Ok(dir_seasons) = db
                    .get_show_seasons_removed(show, |row| {
                        let id = SeasonId::from_row(row)?;
                        let path = row.get::<_, String>("path")?;
                        let tombstone = row.get::<_, bool>("removed")?;

                        Ok(DirSeason {
                            id,
                            path,
                            tombstone,
                        })
                    })
                    .inspect_err(|error| tracing::error!("{error}"))
                {
                    let seasons = dir_seasons
                        .into_iter()
                        .map(|season| {
                            let scanned = scanned_seasons.contains(&season.path);
                            #[allow(clippy::nonminimal_bool)]
                            let insert = (scanned && restore && season.tombstone)
                                || (scanned && !season.tombstone);
                            (season.id, insert)
                        })
                        .collect();

                    tracing::info!("Performing season insert/remove");
                    if let Err(error) = db.insert_remove_seasons(seasons) {
                        tracing::error!("{error}")
                    };
                };
            }

            tracing::info!("Fetching Directory shows");
            if let Ok(dir_shows) = db
                .get_dir_shows(dir.id, |row| {
                    let id = ShowId::from_row(row)?;
                    let path = row.get::<_, String>("path")?;
                    let tombstone = row.get::<_, bool>("removed")?;

                    Ok(DirShow {
                        id,
                        path,
                        tombstone,
                    })
                })
                .inspect_err(|error| tracing::error!("{error}"))
            {
                let shows = dir_shows
                    .into_iter()
                    .map(|show| {
                        let scanned = scanned_shows.contains(&show.path);
                        #[allow(clippy::nonminimal_bool)]
                        let insert =
                            (scanned && restore && show.tombstone) || (scanned && !show.tombstone);
                        (show.id, insert)
                    })
                    .collect();

                tracing::info!("Performing movies insert/remove");
                if let Err(error) = db.insert_remove_shows(shows) {
                    tracing::error!("{error}")
                };
            };
        }
    }

    Some(BatchResult {
        successes,
        failures,
    })
}

fn scan_shows(path: impl AsRef<Path>, discoverer: Option<&Discoverer>) -> Option<Vec<ShowPrim>> {
    let path = path.as_ref();

    let mut shows = vec![];
    let read = path
        .read_dir()
        .inspect_err(|error| {
            tracing::error!("Scan show dir on {}. Error \n{error}", path.display())
        })
        .ok()?;

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };

        let is_dir = match item.file_type() {
            Ok(file) => file.is_dir(),
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };

        if !is_dir {
            continue;
        }
        let path = item.path();

        shows.push(scan_show_dir(path, discoverer)?);
    }

    Some(shows)
}

fn scan_show_dir(path: impl AsRef<Path>, discoverer: Option<&Discoverer>) -> Option<ShowPrim> {
    let path = path.as_ref();
    let dir = path_name(path);

    let read = path
        .read_dir()
        .inspect_err(|error| {
            tracing::error!("Scan show dir on {}. Error \n{error}", path.display())
        })
        .ok()?;

    let mut seasons = vec![];

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };

        let is_dir = match item.file_type() {
            Ok(file) => file.is_dir(),
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };

        if !is_dir {
            continue;
        }

        let path = item.path();
        let name = path_name(&path).to_owned();
        if let Some(videos) = scan_video_dir(path, discoverer, 0, None) {
            let season = SeasonPrim {
                path: name,
                episodes: videos,
            };

            seasons.push(season)
        };
    }

    let show = ShowPrim {
        path: dir.to_owned(),
        seasons,
    };

    Some(show)
}

fn scan_video_dir(
    path: impl AsRef<Path>,
    discoverer: Option<&Discoverer>,
    depth: u8,
    prefix: Option<String>,
) -> Option<Vec<Video>> {
    let path = path.as_ref();

    tracing::info!("Scanning video directory {}", path.display());

    let path = path
        .canonicalize()
        .inspect_err(|error| {
            tracing::error!(
                "Scan video dir error on {}. Error \n{error}",
                path.display()
            );
        })
        .ok()?;

    let mut videos = vec![];

    let read = path
        .read_dir()
        .inspect_err(|error| {
            tracing::error!("Scan video dir on {}. Error \n{error}", path.display())
        })
        .ok()?;

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };

        let is_file = match item.file_type() {
            Ok(file) => file.is_file(),
            Err(error) => {
                tracing::error!("{error}");
                continue;
            }
        };
        let path = item.path();

        if is_file {
            if let Some((path, name, duration)) = scan_file(path, discoverer) {
                let path = match prefix.as_ref() {
                    Some(prefix) => format!("{prefix}{MAIN_SEPARATOR_STR}{path}"),
                    None => path,
                };

                videos.push(Video {
                    path,
                    name,
                    duration,
                })
            };
        } else if depth > 0 {
            let prefix = match &prefix {
                Some(prefix) => format!("{prefix}{MAIN_SEPARATOR_STR}{}", path_name(&path)),
                None => path_name(&path).to_owned(),
            };
            let subs = scan_video_dir(path, discoverer, depth.saturating_sub(1), Some(prefix))?;

            videos.extend(subs);
        }
    }

    Some(videos)
}

/// Returns the (path, name) if valid extension and valid utf8 name
fn scan_file(path: PathBuf, discoverer: Option<&Discoverer>) -> Option<(String, String, u64)> {
    tracing::info!("Scanning file path {}", path.display());
    let is_video = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EXTENSIONS.contains(&ext))
        .unwrap_or_default();

    if !is_video {
        return None;
    }

    let url = url::Url::from_file_path(&path)
        .inspect_err(|_| tracing::error!("Scan file url error on {}", path.display()))
        .ok();
    let duration = match discoverer.zip(url) {
        Some((discoverer, url)) => discoverer
            .discover_uri(url.as_str())
            .inspect_err(|error| {
                tracing::error!("Scan discover error on {}. Error {error}", path.display())
            })
            .ok()
            .and_then(|info| info.duration().map(|clock| clock.seconds()))
            .unwrap_or_default(),
        _ => 0,
    };

    let name = path.file_stem().and_then(|name| name.to_str())?.to_owned();

    let path = path.file_name().and_then(|path| path.to_str())?.to_owned();

    Some((path, name, duration))
}

fn path_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Invalid UTF8 name")
}

fn get_existing_show(db: &Database, dir: &Directory, path: &str) -> rusqlite::Result<ShowId> {
    use rusqlite::types::ToSqlOutput;
    let sql = "SELECT * FROM tv_show WHERE directory=:dir AND path=:path";

    let mut statement = db.prepare_cached(sql)?;
    statement.query_row(
        &[
            (":dir", &ToSqlOutput::from(dir.id)),
            (":path", &ToSqlOutput::from(path)),
        ],
        ShowId::from_row,
    )
}

fn get_existing_season(db: &Database, show: ShowId, path: &str) -> rusqlite::Result<SeasonId> {
    use rusqlite::types::ToSqlOutput;
    let sql = "SELECT * FROM season WHERE show_id=:show_id AND path=:path";

    let mut statement = db.prepare_cached(sql)?;
    statement.query_row(
        &[
            (":show_id", &ToSqlOutput::from(show)),
            (":path", &ToSqlOutput::from(path)),
        ],
        SeasonId::from_row,
    )
}

fn process_name(name: &str) -> Option<String> {
    let value = MOVIE_REG1
        .find(name)
        .inspect_err(|err| tracing::error!("Name processing Error {name:}.\n{err:?}"))
        .ok()
        .and_then(|val| val)
        .map(|val| val.as_str())?;

    let value = MOVIE_REG2
        .find(value)
        .inspect_err(|err| tracing::error!("Name processing Error {name:}.\n{err:?}"))
        .ok()
        .and_then(|val| val)
        .map(|value| value.as_str())?;

    let cleaned = CLEANER.replace_all(value, " ").trim().to_owned();

    Some(cleaned)
}

fn process_season(name: &str) -> Option<u16> {
    let value = SEASON_REG
        .find(name)
        .inspect_err(|err| tracing::error!("{err:?}"))
        .ok()
        .and_then(|val| val);

    let value = value?.as_str().trim();

    value
        .parse::<u16>()
        .inspect_err(|err| tracing::error!("Season processing Error {name:}.\n{err:?}"))
        .ok()
}

fn process_episode(name: &str) -> Option<u16> {
    let value = EPISODE_REG
        .find(name)
        .inspect_err(|err| tracing::error!("{err:?}"))
        .ok()
        .and_then(|val| val);

    let value = value?.as_str().trim();

    value
        .parse::<u16>()
        .inspect_err(|err| tracing::error!("Episode processing Error {name:}.\n{err:?}"))
        .ok()
}
