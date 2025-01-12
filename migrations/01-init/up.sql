CREATE TABLE polls(id INTEGER, question TEXT);
CREATE TABLE poll_answers(
	id INTEGER,
	answer TEXT,
	votes INTEGER,
	poll_id INTEGER,
	FOREIGN KEY(poll_id) REFERENCES polls(id)
);
