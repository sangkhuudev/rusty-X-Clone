use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uchat_endpoint::RequestFailed;

pub type ApiResult<T> = Result<T, ApiError>;
pub struct ApiError {
    pub code: Option<StatusCode>,
    pub error: anyhow::Error,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Login ailed")]
    Login((StatusCode, String)),
}

impl ServerError {
    pub fn missing_login() -> Self {
        Self::Login((StatusCode::NOT_FOUND, "User not found".to_string()))
    }

    pub fn wrong_password() -> Self {
        Self::Login((StatusCode::BAD_REQUEST, "Invalid password".to_string()))
    }
}

pub fn error_response<T: Into<String>>(code: StatusCode, msg: T) -> Response {
    (code, Json(RequestFailed { msg: msg.into() })).into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Some(code) = self.code {
            return error_response(code, format!("{}", self.error));
        }

        if let Some(server_err) = self.error.downcast_ref::<ServerError>() {
            return match server_err {
                ServerError::Login((code, msg)) => error_response(*code, msg),
            };
        }

        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error"),
        )
    }
}

impl ApiError {
    pub fn new<T: Into<anyhow::Error>>(code: StatusCode, error: T) -> Self {
        Self {
            code: Some(code),
            error: error.into(),
        }
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self {
            code: None,
            error: error.into(),
        }
    }
}
