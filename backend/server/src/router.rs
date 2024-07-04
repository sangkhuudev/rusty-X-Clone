use axum::{routing::get, Router};

use crate::AppState;



pub async fn new_router(state: AppState) -> Router {
    let public_router = Router::new()
        .route("/", get(move || async {"This is a route page"}));
    let authorized_router = Router::new();
    Router::new()
        .merge(public_router)
        .merge(authorized_router)
}