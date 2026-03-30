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

    #[error(
        "workspace not found: could not locate aae-orc root from {start}\n  hint: use --workspace <path> or set KOS_WORKSPACE to point to the aae-orc root"
    )]
    WorkspaceNotFound { start: String },

    #[error("explicit workspace path does not contain a kos/ directory: {path}")]
    InvalidWorkspace { path: String },

    #[error("manifest error in {path}: {message}")]
    Manifest { path: String, message: String },

    #[error("graph already exists at {path}")]
    GraphExists { path: String },

    #[error("init error: {message}")]
    Init { message: String },
}

pub type Result<T> = std::result::Result<T, KosError>;
