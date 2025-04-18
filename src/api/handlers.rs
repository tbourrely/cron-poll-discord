use crate::api::dto::{CreatePoll, Poll, UpdatePoll};
use crate::poll::domain::Poll as DomainPoll;
use crate::poll::repository::PollRepository;
use axum::{extract::Path, extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

fn handle_error(e: Box<dyn Error>) -> StatusCode {
    println!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

fn init_repo(pool: &PgPool) -> PollRepository {
    PollRepository { pool: &pool }
}

// handlers
pub async fn create_poll(
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePoll>,
) -> StatusCode {
    // TODO: input validation
    println!("payload : {:?}", payload);

    let poll = DomainPoll::new()
        .cron(payload.cron)
        .question(payload.question)
        .answers(payload.answers)
        .multiselect(payload.multiselect)
        .guild(payload.guild)
        .channel(payload.channel)
        .duration(payload.duration);

    println!("poll : {:?}", poll);
    let repo = init_repo(&pool);
    let status = match repo.save(poll).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => handle_error(e),
    };

    return status;
}

pub async fn get_polls(State(pool): State<PgPool>) -> Result<Json<Vec<Poll>>, StatusCode> {
    let mut polls: Vec<Poll> = Vec::new();
    let db_polls = match init_repo(&pool).get_all().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to read file: {e}");
            Vec::new()
        }
    };

    for p in db_polls {
        polls.push(Poll {
            id: p.id,
            cron: p.cron,
            question: p.question,
            answers: p.answers,
            multiselect: p.multiselect,
            guild: p.guild,
            channel: p.channel,
            duration: p.duration,
        });
    }

    Ok(Json(polls))
}

pub async fn get_poll(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Poll>, StatusCode> {
    let poll = match init_repo(&pool).find_by_id(id).await {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(Poll {
        id: poll.id,
        cron: poll.cron,
        question: poll.question,
        answers: poll.answers,
        multiselect: poll.multiselect,
        guild: poll.guild,
        channel: poll.channel,
        duration: poll.duration,
    }))
}

pub async fn delete_poll(Path(id): Path<Uuid>, State(pool): State<PgPool>) -> impl IntoResponse {
    match init_repo(&pool).delete_poll(id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn update_poll(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdatePoll>,
) -> impl IntoResponse {
    // TODO: input validation
    println!("payload : {:?}", payload);

    let poll = DomainPoll::new()
        .id(id)
        .cron(payload.cron)
        .question(payload.question)
        .answers(payload.answers)
        .multiselect(payload.multiselect)
        .guild(payload.guild)
        .channel(payload.channel)
        .duration(payload.duration);

    println!("poll : {:?}", poll);

    match init_repo(&pool).save(poll).await {
        Ok(_) => StatusCode::OK,
        Err(e) => handle_error(e),
    }
}
