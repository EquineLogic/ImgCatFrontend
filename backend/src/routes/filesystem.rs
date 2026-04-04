use crate::AppData;
use crate::models::auth::LoggedInUser;
use crate::models::filesystem::{
    DeleteFolder, Folder, ListFoldersParams, NewFile, NewFolder, RenameFolder,
};
use axum::{
    Json,
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::Row;

pub async fn create_folder(
    State(app): State<AppData>,
    user: LoggedInUser,
    Json(payload): Json<NewFolder>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let row = sqlx::query("INSERT INTO filesystem (name, type, owner_username, parent_id) VALUES ($1, 'folder', $2, $3) ON CONFLICT (parent_id, name) DO NOTHING")
        .bind(&payload.name)
        .bind(&user.username)
        .bind(payload.parent_id)
        .execute(&app.pool)
        .await.map_err(|e|
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Can't make folder error: {e}"))
        )?;

    // check if folder was created successfully
    if row.rows_affected() == 0 {
        return Err((
            StatusCode::CONFLICT,
            "A folder with that name already exists".to_string(),
        ));
    }

    Ok(StatusCode::CREATED)
}

pub async fn list_folders(
    State(app): State<AppData>,
    user: LoggedInUser,
    Query(params): Query<ListFoldersParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT id, name FROM filesystem WHERE owner_username = $1 AND type = 'folder' AND parent_id IS NOT DISTINCT FROM $2"
    )
    .bind(&user.username)
    .bind(params.parent_id)
    .fetch_all(&app.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    let folders: Vec<Folder> = rows
        .into_iter()
        .map(|row| Folder {
            id: row.get("id"),
            name: row.get("name"),
        })
        .collect();

    Ok(Json(folders))
}

pub async fn delete_folder(
    State(app): State<AppData>,
    user: LoggedInUser,
    Json(payload): Json<DeleteFolder>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let res = sqlx::query(
        "DELETE FROM filesystem WHERE id = $1 AND owner_username = $2 AND type = 'folder'",
    )
    .bind(payload.id)
    .bind(&user.username)
    .execute(&app.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn rename_folder(
    State(app): State<AppData>,
    user: LoggedInUser,
    Json(payload): Json<RenameFolder>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let res = sqlx::query(
        "UPDATE filesystem SET name = $1 WHERE id = $2 AND owner_username = $3 AND type = 'folder'",
    )
    .bind(&payload.name)
    .bind(payload.id)
    .bind(&user.username)
    .execute(&app.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_file(
    State(app): State<AppData>,
    user: LoggedInUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let payload = NewFile::from_multipart(multipart).await?;

    let mut tx = app.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    let f_row = sqlx::query("INSERT INTO files (owner_username, s3_fileid, size_bytes, mime_type) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(&user.username)
        .bind("placeholder_s3_id") // TODO: Upload to S3 and get real ID
        .bind(payload.data.len() as i64)
        .bind(&payload.mime_type)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            )
        })?;

    // don't have to check if rows b/c of fetch_one

    let f_id: uuid::Uuid = f_row.get("id");

    let fs_row = sqlx::query("INSERT INTO filesystem (name, type, owner_username, parent_id, file_id) VALUES ($1, 'file_link', $2, $3, $4)")
        .bind(&payload.name)
        .bind(&user.username)
        .bind(payload.parent_id)
        .bind(f_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            )
        })?;

    if fs_row.rows_affected() == 0 {
        return Err((
            StatusCode::CONFLICT,
            "A file with that name already exists".to_string(),
        ));
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    Ok(StatusCode::CREATED)
}
