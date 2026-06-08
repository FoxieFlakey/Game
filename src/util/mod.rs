pub mod error;
pub mod sig_safe;
pub mod identifier;

mod idented_writer;

pub use idented_writer::IdentedWriter;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct StringError {
    pub message: String,
}

impl StringError {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}
