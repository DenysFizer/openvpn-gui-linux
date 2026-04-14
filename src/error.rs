use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    Io(String),
    ProcessSpawn(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::ProcessSpawn(msg) => write!(f, "Process spawn failed: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
