use crate::{
    error::{ApiError, ApiResult},
    extractor::{DbConnection, UserSession},
    AppState,
};
use axum::{
    async_trait,
    body::Body,
    extract::{Path, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use core::fmt::Debug;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;
use uchat_query::ImageId;
use uuid::Uuid;

pub mod post;
pub mod user;

const USER_CONTEND_DIR: &str = "usercontent";
#[async_trait]
pub trait PublicApiRequest {
    type Response: IntoResponse;
    async fn process_request(
        self,
        conn: DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response>;
}

pub async fn with_public_handler<'a, Req>(
    conn: DbConnection,
    State(state): State<AppState>,
    Json(payload): Json<Req>,
) -> ApiResult<Req::Response>
where
    Req: PublicApiRequest + Deserialize<'a>,
{
    payload.process_request(conn, state).await
}

#[async_trait]
pub trait AuthorizedApiRequest {
    type Response: IntoResponse;
    async fn process_request(
        self,
        conn: DbConnection,
        session: UserSession,
        state: AppState,
    ) -> ApiResult<Self::Response>;
}

pub async fn with_handler<'a, Req>(
    conn: DbConnection,
    session: UserSession,
    State(state): State<AppState>,
    Json(payload): Json<Req>,
) -> ApiResult<Req::Response>
where
    Req: AuthorizedApiRequest + Deserialize<'a> + Debug,
{
    payload.process_request(conn, session, state).await
}

pub async fn save_image<T: AsRef<[u8]>>(id: ImageId, data: T) -> Result<(), ApiError> {
    let mut path = PathBuf::from(USER_CONTEND_DIR);
    fs::create_dir_all(&path).await?;
    path.push(id.to_string());
    fs::write(&path, data).await?;

    Ok(())
}

#[tracing::instrument(name = "Getting image from server", skip_all)]
pub async fn load_image(Path(img_id): Path<Uuid>) -> Result<Response<Body>, ApiError> {
    let mut path = PathBuf::from(USER_CONTEND_DIR);
    path.push(img_id.to_string());
    tracing::info!("Reading image...");
    // Attempt to read the image file
    let raw = fs::read_to_string(path).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::Error::msg(format!("Failed to read image file: {}", e)),
        )
    })?;

    // Split the data into header and image data
    let (header, image_data) = raw.split_once(",").ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            anyhow::Error::msg("Invalid image format"),
        )
    })?;

    // Extract the MIME type from the header
    let mime = header
        .split_once("data:")
        .and_then(|(_, mime)| mime.split_once(";base64"))
        .map(|(mime, _)| mime)
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                anyhow::Error::msg("Invalid MIME type format"),
            )
        })?;

    // Decode the base64 image data
    tracing::info!("Decoding base64 image data");
    let image_data = general_purpose::STANDARD.decode(image_data).map_err(|e| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            anyhow::Error::msg(format!("Failed to decode base64 image data: {}", e)),
        )
    })?;

    // Build and return the HTTP response
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, mime)
        .body(Body::from(image_data))
        .map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow::Error::msg(format!("Failed to build response: {}", e)),
            )
        })
}
