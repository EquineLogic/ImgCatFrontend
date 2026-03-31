use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{distr::SampleString, rng};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub username: String,
    pub token: String,
}

impl Session {
    const TOKEN_LENGTH: usize = 256;
    const EXPIRY: Duration = Duration::from_secs(60 * 60 * 24); // 24 hours

    /// Create a new session given a tx/pool and the username to create the session for
    pub async fn new<'c, E>(executor: E, username: String) -> Result<Self, crate::Error>
    where
        // method is generic over either pool or transaction executor
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let token: String = Alphanumeric.sample_string(&mut rng(), Self::TOKEN_LENGTH);

        sqlx::query("INSERT INTO sessions (username, token, expires_at) VALUES ($1, $2, $3)")
            .bind(&username)
            .bind(&token)
            .bind(Utc::now() + Self::EXPIRY)
            .execute(executor)
            .await
            .map_err(|e| format!("Failed to create session: {e}"))?;

        Ok(Self { username, token })
    }
}
