use std::fmt;

// Custom error type for font-related errors
#[derive(Debug)]
pub enum FontError {
    InvalidFontType(String),
    FontLoadError(String),
    SaveError(String),
    LoadError(String),
    // Add more error types as needed
}

impl std::error::Error for FontError {}

impl fmt::Display for FontError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            FontError::InvalidFontType(t) => write!(
                f,
                "Invalid font type '{}'. Valid types are: ASCII, UNICODE",
                t
            ),
            FontError::FontLoadError(msg) => {
                write!(f, "Font load error: {}", msg)
            }
            FontError::SaveError(msg) => {
                write!(f, "Save error: {}", msg)
            }
            FontError::LoadError(msg) => {
                write!(f, "Load error: {}", msg)
            }
        }
    }
}
