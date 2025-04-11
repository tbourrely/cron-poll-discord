CREATE TABLE polls(
	id TEXT PRIMARY KEY,
	cron TEXT,
	question TEXT,
	multiselect BOOLEAN,
	guild TEXT,
	channel TEXT,
	duration INT -- in seconds
);

CREATE TABLE answers(
	id SERIAL PRIMARY KEY,
	answer TEXT,
	poll_id TEXT,
	FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

CREATE TABLE poll_instances(
	id BIGINT PRIMARY KEY,
	sent_at BIGINT,
	poll_id TEXT,
	FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

CREATE TABLE poll_instance_answers(
	internal_id SERIAL PRIMARY KEY,
	id BIGINT,
	votes INTEGER,
	answer TEXT,
	instance_id BIGINT,
	FOREIGN KEY (instance_id) REFERENCES poll_instances(id) ON DELETE CASCADE
);
