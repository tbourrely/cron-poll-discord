use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Poll {
    pub id: Uuid,
    pub cron: String,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreatePoll {
    pub cron: String,
    pub question: String,
    pub answers: Vec<String>,
    pub multiselect: bool,
}

pub type UpdatePoll = CreatePoll;
