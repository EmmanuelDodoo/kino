PRAGMA recursive_triggers = ON;
PRAGMA user_version = 3;

-- 
ALTER TABLE episode ADD COLUMN generate_poster BOOLEAN DEFAULT TRUE;

ALTER TABLE movie ADD COLUMN generate_poster BOOLEAN DEFAULT TRUE;


--
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
CASE WHEN (NOT episode.fetched) AND episode.generate_poster THEN NULL ELSE episode.poster END AS poster,
episode.*
FROM episode 
INNER JOIN season ON episode.season_id = season.id
INNER JOIN tv_show ON season.show_id = tv_show.id
INNER JOIN directory ON tv_show.directory = directory.id;


--
DROP VIEW IF EXISTS get_collection_posters;

CREATE VIEW get_collection_posters AS SELECT collection_id, poster 
FROM (
	SELECT CASE WHEN (NOT movie.fetched) AND movie.generate_poster THEN NULL ELSE movie.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN movie ON movie.id = item.media_id
	WHERE item.media_type = 'movie' AND poster IS NOT NULL

	UNION ALL

	SELECT CASE WHEN NOT tv_show.fetched THEN NULL ELSE tv_show.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN tv_show ON tv_show.id = item.media_id
	WHERE item.media_type = 'show' AND poster IS NOT NULL


	UNION ALL

	SELECT CASE WHEN NOT season.fetched THEN NULL ELSE season.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN season ON season.id = item.media_id
	WHERE item.media_type = 'season' AND poster IS NOT NULL

	UNION ALL

	SELECT CASE WHEN (NOT episode.fetched) AND episode.generate_poster THEN NULL ELSE episode.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN episode ON episode.id = item.media_id
	WHERE item.media_type = 'episode' AND episode.poster IS NOT NULL
) 
ORDER BY collection_id;


--
DROP TRIGGER IF EXISTS episode_update_tr;

CREATE TRIGGER episode_update_tr AFTER UPDATE ON episode
BEGIN
	UPDATE season SET watch_count = COALESCE((
			SELECT MIN(episode.watch_count) FROM episode WHERE episode.season_id = NEW.season_id
	), 0),
progress = COALESCE((
		SELECT AVG(episode.progress) FROM episode WHERE episode.season_id = NEW.season_id
),0.0),
    rating = (
	SELECT AVG(episode.rating) FROM episode WHERE episode.season_id = NEW.season_id
),
last_watched = (
	SELECT MAX(episode.last_watched) FROM episode WHERE episode.season_id = NEW.season_id
),
	duration = COALESCE((
			SELECT SUM(episode.duration) FROM episode WHERE episode.season_id = NEW.season_id
	),0),
recent_episode = (
	SELECT episode.id FROM episode WHERE episode.season_id = NEW.season_id AND episode.last_watched = (SELECT MAX(last_watched) FROM episode WHERE season_id = NEW.season_id)
),
comment_count = COALESCE((
		SELECT SUM(episode.comment_count) FROM episode WHERE episode.season_id = NEW.season_id
),0)
    WHERE id = NEW.season_id;	
END;
