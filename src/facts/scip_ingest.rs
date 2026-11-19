//! Reading a SCIP index into the fact store (aae-orc-yto7v, brief
//! `_kos/probes/brief-scip-ingest-build.md`, SI-1 to SI-4).
//!
//! The ingest decodes an index file an indexer already wrote and writes only
//! through `FactStore`. Four fact shapes come out of it:
//!
//! - `code/defines (document, symbol)`: an occurrence with the definition role.
//! - `code/references (document, symbol)`: any other occurrence.
//! - `code/implements (symbol, symbol)`: a relationship marked as an
//!   implementation.
//! - `code/depends_on (repo, package)`: each external package a symbol names.
//!
//! A document entity is `<repo>:<path>`, so the same path in two repos is two
//! entities. A repo's own version inside a symbol is replaced by `.`, so a
//! version bump does not end and restart every own fact. Documents whose path
//! leaves the repo root, and local symbols, are skipped and counted.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use protobuf::Message;
use scip::symbol::is_local_symbol;
use scip::types::Index;

use super::{Fact, FactStore, Version};
use crate::error::{KosError, Result};

/// `SymbolRole::Definition`, the bit that makes an occurrence a definition.
const DEFINITION_ROLE: i32 = 1;

/// What one ingest wrote and what its filters dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Report {
    /// Facts newly asserted (a fact already live is not counted).
    pub added: usize,
    /// Documents skipped because their path leaves the repo root.
    pub out_of_root_documents: usize,
    /// Local occurrences and local relationships skipped.
    pub local_symbols: usize,
}

fn refuse(message: impl Into<String>) -> KosError {
    KosError::FactCommand {
        message: message.into(),
    }
}

/// Read the SCIP index at `path`.
///
/// # Errors
/// Fails when the file cannot be read or is not a SCIP index.
pub fn read_index(path: &Path) -> Result<Index> {
    let bytes = fs::read(path)
        .map_err(|e| refuse(format!("cannot read the index {}: {e}", path.display())))?;
    Index::parse_from_bytes(&bytes).map_err(|e| {
        refuse(format!(
            "{} is not a SCIP index: {e}\n  hint: pass the index file rust-analyzer or scip-go wrote",
            path.display()
        ))
    })
}

/// The parts of a global SCIP symbol that a fact keeps; the symbol is five
/// space-separated parts, `<scheme> <manager> <package> <version> <descriptors>`.
struct Parts<'a> {
    scheme: &'a str,
    manager: &'a str,
    package: &'a str,
    descriptors: &'a str,
}

fn split_symbol(symbol: &str) -> Option<Parts<'_>> {
    if is_local_symbol(symbol) {
        return None;
    }
    let mut it = symbol.splitn(5, ' ');
    let scheme = it.next()?;
    let manager = it.next()?;
    let package = it.next()?;
    // The fourth part is the package version; no fact keeps it.
    it.next()?;
    Some(Parts {
        scheme,
        manager,
        package,
        descriptors: it.next()?,
    })
}

/// The repo's own packages: the `(manager, package)` of every definition the
/// index makes in a document under the repo root.
fn own_packages(index: &Index) -> BTreeSet<(String, String)> {
    let mut own = BTreeSet::new();
    for document in index
        .documents
        .iter()
        .filter(|d| !leaves_root(&d.relative_path))
    {
        for occurrence in &document.occurrences {
            if occurrence.symbol_roles & DEFINITION_ROLE == 0 {
                continue;
            }
            if let Some(parts) = split_symbol(&occurrence.symbol) {
                own.insert((parts.manager.to_string(), parts.package.to_string()));
            }
        }
    }
    own
}

fn leaves_root(path: &str) -> bool {
    path.starts_with('/') || path == ".." || path.starts_with("../")
}

/// The symbol with its own package's version replaced by `.`.
fn normalise(symbol: &str, own: &BTreeSet<(String, String)>) -> String {
    match split_symbol(symbol) {
        Some(p) if own.contains(&(p.manager.to_string(), p.package.to_string())) => {
            format!(
                "{} {} {} . {}",
                p.scheme, p.manager, p.package, p.descriptors
            )
        }
        _ => symbol.to_string(),
    }
}

/// The package a symbol names, when it is a global symbol of another package.
fn external_package(symbol: &str, own: &BTreeSet<(String, String)>) -> Option<String> {
    let parts = split_symbol(symbol)?;
    let key = (parts.manager.to_string(), parts.package.to_string());
    if own.contains(&key) {
        return None;
    }
    Some(parts.package.to_string())
}

/// The facts of `index` for `repo`, and what the filters dropped.
#[must_use]
pub fn facts_from_index(repo: &str, index: &Index) -> (BTreeSet<Fact>, Report) {
    let own = own_packages(index);
    let mut facts = BTreeSet::new();
    let mut report = Report::default();
    let mut packages = BTreeSet::new();
    let mut note_package = |symbol: &str| {
        if let Some(package) = external_package(symbol, &own) {
            packages.insert(package);
        }
    };
    for document in &index.documents {
        if leaves_root(&document.relative_path) {
            report.out_of_root_documents += 1;
            continue;
        }
        let entity = format!("{repo}:{}", document.relative_path);
        for occurrence in &document.occurrences {
            if is_local_symbol(&occurrence.symbol) {
                report.local_symbols += 1;
                continue;
            }
            let attribute = if occurrence.symbol_roles & DEFINITION_ROLE == 0 {
                "code/references"
            } else {
                "code/defines"
            };
            note_package(&occurrence.symbol);
            facts.insert(Fact::new(
                &entity,
                attribute,
                normalise(&occurrence.symbol, &own),
            ));
        }
        for info in &document.symbols {
            for relationship in info.relationships.iter().filter(|r| r.is_implementation) {
                if is_local_symbol(&info.symbol) || is_local_symbol(&relationship.symbol) {
                    report.local_symbols += 1;
                    continue;
                }
                note_package(&info.symbol);
                note_package(&relationship.symbol);
                facts.insert(Fact::new(
                    normalise(&info.symbol, &own),
                    "code/implements",
                    normalise(&relationship.symbol, &own),
                ));
            }
        }
    }
    for package in packages {
        facts.insert(Fact::new(repo, "code/depends_on", package));
    }
    (facts, report)
}

/// Read the index at `index_path` and assert its facts for `repo` at `at`.
///
/// # Errors
/// Fails on an unreadable index or a store error. The index is read before the
/// store is opened, so a refused file creates no store.
pub fn ingest(db: &Path, at: i64, repo: &str, index_path: &Path) -> Result<Report> {
    let index = read_index(index_path)?;
    let (facts, mut report) = facts_from_index(repo, &index);
    let facts: Vec<Fact> = facts.into_iter().collect();
    let mut store = FactStore::open(db)?;
    report.added = store.assert_facts(Version::new(at), &facts)?;
    Ok(report)
}
