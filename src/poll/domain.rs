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
            Self::NotFound => write!(f, "answer not found")
        }
    }
}

pub struct PollInstanceAnswer {
    pub answer: String,
    pub discord_answer_id: u64,
    pub votes: u64,
}

pub struct PollInstance {
    pub id: u64,
    pub sent_at: i64,
    pub answers: Vec<PollInstanceAnswer>,
    pub poll: Poll,
}

pub struct Poll {
    pub cron: String,
    pub id: Uuid,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
}

impl Poll {

    pub fn new() -> Poll {
        Poll{
            cron: "".to_string(),
            id: Uuid::new_v4(),
            answers: vec![],
            question: "".to_string(),
            multiselect: false,
        }
    }
}

impl PollInstance {

    pub fn new() -> PollInstance {
        PollInstance{
            id: 0,
            sent_at: 0,
            answers: vec![],
            poll: Poll::new(),
        }
    }

    pub fn add_vote(&mut self, vote_id: u64) -> Result<(), AnswersError> {
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

    pub fn remove_vote(&mut self, vote_id: u64) -> Result<(), AnswersError> {
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
        let mut poll = PollInstance::new(); 
        poll.answers = vec![PollInstanceAnswer{discord_answer_id: 0, answer: String::new(), votes: 0}];

        assert_eq!(0, poll.answers[0].votes);

        let result = poll.add_vote(0);
        assert_eq!(1, poll.answers[0].votes);
        assert_eq!(true, result.is_ok())
    }

    #[test]
    fn test_add_vote_no_answers() {
        let mut poll = PollInstance::new();
        let result = poll.add_vote(0);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_unexistant_answer() {
        let mut poll = PollInstance::new();
        poll.answers = vec![PollInstanceAnswer{discord_answer_id: 0, answer: String::new(), votes: 0}];

        let result = poll.add_vote(2);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_twice() {
        let mut poll = PollInstance::new();
        poll.answers = vec![
            PollInstanceAnswer{discord_answer_id: 0, answer: String::new(), votes: 0},
            PollInstanceAnswer{discord_answer_id: 1, answer: String::new(), votes: 0},
            PollInstanceAnswer{discord_answer_id: 2, answer: String::new(), votes: 0},

        ];
        let result = poll.add_vote(2);
        assert_eq!(false, result.is_err());
        assert_eq!(1, poll.answers[2].votes);

        let result = poll.add_vote(0);
        assert_eq!(false, result.is_err());
        assert_eq!(1, poll.answers[0].votes);
    }
}

impl fmt::Debug for Poll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Poll")
            .field("id", &self.id)
            .field("cron", &self.cron)
            .field("question", &self.question)
            .field("multiselect", &self.multiselect)
            .finish()
    }
}

impl fmt::Debug for PollInstanceAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollInstanceAnswer")
            .field("discord_answer_id", &self.discord_answer_id)
            .field("answer", &self.answer)
            .field("votes", &self.votes)
            .finish()
    }
}

impl fmt::Debug for PollInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollInstance")
            .field("id", &self.id)
            .field("sent_at", &self.sent_at)
            .field("answers", &self.answers)
            .field("poll", &self.poll)
            .finish()
    }
}
