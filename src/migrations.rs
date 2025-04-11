use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn init_db(url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
