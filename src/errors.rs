use std::fmt;
use std::fmt::Formatter;

use crate::convert_error;
use crate::models::api_error::LcuApiError;

#[derive(Eq, PartialEq)]
pub enum LcuDriverError {
    FailedToFindLeagueProcess,
    FailedToReadLockfileToken,
    FailedToSendRequest(String),
    FailedToReadResponse(String),
    FailedToReadCertificate,
    FailedToFindLutrisPrefix,
    ApiError(LcuApiError),
    Other(String),
}

impl fmt::Display for LcuDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            LcuDriverError::FailedToFindLeagueProcess => "Failed to find LeagueClientUx process",
            LcuDriverError::FailedToReadLockfileToken => "Failed to read lockfile token",
            LcuDriverError::FailedToSendRequest(e) => {
                return write!(f, "Failed to send request to League API - {}", e);
            }
            LcuDriverError::FailedToReadResponse(e) => {
                return write!(f, "Failed to read response text from League API - {}", e);
            }
            LcuDriverError::FailedToReadCertificate => "Failed to read riot certificate file",
            LcuDriverError::FailedToFindLutrisPrefix => "Failed to find lutris prefix",
            LcuDriverError::ApiError(e) => return e.fmt(f),
            LcuDriverError::Other(message) => message,
        };

        write!(f, "{}", message)
    }
}

impl fmt::Debug for LcuDriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl LcuDriverError {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self::Other(message.as_ref().to_string())
    }
}

impl std::error::Error for LcuDriverError {}

convert_error!(reqwest::Error);
convert_error!(reqwest::header::InvalidHeaderValue);
convert_error!(serde_json::Error);
convert_error!(std::io::Error);
convert_error!(std::string::FromUtf8Error);
convert_error!(std::num::ParseIntError);
convert_error!(tokio_tungstenite::tungstenite::Error);
convert_error!(url::ParseError);
convert_error!(http::uri::InvalidUri);
convert_error!(http::Error);
convert_error!(std::path::StripPrefixError);

#[macro_export]
macro_rules! convert_error {
    ($err_type:ty) => {
        impl From<$err_type> for LcuDriverError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LcuDriverError::new(err_str)
            }
        }
    };

    ($err_type:ty, $custom_message:expr) => {
        impl From<$err_type> for LcuDriverError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LcuDriverError::new($custom_message)
            }
        }
    };
}
