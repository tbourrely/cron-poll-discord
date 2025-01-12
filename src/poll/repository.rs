use rusqlite::Connection;
use std::error::Error;
use crate::poll::domain::{Poll, PollAnswerCount};

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
            "INSERT INTO polls (id, question) VALUES (?1, ?2)",
            (p.id, &p.question)
        )?;

        Ok(())
    }

    fn create_answer(&self, a: &PollAnswerCount, poll_id: u64) -> Result<(), Box<dyn Error>> {
            self.conn.execute(
                "INSERT INTO poll_answers (id, answer, votes, poll_id) VALUES (?1, ?2, ?3, ?4)",
                (a.id, &a.answer, a.votes, poll_id)
            )?;

        Ok(())
    }

    fn poll_exists(&self, id: u64) -> Result<bool, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT id FROM polls WHERE id = ?1")?;
        let found = stmt.query_row([id], |_| {
            Ok(true)
        }).unwrap_or(false);

        Ok(found)
    }

    fn update_answer(&self, a: &PollAnswerCount, poll_id: u64) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE poll_answers SET votes = ?1 WHERE poll_id = ?2 AND id = ?3",
            (a.votes, poll_id, a.id)
        )?;

        Ok(())
    }

    pub fn find(&self, id: u64) -> Result<Poll, Box<dyn Error>> {
        println!("find poll id: {:?}", id);

        let mut stmt = self.conn.prepare("SELECT id, question FROM polls WHERE id = ?1")?;
        let mut poll = stmt.query_row([id], |row| {
            Ok(Poll{
                id: row.get(0)?,
                question: row.get(1)?,
                answers: vec![]
            })
        }).unwrap();

        let mut answers_stmt = self.conn.prepare("SELECT id, answer, votes FROM poll_answers WHERE poll_id = ?1")?;
        let found_answers = answers_stmt.query_map([id], |row| {
            Ok(PollAnswerCount{
                id: row.get(0)?,
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
