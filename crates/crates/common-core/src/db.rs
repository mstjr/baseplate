pub async fn init(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::Pool::<sqlx::Postgres>::connect(database_url).await
}
