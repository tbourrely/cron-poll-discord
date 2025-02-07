use axum::{
    routing::get,
    Router,
};

use cron_poll_discord::api::handlers::{
    create_poll,
    get_polls,
    get_poll,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/polls", get(get_polls).post(create_poll))
        .route("/polls/{id}", get(get_poll));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
