PRAGMA recursive_triggers = ON;
PRAGMA user_version = 11;

ALTER TABLE tmdb ADD COLUMN wish_type TEXT;
DROP TRIGGER IF EXISTS tmdb_parent_update;
DROP TRIGGER IF EXISTS tmdb_video_fetched;
DROP TRIGGER IF EXISTS tmdb_season_number_update;
DROP TRIGGER IF EXISTS tmdb_movie_name_update;
DROP TRIGGER IF EXISTS tmdb_show_name_update;
DROP TRIGGER IF EXISTS tmdb_movie_delete;
DROP TRIGGER IF EXISTS tmdb_show_delete;
DROP TRIGGER IF EXISTS tmdb_season_delete;
DROP TRIGGER IF EXISTS tmdb_episode_delete;

CREATE TABLE tmdb_temp (
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
	wish_type		TEXT,

	FOREIGN KEY (parent) REFERENCES tmdb(id) ON DELETE CASCADE,
	CHECK ( wish_type IN ('movie', 'show', 'season', 'episode')),
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode', 'wish'))
);

INSERT INTO tmdb_temp  (id, created_at, tmdb_id, media_type, media_id, status, retry, poster, parent, name, number, backdrop, wish_type) SELECT id, created_at, tmdb_id, media_type, media_id, status, retry, poster, parent, name, number, backdrop, wish_type FROM tmdb;

DROP TABLE tmdb;

ALTER TABLE tmdb_temp RENAME TO tmdb;

CREATE TABLE wish (
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	media_type	     TEXT NOT NULL,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	name                 TEXT NOT NULL    ,
	poster               TEXT     ,
	synopsis             TEXT  DEFAULT '<empty synopsis>'   ,
	release              TEXT NOT NULL    ,
	rating               FLOAT(2, 1)     ,
	removed		     BOOLEAN DEFAULT FALSE,
	completed	     BOOLEAN DEFAULT FALSE,
	request		     TEXT,
	source		     TEXT NOT NULL DEFAULT 'none',

	duration             INTEGER DEFAULT 0   ,
	count                INTEGER DEFAULT 0   ,
	season_number        INTEGER DEFAULT 0   ,
	episode_number       INTEGER DEFAULT 0   ,
	tags                 TEXT DEFAULT '' ,

	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	UNIQUE (media_type, name, season_number, episode_number)
);

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

CREATE TRIGGER tmdb_movie_name_update AFTER UPDATE of name on movie
BEGIN
	UPDATE tmdb SET name=NEW.name, status=1, retry=0 WHERE media_id = NEW.id;
END;

CREATE TRIGGER tmdb_show_name_update AFTER UPDATE of name on tv_show
BEGIN
	UPDATE tmdb SET name=NEW.name, status=1, retry=0 WHERE media_id = NEW.id;
END;

CREATE TRIGGER tmdb_wish_name_update AFTER UPDATE OF name ON wish
BEGIN
	UPDATE tmdb SET name=NEW.name, status=1, retry=0 WHERE media_id=NEW.id AND NEW.source ='tmdb';
END;

CREATE TRIGGER tmdb_wish_season_update AFTER UPDATE OF season_number ON wish
BEGIN
	UPDATE tmdb SET number=NEW.season_number, status=1, retry=0 WHERE media_id=NEW.id AND NEW.source ='tmdb';
END;

CREATE TRIGGER tmdb_wish_episode_update AFTER UPDATE OF episode_number ON wish
BEGIN
	UPDATE tmdb SET backdrop=NEW.episode_number, status=1, retry=0 WHERE media_id=NEW.id AND NEW.source ='tmdb';
END;

CREATE TRIGGER tmdb_wish_media_update AFTER UPDATE OF media_type ON wish
BEGIN
	UPDATE tmdb SET wish_type =NEW.media_type, status=1, retry=0 WHERE media_id=NEW.id AND NEW.source='tmdb';
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

CREATE TRIGGER tmdb_wish_delete AFTER DELETE ON wish
BEGIN
	DELETE FROM tmdb WHERE media_id=OLD.id;
END;

