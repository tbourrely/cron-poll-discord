CREATE TABLE poll_groups(
    id UUID PRIMARY KEY,
    created_at timestamp default current_timestamp NOT NULL
);

ALTER TABLE polls
    ADD COLUMN poll_group_id UUID DEFAULT NULL,
    FOREIGN KEY (poll_group_id) REFERENCES poll_groups(id) ON DELETE CASCADE;