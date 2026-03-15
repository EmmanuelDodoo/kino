PRAGMA recursive_triggers = ON;
PRAGMA user_version = 5;

CREATE TABLE image (
    path            TEXT NOT NULL PRIMARY KEY,
    main            TEXT,
    accent          TEXT,
    generated       BOOL DEFAULT FALSE,
	created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO image (path)
SELECT poster FROM movie
WHERE poster IS NOT NULL;

INSERT INTO image (path)
SELECT poster FROM tv_show
WHERE poster IS NOT NULL;

INSERT INTO image (path)
SELECT poster FROM season
WHERE poster IS NOT NULL;

INSERT INTO image (path)
SELECT poster FROM episode
WHERE poster IS NOT NULL;

DROP VIEW IF EXISTS get_episode_data;

CREATE VIEW get_episode_data AS SELECT
season.show_id,
tv_show.backdrop,
tv_show.tmdb_id AS show_tmdb_id,
tv_show.name AS show_name,
season.path AS season_path,
season.season_number,
tv_show.path AS show_path,
directory.path AS directory_path,
image.main as poster_main,
image.accent as poster_accent,
image.path as poster_path,
CASE WHEN (NOT episode.fetched) AND episode.generate_poster THEN NULL ELSE episode.poster END AS poster,
episode.*
FROM episode 
INNER JOIN season ON episode.season_id = season.id
INNER JOIN tv_show ON season.show_id = tv_show.id
INNER JOIN directory ON tv_show.directory = directory.id
LEFT JOIN image ON episode.poster = image.path;
