//! Multi-hop traversal over the fact store (aae-orc-xxe3n, PR 3).
//!
//! A traversal starts from seed strings and applies steps. A forward step maps
//! an entity to the values of one attribute; a backward step maps a value to
//! the entities that carry it. Each step is one indexed point lookup per
//! element of the frontier, so no hop reads the whole log.
//!
//! RED STUB: every call returns no hops so the tests compile and fail on their
//! assertions. The implementation follows.

use std::collections::BTreeSet;

use super::{FactStore, Version};
use crate::error::Result;

/// Which way a step follows an attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Entity to value.
    Forward,
    /// Value to entity.
    Backward,
}

/// One hop: an attribute and the way to follow it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub attribute: String,
    pub direction: Direction,
}

impl Step {
    #[must_use]
    pub fn forward(attribute: impl Into<String>) -> Self {
        Self {
            attribute: attribute.into(),
            direction: Direction::Forward,
        }
    }

    #[must_use]
    pub fn backward(attribute: impl Into<String>) -> Self {
        Self {
            attribute: attribute.into(),
            direction: Direction::Backward,
        }
    }
}

impl FactStore {
    /// Apply `steps` to `seeds` as the facts hold at `version`. Returns one set
    /// per step, in order: the elements reached by that hop.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn traverse(
        &self,
        _version: Version,
        _seeds: &[String],
        _steps: &[Step],
    ) -> Result<Vec<BTreeSet<String>>> {
        Ok(Vec::new())
    }
}
