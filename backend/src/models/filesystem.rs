use serde::{Deserialize, Serialize};
use uuid;

#[derive(Deserialize)]
pub struct NewFolder {
    pub name: String,
    pub parent_id: Option<uuid::Uuid>,
}

#[derive(Serialize)]
pub struct Folder {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ListFoldersParams {
    pub parent_id: Option<uuid::Uuid>,
}

#[derive(Deserialize)]
pub struct DeleteFolder {
    pub id: uuid::Uuid,
}

#[derive(Deserialize)]
pub struct RenameFolder {
    pub id: uuid::Uuid,
    pub name: String,
}

pub struct NewFile {
    pub data: Vec<u8>,
    pub name: String,
    pub mime_type: String,
    pub parent_id: Option<uuid::Uuid>,
}

impl NewFile {
    pub async fn from_multipart(
        mut multipart: axum::extract::Multipart,
    ) -> Result<Self, (axum::http::StatusCode, String)> {
        use axum::http::StatusCode;

        let mut file_data: Option<Vec<u8>> = None;
        let mut file_name: Option<String> = None;
        let mut mime_type: Option<String> = None;
        let mut parent_id: Option<uuid::Uuid> = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            (StatusCode::BAD_REQUEST, format!("Multipart error: {e}"))
        })? {
            match field.name() {
                Some("file") => {
                    file_name = field.file_name().map(|s| s.to_string());
                    mime_type = field.content_type().map(|s| s.to_string());
                    file_data = Some(
                        field.bytes().await.map_err(|e| {
                            (StatusCode::BAD_REQUEST, format!("Failed to read file: {e}"))
                        })?.to_vec(),
                    );
                }
                Some("parent_id") => {
                    let text = field.text().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Invalid field: {e}"))
                    })?;
                    if !text.is_empty() {
                        parent_id = Some(text.parse().map_err(|_| {
                            (StatusCode::BAD_REQUEST, "Invalid parent_id".to_string())
                        })?);
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            data: file_data.ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?,
            name: file_name.unwrap_or_else(|| "unnamed".to_string()),
            mime_type: mime_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            parent_id,
        })
    }
}
