use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use crate::{auth::AuthUser, error::AppError, state::AppState};

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct NotePublic {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
}

pub async fn create_note(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<NotePublic>), AppError> {
    let note = sqlx::query_as::<_, NotePublic>(
        "INSERT INTO notes (user_id, title, content)
         VALUES ($1, $2, $3)
         RETURNING id, user_id, title, content",
    )
    .bind(claims.sub)
    .bind(&body.title)
    .bind(&body.content)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn list_notes(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<NotePublic>>, AppError> {
    let notes = sqlx::query_as::<_, NotePublic>(
        "SELECT id, user_id, title, content FROM notes WHERE user_id = $1 ORDER BY id",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(notes))
}

pub async fn delete_note(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}