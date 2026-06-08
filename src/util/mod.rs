pub mod error;
pub mod sig_safe;
pub mod identifier;

mod idented_writer;

use std::error::Error;

pub use idented_writer::IdentedWriter;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct StringError {
    pub message: String,
    #[source]
    pub cause: Option<Box<dyn Error>>
}

impl StringError {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            cause: None
        }
    }
    
    pub fn new_with_cause<S: Into<String>, E: Into<Box<dyn Error>>>(message: S, cause: E) -> Self {
        Self {
            message: message.into(),
            cause: Some(cause.into())
        }
    }
}
