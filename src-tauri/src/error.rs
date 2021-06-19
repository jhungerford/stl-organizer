use serde::Serialize;
use std::{fmt::Formatter, sync::PoisonError};

/// AppError is a unified error type for stl-organizer results.
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    message: String,
}

impl AppError {
    /// Constructs a new AppError with the given message.
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError { message: err.to_string() }
    }
}

impl From<refinery::Error> for AppError {
    fn from(err: refinery::Error) -> Self {
        AppError { message: err.to_string() }
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError { message: err.to_string() }
    }
}
