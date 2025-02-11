use auto_doc_backend::{database, routes::user};
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    //Configuration du login
    env_logger::init();
    let db: sqlx::Pool<sqlx::Postgres> = database::connect().await;
    println!("✅ Database connected!");

    let user_routes = user::routes(db.clone());
    let app = Router::new().nest("/api", user_routes);

    let app = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .with_state(pool)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Serveur lancé sur http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/* async fn root() -> &'static str {
    "🚀 AutoDoc Backend is Running!"
}
 */