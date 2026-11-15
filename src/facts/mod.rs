//! SQLite valid-time fact store (aae-orc-xxe3n).
//!
//! A fact is an (entity, attribute, value) triple. The store keeps each fact
//! with the half-open interval of versions in which it holds: `valid_from`
//! inclusive, `valid_to` exclusive, `valid_to` empty while the fact is live.
//! kos names versions and passes them in; the store never mints one.
//!
//! RED STUB: every operation returns an empty or neutral result so the tests
//! compile and fail on their assertions. The implementation follows.

use std::path::Path;

use crate::error::Result;

/// A version of the fact set. Opaque to the store: it only compares them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(i64);

impl Version {
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn get(self) -> i64 {
        self.0
    }
}

/// One (entity, attribute, value) triple.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fact {
    pub e: String,
    pub a: String,
    pub v: String,
}

impl Fact {
    #[must_use]
    pub fn new(e: impl Into<String>, a: impl Into<String>, v: impl Into<String>) -> Self {
        Self {
            e: e.into(),
            a: a.into(),
            v: v.into(),
        }
    }
}

/// The facts added and removed between two versions, each sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactDiff {
    pub added: Vec<Fact>,
    pub removed: Vec<Fact>,
}

pub struct FactStore {}

impl FactStore {
    /// Open or create the store at `path`.
    ///
    /// # Errors
    /// Fails when the file cannot be opened or carries a newer schema.
    pub fn open(_path: &Path) -> Result<Self> {
        Ok(Self {})
    }

    /// Open a private in-memory store.
    ///
    /// # Errors
    /// Fails when SQLite cannot start.
    pub fn open_in_memory() -> Result<Self> {
        Ok(Self {})
    }

    /// Assert facts at `version`; returns how many were newly asserted.
    ///
    /// # Errors
    /// Fails when `version` is behind the latest version written.
    pub fn assert_facts(&mut self, _version: Version, _facts: &[Fact]) -> Result<usize> {
        Ok(0)
    }

    /// Retract facts at `version`; returns how many were live and closed.
    ///
    /// # Errors
    /// Fails when `version` is behind the latest version written.
    pub fn retract_facts(&mut self, _version: Version, _facts: &[Fact]) -> Result<usize> {
        Ok(0)
    }

    /// The facts that hold at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn facts_as_of(&self, _version: Version) -> Result<Vec<Fact>> {
        Ok(Vec::new())
    }

    /// What changed from version `from` to version `to`.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn diff(&self, _from: Version, _to: Version) -> Result<FactDiff> {
        Ok(FactDiff::default())
    }

    /// Values of attribute `a` on entity `e` at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn values_of(&self, _e: &str, _a: &str, _version: Version) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Entities that carry value `v` for attribute `a` at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn entities_with(&self, _a: &str, _v: &str, _version: Version) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// The highest version written so far, or `None` for an empty store.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn latest_version(&self) -> Result<Option<Version>> {
        Ok(None)
    }
}
