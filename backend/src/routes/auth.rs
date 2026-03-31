use crate::AppData;
use crate::models::auth::RegisterRequest;
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

    sqlx::query("INSERT INTO users (username, name, password) VALUES ($1, $2, $3)")
        .bind(&payload.username)
        .bind(&payload.name)
        .bind(&hashed_password)
        .execute(&app.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code().as_deref() == Some("23505") {
                    return (StatusCode::CONFLICT, "Username already exists".to_string());
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}"))
        })?;

    Ok((StatusCode::OK, "User registered successfully".to_string()))
}

fn salt_and_hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
