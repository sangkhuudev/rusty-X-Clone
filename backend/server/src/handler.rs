use crate::{
    error::{ApiError, ApiResult},
    extractor::{DbConnection, UserSession},
    AppState,
};
use axum::{async_trait, body::Body, extract::{Path, State}, http::{header::CONTENT_TYPE, StatusCode}, response::{IntoResponse, Response}, Json};
use base64::{ Engine as _ ,engine::general_purpose};
use tokio::fs;
use uchat_query::ImageId;
use uuid::Uuid;
use core::fmt::Debug;
use std::path::PathBuf;
use serde::Deserialize;

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
    tracing::debug!("with_handler called with payload: {:?}", payload);
    payload.process_request(conn, session, state).await
}

pub async fn save_image<T: AsRef<[u8]>>(id: ImageId, data: T) -> Result<(), ApiError> {
    let mut path = PathBuf::from(USER_CONTEND_DIR);
    fs::create_dir_all(&path).await?;
    path.push(id.to_string());
    fs::write(&path, data).await?;

    Ok(())
}

pub async fn load_image(
    Path(img_id): Path<Uuid>,
) -> Result<Response<Body>, ApiError> {
    let mut path = PathBuf::from(USER_CONTEND_DIR);
    path.push(img_id.to_string());
    let raw = fs::read_to_string(path).await?;

    // Data url reference
    // data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==
    let (header, image_data) = raw.split_once(",").unwrap();
    //header=data:text/plain;base64
    // image_date=SGVsbG8sIFdvcmxkIQ==
    // mime=text/plain;base64
    let mime = header
        .split_once("data:")
        .unwrap()
        // 0: data
        // 1: text/plain;base64
        .1
        .split_once(";base64")
        .unwrap()
        // 0: text/plain
        // 1: ;base64
        .0;
    let image_data = general_purpose::STANDARD.decode(image_data).unwrap();
    
    Ok(
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, mime)
            .body(Body::from(image_data))
            .unwrap()
    )
    
}