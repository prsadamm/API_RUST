use serde::Serialize;

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub password_hash: String,
    pub role: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserPublic {
    pub id: i64,
    pub email: String,
    pub role: String,
}