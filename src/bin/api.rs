use axum::{routing::get, Router};
use cron_poll_discord::api::handlers::{
    create_poll, delete_poll, get_poll, get_polls, update_poll,
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
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
