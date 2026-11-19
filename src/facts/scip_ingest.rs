//! Reading a SCIP index into the fact store (aae-orc-yto7v, brief
//! `_kos/probes/brief-scip-ingest-build.md`, SI-1 to SI-4, SI-6 to SI-9, SI-11).
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
//!   several repos. A store holds one repo per package: `code/implements`
//!   facts belong to the repo that defines their subject symbol's package, so
//!   two repos that define the same package would end each other's. The repo
//!   name may not contain `:` or `@`, the delimiters of an entity and of a
//!   load entity.
//!
//! A repo may name crates in its knowledge nodes. The names that count come
//! from one checked-in list per repo, `.kos/crate-names.txt` in the checkout
//! (one name per line, `#` starts a comment line), and not from any window of
//! commits or from a commit's manifest, which moved a probe's answer from 3
//! facts to 0 when a dependency left. For each listed name that the text of a
//! node under `_kos/nodes/*/<id>.yaml`, or a finding under
//! `_kos/findings/<id>.yaml` or `<id>.md`, names as a whole word, the load
//! asserts `kos/names-dep (<repo>:<id>, <name>)`. A name is matched as written: a Rust
//! path spells a crate with an underscore (`serde_saphyr::`) where the list may
//! hold the hyphenated name (`serde-saphyr`), and the underscore form does not
//! match the hyphenated one, so the list holds each spelling the text uses.
//!
//! An input the ingest cannot read is refused before anything is written, and
//! so is an index it cannot parse. Only the index health check below refuses
//! with a message that starts `GAP <repo>@<commit>:` and names every check
//! that failed. The other refusals are plain errors, each in this form:
//!
//! - `cannot read the index <path>: <error>` and `cannot read the indexer log
//!   <path>: <error>`;
//! - `<path> is not a SCIP index: <error>` for an index file that does not
//!   parse;
//! - `cannot read <path>: <error>` for the crate-name list and for a node file
//!   or a node directory;
//! - `--repo "<name>" must be non-empty and hold no ':' or '@', ...` for a repo
//!   name that holds a delimiter;
//! - `cannot run git in <checkout>: <error>` when `git` cannot start, and
//!   `<checkout> is not a git checkout: <git's message>` when `git ls-files`
//!   fails, since the tracked files are how the health check knows what the
//!   index must cover.
//!
//! All of them return before the store is opened, so a refused load creates no
//! store file.
//!
//! An index that the indexer degraded is refused (SI-8):
//!
//! - Rust: the log has no `can't load standard library` line, the index names
//!   at least one external symbol from `std`, and it has a document for every
//!   tracked `.rs` file under the root crate's `src/` and under the `src/` of each
//!   workspace member the root `Cargo.toml` lists. A project tracked inside the
//!   repo that is not a member is not the crate the index covers.
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
    /// Facts newly asserted, the load entity's values included (a fact already
    /// live is not counted).
    pub added: usize,
    /// Facts of this repo that the index no longer holds, whose interval ended.
    pub ended: usize,
    /// Documents skipped because their path leaves the repo root.
    pub out_of_root_documents: usize,
    /// Distinct facts those documents would have made.
    pub out_of_root_facts: usize,
    /// Local occurrences and local relationships skipped.
    pub local_symbols: usize,
    /// Distinct facts the skipped local items would have made.
    pub local_facts: usize,
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

/// The attribute an occurrence's roles give it.
fn occurrence_attribute(roles: i32) -> &'static str {
    if roles & DEFINITION_ROLE == 0 {
        "code/references"
    } else {
        "code/defines"
    }
}

fn leaves_root(path: &str) -> bool {
    path.starts_with('/') || path.split('/').any(|segment| segment == "..")
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
    let mut out_of_root = BTreeSet::new();
    let mut local = BTreeSet::new();
    let mut note_package = |symbol: &str| {
        if let Some(package) = external_package(symbol, &own) {
            packages.insert(package);
        }
    };
    for document in &index.documents {
        if leaves_root(&document.relative_path) {
            report.out_of_root_documents += 1;
            let entity = format!("{repo}:{}", document.relative_path);
            for occurrence in document
                .occurrences
                .iter()
                .filter(|o| !is_local_symbol(&o.symbol))
            {
                out_of_root.insert(Fact::new(
                    &entity,
                    occurrence_attribute(occurrence.symbol_roles),
                    &occurrence.symbol,
                ));
            }
            for info in &document.symbols {
                for relationship in info.relationships.iter().filter(|r| r.is_implementation) {
                    out_of_root.insert(Fact::new(
                        &info.symbol,
                        "code/implements",
                        &relationship.symbol,
                    ));
                }
            }
            continue;
        }
        let entity = format!("{repo}:{}", document.relative_path);
        for occurrence in &document.occurrences {
            if is_local_symbol(&occurrence.symbol) {
                report.local_symbols += 1;
                local.insert(Fact::new(
                    &entity,
                    occurrence_attribute(occurrence.symbol_roles),
                    &occurrence.symbol,
                ));
                continue;
            }
            let attribute = occurrence_attribute(occurrence.symbol_roles);
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
                    local.insert(Fact::new(
                        &info.symbol,
                        "code/implements",
                        &relationship.symbol,
                    ));
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
    report.out_of_root_facts = out_of_root.len();
    report.local_facts = local.len();
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
    /// `go list -json ./...` output for a Go repo. When given, it names the
    /// files the module builds; when not, a path rule stands in for it.
    pub go_list: Option<&'a Path>,
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
        "kos/names-dep" => fact
            .e
            .strip_prefix(repo)
            .is_some_and(|rest| rest.starts_with(':')),
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

/// The crate names the repo lists, one per line.
fn crate_names(checkout: &Path) -> Result<Vec<String>> {
    let path = checkout.join(".kos/crate-names.txt");
    match fs::read_to_string(&path) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(refuse(format!("cannot read {}: {e}", path.display()))),
    }
}

/// Whether `text` holds `name` bounded by characters that cannot be part of a
/// crate name, so `serde-saphyr` is not found inside `serde-saphyr-extras`.
fn names_whole_word(text: &str, name: &str) -> bool {
    let part_of_name = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    text.match_indices(name).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + name.len()..].chars().next();
        !before.is_some_and(part_of_name) && !after.is_some_and(part_of_name)
    })
}

/// `kos/names-dep` facts: each listed crate name that a node's text names.
fn names_dep_facts(repo: &str, checkout: &Path) -> Result<BTreeSet<Fact>> {
    let names = crate_names(checkout)?;
    let mut facts = BTreeSet::new();
    if names.is_empty() {
        return Ok(facts);
    }
    let unreadable =
        |path: &Path, e: std::io::Error| refuse(format!("cannot read {}: {e}", path.display()));
    // Every file of one directory: the text is read whole, and a file that
    // cannot be read is a refusal.
    let mut scan = |dir: &Path, extensions: &[&str]| -> Result<()> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(unreadable(dir, e)),
        };
        for entry in entries {
            let path = entry.map_err(|e| unreadable(dir, e))?.path();
            let wanted = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| extensions.contains(&e));
            if !wanted || path.is_dir() {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|e| unreadable(&path, e))?;
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            for name in names.iter().filter(|n| names_whole_word(&text, n)) {
                facts.insert(Fact::new(format!("{repo}:{id}"), "kos/names-dep", name));
            }
        }
        Ok(())
    };
    // Nodes live one directory down, by confidence tier.
    let nodes = checkout.join("_kos/nodes");
    match fs::read_dir(&nodes) {
        Ok(tiers) => {
            for tier in tiers {
                let tier = tier.map_err(|e| unreadable(&nodes, e))?.path();
                if tier.is_dir() {
                    scan(&tier, &["yaml"])?;
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(unreadable(&nodes, e)),
    }
    // Findings are YAML or markdown files in one directory.
    scan(&checkout.join("_kos/findings"), &["yaml", "md"])?;
    Ok(facts)
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

/// The files `go list -json ./...` says the module builds, as paths relative
/// to the module root. The output is a stream of one JSON object per package.
fn go_list_files(path: &Path) -> std::result::Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("cannot read the go list output {}: {e}", path.display()))?;
    let bad = |why: &str| format!("the go list output {} is not usable: {why}", path.display());
    // The recipe appends `exit: <n>` after go list's output, the way it does
    // for scip-go's log, so a partial list is not trusted.
    let trimmed = text.trim_end();
    let (json, last) = trimmed.rsplit_once('\n').unwrap_or(("", trimmed));
    match last.trim() {
        "exit: 0" => {}
        other if other.starts_with("exit: ") => {
            return Err(format!("go list did not succeed (the list ends `{other}`)"));
        }
        _ => {
            return Err(bad(
                "it does not end with go list's `exit: <n>` line, which the recipe appends",
            ));
        }
    }
    let mut files = BTreeSet::new();
    let mut packages = 0;
    for value in serde_json::Deserializer::from_str(json).into_iter::<serde_json::Value>() {
        let value = value.map_err(|e| bad(&e.to_string()))?;
        let text_of =
            |v: Option<&serde_json::Value>| v.and_then(|v| v.as_str()).map(str::to_string);
        let (Some(dir), Some(root)) = (
            text_of(value.get("Dir")),
            text_of(value.get("Module").and_then(|m| m.get("Dir"))),
        ) else {
            return Err(bad("a package has no Dir or Module.Dir"));
        };
        let Some(rel) = Path::new(&dir).strip_prefix(&root).ok() else {
            continue;
        };
        packages += 1;
        for key in ["GoFiles", "CgoFiles"] {
            for name in value
                .get(key)
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                files.insert(rel.join(name).to_string_lossy().into_owned());
            }
        }
    }
    if packages == 0 {
        return Err(bad("it holds no package of the module"));
    }
    Ok(files)
}

/// The member patterns of the root `Cargo.toml`'s `[workspace]` table: the
/// quoted strings of its `members` array. This is a line reader, not a TOML
/// parser, so a `members` written some other way reads as no members, and the
/// check then covers the root crate alone. `exclude` is not read.
fn workspace_members(manifest: &str) -> Vec<String> {
    let mut in_workspace = false;
    let mut in_members = false;
    let mut members = Vec::new();
    for line in manifest.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_workspace = line == "[workspace]";
            in_members = false;
            continue;
        }
        if !in_workspace {
            continue;
        }
        let mut rest = line;
        if !in_members {
            let Some(after) = line
                .strip_prefix("members")
                .map(str::trim_start)
                .and_then(|r| r.strip_prefix('='))
            else {
                continue;
            };
            rest = after.trim_start();
            if !rest.starts_with('[') {
                continue;
            }
            in_members = true;
        }
        let mut quoted = rest.split('"');
        quoted.next();
        members.extend(quoted.step_by(2).map(str::to_string));
        if rest.contains(']') {
            in_members = false;
        }
    }
    members
}

/// Whether `path` is a source file of the root crate or of a workspace member.
/// A member is a plain directory, or `dir/*` for each directory under `dir`.
fn is_rust_source(path: &str, members: &[String]) -> bool {
    if !path.ends_with(".rs") {
        return false;
    }
    path.starts_with("src/")
        || members.iter().any(|m| match m.strip_suffix("/*") {
            Some(dir) => path
                .strip_prefix(dir)
                .and_then(|p| p.strip_prefix('/'))
                .and_then(|p| p.split_once('/'))
                .is_some_and(|(_, rest)| rest.starts_with("src/")),
            None => path
                .strip_prefix(m.trim_end_matches('/'))
                .and_then(|p| p.strip_prefix("/src/"))
                .is_some(),
        })
}

/// The reasons `index` is degraded, empty when it is healthy.
fn degraded_reasons(
    index: &Index,
    log: &str,
    tracked: &[String],
    go_files: Option<&BTreeSet<String>>,
    members: &[String],
) -> std::result::Result<Vec<String>, String> {
    let documents: BTreeSet<&str> = index
        .documents
        .iter()
        .map(|d| d.relative_path.as_str())
        .collect();
    let missing = |wanted: &dyn Fn(&str) -> bool| -> Vec<&str> {
        tracked
            .iter()
            .map(String::as_str)
            .filter(|p| wanted(p) && !documents.contains(p))
            .collect()
    };
    let mut reasons = Vec::new();
    if index.metadata.tool_info.version.trim().is_empty() {
        reasons.push("the index names no indexer version".to_string());
    }
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
            for path in missing(&|p| is_rust_source(p, members)) {
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
            if let Some(built) = go_files {
                for path in built.iter().filter(|p| !documents.contains(p.as_str())) {
                    reasons.push(format!(
                        "the index has no document for {path}, which go list says the module builds"
                    ));
                }
            } else {
                for path in missing(&is_go_source) {
                    reasons.push(format!("the index has no document for tracked file {path}"));
                }
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
    let go_files = match request.go_list {
        Some(_) if index.metadata.tool_info.name != "scip-go" => {
            return Err(gap(&format!(
                "--golist is for an index scip-go wrote, and this one was written by `{}`",
                index.metadata.tool_info.name
            )));
        }
        Some(path) => Some(go_list_files(path).map_err(|e| gap(&e))?),
        None => None,
    };
    let manifest = fs::read_to_string(request.checkout.join("Cargo.toml")).unwrap_or_default();
    match degraded_reasons(
        index,
        &log,
        &tracked,
        go_files.as_ref(),
        &workspace_members(&manifest),
    ) {
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
    if request.repo.is_empty() || request.repo.contains([':', '@']) {
        return Err(refuse(format!(
            "--repo {:?} must be non-empty and hold no ':' or '@', which delimit an entity and a load entity",
            request.repo
        )));
    }
    let (index, sha256) = read_index_bytes(request.index)?;
    check_healthy(request, &index)?;
    let (mut facts, mut report) = facts_from_index(request.repo, &index);
    facts.extend(names_dep_facts(request.repo, request.checkout)?);
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

    // One transaction: a failed write keeps none of the load.
    let mut asserted: Vec<Fact> = facts.into_iter().collect();
    asserted.extend(load_facts(request, &index, &sha256));
    let done = store.apply(version, &stale, &asserted)?;
    report.ended = done.retracted;
    report.added = done.asserted;
    Ok(report)
}
