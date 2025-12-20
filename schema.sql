PRAGMA recursive_triggers = ON;
PRAGMA user_version = 0;

CREATE TABLE directory ( 
	id		TEXT NOT NULL PRIMARY KEY,
	path		TEXT NOT NULL UNIQUE,
	active		BOOLEAN NOT NULL,
	media_type	TEXT NOT NULL,
	last_scan       DATETIME  NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CHECK ( media_type IN ('movies', 'shows'))
);

CREATE TABLE tv_show ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	tmdb_id              INTEGER,
	name                 TEXT NOT NULL    ,
	original_name	     TEXT NOT NULL,
	directory            TEXT NOT NULL,
	path                 TEXT NOT NULL,
	poster               TEXT     ,
	tags                 TEXT     ,
	synopsis             TEXT  DEFAULT '<empty synopsis>'   ,
	release              TEXT NOT NULL    ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	backdrop             TEXT     ,
	watch_count          INTEGER NOT NULL DEFAULT 0   ,
	season_count         INT NOT NULL DEFAULT 0   ,
	progress             FLOAT(2,1) NOT NULL DEFAULT 0.0   ,
	rating               FLOAT(2, 1)     ,
	last_watched         DATETIME     ,
	recent_season        TEXT     ,
	duration             INTEGER NOT NULL DEFAULT 0   ,
	comment_count        INTEGER NOT NULL DEFAULT 0   ,
	fetched		     BOOLEAN DEFAULT FALSE,
	removed		     BOOLEAN DEFAULT FALSE,
	UNIQUE(directory, path),
	FOREIGN KEY ( directory ) REFERENCES directory( id ) ON DELETE CASCADE
);

CREATE INDEX idx_tv_show_directory ON tv_show ( directory );

CREATE TABLE season ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	tmdb_id              INTEGER,
	name                 TEXT NOT NULL    ,
	original_name	     TEXT NOT NULL,
	path        	     TEXT NOT NULL,
	poster               TEXT     ,
	synopsis             TEXT  DEFAULT '<empty synopsis>'   ,
	release              TEXT NOT NULL    ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	show_id              TEXT NOT NULL    ,
	season_number        INT NOT NULL DEFAULT 0   ,
	watch_count          INTEGER NOT NULL DEFAULT 0   ,
	episode_count        INTEGER NOT NULL DEFAULT 0   ,
	progress             FLOAT(2,1) NOT NULL DEFAULT 0.0   ,
	rating               FLOAT(2, 1)     ,
	last_watched         DATETIME     ,
	recent_episode       TEXT     ,
	duration             INTEGER NOT NULL DEFAULT 0   ,
	comment_count        INTEGER NOT NULL DEFAULT 0   ,
	fetched		     BOOLEAN DEFAULT FALSE,
	removed		     BOOLEAN DEFAULT FALSE,
	UNIQUE(show_id, path),
	FOREIGN KEY ( show_id ) REFERENCES tv_show( id ) ON DELETE CASCADE 
);

CREATE INDEX idx_season_show_id ON season ( show_id );

CREATE TABLE episode ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	tmdb_id              INTEGER,
	name                 TEXT NOT NULL    ,
	original_name	     TEXT NOT NULL,
	path                 TEXT NOT NULL,
	progress             FLOAT(2,1) NOT NULL DEFAULT 0.0   ,
	synopsis             TEXT  DEFAULT '<empy synopsis>'   ,
	rating               FLOAT(2, 1)     ,
	poster               TEXT     ,
	season_id            TEXT NOT NULL    ,
	release              TEXT NOT NULL    ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	duration             INT NOT NULL DEFAULT 0   ,
	last_watched         DATETIME     ,
	watch_count          INT NOT NULL DEFAULT 0   ,
	episode_number       INT NOT NULL DEFAULT 0   ,
	comment_count        INTEGER NOT NULL DEFAULT 0   ,
	fetched		     BOOLEAN DEFAULT FALSE,
	subtitle_uri	     TEXT,
	removed		     BOOLEAN DEFAULT FALSE,
	UNIQUE(season_id, path),
	FOREIGN KEY ( season_id ) REFERENCES season( id ) ON DELETE CASCADE ,
	CHECK ( 0.0 <= progress AND progress <= 1.0 ),
	CHECK ( 0 <= rating AND rating <= 5 )
);

CREATE INDEX idx_episode_season_id ON episode ( season_id );

CREATE TABLE episode_comment ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	content              TEXT  NOT NULL   ,
	episode_id           TEXT NOT NULL    ,
	episode_timestamp    INT     ,
	FOREIGN KEY ( episode_id ) REFERENCES episode( id ) ON DELETE CASCADE 
);

CREATE INDEX idx_comment_episode_id ON episode_comment ( episode_id );

CREATE TABLE movie (
	id		            TEXT NOT NULL PRIMARY KEY,
	tmdb_id              INTEGER,
	name		        TEXT NOT NULL,
	original_name	    TEXT NOT NULL,
	directory            TEXT NOT NULL,
	path                TEXT NOT NULL,
	poster		        TEXT,
	backdrop	        TEXT,
	tags		        TEXT,
	synopsis	        TEXT DEFAULT '<empty synopsis>',
	release		        TEXT NOT NULL,
	created_at	        DATETIME DEFAULT CURRENT_TIMESTAMP,
	watch_count	        INTEGER NOT NULL DEFAULT 0,
	progress	        FLOAT(2,1) NOT NULL DEFAULT 0.0,
	rating		        FLOAT(2,1),
	last_watched	    DATETIME,
	duration	        INTEGER NOT NULL DEFAULT 0,
	comment_count	    INTEGER NOT NULL DEFAULT 0,
	fetched		     BOOLEAN DEFAULT FALSE,
	subtitle_uri	     TEXT,
	removed		     BOOLEAN DEFAULT FALSE,
	UNIQUE(directory, path),
	CHECK ( 0.0 <= progress AND progress <= 1.0 ),
	CHECK ( 0 <= rating AND rating <= 5 ),
	FOREIGN KEY ( directory ) REFERENCES directory( id ) ON DELETE CASCADE
);

CREATE INDEX idx_movie_directory ON movie ( directory );

CREATE TABLE movie_comment ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	content              TEXT  NOT NULL   ,
	movie_id             TEXT NOT NULL    ,
	movie_timestamp      INT     ,
	FOREIGN KEY ( movie_id ) REFERENCES movie( id ) ON DELETE CASCADE 
);

CREATE INDEX idx_comment_movie_id ON movie_comment ( movie_id );

CREATE TABLE collection (
	id              TEXT NOT NULL PRIMARY KEY,
	name            TEXT NOT NULL,
	description     TEXT,
	view            TEXT NOT NULL,
	icon            INT,
	custom          TEXT,
	theme           INT,
	created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
	CHECK (view IN ('shown', 'hidden', 'pinned'))
);

CREATE TABLE collection_item (
	collection_id		TEXT NOT NULL,
	media_type		TEXT NOT NULL,
	media_id		TEXT NOT NULL,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	PRIMARY KEY ( collection_id, media_type, media_id),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE
);

CREATE VIEW get_episode_data AS SELECT
season.show_id,
tv_show.backdrop,
tv_show.tmdb_id AS show_tmdb_id,
tv_show.name AS show_name,
season.path AS season_path,
season.season_number,
tv_show.path AS show_path,
directory.path AS directory_path,
CASE WHEN NOT episode.fetched THEN NULL ELSE episode.poster END AS poster,
episode.*
FROM episode 
INNER JOIN season ON episode.season_id = season.id
INNER JOIN tv_show ON season.show_id = tv_show.id
INNER JOIN directory ON tv_show.directory = directory.id;

CREATE VIEW get_collection_posters AS SELECT collection_id, poster 
FROM (
	SELECT movie.poster, item.collection_id
	FROM collection_item item
	JOIN movie ON movie.id = item.media_id
	WHERE item.media_type = 'movie' AND movie.poster IS NOT NULL

	UNION ALL

	SELECT tv_show.poster, item.collection_id
	FROM collection_item item
	JOIN tv_show ON tv_show.id = item.media_id
	WHERE item.media_type = 'show' AND tv_show.poster IS NOT NULL


	UNION ALL

	SELECT season.poster, item.collection_id
	FROM collection_item item
	JOIN season ON season.id = item.media_id
	WHERE item.media_type = 'season' AND season.poster IS NOT NULL

	UNION ALL

	SELECT episode.poster, item.collection_id
	FROM collection_item item
	JOIN episode ON episode.id = item.media_id
	WHERE item.media_type = 'episode' AND episode.poster IS NOT NULL
) 
ORDER BY collection_id
LIMIT 4;

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


CREATE VIRTUAL TABLE media_fts USING fts5(
	name,
	synopsis,
	tags,
	tokenize='porter trigram'
);

CREATE TABLE media_fts_index (
	rowid INTEGER PRIMARY KEY,
	media_type TEXT NOT NULL,
	media_id TEXT NOT NULL,
	poster TEXT,
	removed BOOLEAN DEFAULT FALSE
);

CREATE TRIGGER fts_movie_insert_tr AFTER INSERT ON movie
BEGIN
	INSERT INTO media_fts (name, synopsis, tags)
	VALUES (NEW.name, NEW.synopsis, NEW.tags);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster)
	VALUES (last_insert_rowid(), 'movie', NEW.id, NEW.poster);
END;

CREATE TRIGGER fts_show_insert_tr AFTER INSERT ON tv_show
BEGIN
	INSERT INTO media_fts (name, synopsis, tags)
	VALUES (NEW.name, NEW.synopsis, NEW.tags);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster)
	VALUES (last_insert_rowid(), 'show', NEW.id, NEW.poster);
END;

CREATE TRIGGER fts_season_insert_tr AFTER INSERT ON season
BEGIN
	INSERT INTO media_fts (name, synopsis)
	VALUES (NEW.name, NEW.synopsis);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster)
	VALUES (last_insert_rowid(), 'season', NEW.id, NEW.poster);
END;

CREATE TRIGGER fts_episode_insert_tr AFTER INSERT ON episode
BEGIN
	INSERT INTO media_fts (name, synopsis)
	VALUES (NEW.name, NEW.synopsis);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster)
	VALUES (last_insert_rowid(), 'episode', NEW.id, NEW.poster);
END;

CREATE TRIGGER fts_movie_update_tr
AFTER UPDATE ON movie
BEGIN
	UPDATE media_fts
	SET name = NEW.name,
	synopsis = NEW.synopsis,
	tags = NEW.tags
	WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'movie' AND media_id = NEW.id);

    UPDATE media_fts_index
    SET poster = NEW.poster,
    removed = NEW.removed
    WHERE media_type = 'movie' AND media_id = NEW.id;

END;

CREATE TRIGGER fts_show_update_tr
AFTER UPDATE ON tv_show
BEGIN
	UPDATE media_fts
	SET name = NEW.name,
	synopsis = NEW.synopsis,
	tags = NEW.tags
	WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'show' AND media_id = NEW.id);

    UPDATE media_fts_index
    SET poster = NEW.poster,
    removed = NEW.removed
    WHERE media_type = 'show' AND media_id = NEW.id;
END;

CREATE TRIGGER fts_season_update_tr
AFTER UPDATE ON season
BEGIN
	UPDATE media_fts
	SET name = NEW.name,
	synopsis = NEW.synopsis
	WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'season' AND media_id = NEW.id);

    UPDATE media_fts_index
    SET poster = NEW.poster,
    removed = NEW.removed
    WHERE media_type = 'season' AND media_id = NEW.id;
END;

CREATE TRIGGER fts_episode_update_tr
AFTER UPDATE ON episode
BEGIN
	UPDATE media_fts
	SET name = NEW.name,
	synopsis = NEW.synopsis
	WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'episode' AND media_id = NEW.id);

    UPDATE media_fts_index
    SET poster = NEW.poster,
    removed = NEW.removed
    WHERE media_type = 'episode' AND media_id = NEW.id;
END;

CREATE TRIGGER fts_movie_delete_tr
AFTER DELETE ON movie
BEGIN
    DELETE FROM media_fts
    WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'movie' AND media_id = OLD.id);

    DELETE FROM media_fts_index
    WHERE media_type = 'movie' AND media_id = OLD.id;
END;

CREATE TRIGGER fts_show_delete_tr
AFTER DELETE ON tv_show
BEGIN
    DELETE FROM media_fts
    WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'show' AND media_id = OLD.id);

    DELETE FROM media_fts_index
    WHERE media_type = 'show' AND media_id = OLD.id;
END;

CREATE TRIGGER fts_season_delete_tr
AFTER DELETE ON season
BEGIN
    DELETE FROM media_fts
    WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'season' AND media_id = OLD.id);

    DELETE FROM media_fts_index
    WHERE media_type = 'season' AND media_id = OLD.id;
END;

CREATE TRIGGER fts_episode_delete_tr
AFTER DELETE ON episode
BEGIN
    DELETE FROM media_fts
    WHERE rowid = (SELECT rowid FROM media_fts_index WHERE media_type = 'episode' AND media_id = OLD.id);

    DELETE FROM media_fts_index
    WHERE media_type = 'episode' AND media_id = OLD.id;
END;

CREATE TRIGGER item_movie_delete_tr AFTER DELETE ON movie
BEGIN
	DELETE FROM collection_item WHERE media_type = 'movie' AND media_id = OLD.id;
END;

CREATE TRIGGER item_show_delete_tr AFTER DELETE ON tv_show
BEGIN
	DELETE FROM collection_item WHERE media_type = 'show' AND media_id = OLD.id;
END;

CREATE TRIGGER item_season_delete_tr AFTER DELETE ON season
BEGIN
	DELETE FROM collection_item WHERE media_type = 'season' AND media_id = OLD.id;
END;

CREATE TRIGGER item_episode_delete_tr AFTER DELETE ON episode
BEGIN
	DELETE FROM collection_item WHERE media_type = 'episode' AND media_id = OLD.id;
END;

CREATE TRIGGER mcomment_delete_tr AFTER DELETE ON movie_comment
BEGIN
	UPDATE movie SET comment_count = (
		SELECT COUNT(*) FROM movie_comment WHERE movie_comment.movie_id = OLD.movie_id
	) WHERE id = OLD.movie_id;
END;

CREATE TRIGGER mcomment_insert_tr AFTER DELETE ON movie_comment
BEGIN
	UPDATE movie SET comment_count = (
		SELECT COUNT(*) FROM movie_comment WHERE movie_comment.movie_id = OLD.movie_id
	) WHERE id = OLD.movie_id;
END;

CREATE TRIGGER ecomment_delete_tr AFTER DELETE ON episode_comment
BEGIN
	UPDATE episode SET comment_count = (
		SELECT COUNT(*) FROM episode_comment WHERE episode_comment.episode_id = OLD.episode_id
	) WHERE id = OLD.episode_id;
END;

CREATE TRIGGER ecomment_insert_tr AFTER INSERT ON episode_comment
BEGIN
	UPDATE episode SET comment_count = (
		SELECT COUNT(*) FROM episode_comment WHERE episode_comment.episode_id = NEW.episode_id
	) WHERE id = NEW.episode_id;
END;

CREATE TRIGGER episode_delete_tr AFTER DELETE ON episode
BEGIN
	UPDATE season SET watch_count = COALESCE((
			SELECT MIN(episode.watch_count) FROM episode WHERE episode.season_id = OLD.season_id
	), 0),
progress = COALESCE((
		SELECT AVG(episode.progress) FROM episode WHERE episode.season_id = OLD.season_id
), 0.0),
    rating = (
	SELECT AVG(episode.rating) FROM episode WHERE episode.season_id = OLD.season_id
),
last_watched = (
	SELECT MAX(episode.last_watched) FROM episode WHERE episode.season_id = OLD.season_id
),
recent_episode = (
	SELECT episode.id FROM episode WHERE episode.season_id = OLD.season_id AND episode.last_watched = (SELECT MAX(last_watched) FROM episode WHERE season_id = OLD.season_id)
),
comment_count = COALESCE((
		SELECT SUM(episode.comment_count) FROM episode WHERE episode.season_id = OLD.season_id
), 0),
    episode_count = COALESCE((
		SELECT COUNT(episode.id) FROM episode WHERE episode.season_id = OLD.season_id
), 0),
    duration = COALESCE((
		SELECT SUM(episode.duration) FROM episode WHERE episode.season_id = OLD.season_id
), 0)
    WHERE id = OLD.season_id;	
END;

CREATE TRIGGER episode_insert_tr AFTER INSERT ON episode 
BEGIN
	UPDATE season SET episode_count = (
		SELECT COUNT(*) FROM episode WHERE episode.season_id = NEW.season_id
	),
	duration = COALESCE((
			SELECT SUM(episode.duration) FROM episode WHERE episode.season_id = NEW.season_id
	),0),
watch_count = COALESCE((
		SELECT MIN(episode.watch_count) FROM episode WHERE episode.season_id = NEW.season_id
),0),
    progress = COALESCE((
		SELECT AVG(episode.progress) FROM episode WHERE episode.season_id = NEW.season_id
),0.0),
    rating = (
	SELECT AVG(episode.rating) FROM episode WHERE episode.season_id = NEW.season_id
),
last_watched = (
	SELECT MAX(episode.last_watched) FROM episode WHERE episode.season_id = NEW.season_id
),
recent_episode = (
	SELECT episode.id FROM episode WHERE episode.season_id = NEW.season_id AND episode.last_watched = (SELECT MAX(last_watched) FROM episode WHERE season_id = NEW.season_id)
),
comment_count = COALESCE((
		SELECT SUM(episode.comment_count) FROM episode WHERE episode.season_id = NEW.season_id
), 0)
    WHERE id = NEW.season_id;	
END;

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
recent_episode = (
	SELECT episode.id FROM episode WHERE episode.season_id = NEW.season_id AND episode.last_watched = (SELECT MAX(last_watched) FROM episode WHERE season_id = NEW.season_id)
),
comment_count = COALESCE((
		SELECT SUM(episode.comment_count) FROM episode WHERE episode.season_id = NEW.season_id
),0)
    WHERE id = NEW.season_id;	
END;

CREATE TRIGGER season_delete_tr AFTER DELETE ON season
BEGIN
	UPDATE tv_show SET watch_count = COALESCE((
			SELECT MIN(season.watch_count) FROM season WHERE season.show_id = OLD.show_id
	), 0),
progress = COALESCE((
		SELECT AVG(season.progress) FROM season WHERE season.show_id = OLD.show_id
),0.0),
    rating = (
	SELECT AVG(season.rating) FROM season WHERE season.show_id = OLD.show_id
),
last_watched = (
	SELECT MAX(season.last_watched) FROM season WHERE season.show_id = OLD.show_id
),
recent_season = (
	SELECT season.id FROM season WHERE season.show_id = OLD.show_id AND season.last_watched = (SELECT MAX(last_watched) FROM season WHERE show_id = OLD.show_id)
),
comment_count = COALESCE((
		SELECT SUM(season.comment_count) FROM season WHERE season.show_id = OLD.show_id
),0),
    season_count = COALESCE((
		SELECT COUNT(*) FROM season WHERE season.show_id = OLD.show_id
),0),
    duration = COALESCE((
		SELECT SUM(season.duration) FROM season WHERE season.show_id = OLD.show_id
), 0)
    WHERE id = OLD.show_id;
END;

CREATE TRIGGER season_insert_tr AFTER INSERT ON season 
BEGIN
	UPDATE tv_show SET season_count = (
		SELECT COUNT(*) FROM season WHERE season.show_id = NEW.show_id
	),
	duration = COALESCE((
			SELECT SUM(season.duration) FROM season WHERE season.show_id = NEW.show_id
	),0),
watch_count = COALESCE((
		SELECT MIN(season.watch_count) FROM season WHERE season.show_id = NEW.show_id
),0),
    progress = COALESCE((
		SELECT AVG(season.progress) FROM season WHERE season.show_id = NEW.show_id
),0.0),
    rating = (
	SELECT AVG(season.rating) FROM season WHERE season.show_id = NEW.show_id
),
last_watched = (
	SELECT MAX(season.last_watched) FROM season WHERE season.show_id = NEW.show_id
),
recent_season = (
	SELECT season.id FROM season WHERE season.show_id = NEW.show_id AND season.last_watched = (SELECT MAX(last_watched) FROM season WHERE show_id = NEW.show_id)
),
comment_count = COALESCE((
		SELECT SUM(season.comment_count) FROM season WHERE season.show_id = NEW.show_id
), 0)
    WHERE id = NEW.show_id;
END;

CREATE TRIGGER season_update_tr AFTER UPDATE ON season
BEGIN
	UPDATE tv_show SET watch_count = COALESCE((
			SELECT MIN(season.watch_count) FROM season WHERE season.show_id = NEW.show_id
	), 0),
progress = COALESCE((
		SELECT AVG(season.progress) FROM season WHERE season.show_id = NEW.show_id
),0.0),
    rating = (
	SELECT AVG(season.rating) FROM season WHERE season.show_id = NEW.show_id
),
last_watched = (
	SELECT MAX(season.last_watched) FROM season WHERE season.show_id = NEW.show_id
),
recent_season = (
	SELECT season.id FROM season WHERE season.show_id = NEW.show_id AND season.last_watched = (SELECT MAX(last_watched) FROM season WHERE show_id = NEW.show_id)
),
duration = COALESCE((
		SELECT SUM(season.duration) FROM season WHERE season.show_id = NEW.show_id
),0),
    comment_count = COALESCE((
		SELECT SUM(season.comment_count) FROM season WHERE season.show_id = NEW.show_id
),0) 
    WHERE id = NEW.show_id;
END;

CREATE TRIGGER show_refetch_tr AFTER UPDATE OF tmdb_id ON tv_show WHEN NEW.tmdb_id IS NULL
BEGIN
	UPDATE season
	SET tmdb_id=NULL,
	fetched=FALSE,
	removed=NEW.removed
	WHERE show_id = NEW.id;
END;

CREATE TRIGGER season_refetch_tr AFTER UPDATE OF tmdb_id ON season WHEN NEW.tmdb_id IS NULL
BEGIN
	UPDATE episode
	SET tmdb_id=NULL,
	fetched=FALSE,
	removed=NEW.removed
	WHERE season_id = NEW.id;
END;
