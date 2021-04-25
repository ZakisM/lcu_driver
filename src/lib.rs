use crate::models::errors::LcuHelperError;

pub mod models;

pub type Result<T> = std::result::Result<T, LcuHelperError>;
