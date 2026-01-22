PRAGMA recursive_triggers = ON;
PRAGMA user_version = 4;

DROP TABLE IF EXISTS episode_comment;

DROP TABLE IF EXISTS movie_comment;

DROP INDEX IF EXISTS idx_comment_episode_id;

DROP INDEX IF EXISTS idx_comment_movie_id;

CREATE TABLE comment ( 
	id                   TEXT NOT NULL  PRIMARY KEY  ,
	created_at           DATETIME  DEFAULT CURRENT_TIMESTAMP   ,
	content              TEXT  NOT NULL   ,
	media_id             TEXT NOT NULL    ,
	media_type	         TEXT NOT NULL,
	timestamp      	     INT     ,
	removed		         BOOLEAN DEFAULT FALSE,
	CHECK (media_type IN ('movie', 'episode'))
);

DROP TRIGGER IF EXISTS mcomment_delete_tr;

DROP TRIGGER IF EXISTS mcomment_insert_tr;

DROP TRIGGER IF EXISTS ecomment_insert_tr;

DROP TRIGGER IF EXISTS ecomment_delete_tr;

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
