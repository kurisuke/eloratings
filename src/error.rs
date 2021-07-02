use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AppError {
    text: String,
}

impl AppError {
    pub fn new(text: &str) -> AppError {
        AppError {
            text: text.to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        &self.text
    }
}
