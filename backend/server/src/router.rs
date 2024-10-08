use axum::{
    extract::DefaultBodyLimit,
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::{get, post},
    Router,
};

use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;
use uchat_endpoint::{
    post::endpoint::{
        Bookmark, BookmarkedPost, Boost, HomePost, LikedPost, NewPost, React, TrendingPost, Vote,
    },
    user::endpoint::{CreateUser, FollowUser, GetMyProfile, Login, UpdateProfile, ViewProfile},
    Endpoint,
};

use uchat_endpoint::app_url::user_content;

use crate::{
    handler::{load_image, with_handler, with_public_handler},
    AppState,
};

pub async fn new_router(state: AppState) -> Router {
    let image_route = format!("{}{}", user_content::ROOT, user_content::IMAGE);
    let public_router = Router::new()
        .route("/", get(move || async { "This is a route page" }))
        .route(&format!("/{}:id", image_route), get(load_image))
        .route(CreateUser::URL, post(with_public_handler::<CreateUser>))
        .route(Login::URL, post(with_public_handler::<Login>));

    let authorized_router = Router::new()
        .route(NewPost::URL, post(with_handler::<NewPost>))
        .route(Bookmark::URL, post(with_handler::<Bookmark>))
        .route(Boost::URL, post(with_handler::<Boost>))
        .route(Vote::URL, post(with_handler::<Vote>))
        .route(React::URL, post(with_handler::<React>))
        .route(TrendingPost::URL, post(with_handler::<TrendingPost>))
        .route(GetMyProfile::URL, post(with_handler::<GetMyProfile>))
        .route(UpdateProfile::URL, post(with_handler::<UpdateProfile>))
        .route(ViewProfile::URL, post(with_handler::<ViewProfile>))
        .route(FollowUser::URL, post(with_handler::<FollowUser>))
        .route(HomePost::URL, post(with_handler::<HomePost>))
        .route(LikedPost::URL, post(with_handler::<LikedPost>))
        .route(BookmarkedPost::URL, post(with_handler::<BookmarkedPost>))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(8 * 1024 * 1024))
        .layer(CompressionLayer::new());

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
                            DefaultOnResponse::new()
                                .level(Level::DEBUG)
                                .include_headers(true)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                        .allow_credentials(true)
                        .allow_origin(
                            std::env::var("FRONTEND_URL")
                                .unwrap()
                                .parse::<HeaderValue>()
                                .unwrap(),
                        )
                        .allow_headers(vec![CONTENT_TYPE]),
                )
                .layer(axum::Extension(state.clone())),
        )
        .with_state(state)
}
