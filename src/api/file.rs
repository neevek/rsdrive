use super::entity::{AppState, User};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tracing::info;

pub async fn upload_file(user: User, state: State<AppState>) -> impl IntoResponse {
    info!(">>>>>>>>>> haha user:{}", user.uid());
    StatusCode::OK
}
