use super::entity::{AppState, User};
use crate::result::{ApiError, Result};
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

pub async fn login(
    mut state: State<AppState>,
    cookies: Cookies,
    login_info: Json<LoginInfo>,
) -> Result<Response> {
    if login_info.username != "test" || login_info.password != "test" {
        cookies.remove(Cookie::from(AUTH_TOKEN));
        return Err(ApiError::NotAuthenticated);
    }

    // query the user from DB
    let user = User::new("123", login_info.username.as_str(), "test");

    let mut cookie = Cookie::new(AUTH_TOKEN, user.uid().to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    info!("new user logged in: {}", user.uid());

    state.put_user(user);

    Ok(StatusCode::OK.into_response())
}

pub async fn user_resolver(
    state: State<AppState>,
    cookies: Cookies,
    mut request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let user = match cookies
        .get(AUTH_TOKEN)
        .map(|cookie| cookie.value().to_string())
    {
        Some(uid) => state.get_user(uid.as_str()),
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

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self> {
        parts
            .extensions
            .get::<Result<User>>()
            .ok_or(ApiError::NotAuthenticated)?
            .clone()
    }
}
