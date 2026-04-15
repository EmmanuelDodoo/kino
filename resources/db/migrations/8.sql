PRAGMA recursive_triggers = ON;
PRAGMA user_version = 8;

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


ALTER TABLE episode ADD COLUMN audio_id TEXT;
ALTER TABLE movie ADD COLUMN audio_id TEXT;

DROP TRIGGER IF EXISTS episode_subtitle_delete_tr;
DROP TRIGGER IF EXISTS movie_subtitle_delete_tr;

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
