use std::env;
use dotenv::dotenv;
use cron_poll_discord::migrations::init_db;
use axum::{
    routing::get,
    Router,
};
use cron_poll_discord::api::handlers::{
    create_poll,
    get_polls,
    get_poll,
    delete_poll,
    update_poll,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE").expect("Expected DATABASE in the environment");
    init_db(&database).ok();

    let app = Router::new()
        .route(
            "/polls",
            get(get_polls)
                .post(create_poll)
        )
        .route(
            "/polls/{id}",
            get(get_poll)
                .delete(delete_poll)
                .put(update_poll)
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
