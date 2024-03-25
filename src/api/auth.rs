use super::entity::AppState;
use crate::{
    result::{ApiError, Result},
    server::entity::User,
};
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};

pub const AUTH_TOKEN: &str = "auth-token";

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

pub async fn login(mut state: State<AppState>, cookies: Cookies, login_info: Json<LoginInfo>) -> Result<Response> {
    if login_info.username != "test" || login_info.password != "test" {
        cookies.remove(Cookie::from(AUTH_TOKEN));
        return Err(ApiError::NotAuthenticated);
    }

    // query the user from DB
    let user = User {
        id: 123,
        username: login_info.username.to_string(),
        password: "password".to_string(),
        phone_number: "13800138000".to_string(),
        email: "test@gmail.com".to_string(),
        create_time: "haha".to_string(),
    };

    let db = state.get_database();
    db.save_user(&user).unwrap();
    let user = db.query_user(&user.username, &user.password).unwrap();

    let mut cookie = Cookie::new(AUTH_TOKEN, user.id.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    info!("new user logged in:{}", user.id);

    state.put_user(user);

    Ok(StatusCode::OK.into_response())
}

pub async fn user_resolver(state: State<AppState>, cookies: Cookies, mut request: Request<Body>, next: Next) -> impl IntoResponse {
    let user = match cookies.get(AUTH_TOKEN).map(|cookie| cookie.value().parse().unwrap_or(0)) {
        Some(uid) if uid > 0 => state.get_user(uid),
        _ => None,
    };

    match user {
        Some(user) => request.extensions_mut().insert(Ok::<User, ApiError>(user)),
        _ => {
            error!("user not authorized!");
            return ApiError::NotAuthenticated.into_response();
        }
    };

    next.run(request).await.into_response()
}

#[async_trait::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for User {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self> {
        parts.extensions.get::<Result<User>>().ok_or(ApiError::NotAuthenticated)?.clone()
    }
}
