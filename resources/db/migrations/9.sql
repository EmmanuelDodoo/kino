PRAGMA recursive_triggers = ON;
PRAGMA user_version = 9;

DROP VIEW IF EXISTS get_episode_data;
DROP VIEW IF EXISTS get_collection_posters;
DROP VIEW IF EXISTS get_collection;
DROP TRIGGER IF EXISTS show_refetch_tr;
DROP TRIGGER IF EXISTS show_season_user_tmdb_tr;
DROP TRIGGER IF EXISTS season_refetch_tr;

CREATE TABLE tmdb (
	id              TEXT NOT NULL  PRIMARY KEY, 
	created_at      DATETIME  DEFAULT CURRENT_TIMESTAMP,
	tmdb_id			INTEGER,
	media_type		TEXT NOT NULL,
	media_id		TEXT NOT NULL,
	status			INTEGER NOT NULL,
	retry			INTEGER DEFAULT 0,
	poster			TEXT,

	parent			TEXT,
	name			TEXT,
	number			INTEGER DEFAULT 0,
	backdrop		TEXT,

	FOREIGN KEY (parent) REFERENCES tmdb(id) ON DELETE CASCADE,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode'))
);

ALTER TABLE tv_show DROP COLUMN tmdb_id;
ALTER TABLE tv_show DROP COLUMN user_tmdb_id;
ALTER TABLE tv_show DROP COLUMN fetched;
ALTER TABLE tv_show ADD COLUMN request;
ALTER TABLE tv_show ADD COLUMN source NOT NULL DEFAULT 'none';

ALTER TABLE season DROP COLUMN tmdb_id;
ALTER TABLE season DROP COLUMN user_tmdb_id;
ALTER TABLE season DROP COLUMN fetched;
ALTER TABLE season ADD COLUMN request;
ALTER TABLE season ADD COLUMN source NOT NULL DEFAULT 'none';

ALTER TABLE episode DROP COLUMN tmdb_id;
ALTER TABLE episode DROP COLUMN user_tmdb_id;
ALTER TABLE episode ADD COLUMN request;
ALTER TABLE episode ADD COLUMN source NOT NULL DEFAULT 'none';

ALTER TABLE movie DROP COLUMN tmdb_id;
ALTER TABLE movie DROP COLUMN user_tmdb_id;
ALTER TABLE movie ADD COLUMN request;
ALTER TABLE movie ADD COLUMN source NOT NULL DEFAULT 'none';

CREATE VIEW get_episode_data AS SELECT
season.show_id,
tv_show.backdrop,
tv_show.name AS show_name,
season.path AS season_path,
season.season_number,
tv_show.path AS show_path,
directory.path AS directory_path,
image.main as poster_main,
image.accent as poster_accent,
image.path as poster_path,
image.generated as poster_generated,
episode.*
FROM episode 
INNER JOIN season ON episode.season_id = season.id
INNER JOIN tv_show ON season.show_id = tv_show.id
INNER JOIN directory ON tv_show.directory = directory.id
LEFT JOIN image ON episode.poster = image.path;

CREATE VIEW get_collection_posters AS SELECT collection_id, poster 
FROM (
	SELECT movie.poster AS poster, item.collection_id
	FROM collection_item item
	JOIN movie ON movie.id = item.media_id
	WHERE item.media_type = 'movie' AND poster IS NOT NULL

	UNION ALL

	SELECT tv_show.poster AS poster, item.collection_id
	FROM collection_item item
	JOIN tv_show ON tv_show.id = item.media_id
	WHERE item.media_type = 'show' AND poster IS NOT NULL


	UNION ALL

	SELECT season.poster AS poster, item.collection_id
	FROM collection_item item
	JOIN season ON season.id = item.media_id
	WHERE item.media_type = 'season' AND poster IS NOT NULL

	UNION ALL

	SELECT episode.poster AS poster, item.collection_id
	FROM collection_item item
	JOIN episode ON episode.id = item.media_id
	WHERE item.media_type = 'episode' AND episode.poster IS NOT NULL
) 
ORDER BY collection_id
;

CREATE VIEW get_collection AS SELECT collection.*,
(
	SELECT posters.poster 
	FROM get_collection_posters posters
	WHERE posters.collection_id = collection.id
	LIMIT 1 OFFSET 0
) AS poster1,
(
	SELECT posters.poster 
	FROM get_collection_posters posters
	WHERE posters.collection_id = collection.id
	LIMIT 1 OFFSET 1
) AS poster2,
(
	SELECT posters.poster  
	FROM get_collection_posters posters
	WHERE posters.collection_id = collection.id
	LIMIT 1 OFFSET 2
) AS poster3,
(
	SELECT posters.poster 
	FROM get_collection_posters posters
	WHERE posters.collection_id = collection.id
	LIMIT 1 OFFSET 3
) AS poster4
FROM collection;

CREATE TRIGGER tmdb_parent_update AFTER UPDATE ON tmdb
BEGIN
	-- New TMDB Id
	UPDATE tmdb SET tmdb_id=NEW.tmdb_id, retry=0 WHERE parent = NEW.id;

	-- Parent Data done
	UPDATE tmdb SET status=2, retry=0 WHERE parent = NEW.id AND NEW.status > 2;

END;

CREATE TRIGGER tmdb_video_fetched AFTER UPDATE OF status ON tmdb WHEN NEW.status <= 2
BEGIN
	UPDATE movie SET fetched=FALSE WHERE id=NEW.media_id;

	UPDATE episode SET fetched=FALSE WHERE id=NEW.media_id;
END;

CREATE TRIGGER tmdb_season_number_update AFTER UPDATE OF number ON tmdb WHEN NEW.media_type = 'season'
BEGIN
	UPDATE tmdb SET name=NEW.number, retry=0 WHERE media_type='episode' AND parent = NEW.id;
END;

CREATE TRIGGER tmdb_show_source_update AFTER UPDATE OF source ON tv_show
BEGIN
	UPDATE season SET source=NEW.source WHERE show_id=NEW.id;
END;

CREATE TRIGGER tmdb_season_source_update AFTER UPDATE of source on season
BEGIN
	UPDATE episode SET source=NEW.source WHERE season_id=NEW.id;
END;

CREATE TRIGGER tmdb_movie_name_update AFTER UPDATE of name on movie
BEGIN
	UPDATE tmdb SET name=NEW.name, status=1, retry=0 WHERE media_id = NEW.id;
END;

CREATE TRIGGER tmdb_show_name_update AFTER UPDATE of name on tv_show
BEGIN
	UPDATE tmdb SET name=NEW.name, status=1, retry=0 WHERE media_id = NEW.id;
END;

CREATE TRIGGER tmdb_movie_delete AFTER DELETE ON movie
BEGIN
	DELETE FROM tmdb WHERE media_id = OLD.id;
END;

CREATE TRIGGER tmdb_show_delete AFTER DELETE ON tv_show
BEGIN
	DELETE FROM tmdb WHERE media_id = OLD.id;
END;

CREATE TRIGGER tmdb_season_delete AFTER DELETE ON season
BEGIN
	DELETE FROM tmdb WHERE media_id = OLD.id;
END;

CREATE TRIGGER tmdb_episode_delete AFTER DELETE ON episode
BEGIN
	DELETE FROM tmdb WHERE media_id = OLD.id;
END;

