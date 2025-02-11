use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // Charger les variables d'environnement
    dotenv().ok();

    // Adresse de l'application
    let addr = env::var("APP_ADDR").unwrap_or_else(|_| String::from("127.0.0.1:3000"));
    
    // Configurer les routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/login", post(login_handler));

    // Lancer le serveur
    println!("Server running on http://{}", addr);
    axum::Server::bind(&addr.parse::<SocketAddr>().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Bienvenue sur le backend de rédaction automatique de documents !"
}

async fn login_handler() -> &'static str {
    // Logique de login ici
    "Connexion en cours..."
}
