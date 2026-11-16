//! Moving facts in and out of the store (aae-orc-xxe3n, PR 2).
//!
//! - JSONL export and import of the whole history, one row per fact interval.
//! - Harvest: a chosen set of facts, as they hold at one version, written as a
//!   `harvested-facts` YAML file, and the matching read and apply.
//!
//! RED STUB: every operation returns an empty or neutral result so the tests
//! compile and fail on their assertions. The implementation follows.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use super::{Fact, FactStore, Version};
use crate::error::Result;

/// What a harvested-facts file carries: a caller-supplied source map and the
/// facts, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Harvest {
    pub source: BTreeMap<String, String>,
    pub facts: Vec<Fact>,
}

impl Harvest {
    /// Parse a harvested-facts YAML file.
    ///
    /// # Errors
    /// Fails when the text is not YAML, has no `kind: harvested-facts`, or has
    /// a fact without all of `e`, `a` and `v`.
    pub fn parse(_yaml: &str) -> Result<Self> {
        Ok(Self::default())
    }
}

impl FactStore {
    /// Write every fact interval as one JSON line, sorted by entity,
    /// attribute, value and start version.
    ///
    /// # Errors
    /// Fails on a SQLite or write error.
    pub fn export_jsonl(&self, _out: &mut impl Write) -> Result<()> {
        Ok(())
    }

    /// Replay an export into this store, which must be empty. Returns the
    /// number of rows read.
    ///
    /// # Errors
    /// Fails when the store holds facts, or on the first malformed row, naming
    /// its line.
    pub fn import_jsonl(&mut self, _input: impl BufRead) -> Result<usize> {
        Ok(0)
    }

    /// Write the given facts, which must hold at `version`, as a
    /// harvested-facts YAML file.
    ///
    /// # Errors
    /// Fails when a fact does not hold at `version`.
    pub fn harvest(
        &self,
        _version: Version,
        _picks: &[Fact],
        _source: &BTreeMap<String, String>,
    ) -> Result<String> {
        Ok(String::new())
    }

    /// Assert the facts of a harvested-facts file at `version`. Returns how
    /// many were newly asserted.
    ///
    /// # Errors
    /// Fails on a malformed file or when `version` is behind the latest.
    pub fn apply_harvest(&mut self, _version: Version, _yaml: &str) -> Result<usize> {
        Ok(0)
    }
}
