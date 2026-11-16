//! Multi-hop traversal over the fact store (aae-orc-xxe3n, PR 3).
//!
//! A traversal starts from seed strings and applies steps. A forward step maps
//! an entity to the values of one attribute; a backward step maps a value to
//! the entities that carry it. Each step is one indexed point lookup per
//! element of the frontier, so no hop reads the whole log.

use std::collections::BTreeSet;

use rusqlite::params;

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

fn first_column(row: &rusqlite::Row<'_>) -> rusqlite::Result<String> {
    row.get(0)
}

impl FactStore {
    /// Apply `steps` to `seeds` as the facts hold at `version`. Returns one set
    /// per step, in order: the elements reached by that hop. Repeated seeds
    /// count once. An attribute no fact carries gives an empty hop.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn traverse(
        &self,
        version: Version,
        seeds: &[String],
        steps: &[Step],
    ) -> Result<Vec<BTreeSet<String>>> {
        let mut forward = self.conn.prepare_cached(concat!(
            "SELECT v FROM facts WHERE e = ?1 AND a = ?2 AND ",
            live_at!("3")
        ))?;
        let mut backward = self.conn.prepare_cached(concat!(
            "SELECT e FROM facts WHERE a = ?1 AND v = ?2 AND ",
            live_at!("3")
        ))?;
        let mut frontier: BTreeSet<String> = seeds.iter().cloned().collect();
        let mut hops = Vec::with_capacity(steps.len());
        for step in steps {
            let mut reached = BTreeSet::new();
            for element in &frontier {
                let rows = match step.direction {
                    Direction::Forward => forward
                        .query_map(params![element, step.attribute, version.0], first_column)?,
                    Direction::Backward => backward
                        .query_map(params![step.attribute, element, version.0], first_column)?,
                };
                for row in rows {
                    reached.insert(row?);
                }
            }
            hops.push(reached.clone());
            frontier = reached;
        }
        Ok(hops)
    }
}
