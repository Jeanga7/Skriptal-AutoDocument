use auto_doc_backend::{database, routes};
use axum::{extract::State, routing::get, Router};


#[tokio::main]
async fn main() {
    // Charge la configuration à partir du fichier .env
    dotenv::dotenv().ok();

    // Connexion à la base de données
    let db = database::connect().await;
    println!("✅ Database connected!");

    // Créer les routes
    let user_routes = routes::auth::routes(db.clone());

    // Définir les routes pour l'application
    let app = Router::new()
        .route("/", get(root))
        .nest("/api", user_routes)
        .with_state(db);

    // Démarrer le serveur
    println!("🚀 Starting server on http://127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Route d'accueil
async fn root(State(_db): State<sqlx::PgPool>) -> &'static str {
    "Welcome to the API! 🚀"
}
