use crate::api::dto::{AnswerPoll, CreatePoll, Poll, PollInstance, PollInstanceAnswer, UpdatePoll};
use crate::poll::domain::Poll as DomainPoll;
use crate::poll::repository::{PollInstanceRepository, PollRepository};
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

    let poll_id = poll.id;

    println!("poll : {:?}", poll);
    let repo = init_repo(&pool);
    let status = match repo.save(poll).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => handle_error(e),
    };

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return status.into_response()
    }

    (status, Json(poll_id)).into_response()
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
            onetime: p.onetime,
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
        onetime: poll.onetime,
    }))
}

pub async fn get_poll_instances(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<PollInstance>>, StatusCode> {
    let instance_repo = PollInstanceRepository {
        pool: &pool,
        poll_repository: &PollRepository { pool: &pool },
    };

    let instances = match instance_repo.find_by_poll(id).await {
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
    let instance_repo = PollInstanceRepository {
        pool: &pool,
        poll_repository: &PollRepository { pool: &pool },
    };

    let instances = match instance_repo.find_by_poll(id).await {
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
        .duration(payload.duration)
        .onetime(payload.onetime);

    println!("poll : {:?}", poll);

    match init_repo(&pool).save(poll).await {
        Ok(_) => StatusCode::OK,
        Err(e) => handle_error(e),
    }
}

pub async fn get_answers_from_most_recent_poll(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<AnswerPoll>>, StatusCode> {
    let poll_instance_answers = init_repo(&pool)
        .get_most_voted_answer_from_latest_poll()
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to read file: {e}");
            Vec::new()
        });

    let mut answers: Vec<AnswerPoll> = Vec::new();

    for p in poll_instance_answers {
        answers.push(AnswerPoll { answer: p.answer });
    }

    Ok(Json(answers))
}
