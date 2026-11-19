use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KosError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml parse error in {path}: {source}")]
    Yaml {
        path: String,
        source: Box<serde_saphyr::Error>,
    },

    #[error(
        "workspace not found: no _kos/ graph or aae-orc root found from {start}\n  hint: run `kos init` to create a graph, or use --workspace <path>"
    )]
    WorkspaceNotFound { start: String },

    #[error("explicit workspace path does not contain a _kos/ graph or kos/ directory: {path}")]
    InvalidWorkspace { path: String },

    #[error("manifest error in {path}: {message}")]
    Manifest { path: String, message: String },

    #[error("graph already exists at {path}")]
    GraphExists { path: String },

    #[error("init error: {message}")]
    Init { message: String },

    #[error("update error: {message}")]
    Update { message: String },

    #[error("id error: {message}")]
    Id { message: String },

    #[error("fact store: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error(
        "fact store: write at version {got} is behind the store's latest version {latest}\n  hint: versions only move forward; pass a version at or above {latest}"
    )]
    FactVersionOrder { got: i64, latest: i64 },

    #[error(
        "fact store: {path} has schema version {found}, this kos reads up to {supported}\n  hint: upgrade kos, or open a store written by this version"
    )]
    FactSchemaTooNew {
        path: String,
        found: i64,
        supported: i64,
    },

    #[error(
        "fact store: import needs an empty store\n  hint: import into a new file, or open an empty store"
    )]
    FactImportNotEmpty,

    #[error("kos facts: {message}")]
    FactCommand { message: String },

    #[error(
        "fact import: header: {message}\n  hint: line 1 must be the header that `kos facts export` writes; regenerate the file with that command"
    )]
    FactImportHeader { message: String },

    #[error(
        "fact import: line {line}: {message}\n  hint: each line after the header is one fact interval as `kos facts export` writes it; fix the line or regenerate the file"
    )]
    FactImportLine { line: usize, message: String },

    #[error(
        "fact harvest: {e} {a} is a code or load fact, and a harvest file drops its provenance and intervals\n  hint: move code facts with `kos facts export` and `kos facts import` (JSONL), which keep every interval and load entity"
    )]
    FactHarvestCodeFact { e: String, a: String },

    #[error(
        "fact harvest: {e} {a} {v} does not hold at version {version}\n  hint: a retraction must name a fact that holds at that version; check the version and the fact"
    )]
    FactHarvestNotLive {
        e: String,
        a: String,
        v: String,
        version: i64,
    },

    #[error(
        "fact harvest file: {message}\n  hint: a harvest file is YAML with `kind: harvested-facts`, a `source` naming `repo` and `commit`, and a `facts` list of e, a, v"
    )]
    FactHarvestFormat { message: String },

    #[error("promote: {message}")]
    Promote { message: String },

    #[error(
        "{path} exists but has no kos.yaml manifest\n  A bare _kos/ is not a graph, so this command would write into a parent graph and attribute the result to this repo.\n  hint: run `kos init` here to create the graph, or remove the empty {path} directory if this repo carries no graph"
    )]
    BareKos { path: String },
}

pub type Result<T> = std::result::Result<T, KosError>;
