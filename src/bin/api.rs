use axum::{routing::get, Router};
use cron_poll_discord::api::handlers::{
    create_poll, delete_poll, get_answers_from_most_recent_poll, get_poll, get_poll_instances,
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
        .route(
            "/poll_answers/most-voted-from-last-poll",
            get(get_answers_from_most_recent_poll),
        )
        .with_state(pool);

    let port_api = env::var("PORT_API").expect("Expected PORT_API in the environment");
    let host = "0.0.0.0:".to_owned() + port_api.as_str();
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
