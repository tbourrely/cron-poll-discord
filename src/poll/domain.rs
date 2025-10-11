use chrono::NaiveDate;
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AnswersError {
    Empty,
    NotFound,
}

impl fmt::Display for AnswersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty answers"),
            Self::NotFound => write!(f, "answer not found"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PollInstanceAnswer {
    pub answer: String,
    pub discord_answer_id: i64,
    pub votes: i32,
}

#[derive(Debug, Clone)]
pub struct PollInstance {
    pub id: i64,
    pub sent_at: i64,
    pub answers: Vec<PollInstanceAnswer>,
    pub poll_uuid: Option<Uuid>,
    pub poll: Option<Poll>,
}

#[derive(Debug, Clone)]
pub struct PollGroup {
    pub id: Uuid,
    pub created_at: Option<NaiveDate>,
    pub polls: Vec<Poll>,
}

impl PollGroup {
    pub fn new(id: Option<Uuid>) -> PollGroup {
        let mut uuid = Uuid::new_v4();
        if id.is_some() {
            uuid = id.unwrap();
        }

        PollGroup {
            id: uuid,
            created_at: None,
            polls: Vec::new(),
        }
    }

    pub fn add_poll(mut self, poll: Poll) -> Self {
        self.polls.push(poll);
        return self;
    }

    pub fn add_polls(mut self, polls: Vec<Poll>) -> Self {
        self.polls = polls;
        return self;
    }
}

#[derive(Debug, Clone)]
pub struct Poll {
    pub cron: String,
    pub id: Uuid,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
    pub guild: String,
    pub channel: String,
    pub duration: i32,
    pub onetime: bool,
    pub sent: bool,
    pub poll_group_id: Option<Uuid>,
}

impl Poll {
    pub fn new() -> Poll {
        Poll {
            cron: "".to_string(),
            id: Uuid::new_v4(),
            answers: vec![],
            question: "".to_string(),
            multiselect: false,
            guild: "".to_string(),
            channel: "".to_string(),
            duration: 0,
            onetime: false,
            sent: false,
            poll_group_id: None,
        }
    }

    pub fn cron(mut self, cron: String) -> Self {
        self.cron = cron;
        return self;
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        return self;
    }

    pub fn question(mut self, question: String) -> Self {
        self.question = question;
        return self;
    }

    pub fn answers(mut self, answers: Vec<String>) -> Self {
        self.answers = answers;
        return self;
    }

    pub fn multiselect(mut self, multiselect: bool) -> Self {
        self.multiselect = multiselect;
        return self;
    }

    pub fn guild(mut self, guild: String) -> Self {
        self.guild = guild;
        return self;
    }

    pub fn channel(mut self, channel: String) -> Self {
        self.channel = channel;
        return self;
    }

    pub fn duration(mut self, duration: i32) -> Self {
        self.duration = duration;
        return self;
    }

    pub fn onetime(mut self, onetime: bool) -> Self {
        self.onetime = onetime;
        return self;
    }

    pub fn sent(mut self, sent: bool) -> Self {
        self.sent = sent;
        return self;
    }

    pub fn poll_group_id(mut self, poll_group_id: Option<Uuid>) -> Self {
        self.poll_group_id = poll_group_id;
        return self;
    }
}

impl PollInstance {
    pub fn new(p: Poll) -> PollInstance {
        PollInstance {
            id: 0,
            sent_at: 0,
            answers: vec![],
            poll_uuid: None,
            poll: Some(p),
        }
    }

    pub fn add_vote(&mut self, vote_id: i64) -> Result<(), AnswersError> {
        if self.answers.is_empty() {
            return Err(AnswersError::Empty);
        }

        let mut found = false;
        for answer in self.answers.iter_mut() {
            if answer.discord_answer_id == vote_id {
                answer.votes = answer.votes + 1;
                found = true;
            }
        }

        if !found {
            return Err(AnswersError::NotFound);
        }

        Ok(())
    }

    pub fn remove_vote(&mut self, vote_id: i64) -> Result<(), AnswersError> {
        if self.answers.is_empty() {
            return Err(AnswersError::Empty);
        }

        let mut found = false;
        for answer in self.answers.iter_mut() {
            if answer.discord_answer_id == vote_id {
                if answer.votes - 1 > 0 {
                    answer.votes = answer.votes - 1;
                } else {
                    answer.votes = 0;
                }

                found = true;
            }
        }

        if !found {
            return Err(AnswersError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_vote() {
        let p = Poll::new();
        let mut poll = PollInstance::new(p);
        poll.answers = vec![PollInstanceAnswer {
            discord_answer_id: 0,
            answer: String::new(),
            votes: 0,
        }];

        assert_eq!(0, poll.answers[0].votes);

        let result = poll.add_vote(0);
        assert_eq!(1, poll.answers[0].votes);
        assert_eq!(true, result.is_ok())
    }

    #[test]
    fn test_add_vote_no_answers() {
        let p = Poll::new();
        let mut poll = PollInstance::new(p);
        let result = poll.add_vote(0);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_unexistant_answer() {
        let p = Poll::new();
        let mut poll = PollInstance::new(p);
        poll.answers = vec![PollInstanceAnswer {
            discord_answer_id: 0,
            answer: String::new(),
            votes: 0,
        }];

        let result = poll.add_vote(2);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_twice() {
        let p = Poll::new();
        let mut poll = PollInstance::new(p);
        poll.answers = vec![
            PollInstanceAnswer {
                discord_answer_id: 0,
                answer: String::new(),
                votes: 0,
            },
            PollInstanceAnswer {
                discord_answer_id: 1,
                answer: String::new(),
                votes: 0,
            },
            PollInstanceAnswer {
                discord_answer_id: 2,
                answer: String::new(),
                votes: 0,
            },
        ];
        let result = poll.add_vote(2);
        assert_eq!(false, result.is_err());
        assert_eq!(1, poll.answers[2].votes);

        let result = poll.add_vote(0);
        assert_eq!(false, result.is_err());
        assert_eq!(1, poll.answers[0].votes);
    }
}
