use crate::api::dto::{CreatePoll, Poll, PollInstance, PollInstanceAnswer, UpdatePoll};
use crate::poll::domain::Poll as DomainPoll;
use crate::poll::poll_instance_use_cases::PollUseCases;
use axum::{extract::Path, extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

fn handle_error(e: Box<dyn Error>) -> StatusCode {
    println!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

// handlers
pub async fn create_poll(
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePoll>,
) -> impl IntoResponse {
    // TODO: input validation
    println!("payload : {:?}", payload);

    let poll = DomainPoll::new()
        .cron(payload.cron)
        .question(payload.question)
        .answers(payload.answers)
        .multiselect(payload.multiselect)
        .guild(payload.guild)
        .channel(payload.channel)
        .duration(payload.duration)
        .onetime(payload.onetime);

    println!("poll : {:?}", poll);
    let poll_use_cases = PollUseCases::new(&pool);
    let (id, status) = match poll_use_cases.save_poll(poll).await {
        Ok(id) => (Some(id), StatusCode::CREATED),
        Err(e) => (None, handle_error(e)),
    };

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return status.into_response();
    }

    (status, Json(id)).into_response()
}

pub async fn get_polls(State(pool): State<PgPool>) -> Result<Json<Vec<Poll>>, StatusCode> {
    let mut polls: Vec<Poll> = Vec::new();
    let poll_use_cases = PollUseCases::new(&pool);
    let db_polls = poll_use_cases.get_polls().await.unwrap_or_else(|e| {
        eprintln!("failed to read file: {e}");
        Vec::new()
    });

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
            onetime: p.onetime,
        });
    }

    Ok(Json(polls))
}

pub async fn get_poll(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Poll>, StatusCode> {
    let poll_use_cases = PollUseCases::new(&pool);
    let poll = match poll_use_cases.get_poll_by_id(id).await {
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
        onetime: poll.onetime,
    }))
}

pub async fn get_poll_instances(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<PollInstance>>, StatusCode> {
    let poll_use_cases = PollUseCases::new(&pool);

    let instances = match poll_use_cases.get_poll_instances_by_poll_id(id).await {
        Ok(v) => v,
        Err(e) => return Err(handle_error(e)),
    };

    let mut results: Vec<PollInstance> = Vec::new();
    for i in instances {
        results.push(PollInstance {
            answers: i
                .answers
                .iter()
                .map(|a| {
                    return PollInstanceAnswer {
                        answer: a.answer.clone(),
                        votes: a.votes,
                    };
                })
                .collect(),
            id: i.id,
            sent_at: i.sent_at,
        });
    }

    Ok(Json(results))
}

pub async fn get_poll_instance(
    Path((id, instance)): Path<(Uuid, i64)>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let poll_use_cases = PollUseCases::new(&pool);

    let instances = match poll_use_cases.get_poll_instances_by_poll_id(id).await {
        Ok(v) => v,
        Err(e) => return Err(handle_error(e)),
    };

    for i in instances {
        if i.id == instance {
            return Ok(Json(PollInstance {
                answers: i
                    .answers
                    .iter()
                    .map(|a| {
                        return PollInstanceAnswer {
                            answer: a.answer.clone(),
                            votes: a.votes,
                        };
                    })
                    .collect(),
                id: i.id,
                sent_at: i.sent_at,
            }));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn delete_poll(Path(id): Path<Uuid>, State(pool): State<PgPool>) -> impl IntoResponse {
    let poll_use_cases = PollUseCases::new(&pool);
    match poll_use_cases.delete_poll_by_id(id).await {
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
        .duration(payload.duration)
        .onetime(payload.onetime);

    println!("poll : {:?}", poll);

    let poll_use_cases = PollUseCases::new(&pool);
    match poll_use_cases.save_poll(poll).await {
        Ok(_) => StatusCode::OK,
        Err(e) => handle_error(e),
    }
}

pub async fn get_answers_from_poll(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<PollInstanceAnswer>>, StatusCode> {
    let poll_use_cases = PollUseCases::new(&pool);
    let poll_instance_answers = poll_use_cases
        .get_poll_instance_answers_from_poll_id(id)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to read file: {e}");
            Vec::new()
        });

    let mut answers: Vec<PollInstanceAnswer> = Vec::new();

    for p in poll_instance_answers {
        answers.push(PollInstanceAnswer {
            answer: p.answer,
            votes: p.votes,
        });
    }

    Ok(Json(answers))
}
