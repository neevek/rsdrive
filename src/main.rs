use api::file;
use axum::{
    http::{Method, Request, Response, StatusCode, Uri},
    middleware,
    routing::{get, get_service, post},
    Router,
};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing::{info, Level};

use crate::api::{auth::user_resolver, entity::AppState};

mod api;
mod result;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app_state = AppState::new();
    let addr = "127.0.0.1:8080";
    let router = Router::new()
        .route("/login", post(api::auth::login))
        .nest("/api", api_router(app_state.clone()))
        .layer(CookieManagerLayer::new())
        .fallback_service(static_router())
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on {addr}");

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

fn api_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/hello", get(|| async { "hello" }))
        .route("/upload", get(file::upload_file))
        .route_layer(
            ServiceBuilder::new()
                // .layer(CookieManagerLayer::new())
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    user_resolver,
                ))
                .layer(middleware::map_request(request_interceptor))
                .layer(middleware::map_response(response_interceptor)),
        )
}

fn static_router() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

async fn request_interceptor<Body>(
    uri: Uri,
    method: Method,
    request: Request<Body>,
) -> Result<Request<Body>, StatusCode> {
    info!("--> {method} {uri}");
    Ok(request)
}

async fn response_interceptor<Body>(uri: Uri, response: Response<Body>) -> Response<Body> {
    info!("<-- {} {uri}", response.status());
    response
}
