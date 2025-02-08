use axum::{
    Json,
    http::StatusCode,
    extract::Path,
    response::IntoResponse,
};
use uuid::Uuid;
use rusqlite::Connection;
use std::env;
use crate::api::dto::{
    CreatePoll,
    Poll,
    UpdatePoll,
};
use crate::poll::repository::PollRepository;
use crate::poll::domain::{Poll as DomainPoll, PollAnswer as DomainPollAnswer};
use std::error::Error;

// utils
pub fn init_repo() -> PollRepository {
    // TODO: use another library that supports
    // async instead of rusqlite
    let database = env::var("DATABASE").expect("Expected DATABASE in the environment");
    let conn = Connection::open(database.to_string()).unwrap();
    PollRepository{ conn }
}

fn handle_error(e: Box<dyn Error>) -> StatusCode {
    println!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

// handlers
pub async fn create_poll(
    Json(payload): Json<CreatePoll>, 
) -> StatusCode {
    // TODO: input validation
    println!("payload : {:?}", payload);
    let mut answers: Vec<DomainPollAnswer> = vec![];
    for answer in payload.answers {
        answers.push(DomainPollAnswer{
            discord_answer_id: 0,
            votes: 0,
            answer
        })
    }
    let poll = DomainPoll{
        id: Uuid::new_v4(),
        discord_poll_id: 0,
        cron: payload.cron,
        question: payload.question,
        answers,
        sent: false
    };
    println!("poll : {:?}", poll);
    let repo = init_repo();
    let status = match repo.save(poll) {
        Ok(_) => StatusCode::CREATED,
        Err(e) =>  handle_error(e)
    };

    return status;
}

pub async fn get_polls() -> Result<Json<Vec<Poll>>, StatusCode> {
    let mut polls: Vec<Poll> = Vec::new();
    let db_polls = match init_repo().get_all() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to read file: {e}");
            Vec::new()
        }
    };

    for p in db_polls {
        let answers = p.answers.iter().map(|answer| {
            answer.answer.clone()
        }).collect();

        polls.push(Poll{
            id: p.id,
            cron: p.cron,
            question: p.question,
            answers
        });
    }
    
    Ok(Json(polls))
}

pub async fn get_poll(
    Path(id): Path<Uuid>
) -> Json<Poll> {
    Json(Poll{
        id,
        cron: "cron".to_string(),
        question: "question".to_string(),
        answers: vec![],
    })
}

pub async fn delete_poll(
    Path(_id): Path<Uuid>
) -> impl IntoResponse {
    StatusCode::OK
}

pub async fn update_poll(
    Path(_id): Path<Uuid>,
    Json(_input): Json<UpdatePoll>
) -> impl IntoResponse {
    StatusCode::OK
}
