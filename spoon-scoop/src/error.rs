use std::path::PathBuf;

use spoon_core::CoreError;
use thiserror::Error;

/// Errors that can occur in the scoop domain.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ScoopError {
    /// Wraps a core infrastructure error.
    #[error("{0}")]
    Core(#[from] CoreError),

    /// Wrap another error with additional context.
    #[error("{message}: {source}")]
    Context {
        message: String,
        #[source]
        source: Box<ScoopError>,
    },

    /// Standard I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Filesystem operation error with context.
    #[error("filesystem {action} failed for {path}: {source}")]
    Fs {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// JSON manifest parsing error.
    #[error("manifest parse error: {0}")]
    ManifestParse(#[from] serde_json::Error),

    /// Manifest validation error (structural, not JSON parsing).
    #[error("manifest validation failed: {0}")]
    ManifestValidation(String),

    /// Required manifest was unavailable.
    #[error("package manifest could not be resolved")]
    ManifestUnavailable,

    /// Configuration error.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl ScoopError {
    pub fn context(self, message: impl Into<String>) -> Self {
        Self::Context {
            message: message.into(),
            source: Box::new(self),
        }
    }

    pub fn fs(action: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Fs {
            action,
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, ScoopError>;
