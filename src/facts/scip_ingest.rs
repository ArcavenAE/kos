//! Reading a SCIP index into the fact store (aae-orc-yto7v, brief
//! `_kos/probes/brief-scip-ingest-build.md`, SI-1 to SI-4, SI-6 to SI-9).
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
//!
//! A load also writes provenance and ends what the index no longer holds:
//!
//! - The entity `load/<repo>@<at>` carries six values (`prov/repo`,
//!   `prov/commit`, `prov/indexer`, `prov/indexer_version`,
//!   `prov/index_sha256`, `prov/grade`). It is asserted at the load's version
//!   and never retracted, so a `code/` row's provenance is its repo's load
//!   entity at the row's start version, and at its end version for an ended
//!   row. The store has no provenance column, and putting provenance in a
//!   fact's value would change every fact's identity at every commit.
//! - A fact this repo's earlier load asserted, and this index does not hold,
//!   has its interval ended. It is stale, not graveyard: no `code/` fact has a
//!   confidence tier, and the source grade is `compiler` for all of them.
//! - A load asserts and ends only its own repo's facts, so one store holds
//!   several repos.
//!
//! An index that the indexer degraded is refused before anything is written
//! (SI-8). A refusal is an error whose message starts `GAP <repo>@<commit>:`
//! and names every check that failed, and it never opens the store:
//!
//! - Rust: the log has no `can't load standard library` line, the index names
//!   at least one external symbol from `std`, and it has a document for every
//!   tracked `.rs` file under a `src/` directory.
//! - Go: the log's last line is `exit: 0` (scip-go's exit status, which the
//!   indexing recipe appends to the log), and the index has a document for
//!   every tracked non-test `.go` file outside `testdata`, `vendor` and
//!   directories whose name starts with `_` or `.`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use protobuf::Message;
use scip::symbol::is_local_symbol;
use scip::types::Index;
use sha2::{Digest, Sha256};

use super::{Fact, FactStore, Version};
use crate::error::{KosError, Result};

/// `SymbolRole::Definition`, the bit that makes an occurrence a definition.
const DEFINITION_ROLE: i32 = 1;

/// What one ingest wrote and what its filters dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Report {
    /// Facts newly asserted (a fact already live is not counted).
    pub added: usize,
    /// Facts of this repo that the index no longer holds, whose interval ended.
    pub ended: usize,
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
    read_index_bytes(path).map(|(index, _)| index)
}

/// The index at `path` and the sha256 of its bytes, as lowercase hex.
fn read_index_bytes(path: &Path) -> Result<(Index, String)> {
    let bytes = fs::read(path)
        .map_err(|e| refuse(format!("cannot read the index {}: {e}", path.display())))?;
    let index = Index::parse_from_bytes(&bytes).map_err(|e| {
        refuse(format!(
            "{} is not a SCIP index: {e}\n  hint: pass the index file rust-analyzer or scip-go wrote",
            path.display()
        ))
    })?;
    Ok((index, format!("{:x}", Sha256::digest(&bytes))))
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

/// What one ingest needs: the store, the version, which repo and commit the
/// index describes, and the index file.
pub struct Request<'a> {
    pub db: &'a Path,
    pub at: i64,
    pub repo: &'a str,
    pub commit: &'a str,
    pub index: &'a Path,
    pub log: &'a Path,
    pub checkout: &'a Path,
}

/// The provenance of one load, as the six values of `load/<repo>@<at>`.
fn load_facts(request: &Request<'_>, index: &Index, sha256: &str) -> Vec<Fact> {
    let entity = format!("load/{}@{}", request.repo, request.at);
    let tool = &index.metadata.tool_info;
    [
        ("prov/repo", request.repo),
        ("prov/commit", request.commit),
        ("prov/indexer", tool.name.as_str()),
        ("prov/indexer_version", tool.version.as_str()),
        ("prov/index_sha256", sha256),
        ("prov/grade", "compiler"),
    ]
    .into_iter()
    .map(|(attribute, value)| Fact::new(&entity, attribute, value))
    .collect()
}

/// Whether `fact` belongs to `repo`: its documents and dependencies by name,
/// and its implementation facts by the packages the repo defines.
fn belongs_to(fact: &Fact, repo: &str, packages: &BTreeSet<(String, String)>) -> bool {
    match fact.a.as_str() {
        "code/defines" | "code/references" => fact
            .e
            .strip_prefix(repo)
            .is_some_and(|rest| rest.starts_with(':')),
        "code/depends_on" => fact.e == repo,
        "code/implements" => split_symbol(&fact.e)
            .is_some_and(|p| packages.contains(&(p.manager.to_string(), p.package.to_string()))),
        _ => false,
    }
}

/// The packages `repo` defines, from the definitions in `facts`.
fn defined_packages<'a>(
    repo: &str,
    facts: impl Iterator<Item = &'a Fact>,
) -> BTreeSet<(String, String)> {
    facts
        .filter(|f| {
            f.a == "code/defines"
                && f.e
                    .strip_prefix(repo)
                    .is_some_and(|rest| rest.starts_with(':'))
        })
        .filter_map(|f| split_symbol(&f.v))
        .map(|p| (p.manager.to_string(), p.package.to_string()))
        .collect()
}

/// The paths of the files git tracks in `checkout`.
fn tracked_files(checkout: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| refuse(format!("cannot run git in {}: {e}", checkout.display())))?;
    if !out.status.success() {
        return Err(refuse(format!(
            "{} is not a git checkout: {}",
            checkout.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect())
}

/// Whether the index names an external symbol from the standard library.
fn names_std(index: &Index) -> bool {
    let named = |symbol: &str| split_symbol(symbol).is_some_and(|p| p.package == "std");
    index.documents.iter().any(|d| {
        d.occurrences.iter().any(|o| named(&o.symbol))
            || d.symbols
                .iter()
                .any(|s| named(&s.symbol) || s.relationships.iter().any(|r| named(&r.symbol)))
    })
}

fn is_go_source(path: &str) -> bool {
    let ignored_dir = path
        .split('/')
        .rev()
        .skip(1)
        .any(|d| d == "testdata" || d == "vendor" || d.starts_with('_') || d.starts_with('.'));
    path.ends_with(".go") && !path.ends_with("_test.go") && !ignored_dir
}

fn is_rust_source(path: &str) -> bool {
    path.ends_with(".rs") && (path.starts_with("src/") || path.contains("/src/"))
}

/// The reasons `index` is degraded, empty when it is healthy.
fn degraded_reasons(
    index: &Index,
    log: &str,
    tracked: &[String],
) -> std::result::Result<Vec<String>, String> {
    let documents: BTreeSet<&str> = index
        .documents
        .iter()
        .map(|d| d.relative_path.as_str())
        .collect();
    let missing = |wanted: fn(&str) -> bool| -> Vec<&str> {
        tracked
            .iter()
            .map(String::as_str)
            .filter(|p| wanted(p) && !documents.contains(p))
            .collect()
    };
    let mut reasons = Vec::new();
    match index.metadata.tool_info.name.as_str() {
        "rust-analyzer" => {
            if log
                .lines()
                .any(|l| l.contains("can't load standard library"))
            {
                reasons.push("the indexer log has `can't load standard library`".to_string());
            }
            if !names_std(index) {
                reasons.push("the index names no external symbol from std".to_string());
            }
            for path in missing(is_rust_source) {
                reasons.push(format!("the index has no document for tracked file {path}"));
            }
        }
        "scip-go" => {
            match log.lines().rev().find(|l| !l.trim().is_empty()) {
                Some(last) if last.trim() == "exit: 0" => {}
                Some(last) if last.trim().starts_with("exit: ") => reasons.push(format!(
                    "scip-go's exit status is not 0 (the log ends `{}`)",
                    last.trim()
                )),
                _ => {
                    reasons.push("the log does not end with scip-go's `exit: <n>` line".to_string())
                }
            }
            for path in missing(is_go_source) {
                reasons.push(format!("the index has no document for tracked file {path}"));
            }
        }
        other => {
            return Err(format!(
                "there is no health check for the indexer `{other}`"
            ));
        }
    }
    Ok(reasons)
}

/// Refuse a degraded index (SI-8). Nothing is written, and the store is not
/// opened.
fn check_healthy(request: &Request<'_>, index: &Index) -> Result<()> {
    let gap = |reasons: &str| {
        refuse(format!(
            "GAP {}@{}: {reasons}\n  hint: nothing was loaded; fix the indexing and run it again",
            request.repo, request.commit
        ))
    };
    let log = fs::read(request.log).map_err(|e| {
        refuse(format!(
            "cannot read the indexer log {}: {e}",
            request.log.display()
        ))
    })?;
    let log = String::from_utf8_lossy(&log);
    let tracked = tracked_files(request.checkout)?;
    match degraded_reasons(index, &log, &tracked) {
        Ok(reasons) if reasons.is_empty() => Ok(()),
        Ok(reasons) => Err(gap(&reasons.join("; "))),
        Err(reason) => Err(gap(&reason)),
    }
}

/// Read the index and load it for the repo at version `at`: assert its facts,
/// end this repo's facts that it no longer holds, and write the load entity.
///
/// # Errors
/// Fails on an unreadable index or a store error. The index is read before the
/// store is opened, so a refused file creates no store.
pub fn ingest(request: &Request<'_>) -> Result<Report> {
    let (index, sha256) = read_index_bytes(request.index)?;
    check_healthy(request, &index)?;
    let (facts, mut report) = facts_from_index(request.repo, &index);
    let version = Version::new(request.at);
    let mut store = FactStore::open(request.db)?;

    let earlier = match store.latest_version()? {
        Some(latest) => store.facts_as_of(latest)?,
        None => Vec::new(),
    };
    let packages = defined_packages(request.repo, earlier.iter().chain(facts.iter()));
    let stale: Vec<Fact> = earlier
        .into_iter()
        .filter(|f| belongs_to(f, request.repo, &packages) && !facts.contains(f))
        .collect();

    report.ended = store.retract_facts(version, &stale)?;
    let facts: Vec<Fact> = facts.into_iter().collect();
    report.added = store.assert_facts(version, &facts)?;
    store.assert_facts(version, &load_facts(request, &index, &sha256))?;
    Ok(report)
}
