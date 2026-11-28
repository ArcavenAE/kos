//! The read verbs `kos facts refs | impls | dependents` (aae-orc I-853).
//!
//! Each verb resolves a query to one symbol (or package) in the store, then
//! takes one backward step of `FactStore::traverse` over one attribute. The
//! query is the whole name or its tail, which must start after a space or a
//! `/`, so a descriptor such as `FactStore#traverse().` is enough.

use std::io::{self, BufWriter, Write};

use serde_json::{Map, Value, json};

use super::cli::{ReadArgs, escape, open_existing, refuse};
use super::{FactStore, Step, Version};
use crate::error::Result;

/// How many candidates an ambiguity refusal lists.
const CANDIDATES_SHOWN: usize = 10;

/// What one verb reads.
pub(super) struct Verb {
    /// What the query names: "symbol" or "package".
    noun: &'static str,
    /// Where the query is resolved: the attributes whose values it may name.
    resolve_in: &'static [&'static str],
    /// The attribute followed backward from the resolved name.
    edge: &'static str,
    /// The JSON key of each result.
    item: &'static str,
    /// What a refusal tells the user to write.
    example: &'static str,
}

pub(super) const REFS: Verb = Verb {
    noun: "symbol",
    resolve_in: &["code/defines", "code/references", "code/implements"],
    edge: "code/references",
    item: "document",
    example: "FactStore#traverse().",
};

pub(super) const IMPLS: Verb = Verb {
    noun: "symbol",
    resolve_in: &["code/defines", "code/references", "code/implements"],
    edge: "code/implements",
    item: "implementer",
    example: "Reader#",
};

pub(super) const DEPENDENTS: Verb = Verb {
    noun: "package",
    resolve_in: &["code/depends_on"],
    edge: "code/depends_on",
    item: "repo",
    example: "serde 1.0",
};

/// The version to read: `--at` capped at the latest, or the latest.
fn read_version(store: &FactStore, db: &std::path::Path, at: Option<i64>) -> Result<Version> {
    let (Some(first), Some(latest)) = (store.first_version()?, store.latest_version()?) else {
        return Err(refuse(
            format!("{} holds no facts", db.display()),
            "`kos facts load`, `kos facts ingest-scip` or `kos facts import` adds them",
        ));
    };
    match at {
        None => Ok(latest),
        Some(n) if n < first.get() => Err(refuse(
            format!(
                "{} holds no facts at version {n} (versions {} to {})",
                db.display(),
                first.get(),
                latest.get()
            ),
            "pass an --at inside that range, or leave it out to read the latest version",
        )),
        Some(n) => Ok(Version::new(n.min(latest.get()))),
    }
}

fn resolve(verb: &Verb, store: &FactStore, version: Version, query: &str) -> Result<String> {
    let mut found = store.symbols_matching(version, verb.resolve_in, query)?;
    let noun = verb.noun;
    match found.len() {
        0 => Err(refuse(
            format!(
                "no {noun} matches \"{}\" at version {}",
                escape(query),
                version.get()
            ),
            format!(
                "write the whole {noun} or its end, which must start after a space or a `/`, for example `{}`",
                verb.example
            ),
        )),
        1 => Ok(found.remove(0)),
        n => {
            let mut message = format!(
                "\"{}\" matches {n} {noun}s at version {}:",
                escape(query),
                version.get()
            );
            for candidate in found.iter().take(CANDIDATES_SHOWN) {
                message.push_str("\n  ");
                message.push_str(&escape(candidate));
            }
            if n > CANDIDATES_SHOWN {
                message.push_str(&format!("\n  (and {} more)", n - CANDIDATES_SHOWN));
            }
            Err(refuse(
                message,
                format!(
                    "write more of the {noun}, starting at an earlier space or `/`, until one matches"
                ),
            ))
        }
    }
}

pub(super) fn run(verb: &Verb, args: &ReadArgs<'_>) -> Result<()> {
    let store = open_existing(args.db)?;
    let version = read_version(&store, args.db, args.at)?;
    let name = resolve(verb, &store, version, args.query)?;
    let hops = store.traverse(
        version,
        std::slice::from_ref(&name),
        &[Step::backward(verb.edge)],
    )?;
    let all: Vec<String> = hops
        .into_iter()
        .next()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let count = all.len();
    // --limit 0 means no cap.
    let shown = if args.limit == 0 {
        count
    } else {
        count.min(args.limit)
    };
    let truncated = shown < count;
    let mut w = BufWriter::new(io::stdout().lock());
    if args.json {
        for item in &all[..shown] {
            writeln!(w, "{}", json!({ verb.item: item }))?;
        }
        let mut trailer = Map::new();
        trailer.insert("query".into(), Value::from(args.query));
        trailer.insert(verb.noun.into(), Value::from(name));
        trailer.insert("at".into(), Value::from(version.get()));
        trailer.insert("count".into(), Value::from(count));
        trailer.insert("truncated".into(), Value::from(truncated));
        writeln!(w, "{}", Value::Object(trailer))?;
        w.flush()?;
        return Ok(());
    }
    for item in &all[..shown] {
        writeln!(w, "{}", escape(item))?;
    }
    w.flush()?;
    let results = if truncated {
        format!("showing {shown} of {count} results")
    } else if count == 1 {
        "1 result".to_string()
    } else {
        format!("{count} results")
    };
    eprintln!(
        "{} {} at version {}: {results}",
        verb.noun,
        escape(&name),
        version.get()
    );
    Ok(())
}
