use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::{get, post},
    Router,
};

use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer}, LatencyUnit,
};
use tracing::Level;
use uchat_endpoint::{user::endpoint::{CreateUser, Login}, Endpoint};

use crate::{handler::with_public_handler, AppState};

pub async fn new_router(state: AppState) -> Router {

    let public_router = Router::new()
        .route("/", get(move || async { "This is a route page" }))
        .route(CreateUser::URL, post(with_public_handler::<CreateUser>))
        .route(Login::URL, post(with_public_handler::<Login>));


    let authorized_router = Router::new();
    Router::new()
        .merge(public_router)
        .merge(authorized_router)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                        .on_response(
                            DefaultOnResponse::new().level(Level::DEBUG)
                            .latency_unit(LatencyUnit::Micros)
                            .include_headers(true),
                        ), 
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
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
