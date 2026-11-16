//! The `kos facts` commands (aae-orc-xxe3n, PR 3).
//!
//! Text output is for people and escapes a backslash, a tab and a newline in
//! every field as `\\`, `\t` and `\n`. `--json` output is the exact form, one
//! JSON object per line.
//!
//! RED STUB: every command does nothing and succeeds, so the tests compile and
//! fail on their assertions. The implementation follows.

use std::path::Path;

use crate::error::Result;

/// Assert the facts of a harvested-facts YAML file at `at`.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn load(_db: &Path, _at: i64, _file: &Path) -> Result<()> {
    Ok(())
}

/// Replay a JSONL export into an empty store.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn import(_db: &Path, _file: &Path) -> Result<()> {
    Ok(())
}

/// Write the whole history as JSONL to stdout or to a new file.
///
/// # Errors
/// Fails when `out` exists, or on a store or write error.
pub fn export(_db: &Path, _out: Option<&Path>) -> Result<()> {
    Ok(())
}

/// Print the facts that hold at `at`.
///
/// # Errors
/// Fails on a store error.
pub fn as_of(
    _db: &Path,
    _at: i64,
    _entity: Option<&str>,
    _attribute: Option<&str>,
    _json: bool,
) -> Result<()> {
    Ok(())
}

/// Print what changed from `from` to `to`.
///
/// # Errors
/// Fails on a store error.
pub fn diff(_db: &Path, _from: i64, _to: i64, _json: bool) -> Result<()> {
    Ok(())
}
