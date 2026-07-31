use crate::source::SourceSet;
use core::error::{Context, ContextLog, Log, Result, bail};
use fancy_regex::Regex;
use gstreamer_pbutils::Discoverer;
use registry::db::{self, Database};
use registry::models::{
    Audio, AudioId, Directory, DirectoryId, Episode, EpisodeId, MediaType, Movie, MovieId, Season,
    SeasonId, Show, ShowId, Subtitle, SubtitleId, VideoId, VideoInfoId, media::Status, video,
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
    Regex::new(r"(?<=[e|E])\s?\d{1,3}|(?<=[e|E][p|P][i|I][s|S][o|O][d|D][e|E])\s?\d{1,3}|(?<=\dx)\s?\d{1,3}|^\d{1,3}(?=\s)")
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
    "mpeg",
    "mpg",
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
struct ScannedSeason {
    // Get number at end of this?
    short_path: String,
    full_path: PathBuf,
}

#[derive(Debug)]
struct ScannedShow {
    short_path: String,
    full_path: PathBuf,
}

#[derive(Debug)]
struct SubtitleInfo {
    title: String,
    lang: String,
}

#[derive(Debug, Default)]
struct ScannedVideo {
    duration: u64,
    subtitles: Vec<SubtitleInfo>,
    audios: Vec<AudioInfo>,
    videos: Vec<VideoInfo>,
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

pub fn discoverer_init(discoverer: bool) -> Option<Discoverer> {
    if !discoverer {
        return None;
    }

    gstreamer::init().with_ctx_log(|| format!("Scan Discoverer init gstreamer init error"));

    Discoverer::new(gstreamer::ClockTime::from_seconds(5))
        .with_ctx_log(|| format!("Scan Discoverer error"))
}

pub fn scan_dir(
    db: &str,
    dir: Directory,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<String>,
    preferred_audio_code: Option<String>,
) -> Option<()> {
    let path = dir.path.display();
    tracing::debug!("Scanning directory {}", path);
    let discoverer = discoverer_init(discoverer);

    let mut db = Database::open(db)
        .with_ctx_log(|| format!("Scan directory DB opening error on {}.", path))?;

    scan_dir_helper(
        &mut db,
        dir,
        discoverer.as_ref(),
        movie_depth,
        restore,
        preferred_subtitle_code.as_deref(),
        preferred_audio_code.as_deref(),
    )
}

pub fn scan_dirs(
    db: impl AsRef<Path>,
    dirs: Vec<Directory>,
    discoverer: bool,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<String>,
    preferred_audio_code: Option<String>,
) -> Vec<DirectoryId> {
    tracing::debug!("Scanning {} directories", dirs.len());
    let discoverer = discoverer_init(discoverer);

    let mut db = match Database::open(db).ctx_log("Scan directories error") {
        Some(db) => db,
        None => {
            return Vec::with_capacity(0);
        }
    };

    let mut scanned = vec![];

    for dir in dirs {
        let id = dir.id;
        match scan_dir_helper(
            &mut db,
            dir,
            discoverer.as_ref(),
            movie_depth,
            restore,
            preferred_subtitle_code.as_deref(),
            preferred_audio_code.as_deref(),
        ) {
            Some(_) => {
                scanned.push(id);
            }
            None => continue,
        };
    }

    scanned
}

pub fn scan_dir_helper(
    db: &mut Database,
    dir: Directory,
    discoverer: Option<&Discoverer>,
    movie_depth: u8,
    restore: bool,
    preferred_subtitle_code: Option<&str>,
    preferred_audio_code: Option<&str>,
) -> Option<()> {
    let path = dir.path.display();
    let default_source = SourceSet::from_str(&dir.source);

    match dir.media_type {
        MediaType::Movies => {
            tracing::debug!("Scanning movie directory {}", path);
            struct DirMovie {
                id: MovieId,
                status: Status,
                scanned: bool,
                request: Option<String>,
                source: SourceSet,
                subtitle: bool,
                audio: bool,
                video: bool,
            }

            let videos = scan_video_dir(&dir.path, discoverer, movie_depth, None)
                .with_ctx_log(|| format!("Scanning movies in dir {}", path))?;

            tracing::debug!("Fetching Directory movies");

            let dir_movies = db
                .get_dir_movies(dir.id, |row| {
                    let id = MovieId::from_row(row)?;
                    let path = row.get::<_, String>("path")?;
                    let status = Status::from_row(row)?;
                    let scanned = matches!(status, Status::Archived);

                    let request = row.get::<_, Option<String>>("request")?;
                    let source = SourceSet::from_row(row, "source")?;

                    let subtitle = Movie::subtitle_maybe(row)?.is_some();
                    let video = Movie::video_maybe(row)?.is_some();
                    let audio = Movie::audio_maybe(row)?.is_some();

                    Ok((
                        path,
                        DirMovie {
                            id,
                            status,
                            scanned,
                            request,
                            source,
                            subtitle,
                            audio,
                            video,
                        },
                    ))
                })
                .with_ctx_log(|| format!("Scanning directory {} movies", path))?;

            let mut dir_movies = {
                // todo: Db could return an iterator instead?
                let mut map = HashMap::new();

                map.extend(dir_movies);

                map
            };

            for movie in videos {
                let mut dir_movie = dir_movies.get_mut(&movie.path);

                let pick_sub = !dir_movie
                    .as_ref()
                    .map(|movie| movie.subtitle)
                    .unwrap_or_default();
                let pick_vid = !dir_movie
                    .as_ref()
                    .map(|movie| movie.video)
                    .unwrap_or_default();
                let pick_aud = !dir_movie
                    .as_ref()
                    .map(|movie| movie.audio)
                    .unwrap_or_default();

                if let Some(movie) = &dir_movie {
                    match movie.status {
                        Status::Archived => continue,
                        Status::Tombstone if !restore => continue,
                        _ => {}
                    }
                }

                let name = process_name(&movie.name)
                    .with_ctx_log(|| format!("Movie name processing on {}", movie.name))
                    .unwrap_or(movie.name.clone());
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

                        succ.log()
                    }
                    Err(err) => {
                        err.with_ctx_log(|| format!("Movie {name} in Dir {} insertion", path));
                    }
                };

                let movie_source = |source: SourceSet| {
                    let Some((query, request)) = source.movie_request(id, name.clone()) else {
                        return;
                    };

                    let Some(succ) = query.execute(db).with_ctx_log(|| {
                        format!("Scan movie request with name: {name}, source: {source:?}")
                    }) else {
                        return;
                    };

                    succ.log();
                    let _ = db
                        .execute(
                            "UPDATE movie SET source=:source, request=:request WHERE id=:id",
                            &[
                                (":id", &ToSqlOutput::from(id)),
                                (":source", &ToSqlOutput::from(source)),
                                (":request", &ToSqlOutput::from(request.as_str())),
                            ],
                        )
                        .with_ctx_log(|| {
                            format!("Scan movie failed to update source & request on {name}")
                        });
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
                    id,
                    movie.embedded_subs,
                    movie.loaded_sub,
                    movie.audio,
                    movie.video,
                );

                if pick_sub {
                    pick_subtitle(db, id, preferred_subtitle_code).log_err();
                }

                if pick_aud {
                    pick_audio(db, id, preferred_audio_code).log_err();
                }

                if pick_vid {
                    pick_video(db, id).log_err();
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

            db.insert_remove_movies(deletes).with_ctx_log(|| {
                format!("Scan movies failed to perform insert remove on {}", path)
            });
        }
        MediaType::Shows => {
            struct DirShow {
                id: ShowId,
                scanned: bool,
                status: Status,
                request: Option<String>,
                source: SourceSet,
            }

            tracing::debug!("Scanning shows directory {}", path);
            let shows =
                scan_shows(&dir.path).with_ctx_log(|| format!("Scanning shows in dir {}", path))?;

            tracing::debug!("Fetching Directory shows");
            let dir_shows = db
                .get_dir_shows(dir.id, |row| {
                    let id = ShowId::from_row(row)?;
                    let path = row.get::<_, String>("path")?;
                    let status = Status::from_row(row)?;
                    let scanned = matches!(status, Status::Archived);

                    let request = row.get::<_, Option<String>>("request")?;
                    let source = SourceSet::from_row(row, "source")?;

                    Ok((
                        path,
                        DirShow {
                            id,
                            scanned,
                            status,
                            request,
                            source,
                        },
                    ))
                })
                .with_ctx_log(|| format!("Scanning directory {} shows", path))?;

            let mut dir_shows = {
                let mut map = HashMap::new();
                map.extend(dir_shows);
                map
            };

            for show in shows {
                let mut dir_show = dir_shows.get_mut(&show.short_path);
                let new_show = dir_show.is_none();

                if let Some(show) = &dir_show {
                    match show.status {
                        Status::Archived => continue,
                        Status::Tombstone if !restore => continue,
                        _ => {}
                    }
                }

                let ScannedShow {
                    full_path,
                    short_path,
                } = show;
                let name = process_name(&short_path)
                    .with_ctx_log(|| format!("Show name processing on {}", path))
                    .unwrap_or(short_path.clone());
                let (new, query) =
                    Show::new(dir.id, short_path.clone(), name.clone(), short_path.clone());

                let show = dir_show.as_ref().map(|show| show.id).unwrap_or(new.id);

                match query.execute(db) {
                    Ok(succ) => {
                        succ.log();

                        if let Some(entry) = dir_show.as_mut() {
                            entry.scanned = true;
                        }
                    }
                    Err(err) => {
                        err.with_ctx_log(|| format!("Show {name} in Dir {} insertion", path));
                        continue;
                    }
                };

                let show_source = |source: SourceSet, name: String| {
                    let Some((query, request)) = source.show_request(show, name.clone()) else {
                        return (source, None);
                    };

                    let Some(succ) = query
                        .execute(db)
                        .with_ctx_log(|| format!("Scan show {name}, source: {source:?}"))
                    else {
                        return (source, None);
                    };

                    succ.log();

                    let _ = db
                        .execute(
                            "UPDATE tv_show SET source=:source, request=:request WHERE id=:id",
                            &[
                                (":id", &ToSqlOutput::from(show)),
                                (":source", &ToSqlOutput::from(source)),
                                (":request", &ToSqlOutput::from(request.as_str())),
                            ],
                        )
                        .with_ctx_log(|| {
                            format!("Scan show failed to update source & request on {name}")
                        });

                    (source, Some(request))
                };

                let (source, show_request) = match dir_show {
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

                scan_show(
                    db.into(),
                    discoverer,
                    full_path,
                    source,
                    show,
                    show_request.as_deref(),
                    restore,
                    new_show,
                    preferred_subtitle_code,
                    preferred_audio_code,
                )
                .with_ctx_log(|| format!("Scanning: Show {name} episodes"));
            }

            tracing::debug!("Performing shows insert/remove");

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

            db.insert_remove_shows(deletes).with_ctx_log(|| {
                format!(
                    "Scan shows failed to perform insert remove shows on {}",
                    path
                )
            });
        }
    }

    Some(())
}

fn scan_shows(path: impl AsRef<Path>) -> Result<Vec<ScannedShow>> {
    let path = path.as_ref();

    let mut shows = vec![];
    let read = path
        .read_dir()
        .with_context(|| format!("Scan shows dir on {}", path.display()))?;

    for item in read {
        let Some(item) =
            item.with_ctx_log(|| format!("Scanning shows directory items at {}", path.display()))
        else {
            continue;
        };

        let Some(is_dir) = item
            .file_type()
            .with_ctx_log(|| format!("Scanning shows dir entry at {}", item.path().display()))
            .map(|ft| ft.is_dir())
        else {
            continue;
        };

        if !is_dir {
            continue;
        }
        let path = item.path();

        shows.push(scan_show_dir(path)?);
    }

    Ok(shows)
}

#[allow(clippy::too_many_arguments)]
pub fn scan_season<'a>(
    db: db::Source<'a>,
    discoverer: Option<&Discoverer>,
    path: impl AsRef<Path>,
    source: SourceSet,
    show: ShowId,
    season: SeasonId,
    season_number: u16,
    season_request: Option<&str>,
    restore: bool,
    new_season: bool,
    preferred_subtitle_code: Option<&str>,
    preferred_audio_code: Option<&str>,
) -> Result<()> {
    struct DirEpisode {
        id: EpisodeId,
        status: Status,
        scanned: bool,
        source: SourceSet,
        request: Option<String>,
        subtitle: bool,
        audio: bool,
        video: bool,
    }

    let db = db.as_ref();
    let path = path.as_ref();

    tracing::debug!("Fetching season directory items");
    let episodes = scan_video_dir(path, discoverer, 0, None)?;

    tracing::debug!("Fetching season episodes");
    let dir_episodes = db
        .get_season_episodes_removed(season, |row| {
            let id = EpisodeId::from_row(row)?;
            let path = row.get::<_, String>("path")?;
            let status = Status::from_row(row)?;
            let scanned = matches!(status, Status::Archived);

            let request = row.get::<_, Option<String>>("request")?;
            let source = SourceSet::from_row(row, "source")?;
            let subtitle = Movie::subtitle_maybe(row)?.is_some();
            let video = Movie::video_maybe(row)?.is_some();
            let audio = Movie::audio_maybe(row)?.is_some();

            Ok((
                path,
                DirEpisode {
                    id,
                    status,
                    scanned,
                    request,
                    source,
                    subtitle,
                    audio,
                    video,
                },
            ))
        })
        .with_context(|| format!("Scanning: season {season} removed epiosdes"))?;

    let mut dir_episodes = {
        let mut map = HashMap::new();
        map.extend(dir_episodes);

        map
    };

    for episode in episodes {
        let mut dir_ep = dir_episodes.get_mut(&episode.path);
        let pick_sub = !dir_ep
            .as_ref()
            .map(|movie| movie.subtitle)
            .unwrap_or_default();
        let pick_vid = !dir_ep.as_ref().map(|movie| movie.video).unwrap_or_default();
        let pick_aud = !dir_ep.as_ref().map(|movie| movie.audio).unwrap_or_default();

        if let Some(episode) = &dir_ep {
            match episode.status {
                Status::Archived => continue,
                Status::Tombstone if !restore => continue,
                _ => {}
            }
        }

        let number = process_episode(&episode.path).log_err();
        let name = match number {
            Some(number) => format!("Episode {number:02}"),
            None => episode.name.clone(),
        };

        let episode_number = number.unwrap_or_default();

        let (new, query) = Episode::new(
            show,
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

                succ.log();
            }
            Err(err) => {
                err.with_ctx_log(|| format!("Scanning: Episode {episode_number} query execution"));
            }
        }

        let episode_source = |source: SourceSet| {
            let Some(parent) = season_request else {
                return (source, None);
            };

            let Some((query, request)) =
                source.episode_request(episode_id, parent, season_number, episode_number)
            else {
                return (source, None);
            };

            let Some(succ) = query
                .execute(db)
                .with_ctx_log(|| format!("Scanning: Episode {episode_number} source {source:?}"))
            else {
                return (source, None);
            };

            succ.log();

            let _ = db
                .execute(
                    "UPDATE episode SET source=:source, request=:request WHERE id=:id",
                    &[
                        (":id", &ToSqlOutput::from(episode_id)),
                        (":source", &ToSqlOutput::from(source)),
                        (":request", &ToSqlOutput::from(request.as_str())),
                    ],
                )
                .with_ctx_log(|| {
                    format!("Scanning: Episode {episode_number} failed to update source & request")
                });

            (source, Some(request))
        };

        match dir_ep {
            Some(dir_episode) => match &dir_episode.request {
                Some(_) => {}
                None => {
                    let source = dir_episode.source.merge(source);

                    episode_source(source);
                }
            },
            None => {
                let (source, request) = episode_source(source);

                if !new_season
                    && let Some((id, parent)) = request.as_deref().zip(season_request.as_deref())
                    && let Some(query) = source.episode_sync(id, parent)
                {
                    let _ = query.execute(db).with_ctx_log(|| {
                        format!("Scanning: Failed episode sync {id} request on source {source:?}")
                    });
                };
            }
        }

        save_video_metadata(
            db,
            episode_id,
            episode.embedded_subs,
            episode.loaded_sub,
            episode.audio,
            episode.video,
        );

        if pick_sub {
            pick_subtitle(db, episode_id, preferred_subtitle_code).log_err();
        }

        if pick_aud {
            pick_audio(db, episode_id, preferred_audio_code).log_err();
        }

        if pick_vid {
            pick_video(db, episode_id).log_err();
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

    db.insert_remove_episodes(deletes).with_context(|| {
        format!(
            "Scanning: Episodes failed to insert remove on {}",
            path.display()
        )
    })?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn scan_show<'a>(
    db: db::Source<'a>,
    discoverer: Option<&Discoverer>,
    path: impl AsRef<Path>,
    source: SourceSet,
    show: ShowId,
    show_request: Option<&str>,
    restore: bool,
    new_show: bool,
    preferred_subtitle_code: Option<&str>,
    preferred_audio_code: Option<&str>,
) -> Result<()> {
    struct DirSeason {
        id: SeasonId,
        scanned: bool,
        status: Status,
        request: Option<String>,
        source: SourceSet,
    }

    let db = db.as_ref();
    let path = path.as_ref();

    tracing::debug!("Fetching show directory items");
    let seasons = scan_seasons(path)?;

    tracing::debug!("Fetching show seasons");
    let dir_seasons = db
        .get_show_seasons_removed(show, |row| {
            let id = SeasonId::from_row(row)?;
            let path = row.get::<_, String>("path")?;
            let status = Status::from_row(row)?;
            let scanned = matches!(status, Status::Archived);

            let request = row.get::<_, Option<String>>("request")?;
            let source = SourceSet::from_row(row, "source")?;

            Ok((
                path,
                DirSeason {
                    id,
                    scanned,
                    status,
                    request,
                    source,
                },
            ))
        })
        .with_context(|| format!("Scanning: show {show} removed seasons"))?;

    let mut dir_seasons = {
        let mut map = HashMap::new();
        map.extend(dir_seasons);
        map
    };

    for season in seasons {
        let mut dir_season = dir_seasons.get_mut(&season.short_path);
        let new_season = dir_season.is_none();

        if let Some(season) = &dir_season {
            match season.status {
                Status::Archived => continue,
                Status::Tombstone if !restore => continue,
                _ => {}
            }
        }

        let ScannedSeason {
            short_path: path,
            full_path,
        } = season;
        let number = process_season(&path).log_err();
        let name = match number {
            Some(number) => format!("Season {number:02}"),
            None => path.clone(),
        };

        let season_number = number.unwrap_or_default();

        let (season, query) = Season::new(show, name.clone(), path.clone(), season_number);

        let season = dir_season.as_ref().map(|sea| sea.id).unwrap_or(season.id);

        match query.execute(db) {
            Ok(succ) => {
                let modified = succ.rows > 0;

                if let Some(entry) = dir_season.as_mut() {
                    entry.scanned = true;
                }

                succ.log();
                modified
            }
            Err(err) => {
                err.with_ctx_log(|| format!("Scanning: Season {season_number} query execution",));
                continue;
            }
        };

        let season_source = |source: SourceSet| {
            let Some(parent) = show_request.as_deref() else {
                return (source, None);
            };

            let Some((query, request)) = source.season_request(season, parent, season_number)
            else {
                return (source, None);
            };

            let Some(succ) = query
                .execute(db)
                .with_ctx_log(|| format!("Scanning: season {season_number} source {source:?}"))
            else {
                return (source, None);
            };

            succ.log();

            let _ = db
                .execute(
                    "UPDATE season SET source=:source, request=:request WHERE id=:id",
                    &[
                        (":id", &ToSqlOutput::from(season)),
                        (":source", &ToSqlOutput::from(source)),
                        (":request", &ToSqlOutput::from(request.as_str())),
                    ],
                )
                .with_ctx_log(|| {
                    format!("Scanning: Season {season_number} failed to update source & request")
                });

            (source, Some(request))
        };

        let (season_source, season_request) = match dir_season {
            Some(dir_season) => match &dir_season.request {
                Some(request) => (dir_season.source, Some(request.to_owned())),
                None => {
                    let source = dir_season.source.merge(source);

                    season_source(source)
                }
            },
            None => {
                let (source, request) = season_source(source);

                if !new_show
                    && let Some((id, parent)) = request.as_deref().zip(show_request.as_deref())
                    && let Some(query) = source.season_sync(id, parent)
                {
                    let _ = query.execute(db).with_ctx_log(|| {
                        format!("Scan: Failed season sync {id} request on source {source:?}")
                    });
                };
                (source, request)
            }
        };

        tracing::debug!("Scanning {name} episodes");
        scan_season(
            db.into(),
            discoverer,
            &full_path,
            season_source,
            show,
            season,
            season_number,
            season_request.as_deref(),
            restore,
            new_season,
            preferred_subtitle_code,
            preferred_audio_code,
        )
        .with_ctx_log(|| format!("Scanning: Season {name} episodes"));
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

    db.insert_remove_seasons(deletes).with_ctx_log(|| {
        format!(
            "Scanning: Seasons failed to insert/remove on {}",
            path.display()
        )
    });

    Ok(())
}

fn scan_season_dir(path: impl AsRef<Path>) -> Result<ScannedSeason> {
    let path = path.as_ref();

    if !path.is_dir() {
        bail!(
            "Scan Season path {} not an accessible directory",
            path.display()
        )
    }

    let name = path_name(&path).to_owned();

    Ok(ScannedSeason {
        short_path: name,
        full_path: path.to_path_buf(),
    })
}

fn scan_seasons(path: impl AsRef<Path>) -> Result<Vec<ScannedSeason>> {
    let path = path.as_ref();

    let read = path
        .read_dir()
        .with_context(|| format!("Scan show dir on {}", path.display()))?;

    let mut seasons = vec![];

    for item in read {
        let Some(item) =
            item.with_ctx_log(|| format!("Scanning show directory items at {}", path.display()))
        else {
            continue;
        };

        let season = scan_season_dir(item.path())
            .with_ctx_log(|| format!("Scanning show directory items at {}", path.display()));

        if let Some(season) = season {
            seasons.push(season)
        }
    }

    Ok(seasons)
}

fn scan_show_dir(path: impl AsRef<Path>) -> Result<ScannedShow> {
    let path = path.as_ref();
    let dir = path_name(path);

    if !path.is_dir() {
        bail!(
            "Scan Season path {} not an accessible directory",
            path.display()
        )
    }

    let show = ScannedShow {
        short_path: dir.to_owned(),
        full_path: path.to_path_buf(),
    };

    Ok(show)
}

fn scan_video_dir(
    path: impl AsRef<Path>,
    discoverer: Option<&Discoverer>,
    depth: u8,
    prefix: Option<String>,
) -> Result<Vec<Video>> {
    let path = path.as_ref();

    tracing::debug!("Scanning video directory {}", path.display());

    let path = path
        .canonicalize()
        .with_context(|| format!("Scan video dir canonicalize on {}", path.display()))?;

    let mut videos = vec![];

    let read = path
        .read_dir()
        .with_context(|| format!("Scan video dir on {}", path.display()))?;

    for item in read {
        let Some(item) =
            item.with_ctx_log(|| format!("Scanning video directory items at {}", path.display()))
        else {
            continue;
        };

        let Some(is_file) = item
            .file_type()
            .with_ctx_log(|| format!("Scanning video dir entry at {}", item.path().display()))
            .map(|ft| ft.is_file())
        else {
            continue;
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

    Ok(videos)
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

    let scanned = discoverer
        .zip(url)
        .and_then(|(discoverer, url)| discover(discoverer, url, &path).log_err())
        .unwrap_or_default();

    let ScannedVideo {
        duration,
        subtitles: embedded_subs,
        audios: audio,
        videos: video,
    } = scanned;

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

fn process_name(name: &str) -> Result<String> {
    let value = MOVIE_REG1
        .find(name)
        .with_context(|| format!("Name processing MOVIE_REG1 on {name}"))?;

    let Some(value) = value else {
        bail!("No MOVIE_REG1 match found on {name} ");
    };

    let value = MOVIE_REG2
        .find(value.as_str())
        .with_context(|| format!("Name processing MOVIE_REG2 on {name}"))?;
    let Some(value) = value else {
        bail!("No MOVIE_REG2 match found on {name}")
    };

    let cleaned = CLEANER.replace_all(value.as_str(), " ").trim().to_owned();

    Ok(cleaned)
}

fn process_season(name: &str) -> Result<u16> {
    let value = SEASON_REG
        .find(name)
        .with_context(|| format!("Season number SEASON_REG match on {name}"))?;

    let Some(value) = value else {
        bail!("No season number match on {name}");
    };

    let value = value.as_str().trim();

    value
        .parse::<u16>()
        .with_context(|| format!("Season number parsing on {name} "))
}

fn process_episode(name: &str) -> Result<u16> {
    let value = EPISODE_REG
        .find(name)
        .with_context(|| format!("Episode number EPISODE_REG match on {name}"))?;

    let Some(value) = value else {
        bail!("No episode number match on {name}");
    };

    let value = value.as_str().trim();

    value
        .parse::<u16>()
        .with_context(|| format!("Episode number parsing on {name}"))
}

fn discover(discoverer: &Discoverer, url: url::Url, path: &Path) -> Result<ScannedVideo> {
    let info = discoverer
        .discover_uri(url.as_str())
        .with_context(|| format!("Discovering url {url} for video {}", path.display()))?;

    use gstreamer_pbutils::prelude::DiscovererStreamInfoExt;

    let subtitles = info
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

    for (audio_stream, audio) in info.audio_streams().into_iter().enumerate() {
        let caps = audio.caps();
        let codec = caps
            .as_ref()
            .map(|caps| gstreamer_pbutils::pb_utils_get_codec_description(caps).to_string());

        let stream = audio_stream as u32;

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

    for (video_stream, video) in info.video_streams().into_iter().enumerate() {
        let caps = video.caps();
        let codec = caps
            .as_ref()
            .map(|caps| gstreamer_pbutils::pb_utils_get_codec_description(caps).to_string());

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

        let stream = video_stream as u32;

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

    Ok(ScannedVideo {
        duration,
        subtitles,
        audios,
        videos,
    })
}

fn pick_subtitle(db: &Database, id: impl Into<VideoId>, preferred: Option<&str>) -> Result<()> {
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
    }
    .with_context(|| format!("Selecting subtitle for video {id} with preferred {preferred:?}"))?;

    let sql = format!("UPDATE {table} SET subtitle_id=:subtitle WHERE id=:id");

    match res {
        Some(subtitle_id) => db
            .execute(
                &sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":subtitle", &ToSqlOutput::from(subtitle_id)),
                ],
            )
            .map(|_| ())
            .with_context(|| format!("Setting video {id} subtitle to {subtitle_id}")),
        None => Ok(()),
    }
}

fn pick_audio(db: &Database, id: impl Into<VideoId>, preferred: Option<&str>) -> Result<()> {
    let Some(preferred) = preferred else {
        return Ok(());
    };

    let id = id.into();

    let table = if matches!(id, VideoId::Movie(_)) {
        "movie"
    } else {
        "episode"
    };

    let res = db
        .query_row(
            "SELECT id FROM audio WHERE media=:media ORDER BY CASE WHEN lang=:lang THEN 0 ELSE 1 END LIMIT 1",
            &[
                (
                    ":lang",
                    &ToSqlOutput::Borrowed(ValueRef::Text(preferred.as_bytes())),
                ),
                (":media", &ToSqlOutput::from(id)),
            ],
            AudioId::from_row,
        )
        .optional()
        .with_context(|| format!("Selecting audio for video {id} with preferred {preferred}"))?;

    let sql = format!("UPDATE {table} SET audio_id=:audio WHERE id=:id");
    match res {
        Some(audio_id) => db
            .execute(
                &sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":audio", &ToSqlOutput::from(audio_id)),
                ],
            )
            .map(|_| ())
            .with_context(|| format!("Setting video {id} audio to {audio_id}")),
        None => Ok(()),
    }
}

fn pick_video(db: &Database, id: impl Into<VideoId>) -> Result<()> {
    let id = id.into();

    let table = if matches!(id, VideoId::Movie(_)) {
        "movie"
    } else {
        "episode"
    };

    let res = db
        .query_row(
            "SELECT id FROM video WHERE media=:media ORDER BY height DESC",
            &[(":media", &ToSqlOutput::from(id))],
            VideoInfoId::from_row,
        )
        .optional()
        .with_context(|| format!("Selecting video info for video {id}"))?;

    let sql = format!("UPDATE {table} SET video_id=:video WHERE id=:id");
    match res {
        Some(video_id) => db
            .execute(
                &sql,
                &[
                    (":id", &ToSqlOutput::from(id)),
                    (":video", &ToSqlOutput::from(video_id)),
                ],
            )
            .map(|_| ())
            .with_context(|| format!("Setting video {id} video info to {video_id}")),
        None => Ok(()),
    }
}

fn save_video_metadata(
    db: &Database,
    id: impl Into<VideoId>,
    subtitles: Vec<SubtitleInfo>,
    loaded_sub: Option<String>,
    audio: Vec<AudioInfo>,
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
            Ok(succ) => succ.log(),
            Err(err) => {
                err.with_ctx_log(|| format!("Subtitle {} insertion on video {id}", sub.id));
            }
        }
    }

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
        let query = audio.insert();
        match query.execute(db) {
            Ok(succ) => succ.log(),
            Err(err) => {
                err.with_ctx_log(|| format!("Audio {} insertion on video {id}", audio.id));
            }
        }
    }

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
            Ok(succ) => succ.log(),
            Err(err) => {
                err.with_ctx_log(|| format!("Video Info {} insertion on video {id}", video.id));
            }
        }
    }
}
