PRAGMA foreign_keys = ON;

CREATE TABLE polls(
	id TEXT PRIMARY KEY,
	cron TEXT,
	question TEXT
);

CREATE TABLE answers(
	id INTEGER PRIMARY KEY,
	answer TEXT,
	poll_id TEXT,
	FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

CREATE TABLE poll_instances(
	id INTEGER PRIMARY KEY,
	sent_at INTEGER,
	poll_id TEXT,
	FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

CREATE TABLE poll_instance_answers(
	id INTEGER PRIMARY KEY,
	votes INTEGER,
	answer TEXT,
	instance_id INTEGER,
	FOREIGN KEY (instance_id) REFERENCES poll_instances(id) ON DELETE CASCADE
);
