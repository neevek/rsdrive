use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize)]
#[skip_serializing_none]
pub struct ApiResult<T>
where
    T: Serialize + Clone + Send + Sync,
{
    #[serde(rename(serialize = "type"))]
    result_type: ResultType,
    msg: Option<String>,
    data: Option<T>,
}

impl<T> ApiResult<T>
where
    T: Serialize + Clone + Send + Sync,
{
    pub fn succeeded(data: Option<T>) -> Self {
        Self {
            result_type: ResultType::Succeeded,
            msg: None,
            data,
        }
    }

    pub fn failed(result_type: ResultType, msg: Option<String>) -> Self {
        Self {
            result_type,
            msg,
            data: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
pub enum ResultType {
    Succeeded,
    IncorrectCrecidentials,
    NotAuthenticated,
}

impl<T> IntoResponse for ApiResult<T>
where
    T: Serialize + Clone + Send + Sync,
{
    fn into_response(self) -> axum::response::Response {
        let mut response = match self.result_type {
            ResultType::Succeeded => StatusCode::OK,
            ResultType::IncorrectCrecidentials | ResultType::NotAuthenticated => {
                StatusCode::UNAUTHORIZED
            }
        }
        .into_response();
        response.extensions_mut().insert(self.result_type);
        response
    }
}

impl<T> Display for ApiResult<T>
where
    T: Serialize + Clone + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: {}",
            self.result_type,
            self.msg.clone().unwrap_or("".to_string())
        )
    }
}
