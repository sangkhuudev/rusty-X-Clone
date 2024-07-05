use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::AppState;

pub async fn new_router(state: AppState) -> Router {
    let public_router = Router::new().route("/", get(move || async { "This is a route page" }));
    let authorized_router = Router::new();
    Router::new()
        .merge(public_router)
        .merge(authorized_router)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)), // .latency_unit(LatencyUnit::Micros)
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
                        .allow_credentials(true)
                        .allow_origin(
                            std::env::var("FRONTEND_URL")
                                .unwrap()
                                .parse::<HeaderValue>()
                                .unwrap(),
                        )
                        .allow_headers([CONTENT_TYPE]),
                )
                .layer(axum::Extension(state.clone())),
        )
        .with_state(state)
}
