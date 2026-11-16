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

fn refuse(message: impl Into<String>) -> KosError {
    KosError::FactCommand {
        message: message.into(),
    }
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
        return Err(refuse(format!(
            "no fact store at {}\n  hint: `kos facts load` or `kos facts import` creates one",
            db.display()
        )));
    }
    FactStore::open(db)
}

/// Assert the facts of a harvested-facts YAML file at `at`.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn load(db: &Path, at: i64, file: &Path) -> Result<()> {
    if has_extension(file, &["jsonl"]) {
        return Err(refuse(format!(
            "{} is a JSONL export, not a harvested-facts file\n  hint: use `kos facts import`",
            file.display()
        )));
    }
    let text = fs::read_to_string(file)
        .map_err(|e| refuse(format!("cannot read {}: {e}", file.display())))?;
    // Parse before opening, so a refused file creates no store.
    let harvest = Harvest::parse(&text)?;
    let mut store = FactStore::open(db)?;
    let added = store.assert_facts(Version::new(at), &harvest.facts)?;
    println!("loaded {added} facts at version {at}");
    Ok(())
}

/// Replay a JSONL export into an empty store.
///
/// # Errors
/// Fails on a refused file or a store error.
pub fn import(db: &Path, file: &Path) -> Result<()> {
    if has_extension(file, &["yaml", "yml"]) {
        return Err(refuse(format!(
            "{} is a harvested-facts file, not a JSONL export\n  hint: use `kos facts load`",
            file.display()
        )));
    }
    let input =
        File::open(file).map_err(|e| refuse(format!("cannot read {}: {e}", file.display())))?;
    let mut store = FactStore::open(db)?;
    let rows = store.import_jsonl(BufReader::new(input))?;
    println!("imported {rows} rows");
    Ok(())
}

/// Write the whole history as JSONL to stdout or to a new file.
///
/// # Errors
/// Fails when `out` exists, or on a store or write error.
pub fn export(db: &Path, out: Option<&Path>) -> Result<()> {
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
                        refuse(format!(
                            "{} already exists\n  hint: choose a new file name; export never overwrites",
                            path.display()
                        ))
                    } else {
                        refuse(format!("cannot create {}: {e}", path.display()))
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
