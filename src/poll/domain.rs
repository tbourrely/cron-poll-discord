use std::fmt;

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

pub struct PollAnswerCount {
    pub id: u64,
    pub answer: String,
    pub votes: u64,
}

pub struct Poll {
    pub id: u64,
    pub cron: String,
    pub question: String,
    pub answers: Vec<PollAnswerCount>,
}

impl Poll {
    pub fn add_vote(&mut self, vote_id: u64) -> Result<(), AnswersError> {
        if self.answers.is_empty() {
            return Err(AnswersError::Empty);
        }

        let mut found = false;
        for answer in self.answers.iter_mut() {
            if answer.id == vote_id {
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
            if answer.id == vote_id {
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
        let mut poll = Poll{
            id: 0,
            cron: String::new(),
            question: String::new(),
            answers: vec![PollAnswerCount{id: 0, answer: String::new(), votes: 0}]
        };
        assert_eq!(0, poll.answers[0].votes);
        let result = poll.add_vote(0);
        assert_eq!(1, poll.answers[0].votes);
        assert_eq!(true, result.is_ok())
    }

    #[test]
    fn test_add_vote_no_answers() {
        let mut poll = Poll{
            id: 0,
            cron: String::new(),
            question: String::new(),
            answers: vec![]
        };
        let result = poll.add_vote(0);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_unexistant_answer() {
        let mut poll = Poll{
            id: 0,
            cron: String::new(),
            question: String::new(),
            answers: vec![PollAnswerCount{id: 0, answer: String::new(), votes: 0}]
        };
        let result = poll.add_vote(2);
        assert_eq!(true, result.is_err())
    }

    #[test]
    fn test_add_vote_twice() {
        let mut poll = Poll{
            id: 0,
            cron: String::new(),
            question: String::new(),
            answers: vec![
                PollAnswerCount{id: 0, answer: String::new(), votes: 0},
                PollAnswerCount{id: 1, answer: String::new(), votes: 0},
                PollAnswerCount{id: 2, answer: String::new(), votes: 0},
            ]
        };
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
            .field("question", &self.question)
            .field("answers", &self.answers)
            .finish()
    }
}

impl fmt::Debug for PollAnswerCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollAnswerCount")
            .field("id", &self.id)
            .field("answer", &self.answer)
            .field("votes", &self.votes)
            .finish()
    }
}
