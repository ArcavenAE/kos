//! The `kos facts` commands (aae-orc-xxe3n, PR 3).
//!
//! Text output is for people and escapes a backslash, a tab and a newline in
//! every field as `\\`, `\t` and `\n`. `--json` output is the exact form, one
//! JSON object per line. A command that reads a store never creates one.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

use serde_json::json;

use super::{Fact, FactStore, Harvest, Version};
use crate::error::{KosError, Result};

/// A refusal of the command itself. The hint is a required argument, so no
/// refusal built here can leave it out.
fn refuse(message: impl AsRef<str>, hint: impl AsRef<str>) -> KosError {
    KosError::FactCommand {
        message: format!("{}\n  hint: {}", message.as_ref(), hint.as_ref()),
    }
}

/// A SQLite or I/O error has no hint of its own (both are shared with the rest
/// of kos), so each command's result passes through here and gains one.
fn with_hint<T>(result: Result<T>) -> Result<T> {
    result.map_err(|e| match e {
        KosError::Sqlite(_) | KosError::Io(_) => refuse(
            e.to_string(),
            "check that --db names a fact store this kos can read and write, and that the paths are readable and writable",
        ),
        other => other,
    })
}

fn has_extension(path: &Path, wanted: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| wanted.iter().any(|w| e.eq_ignore_ascii_case(w)))
}

/// Backslash, tab and newline, written as `\\`, `\t` and `\n`.
fn escape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    for c in field.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

fn open_existing(db: &Path) -> Result<FactStore> {
    if !db.exists() {
        return Err(refuse(
            format!("no fact store at {}", db.display()),
            "`kos facts load` or `kos facts import` creates one",
        ));
    }
    FactStore::open(db)
}

/// Assert the facts of a harvested-facts YAML file at `at`.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn load(db: &Path, at: i64, file: &Path) -> Result<()> {
    with_hint(load_inner(db, at, file))
}

fn load_inner(db: &Path, at: i64, file: &Path) -> Result<()> {
    if has_extension(file, &["jsonl"]) {
        return Err(refuse(
            format!(
                "{} is a JSONL export, not a harvested-facts file",
                file.display()
            ),
            "use `kos facts import`",
        ));
    }
    let text = fs::read_to_string(file).map_err(|e| {
        refuse(
            format!("cannot read {}: {e}", file.display()),
            "check the path and that the file is readable",
        )
    })?;
    // Parse before opening, so a refused file creates no store.
    let harvest = Harvest::parse(&text)?;
    let mut store = FactStore::open(db)?;
    let added = store.assert_facts(Version::new(at), &harvest.facts)?;
    println!("loaded {added} facts at version {at}");
    Ok(())
}

/// What `kos facts ingest-scip` was given. The indexer log and the checkout
/// are inputs of the degraded-index check, which a later change adds.
pub struct IngestScip<'a> {
    pub db: &'a Path,
    pub at: i64,
    pub repo: &'a str,
    pub commit: &'a str,
    pub index: &'a Path,
    pub log: &'a Path,
    pub checkout: &'a Path,
}

/// Load the facts of a SCIP index for one repo at version `at`.
///
/// # Errors
/// Fails on an unreadable index or a store error.
pub fn ingest_scip(args: &IngestScip<'_>) -> Result<()> {
    let report = with_hint(super::scip_ingest::ingest(
        args.db, args.at, args.repo, args.index,
    ))?;
    println!(
        "loaded {} facts for {}@{} at version {}",
        report.added, args.repo, args.commit, args.at
    );
    println!(
        "filtered: out_of_root_documents={} local_symbols={}",
        report.out_of_root_documents, report.local_symbols
    );
    Ok(())
}

/// Replay a JSONL export into an empty store.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn import(db: &Path, file: &Path) -> Result<()> {
    with_hint(import_inner(db, file))
}

fn import_inner(db: &Path, file: &Path) -> Result<()> {
    if has_extension(file, &["yaml", "yml"]) {
        return Err(refuse(
            format!(
                "{} is a harvested-facts file, not a JSONL export",
                file.display()
            ),
            "use `kos facts load`",
        ));
    }
    let read = || {
        File::open(file).map(BufReader::new).map_err(|e| {
            refuse(
                format!("cannot read {}: {e}", file.display()),
                "check the path and that the file is readable",
            )
        })
    };
    // Check the whole file against a private in-memory store first, so a
    // refused file creates no store at --db, as `load` does.
    FactStore::open_in_memory()?.import_jsonl(read()?)?;
    let mut store = FactStore::open(db)?;
    let rows = store.import_jsonl(read()?)?;
    println!("imported {rows} rows");
    Ok(())
}

/// Write the whole history as JSONL to stdout or to a new file.
///
/// # Errors
/// Fails when `out` exists, or on a store or write error.
pub fn export(db: &Path, out: Option<&Path>) -> Result<()> {
    with_hint(export_inner(db, out))
}

fn export_inner(db: &Path, out: Option<&Path>) -> Result<()> {
    let store = open_existing(db)?;
    match out {
        None => {
            let mut w = BufWriter::new(io::stdout().lock());
            store.export_jsonl(&mut w)?;
            w.flush()?;
        }
        Some(path) => {
            // create_new never overwrites: the check and the create are one call.
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| {
                    if e.kind() == io::ErrorKind::AlreadyExists {
                        refuse(
                            format!("{} already exists", path.display()),
                            "choose a new file name; export never overwrites",
                        )
                    } else {
                        refuse(
                            format!("cannot create {}: {e}", path.display()),
                            "check that the directory exists and is writable",
                        )
                    }
                })?;
            let mut w = BufWriter::new(file);
            store.export_jsonl(&mut w)?;
            w.flush()?;
        }
    }
    Ok(())
}

fn text_row(sign: Option<char>, fact: &Fact) -> String {
    let fields = format!(
        "{}\t{}\t{}",
        escape(&fact.e),
        escape(&fact.a),
        escape(&fact.v)
    );
    match sign {
        Some(s) => format!("{s}\t{fields}"),
        None => fields,
    }
}

/// Print the facts that hold at `at`.
///
/// # Errors
/// Fails on a store error.
pub fn as_of(
    db: &Path,
    at: i64,
    entity: Option<&str>,
    attribute: Option<&str>,
    json: bool,
) -> Result<()> {
    with_hint(as_of_inner(db, at, entity, attribute, json))
}

fn as_of_inner(
    db: &Path,
    at: i64,
    entity: Option<&str>,
    attribute: Option<&str>,
    json: bool,
) -> Result<()> {
    let store = open_existing(db)?;
    let mut w = BufWriter::new(io::stdout().lock());
    for fact in store.facts_as_of(Version::new(at))? {
        if entity.is_some_and(|e| e != fact.e) || attribute.is_some_and(|a| a != fact.a) {
            continue;
        }
        if json {
            writeln!(w, "{}", json!({"e": fact.e, "a": fact.a, "v": fact.v}))?;
        } else {
            writeln!(w, "{}", text_row(None, &fact))?;
        }
    }
    w.flush()?;
    Ok(())
}

/// Print what changed from `from` to `to`: removed facts first, then added,
/// each group sorted.
///
/// # Errors
/// Fails on a store error.
pub fn diff(db: &Path, from: i64, to: i64, json: bool) -> Result<()> {
    with_hint(diff_inner(db, from, to, json))
}

fn diff_inner(db: &Path, from: i64, to: i64, json: bool) -> Result<()> {
    let store = open_existing(db)?;
    let changes = store.diff(Version::new(from), Version::new(to))?;
    let mut w = BufWriter::new(io::stdout().lock());
    for (op, sign, facts) in [
        ("removed", '-', &changes.removed),
        ("added", '+', &changes.added),
    ] {
        for fact in facts {
            if json {
                writeln!(
                    w,
                    "{}",
                    json!({"op": op, "e": fact.e, "a": fact.a, "v": fact.v})
                )?;
            } else {
                writeln!(w, "{}", text_row(Some(sign), fact))?;
            }
        }
    }
    w.flush()?;
    Ok(())
}
