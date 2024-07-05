use axum::{
    http::StatusCode, response::{IntoResponse, Response}, Json
};
use uchat_endpoint::RequestFailed;


pub type ApiResult<T> = Result<T, ApiError>;
pub struct ApiError {
    pub code: Option<StatusCode>,
    pub error: anyhow::Error
}

pub fn error_response<T: Into<String>>(
    code: StatusCode,
    msg: T
) -> Response {
    (code, Json(
        RequestFailed {
            msg: msg.into()
        }
    )).into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Some(code) = self.code {
            return error_response(code, format!("{}", self.error));
        }
        
        error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error"))
    }
}

impl<E> From<E> for ApiError 
where E: Into<anyhow::Error>
{
    fn from(error: E) -> Self {
        Self {
            code: None,
            error: error.into()
        }
    }
}