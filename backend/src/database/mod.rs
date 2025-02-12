pub mod schema;

use sqlx::PgPool;

pub async fn connect() -> PgPool {
    // Recuperation de la variable d'environnement DATABASE_URL(url postgres)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // Connexion à la base de données
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database")
}
