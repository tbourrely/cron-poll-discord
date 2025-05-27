use crate::poll::domain::{Poll, PollInstance, PollInstanceAnswer};
use futures::TryStreamExt;
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::error::Error;
use uuid::Uuid;

pub struct PollRepository<'a> {
    pub pool: &'a PgPool,
}

pub struct PollInstanceRepository<'a> {
    pub pool: &'a PgPool,
    pub poll_repository: &'a PollRepository<'a>, // TODO: not ddd compatible, a repository must not
                                                 // link another one
}

#[derive(sqlx::FromRow)]
pub struct AnswerRow {
    pub id: i32,
    pub answer: String,
    pub poll_id: String,
}

impl<'a> PollRepository<'a> {
    pub async fn save(&self, p: Poll) -> Result<(), Box<dyn Error>> {
        let exists = self.poll_exists(p.id).await?;

        if !exists {
            self.create(&p).await?;
        } else {
            self.update_poll(&p).await?;

            let saved_answers = self.find_answers(p.id).await?;
            for answer in &p.answers {
                match saved_answers.iter().find(|item| item.answer == *answer) {
                    Some(_) => (),
                    None => self.create_answer(answer.clone(), p.id).await?,
                };
            }
            for answer in &saved_answers {
                if let None = p.answers.iter().find(|item| **item == answer.answer) {
                    self.delete_answer(answer.answer.clone(), p.id).await?;
                }
            }
        }

        Ok(())
    }

    async fn create(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.create_poll(&p).await?;

        for answer in &p.answers {
            self.create_answer(answer.clone(), p.id).await?;
        }

        Ok(())
    }

    async fn create_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "
INSERT INTO polls
(id, cron, question, multiselect, guild, channel, duration, onetime, sent)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(p.id.to_string())
        .bind(p.cron.clone())
        .bind(p.question.clone())
        .bind(p.multiselect)
        .bind(p.guild.clone())
        .bind(p.channel.clone())
        .bind(p.duration)
        .bind(p.onetime.clone())
        .bind(p.sent.clone())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn create_answer(&self, a: String, poll_id: Uuid) -> Result<(), Box<dyn Error>> {
        sqlx::query("INSERT INTO answers (answer, poll_id) VALUES ($1, $2)")
            .bind(a.clone())
            .bind(poll_id.to_string())
            .execute(self.pool)
            .await?;
        Ok(())
    }

    async fn poll_exists(&self, id: Uuid) -> Result<bool, Box<dyn Error>> {
        let result = sqlx::query("SELECT id FROM polls WHERE id = $1 LIMIT 1")
            .bind(id.to_string())
            .fetch_optional(self.pool)
            .await?;

        Ok(result.is_some())
    }

    async fn find_answers(&self, id: Uuid) -> Result<Vec<AnswerRow>, Box<dyn Error>> {
        let answers: Vec<AnswerRow> = sqlx::query_as("SELECT * FROM answers WHERE poll_id = $1")
            .bind(id.to_string())
            .fetch_all(self.pool)
            .await?;
        Ok(answers)
    }

    async fn update_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "
UPDATE polls
SET cron = $1, question = $2, multiselect = $3, guild = $4, channel = $5, duration = $6, onetime = $7, sent = $8
WHERE id = $9",
        )
        .bind(p.cron.clone())
        .bind(p.question.clone())
        .bind(p.multiselect)
        .bind(p.guild.clone())
        .bind(p.channel.clone())
        .bind(p.duration)
        .bind(p.onetime.clone())
        .bind(p.sent.clone())
        .bind(p.id.to_string())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Poll, Box<dyn Error>> {
        let row = sqlx::query("SELECT * FROM polls WHERE id = $1 LIMIT 1")
            .bind(id.to_string())
            .fetch_one(self.pool)
            .await?;

        let answers = self.find_answers(id).await?;

        Ok(Poll {
            id,
            cron: row.try_get(1)?,
            question: row.try_get(2)?,
            multiselect: row.try_get(3)?,
            guild: row.try_get(4)?,
            channel: row.try_get(5)?,
            answers: answers.iter().map(|item| item.answer.clone()).collect(),
            duration: row.try_get(6)?,
            onetime: row.try_get(7)?,
            sent: row.try_get(8)?,
        })
    }

    pub async fn get_all(&self) -> Result<Vec<Poll>, Box<dyn Error>> {
        let mut polls: Vec<Poll> = Vec::new();

        let mut rows = sqlx::query("SELECT * FROM polls").fetch(self.pool);

        while let Some(row) = rows.try_next().await? {
            let id: String = row.try_get(0)?;
            let parsed_uuid = Uuid::parse_str(id.as_str())?;

            let answers = self
                .find_answers(parsed_uuid)
                .await?
                .iter()
                .map(|item| item.answer.clone())
                .collect();

            polls.push(Poll {
                id: parsed_uuid,
                cron: row.try_get(1)?,
                question: row.try_get(2)?,
                multiselect: row.try_get(3)?,
                guild: row.try_get(4)?,
                channel: row.try_get(5)?,
                answers,
                duration: row.try_get(6)?,
                onetime: row.try_get(7)?,
                sent: row.try_get(8)?,
            });
        }

        Ok(polls)
    }

    pub async fn delete_poll(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let found = self.poll_exists(id).await?;
        if !found {
            return Err("not found")?;
        }

        sqlx::query("DELETE FROM polls WHERE id = $1")
            .bind(id.to_string())
            .execute(self.pool)
            .await?;

        Ok(())
    }

    async fn delete_answer(&self, answer: String, id: Uuid) -> Result<(), Box<dyn Error>> {
        sqlx::query("DELETE FROM answers WHERE poll_id = $1 AND answer = $2")
            .bind(id.to_string())
            .bind(answer.clone())
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

impl<'a> PollInstanceRepository<'a> {
    pub async fn save(&self, i: PollInstance) -> Result<(), Box<dyn Error>> {
        let exists = self.exists(i.id).await;

        if !exists {
            self.create(&i).await?;
        } else {
            self.update_votes(&i).await?;
        }

        Ok(())
    }

    pub async fn find(&self, id: i64) -> Result<PollInstance, Box<dyn Error>> {
        let row = sqlx::query("SELECT * FROM poll_instances WHERE id = $1")
            .bind(id)
            .fetch_one(self.pool)
            .await?;

        let poll_id: String = row.try_get(2)?;
        let poll_uuid = Uuid::parse_str(poll_id.as_str()).unwrap();
        let poll = self.poll_repository.find_by_id(poll_uuid).await?;

        let mut instance = PollInstance {
            id: row.try_get(0)?,
            sent_at: row.try_get(1)?,
            answers: Vec::new(),
            poll,
        };

        instance.answers = self.find_answers(id).await?;

        Ok(instance)
    }

    pub async fn find_by_poll(&self, id: Uuid) -> Result<Vec<PollInstance>, Box<dyn Error>> {
        let poll = self.poll_repository.find_by_id(id).await?;

        let mut rows = sqlx::query("SELECT * FROM poll_instances WHERE poll_id = $1")
            .bind(poll.id.to_string())
            .fetch(self.pool);

        let mut instances: Vec<PollInstance> = Vec::new();

        while let Some(row) = rows.try_next().await? {
            let mut instance = PollInstance {
                id: row.try_get(0)?,
                sent_at: row.try_get(1)?,
                answers: Vec::new(),
                poll: poll.clone(),
            };
            instance.answers = self.find_answers(instance.id).await?;
            instances.push(instance)
        }

        Ok(instances)
    }

    async fn exists(&self, id: i64) -> bool {
        let row = sqlx::query("SELECT id FROM poll_instances WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
            .unwrap_or(None);
        row.is_some()
    }

    async fn create(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        self.create_instance(i).await?;

        for answer in &i.answers {
            self.create_answer(answer, i.id).await?;
        }

        Ok(())
    }

    async fn create_instance(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        sqlx::query("INSERT INTO poll_instances (id, sent_at, poll_id) VALUES ($1, $2, $3)")
            .bind(i.id)
            .bind(i.sent_at)
            .bind(i.poll.id.to_string())
            .execute(self.pool)
            .await?;

        Ok(())
    }

    async fn create_answer(
        &self,
        a: &PollInstanceAnswer,
        instance: i64,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query("INSERT INTO poll_instance_answers (id, votes, answer, instance_id) VALUES ($1, $2, $3, $4)")
            .bind(a.discord_answer_id)
            .bind(a.votes)
            .bind(a.answer.clone())
            .bind(instance)
            .execute(self.pool).await?;

        Ok(())
    }

    async fn update_votes(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        for answer in &i.answers {
            sqlx::query(
                "UPDATE poll_instance_answers SET votes = $1 WHERE id = $2 AND instance_id = $3",
            )
            .bind(answer.votes)
            .bind(answer.discord_answer_id)
            .bind(i.id)
            .execute(self.pool)
            .await?;
        }

        Ok(())
    }

    async fn find_answers(&self, id: i64) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let mut rows = sqlx::query(
            "SELECT id, votes, answer FROM poll_instance_answers WHERE instance_id = $1",
        )
        .bind(id)
        .fetch(self.pool);

        let mut answers: Vec<PollInstanceAnswer> = Vec::new();
        while let Some(row) = rows.try_next().await? {
            answers.push(PollInstanceAnswer {
                discord_answer_id: row.try_get(0)?,
                votes: row.try_get(1)?,
                answer: row.try_get(2)?,
            })
        }

        Ok(answers)
    }

    pub async fn find_answers_by_poll_id(&self, id: Uuid) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let mut poll_instance_answers: Vec<PollInstanceAnswer> = Vec::new();

        let mut rows = sqlx::query("
            SELECT votes, answer, pia.id
            FROM poll_instances pi
            LEFT JOIN poll_instance_answers pia ON pia.instance_id = pi.id
            WHERE poll_id = $1
        ")
            .bind(id.to_string())
            .fetch(self.pool);

        while let Some(row) = rows.try_next().await? {
            poll_instance_answers.push(PollInstanceAnswer {
                votes: row.try_get(0)?,
                answer: row.try_get(1)?,
                discord_answer_id: row.try_get(2)?,
            });
        }

        Ok(poll_instance_answers)
    }
}
