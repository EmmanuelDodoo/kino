-- Auto collections
PRAGMA user_version = 1;

CREATE INDEX idx_collection_item_id ON collection_item ( collection_id );

CREATE TABLE collection_inserts (
	id              	TEXT NOT NULL PRIMARY KEY,
	collection_id		TEXT NOT NULL,
	name            	TEXT NOT NULL,
	trigger_name        TEXT NOT NULL UNIQUE,
	media_type		    TEXT NOT NULL,
	created_at      	DATETIME DEFAULT CURRENT_TIMESTAMP,
	logic			    TEXT,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE
);

CREATE INDEX idx_collection_inserts_id ON collection_inserts ( collection_id );

CREATE TABLE collection_deletes (
	id              	TEXT NOT NULL PRIMARY KEY,
	collection_id		TEXT NOT NULL,
	name            	TEXT NOT NULL,
	trigger_name        TEXT NOT NULL UNIQUE,
	media_type		    TEXT NOT NULL,
	created_at      	DATETIME DEFAULT CURRENT_TIMESTAMP,
	logic			    TEXT,
	CHECK ( media_type IN ('movie', 'show', 'season', 'episode')),
	FOREIGN KEY ( collection_id ) REFERENCES collection( id ) ON DELETE CASCADE

);

CREATE INDEX idx_collection_deletes_id ON collection_deletes ( collection_id );

