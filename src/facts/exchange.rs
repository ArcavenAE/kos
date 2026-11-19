//! Moving facts in and out of the store (aae-orc-xxe3n, PR 2).
//!
//! - JSONL export and import of the whole history. Line 1 is a header,
//!   `{"kos_facts_export":1,"schema_version":1,"latest_version":N}`, and one
//!   row per fact interval follows, sorted by entity, attribute, value and
//!   start version. The header carries `latest_version` because the rows alone
//!   cannot rebuild it: a write that changes no row still moves it, and writes
//!   may not go behind it.
//! - Import is a writer, so it validates strictly and refuses a bad file with
//!   a typed error. It reads into an empty store only.
//! - Harvest: a chosen set of facts, as they hold at one version, written as a
//!   `harvested-facts` YAML file, and the matching read and apply. Unknown
//!   extra keys in that file are ignored, so a later optional key does not
//!   break an older reader.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Fact, FactStore, Version};
use crate::error::{KosError, Result};

/// Number written under `kos_facts_export`. Readers refuse a number they do
/// not know.
const EXPORT_FORMAT: i64 = 1;

/// The `kind` value of a harvest file.
const HARVEST_KIND: &str = "harvested-facts";

/// What a harvested-facts file carries: a caller-supplied source map and the
/// facts, in file order. A file this module writes has them sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Harvest {
    pub source: BTreeMap<String, String>,
    pub facts: Vec<Fact>,
}

/// A scalar read from a source map: YAML may carry a number or a boolean where
/// the writer put text.
#[derive(Deserialize)]
#[serde(untagged)]
enum Scalar {
    Text(String),
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl Scalar {
    fn into_text(self) -> String {
        match self {
            Self::Text(s) => s,
            Self::Int(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Float(x) => x.to_string(),
        }
    }
}

#[derive(Deserialize)]
struct RawHarvest {
    kind: Option<String>,
    source: Option<BTreeMap<String, Scalar>>,
    #[serde(default)]
    facts: Vec<RawFact>,
}

#[derive(Deserialize)]
struct RawFact {
    e: String,
    a: String,
    v: String,
}

#[derive(Serialize)]
struct OutHarvest<'a> {
    kind: &'static str,
    source: &'a BTreeMap<String, String>,
    facts: Vec<OutFact<'a>>,
}

#[derive(Serialize)]
struct OutFact<'a> {
    e: &'a str,
    a: &'a str,
    v: &'a str,
}

fn format_error(message: impl Into<String>) -> KosError {
    KosError::FactHarvestFormat {
        message: message.into(),
    }
}

impl Harvest {
    /// Parse a harvested-facts YAML file.
    ///
    /// # Errors
    /// Fails when the text is not YAML, has no `kind: harvested-facts`, has no
    /// `source` or a `source` without `repo` and `commit`, or has a fact
    /// without all of `e`, `a` and `v`.
    pub fn parse(yaml: &str) -> Result<Self> {
        let raw: RawHarvest =
            crate::yaml::from_str(yaml).map_err(|e| format_error(e.to_string()))?;
        match raw.kind.as_deref() {
            Some(HARVEST_KIND) => {}
            Some(other) => {
                return Err(format_error(format!(
                    "kind is {other:?}, this kos reads only {HARVEST_KIND:?}"
                )));
            }
            None => {
                return Err(format_error(format!(
                    "no kind; expected `kind: {HARVEST_KIND}`"
                )));
            }
        }
        let Some(raw_source) = raw.source else {
            return Err(format_error(
                "no source; a harvest file needs source.repo and source.commit",
            ));
        };
        let source: BTreeMap<String, String> = raw_source
            .into_iter()
            .map(|(k, v)| (k, v.into_text()))
            .collect();
        require_provenance(&source)?;
        let facts: Vec<Fact> = raw
            .facts
            .into_iter()
            .map(|f| Fact::new(f.e, f.a, f.v))
            .collect();
        refuse_code_facts(&facts)?;
        Ok(Self { source, facts })
    }
}

/// A fact of a SCIP load: a `code/` attribute, or an entity under `load/`.
/// A harvest file carries a set of facts at one version and drops every
/// interval and four of the six provenance values, so code facts move only as
/// JSONL (SI-10).
fn is_code_fact(fact: &Fact) -> bool {
    fact.a.starts_with("code/") || fact.e.starts_with("load/")
}

fn refuse_code_facts(facts: &[Fact]) -> Result<()> {
    match facts.iter().find(|f| is_code_fact(f)) {
        Some(f) => Err(KosError::FactHarvestCodeFact {
            e: f.e.clone(),
            a: f.a.clone(),
        }),
        None => Ok(()),
    }
}

/// A harvest's source is its provenance: it must name the repo and the
/// commit. The writer and the reader hold the same rule.
fn require_provenance(source: &BTreeMap<String, String>) -> Result<()> {
    for key in ["repo", "commit"] {
        if source.get(key).is_none_or(|v| v.trim().is_empty()) {
            return Err(format_error(format!("source needs a non-empty {key}")));
        }
    }
    Ok(())
}

impl FactStore {
    /// Write the header, then every fact interval as one JSON line, sorted by
    /// entity, attribute, value and start version.
    ///
    /// # Errors
    /// Fails on a SQLite or write error.
    pub fn export_jsonl(&self, out: &mut impl Write) -> Result<()> {
        let latest = self.latest_version()?.map(Version::get);
        let header = json!({
            "kos_facts_export": EXPORT_FORMAT,
            "schema_version": super::SCHEMA_VERSION,
            "latest_version": latest,
        });
        writeln!(out, "{header}")?;
        let mut stmt = self.conn.prepare(
            "SELECT e, a, v, valid_from, valid_to FROM facts ORDER BY e, a, v, valid_from",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let line = json!({
                "e": row.get::<_, String>(0)?,
                "a": row.get::<_, String>(1)?,
                "v": row.get::<_, String>(2)?,
                "valid_from": row.get::<_, i64>(3)?,
                "valid_to": row.get::<_, Option<i64>>(4)?,
            });
            writeln!(out, "{line}")?;
        }
        Ok(())
    }

    /// Replay an export into this store, which must be empty. Returns the
    /// number of rows read. A refused file leaves the store empty.
    ///
    /// # Errors
    /// Fails when the store holds facts, when the header is missing or not one
    /// this kos knows, or on the first row that is malformed or breaks a rule,
    /// naming its line.
    pub fn import_jsonl(&mut self, input: impl BufRead) -> Result<usize> {
        if self.latest_version()?.is_some() {
            return Err(KosError::FactImportNotEmpty);
        }
        let mut lines = input.lines();
        let first = lines.next().transpose()?;
        let latest = read_header(first.as_deref())?;
        let tx = self.conn.transaction()?;
        let mut count = 0;
        let mut previous: Option<Row> = None;
        for (index, text) in lines.enumerate() {
            let line = index + 2;
            let text = text?;
            let row = parse_row(&text, line)?;
            check_row(&row, previous.as_ref(), latest, line)?;
            tx.execute(
                "INSERT INTO facts (e, a, v, valid_from, valid_to) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![row.fact.e, row.fact.a, row.fact.v, row.from, row.to],
            )?;
            previous = Some(row);
            count += 1;
        }
        if let Some(latest) = latest {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![super::LATEST_KEY, latest],
            )?;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Write the given facts, which must hold at `version`, as a
    /// harvested-facts YAML file. `source` must carry `repo` and `commit`; it
    /// may carry the version the facts were read at, as provenance only.
    ///
    /// # Errors
    /// Fails when `source` lacks `repo` or `commit`, or when a fact does not
    /// hold at `version`.
    pub fn harvest(
        &self,
        version: Version,
        picks: &[Fact],
        source: &BTreeMap<String, String>,
    ) -> Result<String> {
        require_provenance(source)?;
        refuse_code_facts(picks)?;
        let mut facts: Vec<&Fact> = picks.iter().collect();
        facts.sort();
        facts.dedup();
        let mut stmt = self.conn.prepare_cached(concat!(
            "SELECT EXISTS (SELECT 1 FROM facts WHERE e = ?1 AND a = ?2 AND v = ?3 AND ",
            live_at!("4"),
            ")"
        ))?;
        for fact in &facts {
            let holds: bool =
                stmt.query_row(params![fact.e, fact.a, fact.v, version.0], |row| row.get(0))?;
            if !holds {
                return Err(KosError::FactHarvestNotLive {
                    e: fact.e.clone(),
                    a: fact.a.clone(),
                    v: fact.v.clone(),
                    version: version.0,
                });
            }
        }
        let out = OutHarvest {
            kind: HARVEST_KIND,
            source,
            facts: facts
                .iter()
                .map(|f| OutFact {
                    e: &f.e,
                    a: &f.a,
                    v: &f.v,
                })
                .collect(),
        };
        serde_saphyr::to_string(&out).map_err(|e| format_error(e.to_string()))
    }

    /// Assert the facts of a harvested-facts file at `version`. The version in
    /// the file's source, if any, is provenance and is not used. Returns how
    /// many facts were newly asserted.
    ///
    /// # Errors
    /// Fails on a malformed file or when `version` is behind the latest.
    pub fn apply_harvest(&mut self, version: Version, yaml: &str) -> Result<usize> {
        let harvest = Harvest::parse(yaml)?;
        self.assert_facts(version, &harvest.facts)
    }
}

/// One import row.
struct Row {
    fact: Fact,
    from: i64,
    to: Option<i64>,
}

fn header_error(message: impl Into<String>) -> KosError {
    KosError::FactImportHeader {
        message: message.into(),
    }
}

fn line_error(line: usize, message: impl Into<String>) -> KosError {
    KosError::FactImportLine {
        line,
        message: message.into(),
    }
}

/// Check line 1 and return its `latest_version`.
fn read_header(first: Option<&str>) -> Result<Option<i64>> {
    let Some(text) = first else {
        return Err(header_error("the file is empty; line 1 must be the header"));
    };
    let value: Value =
        serde_json::from_str(text).map_err(|e| header_error(format!("line 1 is not JSON: {e}")))?;
    let number = value.get("kos_facts_export").and_then(Value::as_i64);
    match number {
        Some(EXPORT_FORMAT) => {}
        Some(other) => {
            return Err(header_error(format!(
                "kos_facts_export is {other}, this kos reads {EXPORT_FORMAT}"
            )));
        }
        None => {
            return Err(header_error(
                "line 1 has no kos_facts_export number, so it is not a header",
            ));
        }
    }
    match value.get("schema_version").and_then(Value::as_i64) {
        Some(n) if n <= super::SCHEMA_VERSION => {}
        Some(n) => {
            return Err(header_error(format!(
                "schema_version is {n}, this kos reads up to {}",
                super::SCHEMA_VERSION
            )));
        }
        None => return Err(header_error("the header has no schema_version")),
    }
    match value.get("latest_version") {
        Some(Value::Null) => Ok(None),
        Some(other) => other
            .as_i64()
            .map(Some)
            .ok_or_else(|| header_error("latest_version must be an integer or null")),
        None => Err(header_error("the header has no latest_version")),
    }
}

fn parse_row(text: &str, line: usize) -> Result<Row> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| line_error(line, format!("not JSON: {e}")))?;
    let field = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| line_error(line, format!("{name} must be a string")))
    };
    let from = value
        .get("valid_from")
        .and_then(Value::as_i64)
        .ok_or_else(|| line_error(line, "valid_from must be an integer"))?;
    let to = match value.get("valid_to") {
        Some(Value::Null) => None,
        Some(other) => Some(
            other
                .as_i64()
                .ok_or_else(|| line_error(line, "valid_to must be an integer or null"))?,
        ),
        None => return Err(line_error(line, "valid_to is missing")),
    };
    Ok(Row {
        fact: Fact::new(field("e")?, field("a")?, field("v")?),
        from,
        to,
    })
}

/// The import rules: one interval is well formed, it ends no later than
/// `latest_version`, and it follows the previous row in the stated order
/// without overlapping an earlier interval of the same fact.
fn check_row(row: &Row, previous: Option<&Row>, latest: Option<i64>, line: usize) -> Result<()> {
    if let Some(to) = row.to
        && to <= row.from
    {
        return Err(line_error(
            line,
            format!(
                "valid_to must be greater than valid_from, got {} to {to}",
                row.from
            ),
        ));
    }
    match latest {
        None => {
            return Err(line_error(
                line,
                "the header's latest_version is null, so the file may hold no rows",
            ));
        }
        Some(latest) => {
            if row.to.is_some_and(|to| to > latest) || row.from > latest {
                return Err(line_error(
                    line,
                    format!("interval ends or starts after latest_version {latest}"),
                ));
            }
        }
    }
    let Some(prev) = previous else {
        return Ok(());
    };
    if row.fact < prev.fact {
        return Err(line_error(
            line,
            "rows out of order: expected sorted by e, a, v, valid_from",
        ));
    }
    if row.fact == prev.fact {
        match prev.to {
            None => {
                return Err(line_error(
                    line,
                    "more than one live interval for the same fact",
                ));
            }
            Some(_) if row.from < prev.from => {
                return Err(line_error(
                    line,
                    "rows out of order: expected sorted by e, a, v, valid_from",
                ));
            }
            Some(end) if row.from < end => {
                return Err(line_error(
                    line,
                    "interval overlaps the previous interval of the same fact",
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}
