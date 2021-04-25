use std::fmt;
use std::fmt::Formatter;

use crate::convert_error;

pub enum LcuHelperError {
    FailedToFindLeagueProcess,
    Other(String),
}

impl fmt::Display for LcuHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            LcuHelperError::FailedToFindLeagueProcess => "Failed to find LeagueClientUx process",
            LcuHelperError::Other(message) => message,
        };

        write!(f, "{}", message)
    }
}

impl fmt::Debug for LcuHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl LcuHelperError {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self::Other(message.as_ref().to_string())
    }
}

impl std::error::Error for LcuHelperError {}

convert_error!(reqwest::Error);
convert_error!(std::io::Error);
convert_error!(std::string::FromUtf8Error);

#[macro_export]
macro_rules! convert_error {
    ($err_type:ty) => {
        impl From<$err_type> for LcuHelperError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LcuHelperError::new(err_str)
            }
        }
    };

    ($err_type:ty, $custom_message:expr) => {
        impl From<$err_type> for LcuHelperError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                LcuHelperError::new($custom_message)
            }
        }
    };
}
