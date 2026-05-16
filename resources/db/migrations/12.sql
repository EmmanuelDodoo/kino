PRAGMA recursive_triggers = ON;
PRAGMA user_version = 12;

ALTER TABLE episode ADD COLUMN video_id TEXT;
ALTER TABLE movie ADD COLUMN video_id TEXT;
