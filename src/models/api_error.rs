use std::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Serialize};

use crate::errors::LcuDriverError;

#[derive(Eq, PartialEq)]
pub enum LcuApiError {
    NoActiveDelegate,
    UnknownError(String),
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub error_code: String,
    pub http_status: i64,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ApiError> for LcuDriverError {
    fn from(api_err: ApiError) -> Self {
        let err = match &*api_err.message {
            "No active delegate" => LcuApiError::NoActiveDelegate,
            msg => LcuApiError::UnknownError(msg.to_owned()),
        };

        LcuDriverError::ApiError(err)
    }
}

impl fmt::Display for LcuApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self {
            LcuApiError::NoActiveDelegate => "No active delegate was found",
            LcuApiError::UnknownError(e) => e,
        };

        write!(f, "{}", message)
    }
}
