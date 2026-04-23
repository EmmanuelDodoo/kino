PRAGMA recursive_triggers = ON;
PRAGMA user_version = 10;

CREATE TABLE directory ( 
	id		TEXT NOT NULL PRIMARY KEY,
	path		TEXT NOT NULL UNIQUE,
	active		BOOLEAN NOT NULL,
	media_type	TEXT NOT NULL,
	last_scan       DATETIME  NOT NULL DEFAULT CURRENT_TIMESTAMP,
	source		TEXT NOT NULL DEFAULT 'none',
	CHECK ( media_type IN ('movies', 'shows'))
);

CREATE TABLE tmdb (
	id                   	TEXT NOT NULL  PRIMARY KEY, 
	created_at           	DATETIME  DEFAULT CURRENT_TIMESTAMP,
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

CREATE TABLE tv_show ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
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
	removed		     BOOLEAN DEFAULT FALSE,
	request		     TEXT,
	source		     TEXT NOT NULL DEFAULT 'none',
	UNIQUE(directory, path),
	FOREIGN KEY ( directory ) REFERENCES directory( id ) ON DELETE CASCADE
);

CREATE INDEX idx_tv_show_directory ON tv_show ( directory );

CREATE TABLE season ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
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
	removed		     BOOLEAN DEFAULT FALSE,
	request		     TEXT,
	source		     TEXT NOT NULL DEFAULT 'none',
	UNIQUE(show_id, path),
	FOREIGN KEY ( show_id ) REFERENCES tv_show( id ) ON DELETE CASCADE 
);

CREATE INDEX idx_season_show_id ON season ( show_id );

CREATE TABLE episode ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	name                 TEXT NOT NULL    ,
	original_name	     TEXT NOT NULL,
	path                 TEXT NOT NULL,
	progress             FLOAT(2,1) NOT NULL DEFAULT 0.0   ,
	synopsis             TEXT  DEFAULT '<empy synopsis>'   ,
	rating               FLOAT(2, 1)     ,
	poster               TEXT     ,
	generate_poster	     BOOLEAN DEFAULT TRUE,
	season_id            TEXT NOT NULL    ,
	release              TEXT NOT NULL    ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	duration             INT NOT NULL DEFAULT 0   ,
	last_watched         DATETIME     ,
	watch_count          INT NOT NULL DEFAULT 0   ,
	episode_number       INT NOT NULL DEFAULT 0   ,
	comment_count        INTEGER NOT NULL DEFAULT 0   ,
	fetched		     BOOLEAN DEFAULT FALSE,
	subtitle_id	     TEXT ,
	audio_id	     TEXT,
	removed		     BOOLEAN DEFAULT FALSE,
	request		     TEXT,
	source		     TEXT NOT NULL DEFAULT 'none',
	UNIQUE(season_id, path),
	FOREIGN KEY ( season_id ) REFERENCES season( id ) ON DELETE CASCADE ,
	FOREIGN KEY ( subtitle_id ) REFERENCES subtitle( id ),
	CHECK ( 0.0 <= progress AND progress <= 1.0 ),
	CHECK ( 0 <= rating AND rating <= 5 )
);

CREATE INDEX idx_episode_season_id ON episode ( season_id );

CREATE TABLE movie (
	id		     TEXT NOT NULL PRIMARY KEY,
	name		     TEXT NOT NULL,
	original_name	     TEXT NOT NULL,
	directory            TEXT NOT NULL,
	path                 TEXT NOT NULL,
	poster		     TEXT,
	generate_poster	     BOOLEAN DEFAULT TRUE,
	backdrop	     TEXT,
	tags		     TEXT,
	synopsis	     TEXT DEFAULT '<empty synopsis>',
	release		     TEXT NOT NULL,
	created_at	     DATETIME DEFAULT CURRENT_TIMESTAMP,
	watch_count	     INTEGER NOT NULL DEFAULT 0,
	progress	     FLOAT(2,1) NOT NULL DEFAULT 0.0,
	rating		     FLOAT(2,1),
	last_watched	     DATETIME,
	duration	     INTEGER NOT NULL DEFAULT 0,
	comment_count	     INTEGER NOT NULL DEFAULT 0,
	fetched		     BOOLEAN DEFAULT FALSE,
	subtitle_id	     TEXT,
	audio_id	     TEXT,
	removed		     BOOLEAN DEFAULT FALSE,
	request		     TEXT,
	source		     TEXT NOT NULL DEFAULT 'none',
	UNIQUE(directory, path),
	FOREIGN KEY ( subtitle_id ) REFERENCES subtitle( id ),
	CHECK ( 0.0 <= progress AND progress <= 1.0 ),
	CHECK ( 0 <= rating AND rating <= 5 ),
	FOREIGN KEY ( directory ) REFERENCES directory( id ) ON DELETE CASCADE
);

CREATE INDEX idx_movie_directory ON movie ( directory );

CREATE TABLE comment ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	content              TEXT  NOT NULL   ,
	media_id             TEXT NOT NULL    ,
	media_type	     TEXT NOT NULL,
	timestamp      	     INT     ,
	removed		     BOOLEAN DEFAULT FALSE,
	CHECK (media_type IN ('movie', 'episode'))
);

CREATE TABLE subtitle (
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	video                TEXT NOT NULL,
	media_type	     TEXT NOT NULL,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	kind		     TEXT NOT NULL,
	path		     TEXT,
	title		     TEXT NOT NULL,
	lang		     TEXT NOT NULL,
	removed		     BOOLEAN DEFAULT FALSE,
	sub_offset	     FLOAT DEFAULT 0.0,
	CHECK (kind IN ('embedded', 'loaded')),
	CHECK (media_type IN ('movie', 'episode'))
);

CREATE TABLE audio (
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	media                TEXT NOT NULL,
	media_type	     TEXT NOT NULL,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	stream		     INTEGER NOT NULL DEFAULT 0,
	codec		     TEXT,
	lang		     TEXT,
	channels	     INTEGER NOT NULL DEFAULT 0,
	sample_rate	     INTEGER NOT NULL DEFAULT 0,
	bitrate	     	     INTEGER NOT NULL DEFAULT 0,
	depth	     	     INTEGER NOT NULL DEFAULT 0,
	UNIQUE(media, stream),
	CHECK (media_type IN ('movie', 'episode'))
);

CREATE TABLE video (
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	media                TEXT NOT NULL,
	media_type	     TEXT NOT NULL,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	stream		     INTEGER NOT NULL DEFAULT 0,
	tag		     TEXT,
	codec		     TEXT,
	bitrate	     	     INTEGER NOT NULL DEFAULT 0,
	width	     	     INTEGER NOT NULL DEFAULT 0,
	height	     	     INTEGER NOT NULL DEFAULT 0,
	depth	     	     INTEGER NOT NULL DEFAULT 0,
	framerate	     FLOAT NOT NULL DEFAULT 0.0,
	interlaced	     BOOLEAN DEFAULT FALSE,
	dar_num	     	     INTEGER NOT NULL DEFAULT 0,
	dar_denom	     INTEGER NOT NULL DEFAULT 0,
	UNIQUE(media, stream),
	CHECK (media_type IN ('movie', 'episode'))
);

CREATE UNIQUE INDEX subtitle_unique_embedded ON subtitle(video, title, lang)
WHERE kind='embedded';

CREATE UNIQUE INDEX subtitle_unique_loaded ON subtitle(video, path) WHERE
kind='loaded';

CREATE TABLE collection (
	id              TEXT NOT NULL PRIMARY KEY,
	name            TEXT NOT NULL,
	description     TEXT,
	view            TEXT NOT NULL,
	icon            INT,
	custom          TEXT,
	theme           INT,
	created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
	removed		BOOLEAN DEFAULT FALSE,
	CHECK (view IN ('shown', 'hidden', 'pinned'))
);

CREATE TABLE collection_item (
	collection_id		TEXT NOT NULL,
	media_type		TEXT NOT NULL,
	media_id		TEXT NOT NULL,
	created_at		DATETIME DEFAULT CURRENT_TIMESTAMP,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	PRIMARY KEY ( collection_id, media_type, media_id),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE
);

CREATE INDEX idx_collection_item_id ON collection_item ( collection_id );

CREATE TABLE collection_inserts (
	id              	TEXT NOT NULL PRIMARY KEY,
	collection_id		TEXT NOT NULL,
	name            	TEXT NOT NULL,
	trigger_name            TEXT NOT NULL UNIQUE,
	media_type		TEXT NOT NULL,
	created_at      	DATETIME DEFAULT CURRENT_TIMESTAMP,
	logic			TEXT,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE
);

CREATE INDEX idx_collection_inserts_id ON collection_inserts ( collection_id );

CREATE TABLE collection_deletes (
	id              	TEXT NOT NULL PRIMARY KEY,
	collection_id		TEXT NOT NULL,
	name            	TEXT NOT NULL,
	trigger_name            TEXT NOT NULL UNIQUE,
	media_type		TEXT NOT NULL,
	created_at      	DATETIME DEFAULT CURRENT_TIMESTAMP,
	logic			TEXT,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE

);

CREATE INDEX idx_collection_deletes_id ON collection_deletes ( collection_id );

CREATE TABLE image (
    path            TEXT NOT NULL PRIMARY KEY,
    main            TEXT,
    accent          TEXT,
    generated       BOOL DEFAULT FALSE,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

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

CREATE TRIGGER comment_delete_tr AFTER DELETE ON comment
BEGIN
	UPDATE movie SET comment_count = (
		SELECT COUNT(*) FROM comment WHERE comment.media_type = 'movie' AND comment.media_id = OLD.media_id 
	) WHERE id = OLD.media_id;

	UPDATE episode SET comment_count = (
		SELECT COUNT(*) FROM comment WHERE comment.media_type = 'episode' AND comment.media_id = OLD.media_id 
	) WHERE id = OLD.media_id;

END;

CREATE TRIGGER comment_insert_tr AFTER INSERT ON comment
BEGIN
	UPDATE movie SET comment_count = (
		SELECT COUNT(*) FROM comment WHERE comment.media_type = 'movie' AND comment.media_id = NEW.media_id 
	) WHERE id = NEW.media_id;

	UPDATE episode SET comment_count = (
		SELECT COUNT(*) FROM comment WHERE comment.media_type = 'episode' AND comment.media_id = NEW.media_id 
	) WHERE id = NEW.media_id;
END;

CREATE TRIGGER episode_comment_delete_tr AFTER DELETE ON episode
BEGIN
	DELETE FROM comment WHERE media_type = 'episode' AND media_id = OLD.id;
END;

CREATE TRIGGER movie_comment_delete_tr AFTER DELETE ON movie
BEGIN
	DELETE FROM comment WHERE media_type = 'movie' AND media_id = OLD.id;
END;

CREATE TRIGGER episode_info_delete_tr AFTER DELETE ON episode
BEGIN
	DELETE FROM subtitle WHERE media_type= 'episode' AND video = OLD.id;
	DELETE FROM audio WHERE media_type= 'episode' AND media = OLD.id;
	DELETE FROM video WHERE media_type= 'episode' AND media = OLD.id;
END;

CREATE TRIGGER movie_info_delete_tr AFTER DELETE ON movie
BEGIN
	DELETE FROM subtitle WHERE media_type= 'movie' AND video = OLD.id;
	DELETE FROM audio WHERE media_type= 'movie' AND media = OLD.id;
	DELETE FROM video WHERE media_type= 'movie' AND media = OLD.id;
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
