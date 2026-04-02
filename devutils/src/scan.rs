use core::error;
use fancy_regex::Regex;
use gstreamer_pbutils::Discoverer;
use registry::db::{BatchResult, Database};
use registry::models::{
    Directory, DirectoryId, Episode, EpisodeId, MediaType, Movie, MovieId, Season, SeasonId, Show,
    ShowId, Subtitle, SubtitleId, VideoId,
};
use rusqlite::OptionalExtension;
use rusqlite::types::{ToSqlOutput, ValueRef};
use std::collections::HashMap;
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

/// Ordered by most likely
#[rustfmt::skip]
pub const SUB_EXT: &[&str] = &[
    "srt", 
    "ass",
    "ssa",
    "vtt",
    "sub",
    "ttml",
    "dfxp",
    "sbv",
    "lrc",
];

/// Ordered by most likely
#[rustfmt::skip]
pub const VIDEO_EXT: &[&str] = &[
    "mp4", 
    "mkv",
    "mov",
    "webm",
    "m4v",
    "avi",
    "wmv",
    "mpeg",
    "mpg",
    "flv",
];

#[derive(Debug)]
struct Video {
    name: String,
    path: String,
    loaded_sub: Option<String>,
    embedded_subs: Vec<(String, String)>,
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
    preferred_subtitle_code: Option<String>,
) -> Option<BatchResult<'a>> {
    tracing::debug!("Scanning directory {}", dir.path);
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

    scan_dir_helper(
        &mut db,
        dir,
        discoverer.as_ref(),
        movie_depth,
        restore,
        preferred_subtitle_code.as_ref(),
    )
}

pub fn scan_dirs<'a>(
    db: impl AsRef<Path>,
    dirs: Vec<Directory>,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<String>,
) -> (Option<BatchResult<'a>>, Vec<DirectoryId>) {
    tracing::debug!("Scanning {} directories", dirs.len());
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
        match scan_dir_helper(
            &mut db,
            dir,
            discoverer.as_ref(),
            movie_depth,
            restore,
            preferred_subtitle_code.as_ref(),
        ) {
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
    preferred_subtitle_code: Option<&String>,
) -> Option<BatchResult<'a>> {
    let mut successes = vec![];
    let mut failures = vec![];

    match dir.media_type {
        MediaType::Movies => {
            tracing::debug!("Scanning movie directory {}", dir.path);
            struct DirMovie {
                id: MovieId,
                tombstone: bool,
                scanned: bool,
            }

            let Some(videos) = scan_video_dir(&dir.path, discoverer, movie_depth, None) else {
                return None;
            };

            tracing::debug!("Fetching Directory movies");
            let mut dir_movies = match db.get_dir_movies(dir.id, |row| {
                let id = MovieId::from_row(row)?;
                let path = row.get::<_, String>("path")?;
                let tombstone = row.get::<_, bool>("removed")?;
                Ok((
                    path,
                    DirMovie {
                        id,
                        tombstone,
                        scanned: false,
                    },
                ))
            }) {
                Ok(dir_movies) => {
                    // todo: Db could return an iterator instead?
                    let mut map = HashMap::new();

                    map.extend(dir_movies.into_iter());

                    map
                }
                Err(error) => {
                    tracing::error!("Directory movies error. \n{error}");
                    return None;
                }
            };

            for movie in videos {
                let dir_movie = dir_movies.get_mut(&movie.path);
                let pick_sub = dir_movie.is_none();

                if dir_movie
                    .as_ref()
                    .map(|mv| mv.tombstone)
                    .unwrap_or_default()
                    && !restore
                {
                    continue;
                }

                let name = process_name(&movie.name).unwrap_or(movie.name.clone());
                let (new, query) =
                    Movie::new(dir.id, movie.path.clone(), name, movie.name, movie.duration);
                let id = dir_movie.as_ref().map(|mv| mv.id).unwrap_or(new.id);

                match query.execute(db) {
                    Ok(succ) => {
                        if let Some(entry) = dir_movie {
                            entry.scanned = true;
                        }

                        successes.push(succ)
                    }
                    Err(fail) => failures.push(fail),
                };

                let loaded = movie.loaded_sub.map(|path| Subtitle::new_loaded(id, path));
                let subtitles = movie
                    .embedded_subs
                    .into_iter()
                    .map(|(title, lang)| Subtitle::new_embedded(id, title, lang))
                    .chain(loaded.into_iter());

                for sub in subtitles {
                    let query = sub.insert();
                    match query.execute(db) {
                        Ok(succ) => successes.push(succ),
                        Err(fail) => failures.push(fail),
                    }
                }

                if pick_sub {
                    pick_subtitle(db, id, preferred_subtitle_code);
                }
            }

            tracing::debug!("Performing movies insert/remove");

            let deletes = dir_movies
                .into_values()
                .filter_map(|value| {
                    if value.scanned {
                        None
                    } else {
                        Some((value.id, false))
                    }
                })
                .collect();

            if let Err(error) = db.insert_remove_movies(deletes) {
                tracing::error!("{error}")
            };
        }
        MediaType::Shows => {
            struct DirEpisode {
                id: EpisodeId,
                tombstone: bool,
                scanned: bool,
            }

            struct DirSeason {
                id: SeasonId,
                scanned: bool,
                tombstone: bool,
            }

            struct DirShow {
                id: ShowId,
                scanned: bool,
                tombstone: bool,
            }

            tracing::debug!("Scanning shows directory {}", dir.path);
            let shows = scan_shows(&dir.path, discoverer)?;

            tracing::debug!("Fetching Directory shows");
            let mut dir_shows = match db.get_dir_shows(dir.id, |row| {
                let id = ShowId::from_row(row)?;
                let path = row.get::<_, String>("path")?;
                let tombstone = row.get::<_, bool>("removed")?;

                Ok((
                    path,
                    DirShow {
                        id,
                        scanned: false,
                        tombstone,
                    },
                ))
            }) {
                Ok(shows) => {
                    let mut map = HashMap::new();
                    map.extend(shows.into_iter());
                    map
                }
                Err(error) => {
                    tracing::error!("Directory shows error. \n{error}");
                    return None;
                }
            };

            for show in shows {
                let dir_show = dir_shows.get_mut(&show.path);

                if dir_show
                    .as_ref()
                    .map(|show| show.tombstone)
                    .unwrap_or_default()
                    && !restore
                {
                    continue;
                }

                let ShowPrim { path, seasons } = show;
                let name = process_name(&path).unwrap_or(path.clone());
                let (new, query) = Show::new(
                    dir.id,
                    path.clone(),
                    name.clone(),
                    path.clone(),
                    seasons.len() as _,
                );

                let show = dir_show.as_ref().map(|show| show.id).unwrap_or(new.id);

                match query.execute(db) {
                    Ok(succ) => {
                        let modified = succ.rows > 0;
                        successes.push(succ);

                        if let Some(entry) = dir_show {
                            entry.scanned = true;
                        }

                        modified
                    }
                    Err(error) => {
                        failures.push(error);
                        continue;
                    }
                };

                tracing::debug!("Scanning {name} seasons");
                tracing::debug!("Fetching show seasons");

                let mut dir_seasons = match db.get_show_seasons_removed(show, |row| {
                    let id = SeasonId::from_row(row)?;
                    let path = row.get::<_, String>("path")?;
                    let tombstone = row.get::<_, bool>("removed")?;

                    Ok((
                        path,
                        DirSeason {
                            id,
                            scanned: false,
                            tombstone,
                        },
                    ))
                }) {
                    Ok(seasons) => {
                        let mut map = HashMap::new();
                        map.extend(seasons.into_iter());
                        map
                    }
                    Err(error) => {
                        tracing::error!("Directory seasons error. \n{error}");
                        continue;
                    }
                };

                for season in seasons {
                    let dir_season = dir_seasons.get_mut(&season.path);

                    if dir_season
                        .as_ref()
                        .map(|sea| sea.tombstone)
                        .unwrap_or_default()
                        && !restore
                    {
                        continue;
                    }

                    let SeasonPrim { path, episodes } = season;
                    let number = process_season(&path);
                    let name = match number {
                        Some(number) => format!("Season {number:02}"),
                        None => path.clone(),
                    };

                    let (season, query) = Season::new(show, name.clone(), path.clone(), number);

                    let season = dir_season.as_ref().map(|sea| sea.id).unwrap_or(season.id);

                    match query.execute(db) {
                        Ok(succ) => {
                            let modified = succ.rows > 0;

                            if let Some(entry) = dir_season {
                                entry.scanned = true;
                            }

                            successes.push(succ);
                            modified
                        }
                        Err(error) => {
                            failures.push(error);
                            continue;
                        }
                    };

                    tracing::debug!("Scanning {name} episodes");
                    tracing::debug!("Fetching season episodes");
                    let mut dir_episodes = match db.get_season_episodes_removed(season, |row| {
                        let id = EpisodeId::from_row(row)?;
                        let path = row.get::<_, String>("path")?;
                        let tombstone = row.get::<_, bool>("removed")?;

                        Ok((
                            path,
                            DirEpisode {
                                id,
                                tombstone,
                                scanned: false,
                            },
                        ))
                    }) {
                        Ok(episodes) => {
                            let mut map = HashMap::new();
                            map.extend(episodes.into_iter());

                            map
                        }
                        Err(error) => {
                            tracing::error!("Directory episodes error. \n{error}");
                            continue;
                        }
                    };

                    for episode in episodes {
                        let dir_ep = dir_episodes.get_mut(&episode.path);
                        let pick_sub = dir_ep.is_none();

                        if dir_ep.as_ref().map(|ep| ep.tombstone).unwrap_or_default() && !restore {
                            continue;
                        }

                        let number = process_episode(&episode.path);
                        let name = match number {
                            Some(number) => format!("Episode {number:02}"),
                            None => episode.name.clone(),
                        };

                        let (new, query) = Episode::new(
                            season,
                            name,
                            episode.name,
                            episode.path.clone(),
                            episode.duration,
                            number,
                        );

                        let episode_id = dir_ep.as_ref().map(|ep| ep.id).unwrap_or(new.id);

                        match query.execute(db) {
                            Ok(succ) => {
                                if let Some(entry) = dir_ep {
                                    entry.scanned = true;
                                }

                                successes.push(succ)
                            }
                            Err(fail) => failures.push(fail),
                        }

                        let loaded = episode
                            .loaded_sub
                            .map(|path| Subtitle::new_loaded(episode_id, path));

                        let subtitles = episode
                            .embedded_subs
                            .into_iter()
                            .map(|(title, lang)| Subtitle::new_embedded(episode_id, title, lang))
                            .chain(loaded.into_iter());

                        for sub in subtitles {
                            let query = sub.insert();
                            match query.execute(db) {
                                Ok(succ) => successes.push(succ),
                                Err(fail) => failures.push(fail),
                            }
                        }

                        if pick_sub {
                            pick_subtitle(db, episode_id, preferred_subtitle_code);
                        }
                    }

                    tracing::debug!("Performing episodes insert/remove");

                    let deletes = dir_episodes
                        .into_values()
                        .filter_map(|value| {
                            if value.scanned {
                                None
                            } else {
                                Some((value.id, false))
                            }
                        })
                        .collect();

                    if let Err(error) = db.insert_remove_episodes(deletes) {
                        tracing::error!("{error}")
                    };
                }

                tracing::debug!("Performing season insert/remove");

                let deletes = dir_seasons
                    .into_values()
                    .filter_map(|value| {
                        if value.scanned {
                            None
                        } else {
                            Some((value.id, false))
                        }
                    })
                    .collect();

                if let Err(error) = db.insert_remove_seasons(deletes) {
                    tracing::error!("{error}")
                };
            }

            tracing::debug!("Performing movies insert/remove");

            let deletes = dir_shows
                .into_values()
                .filter_map(|value| {
                    if value.scanned {
                        None
                    } else {
                        Some((value.id, false))
                    }
                })
                .collect();

            if let Err(error) = db.insert_remove_shows(deletes) {
                tracing::error!("{error}")
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

    tracing::debug!("Scanning video directory {}", path.display());

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
            if let Some(mut video) = scan_file(path, discoverer) {
                if let Some(prefix) = prefix.as_ref() {
                    video.path = format!("{prefix}{MAIN_SEPARATOR_STR}{}", video.path);
                }

                videos.push(video)
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
fn scan_file(path: PathBuf, discoverer: Option<&Discoverer>) -> Option<Video> {
    tracing::debug!("Scanning file path {}", path.display());
    let is_video = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXT.contains(&ext))
        .unwrap_or_default();

    if !is_video {
        return None;
    }

    let url = url::Url::from_file_path(&path)
        .inspect_err(|_| tracing::error!("Scan file url error on {}", path.display()))
        .ok();

    let (duration, embedded_subs) = match discoverer.zip(url) {
        Some((discoverer, url)) => discover(discoverer, url, &path),
        None => (0, vec![]),
    };

    let name = path.file_stem().and_then(|name| name.to_str())?.to_owned();
    let loaded_sub = subtitles(&path);
    let path = path.file_name().and_then(|path| path.to_str())?.to_owned();

    Some(Video {
        name,
        path,
        loaded_sub,
        embedded_subs,
        duration,
    })
}

fn subtitles(path: &PathBuf) -> Option<String> {
    for ext in SUB_EXT {
        let sub = path.with_extension(ext);

        if let Ok(true) = sub.try_exists() {
            return Some(sub.display().to_string());
        }
    }

    None
}

fn path_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Invalid UTF8 name")
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

fn discover(discoverer: &Discoverer, url: url::Url, path: &Path) -> (u64, Vec<(String, String)>) {
    let discovered = discoverer
        .discover_uri(url.as_str())
        .inspect_err(|error| {
            tracing::error!("Scan discover error on {}. Error {error}", path.display())
        })
        .ok();

    match discovered {
        Some(info) => {
            use gstreamer_pbutils::prelude::DiscovererStreamInfoExt;

            let subs = info
                .subtitle_streams()
                .into_iter()
                .map(|sub| {
                    sub.tags().and_then(|info| {
                        let title = info
                            .get::<gstreamer::tags::Title>()
                            .map(|code| code.get().to_owned());

                        let lang = info
                            .get::<gstreamer::tags::LanguageCode>()
                            .map(|code| code.get().to_owned());

                        title.zip(lang)
                    })
                })
                .flatten()
                .collect::<Vec<_>>();

            let duration = info
                .duration()
                .map(|clock| clock.seconds())
                .unwrap_or_default();

            (duration, subs)
        }
        None => (0, vec![]),
    }
}

fn pick_subtitle(db: &Database, id: impl Into<VideoId>, preferred: Option<&String>) {
    let id = id.into();

    let table = if matches!(id, VideoId::Movie(_)) {
        "movie"
    } else {
        "episode"
    };

    let res = match preferred {
        Some(lang) => db
            .query_row(
                "SELECT id FROM subtitle WHERE video=:video AND (path NOT NULL OR lang=:lang)",
                &[
                    (
                        ":lang",
                        &ToSqlOutput::Borrowed(ValueRef::Text(lang.as_bytes())),
                    ),
                    (":video", &ToSqlOutput::from(id)),
                ],
                SubtitleId::from_row,
            )
            .optional(),
        None => db
            .query_row(
                "SELECT id FROM subtitle WHERE video=:video AND path NOT NULL",
                &[(":video", &ToSqlOutput::from(id))],
                SubtitleId::from_row,
            )
            .optional(),
    };

    let sql = format!("UPDATE {table} SET subtitle_id=:subtitle WHERE id=:id");
    match res {
        Ok(None) => {}
        Ok(Some(subtitle_id)) => {
            if let Err(error) = db.execute(
                &sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":subtitle", &ToSqlOutput::from(subtitle_id)),
                ],
            ) {
                tracing::error!("Set movie subtitle error.\n {error}");
            };
        }
        Err(error) => {
            tracing::error!("Select movie subtitle error. \n{error}");
        }
    }
}
