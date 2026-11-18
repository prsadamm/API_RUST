mod auth;
mod error;
mod handlers;
mod jwt;
mod models;
mod password;
mod state;
mod notes;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET manquant");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("connexion à PostgreSQL impossible : le serveur est-il lancé ?");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id BIGSERIAL PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(&db)
    .await
    .expect("création de la table users");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(&db)
    .await
    .expect("création de la table notes");

    let state = AppState { db, jwt_secret };

    let app = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/me", get(handlers::me))
        .route("/admin/users", get(handlers::list_users))
        .route("/notes", post(notes::create_note).get(notes::list_notes))
        .route("/notes/{id}", delete(notes::delete_note))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("impossible d'écouter sur le port 3000 (déjà utilisé ?)");

    println!("Serveur démarré sur http://localhost:3000");

    axum::serve(listener, app).await.expect("erreur du serveur");
}