use rusqlite::Connection;
use std::error::Error;
use crate::poll::domain::{Poll, PollInstanceAnswer, PollInstance};
use uuid::Uuid;

pub struct PollRepository {
    pub conn: Connection
}

pub struct PollInstanceRepository {
    pub conn: Connection,
    pub poll_repository: PollRepository,
}

pub struct AnswerRow {
    pub id: i64,
    pub answer: String,
    pub poll_id: String,
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
                match saved_answers.iter().find(|item| item.answer == *answer) {
                    Some(_) => (),
                    None => self.create_answer(answer.clone(), p.id)?,
                };
            }
            for answer in &saved_answers {
                if let None = p.answers.iter().find(|item| **item == answer.answer) {
                    self.delete_answer(answer.answer.clone(), p.id)?;
                }
            }
        }

        Ok(())
    }

    fn create(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.create_poll(&p)?;

        for answer in &p.answers {
            self.create_answer(answer.clone(), p.id)?;
        }

        Ok(())
    }

    fn create_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT INTO polls (id, cron, question) VALUES (?1, ?2, ?3)",
            (p.id.to_string(), p.cron.clone(), p.question.clone())
        )?;

        Ok(())
    }

    fn create_answer(&self, a: String, poll_id: Uuid) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT INTO answers (answer, poll_id) VALUES (?1, ?2)",
            (a.clone(), poll_id.to_string())
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

    fn find_answers(&self, id: Uuid) -> Result<Vec<AnswerRow>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT * FROM answers WHERE poll_id = ?1")?;
        let mut rows = stmt.query([id.to_string()]).unwrap();
        let mut answers: Vec<AnswerRow> = Vec::new();

        while let Some(row) = rows.next()? {
            answers.push(AnswerRow{
                id: row.get(0)?,
                answer: row.get(1)?,
                poll_id: row.get(2)?,
            })
        }

        Ok(answers)
    }

    fn update_poll(&self, p: &Poll) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE polls SET cron = ?1, question = ?2 WHERE id = ?3",
            (
                p.cron.clone(),
                p.question.clone(),
                p.id.to_string(),
            )
        )?;
        Ok(())
    }

    pub fn find_by_id(&self, id: Uuid) -> Result<Poll, Box<dyn Error>> {
        println!("find poll id: {:?}", id);

        let mut stmt = self.conn.prepare("SELECT * FROM polls WHERE id = ?1")?;
        let mut poll = stmt.query_row([id.to_string()], |row| {
            let id: String = row.get(0)?;
            let parsed_id = Uuid::parse_str(id.as_str()).unwrap();
            Ok(Poll{
                id: parsed_id,
                cron: row.get(1)?,
                question: row.get(2)?,
                answers: vec![],
            })
        }).unwrap();

        poll.answers = self.find_answers(poll.id)?.iter().map(|item| {item.answer.clone()}).collect();

        Ok(poll)
    }

    pub fn get_all(&self) -> Result<Vec<Poll>, Box<dyn Error>> {
        let mut polls: Vec<Poll> = Vec::new();

        let mut stmt = self.conn.prepare("SELECT * FROM polls")?;
        let mut poll_rows = stmt.query([]).unwrap();

        while let Some(row) = poll_rows.next()? {
            let id: String = row.get(0)?;
            let parsed_uuid = Uuid::parse_str(id.as_str())?;
            let answers = self.find_answers(parsed_uuid)?.iter().map(|item| {item.answer.clone()}).collect();

            polls.push(Poll{
                id: parsed_uuid,
                cron: row.get(1)?,
                question: row.get(2)?,
                answers,
            });
        }

        Ok(polls)
    }

    pub fn delete_poll(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let found = self.poll_exists(id)?;
        if !found {
            return Err("not found")?;
        }

        // it relies on CASCADE deletion
        self.conn.execute(
            "DELETE FROM polls WHERE id = ?1",
            [ id.to_string() ]
        )?;

        Ok(())
    }

    fn delete_answer(&self, answer: String, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "DELETE FROM answers WHERE poll_id = ?1 AND answer = ?2",
            (id.to_string(), answer.clone())
        )?;

        Ok(())
    }
}

impl PollInstanceRepository {
    pub fn save(&self, i: PollInstance) -> Result<(), Box<dyn Error>> {
        let exists = self.exists(i.id);

        if !exists {
            self.create(&i)?;
        } else {
            self.update_votes(&i)?;
        }

        Ok(())
    }

    pub fn find(&self, id: u64) -> Result<PollInstance, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT * FROM poll_instances WHERE id = ?1")?;

        let mut instance = stmt.query_row([id], |row| {
            let id: String = row.get(2)?;
            let poll_id = Uuid::parse_str(id.as_str()).unwrap();
            let poll = self.poll_repository.find_by_id(poll_id).unwrap();

            Ok(PollInstance{
                id: row.get(0)?,
                sent_at: row.get(1)?,
                answers: vec![],
                poll,
            })
        }).unwrap();

        instance.answers = self.find_answers(id)?;

        Ok(instance)
    }

    fn exists(&self, id: u64) -> bool {
        let stmt = self.conn.prepare("SELECT id FROM poll_instances WHERE id = ?1");
        stmt.unwrap().query_row([id], |_| {
            Ok(true)
        }).unwrap_or(false)
    }
    
    fn create(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        self.create_instance(i)?;
        
        for answer in &i.answers {
            self.create_answer(answer, i.id)?;
        }

        Ok(())
    }

    fn create_instance(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT INTO poll_instances (id, sent_at, poll_id) VALUES (?1, ?2, ?3)",
            (i.id, i.sent_at, i.poll.id.to_string())
        )?;

        Ok(())
    }

    fn create_answer(&self, a: &PollInstanceAnswer, instance: u64) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT INTO poll_instance_answers (id, votes, answer, instance_id) VALUES (?1, ?2, ?3, ?4)",
            (a.discord_answer_id, a.votes, a.answer.clone(), instance)
        )?;

        Ok(())
    } 

    fn update_votes(&self, i: &PollInstance) -> Result<(), Box<dyn Error>> {
        for answer in &i.answers {
            self.conn.execute(
                "UPDATE poll_instance_answers SET votes = ?1 WHERE id = ?2",
                (answer.votes, answer.discord_answer_id)
            )?;
        }

        Ok(())
    }

    fn find_answers(&self, id: u64) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT * FROM poll_instance_answers WHERE instance_id = ?1")?;
        let mut rows = stmt.query([id]).unwrap();
        let mut answers: Vec<PollInstanceAnswer> = Vec::new();

        while let Some(row) = rows.next()? {
            answers.push(PollInstanceAnswer{
                discord_answer_id: row.get(0)?,
                votes: row.get(1)?,
                answer: row.get(2)?,
            })
        }

        Ok(answers)
    }
}
