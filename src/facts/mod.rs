//! SQLite valid-time fact store (aae-orc-xxe3n).
//!
//! A fact is an (entity, attribute, value) triple. The store keeps each fact
//! with the half-open interval of versions in which it holds: `valid_from`
//! inclusive, `valid_to` exclusive, `valid_to` empty while the fact is live.
//! kos names versions and passes them in; the store never mints one.
//!
//! Semantics, in one place:
//! - A fact holds at version `n` when `valid_from <= n` and either `valid_to`
//!   is empty or `n < valid_to`.
//! - Writes carry a version and must not go backward. Several writes at one
//!   version are allowed.
//! - Asserting a live fact changes nothing. Retracting a fact in the version
//!   that asserted it leaves no trace.
//! - The attribute is free text. The store knows no kos schema.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::error::{KosError, Result};

/// Schema version written to `PRAGMA user_version`.
pub(crate) const SCHEMA_VERSION: i64 = 1;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS facts (
    id         INTEGER PRIMARY KEY,
    e          TEXT NOT NULL,
    a          TEXT NOT NULL,
    v          TEXT NOT NULL,
    valid_from INTEGER NOT NULL,
    valid_to   INTEGER
);
CREATE UNIQUE INDEX IF NOT EXISTS facts_live ON facts (e, a, v) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS facts_eavt ON facts (e, a, v, valid_from, valid_to);
CREATE INDEX IF NOT EXISTS facts_aevt ON facts (a, e, v, valid_from, valid_to);
CREATE INDEX IF NOT EXISTS facts_avet ON facts (a, v, e, valid_from, valid_to);
CREATE INDEX IF NOT EXISTS facts_vaet ON facts (v, a, e, valid_from, valid_to);
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value INTEGER NOT NULL
) WITHOUT ROWID;
";

pub(crate) const LATEST_KEY: &str = "latest_version";

/// SQL predicate: the fact row holds at the version bound to `?N`.
macro_rules! live_at {
    ($n:literal) => {
        concat!(
            "valid_from <= ?",
            $n,
            " AND (valid_to IS NULL OR ?",
            $n,
            " < valid_to)"
        )
    };
}

mod exchange;

pub use exchange::Harvest;

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

pub struct FactStore {
    conn: Connection,
}

impl FactStore {
    /// Open or create the store at `path`.
    ///
    /// # Errors
    /// Fails when the file cannot be opened or carries a newer schema.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init(conn, &path.display().to_string())
    }

    /// Open a private in-memory store.
    ///
    /// # Errors
    /// Fails when SQLite cannot start.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, ":memory:")
    }

    fn init(conn: Connection, label: &str) -> Result<Self> {
        let found: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if found > SCHEMA_VERSION {
            return Err(KosError::FactSchemaTooNew {
                path: label.to_string(),
                found,
                supported: SCHEMA_VERSION,
            });
        }
        if found < SCHEMA_VERSION {
            conn.execute_batch(SCHEMA)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(Self { conn })
    }

    /// Assert facts at `version`; returns how many were newly asserted.
    ///
    /// A fact that is already live is left alone and not counted.
    ///
    /// # Errors
    /// Fails when `version` is behind the latest version written.
    pub fn assert_facts(&mut self, version: Version, facts: &[Fact]) -> Result<usize> {
        self.write(version, facts, |tx, fact| {
            let changed = tx.execute(
                "INSERT INTO facts (e, a, v, valid_from) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (e, a, v) WHERE valid_to IS NULL DO NOTHING",
                params![fact.e, fact.a, fact.v, version.0],
            )?;
            Ok(changed)
        })
    }

    /// Retract facts at `version`; returns how many were live and closed.
    ///
    /// # Errors
    /// Fails when `version` is behind the latest version written.
    pub fn retract_facts(&mut self, version: Version, facts: &[Fact]) -> Result<usize> {
        self.write(version, facts, |tx, fact| {
            // Asserted in this same version: the interval would be empty, so
            // the row goes.
            let removed = tx.execute(
                "DELETE FROM facts
                 WHERE e = ?1 AND a = ?2 AND v = ?3 AND valid_to IS NULL AND valid_from = ?4",
                params![fact.e, fact.a, fact.v, version.0],
            )?;
            if removed > 0 {
                return Ok(removed);
            }
            let closed = tx.execute(
                "UPDATE facts SET valid_to = ?4
                 WHERE e = ?1 AND a = ?2 AND v = ?3 AND valid_to IS NULL",
                params![fact.e, fact.a, fact.v, version.0],
            )?;
            Ok(closed)
        })
    }

    /// Run `apply` for each fact in one transaction, after the version check.
    fn write(
        &mut self,
        version: Version,
        facts: &[Fact],
        apply: impl Fn(&Transaction<'_>, &Fact) -> Result<usize>,
    ) -> Result<usize> {
        if facts.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.transaction()?;
        if let Some(latest) = latest_in(&tx)?
            && version < latest
        {
            return Err(KosError::FactVersionOrder {
                got: version.0,
                latest: latest.0,
            });
        }
        let mut changed = 0;
        for fact in facts {
            changed += apply(&tx, fact)?;
        }
        tx.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![LATEST_KEY, version.0],
        )?;
        tx.commit()?;
        Ok(changed)
    }

    /// The facts that hold at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn facts_as_of(&self, version: Version) -> Result<Vec<Fact>> {
        self.fact_rows(
            concat!(
                "SELECT e, a, v FROM facts WHERE ",
                live_at!("1"),
                " ORDER BY e, a, v"
            ),
            params![version.0],
        )
    }

    /// What changed from version `from` to version `to`.
    ///
    /// `added` holds at `to` and not at `from`; `removed` is the reverse.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn diff(&self, from: Version, to: Version) -> Result<FactDiff> {
        let sql = concat!(
            "SELECT e, a, v FROM facts WHERE ",
            live_at!("1"),
            " EXCEPT SELECT e, a, v FROM facts WHERE ",
            live_at!("2"),
            " ORDER BY e, a, v"
        );
        Ok(FactDiff {
            added: self.fact_rows(sql, params![to.0, from.0])?,
            removed: self.fact_rows(sql, params![from.0, to.0])?,
        })
    }

    /// Values of attribute `a` on entity `e` at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn values_of(&self, e: &str, a: &str, version: Version) -> Result<Vec<String>> {
        self.strings(
            concat!(
                "SELECT v FROM facts WHERE e = ?1 AND a = ?2 AND ",
                live_at!("3"),
                " ORDER BY v"
            ),
            params![e, a, version.0],
        )
    }

    /// Entities that carry value `v` for attribute `a` at `version`, sorted.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn entities_with(&self, a: &str, v: &str, version: Version) -> Result<Vec<String>> {
        self.strings(
            concat!(
                "SELECT e FROM facts WHERE a = ?1 AND v = ?2 AND ",
                live_at!("3"),
                " ORDER BY e"
            ),
            params![a, v, version.0],
        )
    }

    /// The highest version written so far, or `None` for an empty store.
    ///
    /// # Errors
    /// Fails on a SQLite error.
    pub fn latest_version(&self) -> Result<Option<Version>> {
        latest_in(&self.conn)
    }

    fn fact_rows(&self, sql: &str, args: impl rusqlite::Params) -> Result<Vec<Fact>> {
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map(args, |row| {
            Ok(Fact {
                e: row.get(0)?,
                a: row.get(1)?,
                v: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn strings(&self, sql: &str, args: impl rusqlite::Params) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map(args, |row| row.get(0))?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

fn latest_in(conn: &Connection) -> Result<Option<Version>> {
    let latest: Option<i64> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = ?1",
            params![LATEST_KEY],
            |row| row.get(0),
        )
        .optional()?;
    Ok(latest.map(Version))
}
