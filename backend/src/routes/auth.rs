use crate::AppData;
use crate::models::auth::{RegisterRequest, Session};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use axum::{Json, extract::State, response::IntoResponse};
use sqlx;

pub async fn register(
    State(app): State<AppData>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let hashed_password = salt_and_hash_password(&payload.password);

    let mut tx = app.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    let res = sqlx::query("INSERT INTO users (username, name, password) VALUES ($1, $2, $3) ON CONFLICT (username) DO NOTHING")
        .bind(&payload.username)
        .bind(&payload.name)
        .bind(&hashed_password)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}")))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "Username already exists".to_string()));
    }

    let session = Session::new(&mut *tx, payload.username)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create session: {e}"),
            )
        })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    Ok((StatusCode::CREATED, Json(session)))
}

fn salt_and_hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
