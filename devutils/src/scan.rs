use crate::source::SourceSet;
use core::error;
use fancy_regex::Regex;
use gstreamer_pbutils::Discoverer;
use registry::db::{BatchResult, Database};
use registry::models::{
    Audio, AudioId, Directory, DirectoryId, Episode, EpisodeId, MediaType, Movie, MovieId, Season,
    SeasonId, Show, ShowId, Subtitle, SubtitleId, VideoId, video,
};
use rusqlite::OptionalExtension;
use rusqlite::types::{ToSqlOutput, Value, ValueRef};
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
    embedded_subs: Vec<SubtitleInfo>,
    audio: Vec<AudioInfo>,
    video: Vec<VideoInfo>,
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

#[derive(Debug)]
struct SubtitleInfo {
    title: String,
    lang: String,
}

#[derive(Debug)]
struct AudioInfo {
    stream: u32,
    codec: Option<String>,
    lang: Option<String>,
    channels: u32,
    sample_rate: u32,
    bitrate: u32,
    depth: u32,
}

#[derive(Debug)]
struct VideoInfo {
    stream: u32,
    tag: Option<String>,
    codec: Option<String>,
    bitrate: u32,
    width: u32,
    height: u32,
    depth: u32,
    framerate: f32,
    interlaced: bool,
    /// Display Aspect Ratio
    dar_num: u32,
    dar_denom: u32,
}

pub fn scan_dir<'a>(
    db: &str,
    dir: Directory,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<String>,
    preferred_audio_code: Option<String>,
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
        preferred_audio_code.as_ref(),
    )
}

pub fn scan_dirs<'a>(
    db: impl AsRef<Path>,
    dirs: Vec<Directory>,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<String>,
    preferred_audio_code: Option<String>,
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
            preferred_audio_code.as_ref(),
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
    preferred_audio_code: Option<&String>,
) -> Option<BatchResult<'a>> {
    let default_source = SourceSet::from_str(&dir.source);
    let mut successes = vec![];
    let mut failures = vec![];

    match dir.media_type {
        MediaType::Movies => {
            tracing::debug!("Scanning movie directory {}", dir.path);
            struct DirMovie {
                id: MovieId,
                tombstone: bool,
                scanned: bool,
                request: Option<String>,
                source: SourceSet,
            }

            let videos = scan_video_dir(&dir.path, discoverer, movie_depth, None)?;

            tracing::debug!("Fetching Directory movies");

            let mut dir_movies = match db.get_dir_movies(dir.id, |row| {
                let id = MovieId::from_row(row)?;
                let path = row.get::<_, String>("path")?;
                let tombstone = row.get::<_, bool>("removed")?;

                let request = row.get::<_, Option<String>>("request")?;
                let source = SourceSet::from_row(row, "source")?;

                Ok((
                    path,
                    DirMovie {
                        id,
                        tombstone,
                        scanned: false,
                        request,
                        source,
                    },
                ))
            }) {
                Ok(dir_movies) => {
                    // todo: Db could return an iterator instead?
                    let mut map = HashMap::new();

                    map.extend(dir_movies);

                    map
                }
                Err(error) => {
                    tracing::error!("Directory movies error. \n{error}");
                    return None;
                }
            };

            for movie in videos {
                let mut dir_movie = dir_movies.get_mut(&movie.path);
                let pick_preferred = dir_movie.is_none();

                if dir_movie
                    .as_ref()
                    .map(|mv| mv.tombstone)
                    .unwrap_or_default()
                    && !restore
                {
                    continue;
                }

                let name = process_name(&movie.name).unwrap_or(movie.name.clone());
                let (new, query) = Movie::new(
                    dir.id,
                    movie.path.clone(),
                    name.clone(),
                    movie.name,
                    movie.duration,
                );
                let id = dir_movie.as_ref().map(|mv| mv.id).unwrap_or(new.id);

                match query.execute(db) {
                    Ok(succ) => {
                        if let Some(entry) = dir_movie.as_mut() {
                            entry.scanned = true;
                        }

                        successes.push(succ)
                    }
                    Err(fail) => failures.push(fail),
                };

                let movie_source = |source: SourceSet| match source
                    .movie_request(id, name)
                    .map(|(query, request)| (query.execute(db), request))
                {
                    Some((Ok(succ), request)) => {
                        successes.push(succ);

                        if let Err(error) = db.execute(
                            "UPDATE movie SET source=:source, request=:request WHERE id=:id",
                            &[
                                (":id", &ToSqlOutput::from(id)),
                                (":source", &ToSqlOutput::from(source)),
                                (":request", &ToSqlOutput::from(request.as_str())),
                            ],
                        ) {
                            tracing::error!("Could not insert movie request/source. \n{error}");
                        };
                    }
                    Some((Err(fail), _)) => {
                        failures.push(fail);
                    }
                    None => {}
                };

                match dir_movie {
                    Some(dir_movie) => match &dir_movie.request {
                        Some(_) => {}
                        None => {
                            let source = dir_movie.source.merge(default_source);

                            movie_source(source)
                        }
                    },
                    None => movie_source(default_source),
                }

                save_video_metadata(
                    db,
                    &mut successes,
                    &mut failures,
                    id,
                    movie.embedded_subs,
                    movie.loaded_sub,
                    pick_preferred,
                    preferred_subtitle_code,
                    movie.audio,
                    preferred_audio_code,
                    movie.video,
                );
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
                source: SourceSet,
                request: Option<String>,
            }

            struct DirSeason {
                id: SeasonId,
                scanned: bool,
                tombstone: bool,
                request: Option<String>,
                source: SourceSet,
            }

            struct DirShow {
                id: ShowId,
                scanned: bool,
                tombstone: bool,
                request: Option<String>,
                source: SourceSet,
            }

            tracing::debug!("Scanning shows directory {}", dir.path);
            let shows = scan_shows(&dir.path, discoverer)?;

            tracing::debug!("Fetching Directory shows");
            let mut dir_shows = match db.get_dir_shows(dir.id, |row| {
                let id = ShowId::from_row(row)?;
                let path = row.get::<_, String>("path")?;
                let tombstone = row.get::<_, bool>("removed")?;

                let request = row.get::<_, Option<String>>("request")?;
                let source = SourceSet::from_row(row, "source")?;

                Ok((
                    path,
                    DirShow {
                        id,
                        scanned: false,
                        tombstone,
                        request,
                        source,
                    },
                ))
            }) {
                Ok(shows) => {
                    let mut map = HashMap::new();
                    map.extend(shows);
                    map
                }
                Err(error) => {
                    tracing::error!("Directory shows error. \n{error}");
                    return None;
                }
            };

            for show in shows {
                let mut dir_show = dir_shows.get_mut(&show.path);

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
                        successes.push(succ);

                        if let Some(entry) = dir_show.as_mut() {
                            entry.scanned = true;
                        }
                    }
                    Err(error) => {
                        failures.push(error);
                        continue;
                    }
                };

                let mut show_source = |source: SourceSet, name: String| match source
                    .show_request(show, name)
                    .map(|(query, request)| (query.execute(db), request))
                {
                    Some((Ok(succ), request)) => {
                        successes.push(succ);

                        if let Err(error) = db.execute(
                            "UPDATE tv_show SET source=:source, request=:request WHERE id=:id",
                            &[
                                (":id", &ToSqlOutput::from(show)),
                                (":source", &ToSqlOutput::from(source)),
                                (":request", &ToSqlOutput::from(request.as_str())),
                            ],
                        ) {
                            tracing::error!("Could not insert show request/source. \n{error}");
                        };

                        (source, Some(request))
                    }
                    Some((Err(fail), _)) => {
                        failures.push(fail);
                        (source, None)
                    }
                    None => (source, None),
                };

                let (show_source, show_request) = match dir_show {
                    Some(dir_show) => match &dir_show.request {
                        Some(request) => (dir_show.source, Some(request.to_owned())),
                        None => {
                            let source = dir_show.source.merge(default_source);

                            show_source(source, name.clone())
                        }
                    },
                    None => show_source(default_source, name.clone()),
                };

                tracing::debug!("Scanning {name} seasons");
                tracing::debug!("Fetching show seasons");

                let mut dir_seasons = match db.get_show_seasons_removed(show, |row| {
                    let id = SeasonId::from_row(row)?;
                    let path = row.get::<_, String>("path")?;
                    let tombstone = row.get::<_, bool>("removed")?;

                    let request = row.get::<_, Option<String>>("request")?;
                    let source = SourceSet::from_row(row, "source")?;

                    Ok((
                        path,
                        DirSeason {
                            id,
                            scanned: false,
                            tombstone,
                            request,
                            source,
                        },
                    ))
                }) {
                    Ok(seasons) => {
                        let mut map = HashMap::new();
                        map.extend(seasons);
                        map
                    }
                    Err(error) => {
                        tracing::error!("Directory seasons error. \n{error}");
                        continue;
                    }
                };

                for season in seasons {
                    let mut dir_season = dir_seasons.get_mut(&season.path);

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

                    let season_number = number.unwrap_or_default();

                    let (season, query) =
                        Season::new(show, name.clone(), path.clone(), season_number);

                    let season = dir_season.as_ref().map(|sea| sea.id).unwrap_or(season.id);

                    match query.execute(db) {
                        Ok(succ) => {
                            let modified = succ.rows > 0;

                            if let Some(entry) = dir_season.as_mut() {
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

                    let mut season_source = |source: SourceSet| {
                        let Some(parent) = show_request.as_deref() else {
                            return (source, None);
                        };

                        match source
                            .season_request(season, parent, season_number)
                            .map(|(query, request)| (query.execute(db), request))
                        {
                            Some((Ok(succ), request)) => {
                                successes.push(succ);

                                if let Err(error) = db.execute(
                                "UPDATE season SET source=:source, request=:request WHERE id=:id",
                                &[
                                    (":id", &ToSqlOutput::from(season)),
                                    (":source", &ToSqlOutput::from(source)),
                                    (":request", &ToSqlOutput::from(request.as_str())),
                                ],
                            ) {
                                tracing::error!(
                                    "Could not insert season request/source. \n{error}"
                                );
                            };

                                (source, Some(request))
                            }
                            Some((Err(fail), _)) => {
                                failures.push(fail);
                                (source, None)
                            }
                            None => (source, None),
                        }
                    };

                    let (season_source, season_request) = match dir_season {
                        Some(dir_season) => match &dir_season.request {
                            Some(request) => (dir_season.source, Some(request.to_owned())),
                            None => {
                                let source = dir_season.source.merge(show_source);

                                season_source(source)
                            }
                        },
                        None => season_source(show_source),
                    };

                    tracing::debug!("Scanning {name} episodes");
                    tracing::debug!("Fetching season episodes");
                    let mut dir_episodes = match db.get_season_episodes_removed(season, |row| {
                        let id = EpisodeId::from_row(row)?;
                        let path = row.get::<_, String>("path")?;
                        let tombstone = row.get::<_, bool>("removed")?;

                        let request = row.get::<_, Option<String>>("request")?;
                        let source = SourceSet::from_row(row, "source")?;

                        Ok((
                            path,
                            DirEpisode {
                                id,
                                tombstone,
                                scanned: false,
                                request,
                                source,
                            },
                        ))
                    }) {
                        Ok(episodes) => {
                            let mut map = HashMap::new();
                            map.extend(episodes);

                            map
                        }
                        Err(error) => {
                            tracing::error!("Directory episodes error. \n{error}");
                            continue;
                        }
                    };

                    for episode in episodes {
                        let mut dir_ep = dir_episodes.get_mut(&episode.path);
                        let pick_sub = dir_ep.is_none();

                        if dir_ep.as_ref().map(|ep| ep.tombstone).unwrap_or_default() && !restore {
                            continue;
                        }

                        let number = process_episode(&episode.path);
                        let name = match number {
                            Some(number) => format!("Episode {number:02}"),
                            None => episode.name.clone(),
                        };

                        let episode_number = number.unwrap_or_default();

                        let (new, query) = Episode::new(
                            season,
                            name,
                            episode.name,
                            episode.path.clone(),
                            episode.duration,
                            episode_number,
                        );

                        let episode_id = dir_ep.as_ref().map(|ep| ep.id).unwrap_or(new.id);

                        match query.execute(db) {
                            Ok(succ) => {
                                if let Some(entry) = dir_ep.as_mut() {
                                    entry.scanned = true;
                                }

                                successes.push(succ)
                            }
                            Err(fail) => failures.push(fail),
                        }

                        let mut episode_source = |source: SourceSet| {
                            let Some(parent) = season_request.as_deref() else {
                                return;
                            };

                            match source
                                .episode_request(episode_id, parent, season_number, episode_number)
                                .map(|(query, request)| (query.execute(db), request))
                            {
                                Some((Ok(succ), request)) => {
                                    successes.push(succ);

                                    if let Err(error) = db.execute(
                                    "UPDATE episode SET source=:source, request=:request WHERE id=:id",
                                    &[
                                        (":id", &ToSqlOutput::from(episode_id)),
                                        (":source", &ToSqlOutput::from(source)),
                                        (":request", &ToSqlOutput::from(request.as_str())),
                                    ],
                                ) {
                                    tracing::error!(
                                        "Could not insert episode request/source. \n{error}"
                                    );
                                };
                                }
                                Some((Err(fail), _)) => {
                                    failures.push(fail);
                                }
                                None => {}
                            }
                        };

                        match dir_ep {
                            Some(dir_episode) => match &dir_episode.request {
                                Some(_) => {}
                                None => {
                                    let source = dir_episode.source.merge(season_source);

                                    episode_source(source)
                                }
                            },
                            None => episode_source(season_source),
                        }

                        save_video_metadata(
                            db,
                            &mut successes,
                            &mut failures,
                            episode_id,
                            episode.embedded_subs,
                            episode.loaded_sub,
                            pick_sub,
                            preferred_subtitle_code,
                            episode.audio,
                            preferred_audio_code,
                            episode.video,
                        );
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

    let (duration, embedded_subs, audio, video) = match discoverer.zip(url) {
        Some((discoverer, url)) => discover(discoverer, url, &path),
        None => (0, vec![], vec![], vec![]),
    };

    let name = path.file_stem().and_then(|name| name.to_str())?.to_owned();
    let loaded_sub = subtitles(&path);
    let path = path.file_name().and_then(|path| path.to_str())?.to_owned();

    Some(Video {
        name,
        path,
        loaded_sub,
        embedded_subs,
        audio,
        video,
        duration,
    })
}

fn subtitles(path: &Path) -> Option<String> {
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

fn discover(
    discoverer: &Discoverer,
    url: url::Url,
    path: &Path,
) -> (u64, Vec<SubtitleInfo>, Vec<AudioInfo>, Vec<VideoInfo>) {
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
                .filter_map(|sub| {
                    let tags = sub.tags()?;
                    let title = tags
                        .get::<gstreamer::tags::Title>()
                        .map(|code| code.get().to_owned())?;
                    let lang = tags
                        .get::<gstreamer::tags::LanguageCode>()
                        .map(|code| code.get().to_owned())?;

                    Some(SubtitleInfo { title, lang })
                })
                .collect::<Vec<_>>();

            let mut audios = vec![];
            let mut audio_stream = 0;

            for audio in info.audio_streams().into_iter() {
                let caps = audio.caps();
                let codec = caps.as_ref().map(|caps| {
                    gstreamer_pbutils::pb_utils_get_codec_description(caps).to_string()
                });

                let stream = audio_stream;
                audio_stream += 1;

                let depth = audio.depth();
                let lang = audio.language().map(|lang| lang.to_string());
                let channels = audio.channels();
                let sample_rate = audio.sample_rate();
                let bitrate = audio.bitrate();

                let audio = AudioInfo {
                    codec,
                    lang,
                    stream,
                    channels,
                    sample_rate,
                    bitrate,
                    depth,
                };

                audios.push(audio);
            }

            let mut videos = vec![];
            let mut video_stream = 0;

            for video in info.video_streams().into_iter() {
                let caps = video.caps();
                let codec = caps.as_ref().map(|caps| {
                    gstreamer_pbutils::pb_utils_get_codec_description(caps).to_string()
                });

                let tag = video.tags().and_then(|tags| {
                    tags.get::<gstreamer::tags::VideoCodec>()
                        .map(|tag| tag.get().to_owned())
                });

                let bitrate = video.bitrate();
                let par = video.par();

                let depth = video.depth();
                let width = video.width();
                let height = video.height();
                let framerate = video.framerate();
                let framerate = (framerate.numer() as f32) / (framerate.denom() as f32);

                let dar_num = width * par.numer() as u32;
                let dar_denom = height * par.denom() as u32;
                let interlaced = video.is_interlaced();

                let stream = video_stream;
                video_stream += 1;

                let video = VideoInfo {
                    stream,
                    tag,
                    codec,
                    bitrate,
                    width,
                    height,
                    depth,
                    framerate,
                    interlaced,
                    dar_num,
                    dar_denom,
                };

                videos.push(video)
            }

            let duration = info
                .duration()
                .map(|clock| clock.seconds())
                .unwrap_or_default();

            (duration, subs, audios, videos)
        }
        None => (0, vec![], vec![], vec![]),
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
                tracing::error!("Set {table} subtitle error.\n {error}");
            };
        }
        Err(error) => {
            tracing::error!("Select {table} subtitle error. \n{error}");
        }
    }
}

fn pick_audio(db: &Database, id: impl Into<VideoId>, preferred: Option<&String>) {
    let Some(preferred) = preferred else {
        return;
    };

    let id = id.into();

    let table = if matches!(id, VideoId::Movie(_)) {
        "movie"
    } else {
        "episode"
    };

    let res = db
        .query_row(
            "SELECT id FROM audio WHERE media=:media AND lang=:lang",
            &[
                (
                    ":lang",
                    &ToSqlOutput::Borrowed(ValueRef::Text(preferred.as_bytes())),
                ),
                (":media", &ToSqlOutput::from(id)),
            ],
            AudioId::from_row,
        )
        .optional();

    let sql = format!("UPDATE {table} SET audio_id=:audio WHERE id=:id");
    match res {
        Ok(None) => {}
        Ok(Some(audio_id)) => {
            if let Err(error) = db.execute(
                &sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":audio", &ToSqlOutput::from(audio_id)),
                ],
            ) {
                tracing::error!("Set {table} audio error.\n {error}");
            };
        }
        Err(error) => {
            tracing::error!("Select {table} audio error. \n{error}");
        }
    }
}

fn save_video_metadata(
    db: &Database,
    successes: &mut Vec<registry::db::Success>,
    failures: &mut Vec<registry::db::Failure<'_>>,
    id: impl Into<VideoId>,
    subtitles: Vec<SubtitleInfo>,
    loaded_sub: Option<String>,
    pick_preferred: bool,
    preferred_subtitle_code: Option<&String>,
    audio: Vec<AudioInfo>,
    preferred_audio_code: Option<&String>,
    videos: Vec<VideoInfo>,
) {
    let id = id.into();
    let loaded = loaded_sub.map(|path| Subtitle::new_loaded(id, path));
    let subtitles = subtitles
        .into_iter()
        .map(|sub| Subtitle::new_embedded(id, sub.title, sub.lang))
        .chain(loaded);

    for sub in subtitles {
        let query = sub.insert();
        match query.execute(db) {
            Ok(succ) => successes.push(succ),
            Err(fail) => failures.push(fail),
        }
    }

    let mut selected_audio = None;

    let audios = audio.into_iter().map(|audio| {
        Audio::new(
            id,
            audio.codec,
            audio.lang,
            audio.channels,
            audio.sample_rate,
            audio.bitrate,
            audio.depth,
            audio.stream,
        )
    });

    for audio in audios {
        if selected_audio.is_none() {
            selected_audio = Some(audio.id);
        }

        let query = audio.insert();
        match query.execute(db) {
            Ok(succ) => successes.push(succ),
            Err(fail) => failures.push(fail),
        }
    }

    let selected_audio = selected_audio
        .map(|audio| ToSqlOutput::from(audio))
        .unwrap_or(ToSqlOutput::Owned(Value::Null));

    if let Err(error) = db.execute(
        "UPDATE movie SET audio_id=:audio WHERE id=:id",
        &[(":audio", &selected_audio), (":id", &ToSqlOutput::from(id))],
    ) {
        tracing::error!("Failed to save movie audio. \n{error}");
    };

    let videos = videos.into_iter().map(|video| {
        video::VideoInfo::new(
            id,
            video.tag,
            video.codec,
            video.bitrate,
            video.width,
            video.height,
            video.depth,
            video.framerate,
            video.interlaced,
            video.dar_num,
            video.dar_denom,
            video.stream,
        )
    });

    for video in videos {
        let query = video.insert();
        match query.execute(db) {
            Ok(succ) => successes.push(succ),
            Err(fail) => failures.push(fail),
        }
    }

    if pick_preferred {
        pick_subtitle(db, id, preferred_subtitle_code);
        pick_audio(db, id, preferred_audio_code);
    }
}
