use super::entity::{AppState, User};
use crate::result::{ApiResult, ResultType};
use axum::{
    body::Body, extract::State, http::Request, middleware::Next, response::IntoResponse, Json,
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
) -> ApiResult<()> {
    if login_info.username != "test" || login_info.password != "test" {
        cookies.remove(Cookie::from(AUTH_TOKEN));
        return ApiResult::failed(ResultType::IncorrectCrecidentials, None);
    }

    // query the user from DB
    let user = User {
        uid: "123".to_string(),
        username: login_info.username.clone(),
        pwd_hash: "test".to_string(),
    };

    let mut cookie = Cookie::new(AUTH_TOKEN, user.uid.clone());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    info!("new user logged in: {}", user.uid);

    state.put_user(user);

    ApiResult::succeeded(None)
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
        Some(user) => request.extensions_mut().insert(user),
        _ => {
            error!("user not authorized!");
            return ApiResult::<()>::failed(ResultType::NotAuthenticated, None).into_response();
        }
    };

    next.run(request).await
}
