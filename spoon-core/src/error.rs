use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur in spoon-core infrastructure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// Wrap another error with additional context.
    #[error("{message}: {source}")]
    Context {
        message: String,
        #[source]
        source: Box<CoreError>,
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

    /// Network/HTTP request error.
    #[error("network error for {url}: {source}")]
    Network {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    /// HTTP client construction error.
    #[error("http client {operation} failed: {source}")]
    HttpClient {
        operation: &'static str,
        #[source]
        source: reqwest::Error,
    },

    /// Operation cancelled by user.
    #[error("Cancelled by user.")]
    Cancelled,

    /// Configuration error.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Git operation error.
    #[error("git {operation} failed: {message}")]
    Git {
        operation: &'static str,
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// Async task operation error (join, timeout, etc.).
    #[error("task {operation} failed: {source}")]
    Task {
        operation: &'static str,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// External library error.
    #[error("{message}: {source}")]
    External {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl CoreError {
    /// Wrap this error with additional context.
    pub fn context(self, message: impl Into<String>) -> Self {
        Self::Context {
            message: message.into(),
            source: Box::new(self),
        }
    }

    /// Create a filesystem error.
    pub fn fs(action: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Fs {
            action,
            path: path.into(),
            source,
        }
    }

    /// Create a network error.
    pub fn network(url: impl Into<String>, source: reqwest::Error) -> Self {
        Self::Network {
            url: url.into(),
            source,
        }
    }

    /// Create an HTTP client construction error.
    pub fn http_client(operation: &'static str, source: reqwest::Error) -> Self {
        Self::HttpClient { operation, source }
    }

    /// Create an external error from any error type with context.
    pub fn external(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::External {
            message: message.into(),
            source: Box::new(source),
        }
    }

    /// Create a Git error.
    pub fn git(
        operation: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Git {
            operation,
            message: source.to_string(),
            source: Box::new(source),
        }
    }

    /// Create a task operation error.
    pub fn task(
        operation: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Task {
            operation,
            source: Box::new(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;
