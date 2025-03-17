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

#[derive(Debug)]
pub struct PollInstanceAnswer {
    pub answer: String,
    pub discord_answer_id: u64,
    pub votes: u64,
}

#[derive(Debug)]
pub struct PollInstance {
    pub id: u64,
    pub sent_at: i64,
    pub answers: Vec<PollInstanceAnswer>,
    pub poll: Poll,
}

#[derive(Debug)]
pub struct Poll {
    pub cron: String,
    pub id: Uuid,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
    pub guild: String,
    pub channel: String,
    pub duration: u32,
}

impl Poll {

    pub fn new() -> Poll {
        Poll{
            cron: "".to_string(),
            id: Uuid::new_v4(),
            answers: vec![],
            question: "".to_string(),
            multiselect: false,
            guild: "".to_string(),
            channel: "".to_string(),
            duration: 0,
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

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        return self;
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
