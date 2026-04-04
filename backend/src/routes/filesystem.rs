use crate::AppData;
use crate::models::auth::LoggedInUser;
use crate::models::filesystem::{
    DeleteFolder, FileEntry, Folder, ListParams, NewFile, NewFolder, RenameFolder, ReorderRequest,
};
use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};
use sqlx::Row;
use uuid::Uuid;

pub async fn create_folder(
    State(app): State<AppData>,
    user: LoggedInUser,
    Json(payload): Json<NewFolder>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // COALESCE handles empty folders where MAX(sort_order) returns NULL -> defaults to 0, then +1
    let row = sqlx::query(
        "INSERT INTO filesystem (name, type, owner_username, parent_id, sort_order)
         VALUES ($1, 'folder', $2, $3,
            COALESCE((SELECT MAX(sort_order) FROM filesystem WHERE parent_id IS NOT DISTINCT FROM $3 AND owner_username = $2), 0) + 1
         ) ON CONFLICT (parent_id, name) DO NOTHING"
    )
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
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT id, name FROM filesystem WHERE owner_username = $1 AND type = 'folder' AND parent_id IS NOT DISTINCT FROM $2 ORDER BY sort_order"
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

    let s3_key = Uuid::new_v4().to_string();

    app.s3
        .put_object()
        .bucket(&app.bucket)
        .key(&s3_key)
        .body(payload.data.clone().into())
        .content_type(&payload.mime_type)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("S3 upload error: {e:?}"),
            )
        })?;

    let mut tx = app.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    let f_row = sqlx::query("INSERT INTO files (owner_username, s3_fileid, size_bytes, mime_type) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(&user.username)
        .bind(&s3_key)
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

    // COALESCE handles empty folders where MAX(sort_order) returns NULL -> defaults to 0, then +1
    let fs_row = sqlx::query(
        "INSERT INTO filesystem (name, type, owner_username, parent_id, file_id, sort_order)
         VALUES ($1, 'file_link', $2, $3, $4,
            COALESCE((SELECT MAX(sort_order) FROM filesystem WHERE parent_id IS NOT DISTINCT FROM $3 AND owner_username = $2), 0) + 1
         )"
    )
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

pub async fn list_files(
    State(app): State<AppData>,
    user: LoggedInUser,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT fs.id, fs.name, f.size_bytes, f.mime_type 
              FROM filesystem fs 
              JOIN files f ON fs.file_id = f.id 
              WHERE fs.owner_username = $1 AND fs.type = 'file_link'
              AND fs.parent_id IS NOT DISTINCT FROM $2
              ORDER BY fs.sort_order",
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

    let files: Vec<FileEntry> = rows
        .into_iter()
        .map(|row| FileEntry {
            id: row.get("id"),
            name: row.get("name"),
            size_bytes: row.get("size_bytes"),
            mime_type: row.get("mime_type"),
        })
        .collect();

    Ok(Json(files))
}

pub async fn get_file(
    State(app): State<AppData>,
    user: LoggedInUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT f.s3_fileid, f.mime_type
              FROM filesystem fs
              JOIN files f on fs.file_id = f.id
              WHERE fs.id = $1 AND fs.owner_username = $2 AND fs.type = 'file_link'",
    )
    .bind(&id)
    .bind(&user.username)
    .fetch_one(&app.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    let s3_fileid: String = row.get("s3_fileid");
    let mime_type: String = row.get("mime_type");

    let obj = app
        .s3
        .get_object()
        .bucket(&app.bucket)
        .key(s3_fileid)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("S3 error: {e:?}"),
            )
        })?;

    let data = obj.body.collect().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("S3 error: {e:?}"),
        )
    })?;

    Ok(([(CONTENT_TYPE, mime_type)], data.into_bytes()))
}

pub async fn reorder(
    State(app): State<AppData>,
    user: LoggedInUser,
    Json(payload): Json<ReorderRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = app.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    // Assign sort_order 1, 2, 3... based on the order of IDs received
    for (i, id) in payload.ids.iter().enumerate() {
        sqlx::query("UPDATE filesystem SET sort_order = $1 WHERE id = $2 AND owner_username = $3")
            .bind((i + 1) as i32)
            .bind(id)
            .bind(&user.username)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {e}"),
                )
            })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
