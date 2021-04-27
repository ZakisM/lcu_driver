use std::fmt;
use std::fmt::Formatter;

use crate::convert_error;

#[derive(Eq, PartialEq)]
pub enum LcuDriverError {
    FailedToFindLeagueProcess,
    FailedToReadLockfileToken,
    Other(String),
}

impl fmt::Display for LcuDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            LcuDriverError::FailedToFindLeagueProcess => "Failed to find LeagueClientUx process",
            LcuDriverError::FailedToReadLockfileToken => "Failed to read lockfile token",
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
convert_error!(std::io::Error);
convert_error!(std::string::FromUtf8Error);
convert_error!(std::num::ParseIntError);

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
