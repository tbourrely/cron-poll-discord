CREATE TABLE polls(
	id TEXT,
	cron TEXT,
	question TEXT,
	discord_poll_id INTEGER,
	sent INTEGER
);

CREATE TABLE poll_answers(
	discord_answer_id INTEGER,
	answer TEXT,
	votes INTEGER,
	poll_id TEXT,
	FOREIGN KEY(poll_id) REFERENCES polls(id)
);
