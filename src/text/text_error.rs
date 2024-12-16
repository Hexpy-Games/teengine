use std::fmt;

// Custom error type for font-related errors
#[derive(Debug)]
pub enum TextError {
    FontLoadError(String), // Add more error types as needed
}

impl std::error::Error for TextError {}

impl fmt::Display for TextError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            TextError::FontLoadError(msg) => {
                write!(f, "Font load error: {}", msg)
            }
        }
    }
}
