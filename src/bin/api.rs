use axum::{routing::get, Router};
use cron_poll_discord::api::handlers::{
    create_poll, create_poll_in_poll_group, delete_poll, get_answers_from_poll,
    get_answers_from_poll_group, get_poll, get_poll_groups, get_poll_instance, get_poll_instances,
    get_polls, update_poll,
};
use cron_poll_discord::migrations::init_db;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE_URL").expect("Expected DATABASE in the environment");
    let pool = init_db(&database).await.unwrap();

    let app = Router::new()
        .route("/polls", get(get_polls).post(create_poll))
        .route(
            "/polls/{id}",
            get(get_poll).delete(delete_poll).put(update_poll),
        )
        .route("/polls/{id}/instances", get(get_poll_instances))
        .route("/polls/{id}/instances/{instance}", get(get_poll_instance))
        .route("/polls/{id}/instances/answers", get(get_answers_from_poll))
        .route(
            "/poll-groups",
            get(get_poll_groups).post(create_poll_in_poll_group),
        )
        .route(
            "/poll-groups/{id}/instances/answers",
            get(get_answers_from_poll_group),
        )
        .with_state(pool);

    let port_api = env::var("PORT_API").expect("Expected PORT_API in the environment");
    let host = "0.0.0.0:".to_owned() + port_api.as_str();
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
