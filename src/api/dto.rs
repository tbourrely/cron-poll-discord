use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Poll {
    pub id: Uuid,
    pub cron: String,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
    pub guild: String,
    pub channel: String,
    pub duration: i32,
    pub onetime: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePoll {
    pub cron: String,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
    pub guild: String,
    pub channel: String,
    pub duration: i32,
    pub onetime: bool,
}

pub type UpdatePoll = CreatePoll;

#[derive(Deserialize, Serialize, Debug)]
pub struct PollInstance {
    pub id: i64,
    pub sent_at: i64,
    pub answers: Vec<PollInstanceAnswer>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PollInstanceAnswer {
    pub answer: String,
    pub votes: i32,
}
