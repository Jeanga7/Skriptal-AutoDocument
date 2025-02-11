pub mod schema;

use std::env;
use sqlx::{Pool, Postgres};

pub async fn connect() -> Pool<Postgres> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to connect to the database")
}

