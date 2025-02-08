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
            self.update_poll(&p)?;

            let saved_answers = self.find_answers(p.id)?;

            for answer in &p.answers {
                match saved_answers.iter().find(|item| item.answer == answer.answer) {
                    Some(_) => self.update_answer(&answer, p.id)?,
                    None => self.create_answer(&answer, p.id)?,
                };
            }

            for answer in &saved_answers {
                if let None = p.answers.iter().find(|item| item.answer == answer.answer) {
                    self.delete_answer(&answer, p.id)?;
                }
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
            "INSERT INTO polls (id, cron, question, discord_poll_id, sent) VALUES (?1, ?2, ?3, ?4, ?5)",
            (p.id.to_string(), p.cron.clone(), p.question.clone(), p.discord_poll_id, p.sent)
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

    fn find_answers(&self, id: Uuid) -> Result<Vec<PollAnswer>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT * FROM poll_answers WHERE poll_id = ?1")?;
        let mut rows = stmt.query([id.to_string()]).unwrap();
        let mut answers: Vec<PollAnswer> = Vec::new();

        while let Some(row) = rows.next()? {
            answers.push(PollAnswer{
                discord_answer_id: row.get(0)?,
                answer: row.get(1)?,
                votes: row.get(2)?,
            })
        }

        Ok(answers)
    }

    fn update_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE polls SET cron = ?1, question = ?2, sent = ?3 WHERE id = ?4",
            (
                p.cron.clone(),
                p.question.clone(),
                p.sent,
                p.id.to_string(),
            )
        )?;
        Ok(())
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

    pub fn get_all(&self) -> Result<Vec<Poll>, Box<dyn Error>> {
        let mut polls: Vec<Poll> = Vec::new();

        let mut stmt = self.conn.prepare("SELECT * FROM polls")?;
        let mut poll_rows = stmt.query([]).unwrap();

        while let Some(row) = poll_rows.next()? {
            let id: String = row.get(0)?;

            let mut stmt = self.conn.prepare("SELECT * FROM poll_answers WHERE poll_id = ?1")?;
            let mut answer_rows = stmt.query([id.clone()]).unwrap();
            let mut answers: Vec<PollAnswer> = Vec::new();

            while let Some(row) = answer_rows.next()? {
                answers.push(PollAnswer{
                    discord_answer_id: row.get(0)?,
                    answer: row.get(1)?,
                    votes: row.get(2)?,
                });
            }

            let parsed_uuid = Uuid::parse_str(id.as_str())?;
            polls.push(Poll{
                id: parsed_uuid,
                cron: row.get(1)?,
                question: row.get(2)?,
                discord_poll_id: row.get(3)?,
                sent: row.get(4)?,
                answers
            });
        }

        Ok(polls)
    }

    pub fn delete_poll(&mut self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let found = self.poll_exists(id)?;
        if !found {
            return Err("not found")?;
        }
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM poll_answers WHERE poll_id = ?1", [id.to_string()])?;
        tx.execute("DELETE FROM polls WHERE id = ?1", [id.to_string()])?;
        tx.commit()?;
        Ok(())
    }

    fn delete_answer(&self, answer: &PollAnswer, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "DELETE FROM poll_answers WHERE poll_id = ?1 AND answer = ?2",
            (id.to_string(), answer.answer.clone())
        )?;

        Ok(())
    }
}
