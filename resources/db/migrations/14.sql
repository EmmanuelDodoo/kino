PRAGMA recursive_triggers = ON;
PRAGMA user_version = 14;

DROP TRIGGER IF EXISTS season_status_update_tr;
DROP TRIGGER IF EXISTS show_status_update_tr;

CREATE TRIGGER season_status_update_tr AFTER UPDATE OF status ON season
BEGIN
    UPDATE episode SET status=NEW.status WHERE season_id=NEW.id AND NEW.status > 0;
END;

CREATE TRIGGER show_status_update_tr AFTER UPDATE OF status ON tv_show
BEGIN
	UPDATE season SET status=NEW.status WHERE show_id=NEW.id AND NEW.status > 0;
END;
