use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ExecuteError {
    pub message: String,
    // TODO location
}

impl Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ExecuteError {}
