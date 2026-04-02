PRAGMA recursive_triggers = ON;
PRAGMA user_version = 6;

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
	CHECK (kind IN ('embedded', 'loaded')),
	CHECK (media_type IN ('movie', 'episode'))
);

CREATE UNIQUE INDEX subtitle_unique_embedded ON subtitle(video, title, lang)
WHERE kind='embedded';

CREATE UNIQUE INDEX subtitle_unique_loaded ON subtitle(video, path) WHERE
kind='loaded';


ALTER TABLE episode DROP COLUMN subtitle_uri;
ALTER TABLE movie DROP COLUMN subtitle_uri;

ALTER TABLE episode ADD COLUMN subtitle_id REFERENCES subtitle( id );
ALTER TABLE movie ADD COLUMN subtitle_id REFERENCES subtitle( id );

CREATE TRIGGER episode_subtitle_delete_tr AFTER DELETE ON episode
BEGIN
	DELETE FROM subtitle WHERE media_type= 'episode' AND video = OLD.id;
END;

CREATE TRIGGER movie_subtitle_delete_tr AFTER DELETE ON movie
BEGIN
	DELETE FROM subtitle WHERE media_type= 'movie' AND video = OLD.id;
END;

