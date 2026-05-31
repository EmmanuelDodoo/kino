PRAGMA recursive_triggers = ON;
PRAGMA user_version = 13;

DROP TRIGGER IF EXISTS fts_movie_insert_tr;
DROP TRIGGER IF EXISTS fts_show_insert_tr;
DROP TRIGGER IF EXISTS fts_season_insert_tr;
DROP TRIGGER IF EXISTS fts_episode_insert_tr;

DROP TRIGGER IF EXISTS fts_movie_update_tr;
DROP TRIGGER IF EXISTS fts_show_update_tr;
DROP TRIGGER IF EXISTS fts_season_update_tr;
DROP TRIGGER IF EXISTS fts_episode_update_tr;

ALTER TABLE movie ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
UPDATE movie SET status = CASE WHEN removed THEN 1 ELSE 0 END;
ALTER TABLE movie DROP COLUMN removed;

ALTER TABLE tv_show ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
UPDATE tv_show SET status = CASE WHEN removed THEN 1 ELSE 0 END;
ALTER TABLE tv_show DROP COLUMN removed;

ALTER TABLE season ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
UPDATE season SET status = CASE WHEN removed THEN 1 ELSE 0 END;
ALTER TABLE season DROP COLUMN removed;

ALTER TABLE episode ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
UPDATE episode SET status = CASE WHEN removed THEN 1 ELSE 0 END;
ALTER TABLE episode DROP COLUMN removed;

ALTER TABLE media_fts_index ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
UPDATE media_fts_index SET status = CASE WHEN removed THEN 1 ELSE 0 END;
ALTER TABLE media_fts_index DROP COLUMN removed;

CREATE TRIGGER fts_movie_insert_tr AFTER INSERT ON movie
BEGIN
	INSERT INTO media_fts (name, synopsis, tags)
	VALUES (NEW.name, NEW.synopsis, NEW.tags);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster, status)
	VALUES (last_insert_rowid(), 'movie', NEW.id, NEW.poster, NEW.status);
END;

CREATE TRIGGER fts_show_insert_tr AFTER INSERT ON tv_show
BEGIN
	INSERT INTO media_fts (name, synopsis, tags)
	VALUES (NEW.name, NEW.synopsis, NEW.tags);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster, status)
	VALUES (last_insert_rowid(), 'show', NEW.id, NEW.poster, NEW.status);
END;

CREATE TRIGGER fts_season_insert_tr AFTER INSERT ON season
BEGIN
	INSERT INTO media_fts (name, synopsis)
	VALUES (NEW.name, NEW.synopsis);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster, status)
	VALUES (last_insert_rowid(), 'season', NEW.id, NEW.poster, NEW.status);
END;

CREATE TRIGGER fts_episode_insert_tr AFTER INSERT ON episode
BEGIN
	INSERT INTO media_fts (name, synopsis)
	VALUES (NEW.name, NEW.synopsis);

	INSERT INTO media_fts_index (rowid, media_type, media_id, poster, status)
	VALUES (last_insert_rowid(), 'episode', NEW.id, NEW.poster, NEW.status);
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
    status = NEW.status
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
    status = NEW.status
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
    status = NEW.status
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
    status = NEW.status
    WHERE media_type = 'episode' AND media_id = NEW.id;
END;

CREATE TRIGGER season_status_update_tr AFTER UPDATE OF status ON season
BEGIN
    UPDATE episode SET status=NEW.status WHERE season_id=NEW.id;
END;

CREATE TRIGGER show_status_update_tr AFTER UPDATE OF status ON tv_show
BEGIN
	UPDATE season SET status=NEW.status WHERE show_id=NEW.id;
END;
