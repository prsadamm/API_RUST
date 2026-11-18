use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Identifiants invalides")]
    InvalidCredentials,

    #[error("Token manquant ou invalide")]
    Unauthorized,

    #[error("Accès interdit")]
    Forbidden,

    #[error("Email déjà utilisé")]
    EmailTaken,

    #[error("Ressource introuvable")]
    NotFound,

    #[error("Requête invalide")]
    BadRequest,

    #[error("Erreur interne")]
    Internal,
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidCredentials | AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::EmailTaken => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
             AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        eprintln!("Erreur base de données : {e}");
        AppError::Internal
    }
}