use std::error::Error;

#[derive(Debug)]
pub enum ServerError {
    Critical(String),
    // HttpParse(String),
    // Connection(String),
    CustomError(String),
}

// Implement the Display trait for error messages
use std::fmt;
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerError::Critical(ref msg) => write!(f, "Critical Error: {}", msg),
            // ServerError::HttpParse(ref msg) => write!(f, "Http Parse Error: {}", msg),
            // ServerError::Connection(ref msg) => write!(f, "TCP Connection Error: {}", msg),
            ServerError::CustomError(ref msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServerError::Critical(ref _msg) => None,
            // ServerError::HttpParse(ref msg) => None,
            ServerError::CustomError(ref _msg) => None,
        }
    }
}
