use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::fmt::Display;

pub type Result<T> = core::result::Result<T, ApiError>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
pub enum ApiError {
    IncorrectCrecidentials,
    NotAuthenticated,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let mut response = match self {
            ApiError::IncorrectCrecidentials | ApiError::NotAuthenticated => {
                StatusCode::UNAUTHORIZED
            }
            _ => StatusCode::OK,
        }
        .into_response();
        response.extensions_mut().insert(self);
        response
    }
}

// #[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
// pub enum ResultType {}

// #[derive(Clone, Debug, Serialize)]
// #[skip_serializing_none]
// pub struct ApiResult<T>
// where
//     T: Serialize + Clone + Send + Sync,
// {
//     #[serde(rename(serialize = "type"))]
//     result_type: ResultType,
//     data: T,
// }

// impl<T> ApiResult<T>
// where
//     T: Serialize + Clone + Send + Sync,
// {
//     pub fn new(result_type: ResultType, data: T) -> Self {
//         Self { result_type, data }
//     }
// }

// impl<T> Display for ApiResult<T>
// where
//     T: Serialize + Clone + Send + Sync,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.result_type)
//     }
// }
