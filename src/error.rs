use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KosError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml parse error in {path}: {source}")]
    Yaml {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("workspace not found: could not locate aae-orc root from {start}")]
    WorkspaceNotFound { start: String },
}

pub type Result<T> = std::result::Result<T, KosError>;
