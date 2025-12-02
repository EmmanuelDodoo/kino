use crate::db::{BatchResult, Database};
use crate::error;
use crate::models::{
    Directory, DirectoryId, Episode, MediaType, Movie, Season, SeasonId, Show, ShowId,
};
use gstreamer_pbutils::Discoverer;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

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

pub fn scan_dir<'a>(db: &str, dir: Directory, discoverer: bool) -> Option<BatchResult<'a>> {
    let discoverer = if discoverer {
        if let Err(error) = gstreamer::init().map_err(error::GStreamerError::Glib) {
            eprintln!(
                "Scan directory gstreamer init error on {}. Error \n{error}",
                dir.path
            );
        };
        Discoverer::new(gstreamer::ClockTime::from_seconds(5))
            .inspect_err(|error| {
                eprintln!(
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
            eprintln!("Scan directory error on {}. Error \n{error}", dir.path);
            return None;
        }
    };

    scan_dir_helper(&mut db, dir, discoverer.as_ref())
}

pub fn scan_dirs<'a>(
    db: &str,
    dirs: Vec<Directory>,
    discoverer: bool,
) -> (Option<BatchResult<'a>>, Vec<DirectoryId>) {
    let discoverer = if discoverer {
        if let Err(error) = gstreamer::init().map_err(error::GStreamerError::Glib) {
            eprintln!("Scan directories gstreamer init error. Error \n{error}");
        };
        Discoverer::new(gstreamer::ClockTime::from_seconds(5))
            .inspect_err(|error| eprintln!("Scan directories discoverer error. Error \n{error}",))
            .ok()
    } else {
        None
    };

    let mut db = match Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("Scan directories error. Error \n{error}");
            return (None, Vec::with_capacity(0));
        }
    };

    let mut result = BatchResult::empty();
    let mut scanned = vec![];

    for dir in dirs {
        let id = dir.id;
        match scan_dir_helper(&mut db, dir, discoverer.as_ref()) {
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
) -> Option<BatchResult<'a>> {
    let mut successes = vec![];
    let mut failures = vec![];

    match dir.media_type {
        MediaType::Movies => {
            if let Some(videos) = scan_videos(&dir.path, discoverer) {
                for movie in videos {
                    let (_, query) = Movie::new(dir.id, movie.path, movie.name, movie.duration);
                    match query.execute(db) {
                        Ok(succ) => successes.push(succ),
                        Err(fail) => failures.push(fail),
                    };
                }
            }
        }
        MediaType::Shows => {
            let shows = scan_shows(&dir.path, discoverer)?;

            for show in shows {
                let ShowPrim { path, seasons } = show;
                let (show, query) =
                    Show::new(dir.id, path.clone(), path.clone(), seasons.len() as _);

                let new = match query.execute(db) {
                    Ok(succ) => {
                        let modified = succ.rows > 0;
                        successes.push(succ);
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
                            eprintln!("{error}");
                            continue;
                        }
                    }
                };

                for season in seasons {
                    let SeasonPrim { path, episodes } = season;
                    let (season, query) = Season::new(show, path.clone(), path.clone());

                    let new = match query.execute(db) {
                        Ok(succ) => {
                            let modified = succ.rows > 0;
                            successes.push(succ);
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
                                eprintln!("{error}");
                                continue;
                            }
                        }
                    };

                    for episode in episodes {
                        let (_, query) =
                            Episode::new(season, episode.name, episode.path, episode.duration);
                        match query.execute(db) {
                            Ok(succ) => successes.push(succ),
                            Err(fail) => failures.push(fail),
                        }
                    }
                }
            }
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
        .inspect_err(|error| eprintln!("Scan show dir on {}. Error \n{error}", path.display()))
        .ok()?;

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                eprintln!("{error}");
                continue;
            }
        };

        let is_dir = match item.file_type() {
            Ok(file) => file.is_dir(),
            Err(error) => {
                eprintln!("{error}");
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
        .inspect_err(|error| eprintln!("Scan show dir on {}. Error \n{error}", path.display()))
        .ok()?;

    let mut seasons = vec![];

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                eprintln!("{error}");
                continue;
            }
        };

        let is_dir = match item.file_type() {
            Ok(file) => file.is_dir(),
            Err(error) => {
                eprintln!("{error}");
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

fn scan_videos(path: impl AsRef<Path>, discoverer: Option<&Discoverer>) -> Option<Vec<Video>> {
    scan_video_dir(path, discoverer, 2, None)
}

fn scan_video_dir(
    path: impl AsRef<Path>,
    discoverer: Option<&Discoverer>,
    depth: usize,
    prefix: Option<String>,
) -> Option<Vec<Video>> {
    let path = path.as_ref();
    let path = path
        .canonicalize()
        .inspect_err(|error| {
            eprintln!(
                "Scan video dir error on {}. Error \n{error}",
                path.display()
            );
        })
        .ok()?;

    let mut videos = vec![];

    let read = path
        .read_dir()
        .inspect_err(|error| eprintln!("Scan video dir on {}. Error \n{error}", path.display()))
        .ok()?;

    for item in read {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                eprintln!("{error}");
                continue;
            }
        };

        let is_file = match item.file_type() {
            Ok(file) => file.is_file(),
            Err(error) => {
                eprintln!("{error}");
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
    let is_video = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EXTENSIONS.contains(&ext))
        .unwrap_or_default();

    if !is_video {
        return None;
    }

    let url = url::Url::from_file_path(&path)
        .inspect_err(|_| eprintln!("Scan file url error on {}", path.display()))
        .ok();
    let duration = match discoverer.zip(url) {
        Some((discoverer, url)) => discoverer
            .discover_uri(url.as_str())
            .inspect_err(|error| {
                eprintln!("Scan discover error on {}. Error {error}", path.display())
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
