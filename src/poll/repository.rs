use rusqlite::Connection;
use std::error::Error;
use crate::poll::domain::{Poll, PollAnswer};
use uuid::Uuid;

pub struct PollRepository {
    #[allow(dead_code)]
    pub conn: Connection
}

impl PollRepository {
    pub fn save(&self, p: Poll) -> Result<(), Box<dyn Error>> {
        println!("saving poll : {:?}", p);

        let exists = self.poll_exists(p.id)?;

        if !exists {
            self.create(&p)?;
        } else {
            for answer in p.answers {
                self.update_answer(&answer, p.id)?;
            }
        }

        Ok(())
    }

    fn create(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.create_poll(&p)?;

        for answer in &p.answers {
            self.create_answer(&answer, p.id)?;
        }

        Ok(())
    }

    fn create_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT INTO polls (id, discord_poll_id, question, sent) VALUES (?1, ?2)",
            (p.id.to_string(), p.discord_poll_id, &p.question, p.sent)
        )?;

        Ok(())
    }

    fn create_answer(&self, a: &PollAnswer, poll_id: Uuid) -> Result<(), Box<dyn Error>> {
            self.conn.execute(
                "INSERT INTO poll_answers (discord_answer_id, answer, votes, poll_id) VALUES (?1, ?2, ?3, ?4)",
                (a.discord_answer_id, &a.answer, a.votes, poll_id.to_string())
            )?;

        Ok(())
    }

    fn poll_exists(&self, id: Uuid) -> Result<bool, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT id FROM polls WHERE id = ?1")?;
        let found = stmt.query_row([id.to_string()], |_| {
            Ok(true)
        }).unwrap_or(false);

        Ok(found)
    }

    fn update_answer(&self, a: &PollAnswer, poll_id: Uuid) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE poll_answers SET votes = ?1 WHERE poll_id = ?2 AND discord_answer_id = ?3",
            (a.votes, poll_id.to_string(), a.discord_answer_id)
        )?;

        Ok(())
    }

    pub fn find_by_discord_id(&self, id: u64) -> Result<Poll, Box<dyn Error>> {
        println!("find poll id: {:?}", id);

        let mut stmt = self.conn.prepare("SELECT * FROM polls WHERE discord_poll_id = ?1")?;
        let mut poll = stmt.query_row([id], |row| {
            let id: String = row.get(0)?;
            let parsed_id = Uuid::parse_str(id.as_str()).unwrap();
            Ok(Poll{
                id: parsed_id,
                cron: row.get(1)?, // cron value is not saved in db
                question: row.get(2)?,
                discord_poll_id: row.get(3)?,
                answers: vec![],
                sent: row.get(4)?,
            })
        }).unwrap();

        let mut answers_stmt = self.conn.prepare("SELECT discord_answer_id, answer, votes FROM poll_answers WHERE poll_id = ?1")?;
        let found_answers = answers_stmt.query_map([poll.discord_poll_id], |row| {
            Ok(PollAnswer{
                discord_answer_id: row.get(0)?,
                answer: row.get(1)?,
                votes: row.get(2)?,
            })
        }).unwrap();

        for answer in found_answers {
            poll.answers.push(answer?);
        }

        Ok(poll)
    }

    pub fn find_by_id(&self, id: Uuid) -> Result<Poll, Box<dyn Error>> {
        println!("find poll id: {:?}", id);

        let mut stmt = self.conn.prepare("SELECT * FROM polls WHERE id = ?1")?;
        let mut poll = stmt.query_row([id.to_string()], |row| {
            let id: String = row.get(0)?;
            let parsed_id = Uuid::parse_str(id.as_str()).unwrap();
            Ok(Poll{
                id: parsed_id,
                cron: row.get(1)?, // cron value is not saved in db
                question: row.get(2)?,
                discord_poll_id: row.get(3)?,
                answers: vec![],
                sent: row.get(4)?,
            })
        }).unwrap();

        let mut answers_stmt = self.conn.prepare("SELECT discord_answer_id, answer, votes FROM poll_answers WHERE poll_id = ?1")?;
        let found_answers = answers_stmt.query_map([id.to_string()], |row| {
            Ok(PollAnswer{
                discord_answer_id: row.get(0)?,
                answer: row.get(1)?,
                votes: row.get(2)?,
            })
        }).unwrap();

        for answer in found_answers {
            poll.answers.push(answer?);
        }

        Ok(poll)
    }
}
