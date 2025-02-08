use axum::{
    Json,
    http::StatusCode,
    extract::Path,
    response::IntoResponse,
};

use crate::api::dto::{
    CreatePoll,
    Poll,
    UpdatePoll,
};

use uuid::Uuid;

pub async fn create_poll(
    Json(payload): Json<CreatePoll>, 
) -> StatusCode {
    println!("{:?}", payload);
    StatusCode::CREATED
}

pub async fn get_polls() -> Result<Json<Vec<Poll>>, StatusCode> {
    let polls: Vec<Poll> = vec!();
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
