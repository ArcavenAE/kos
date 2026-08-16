#![forbid(unsafe_code)]

//! Read telemetry: which graph IDs a read verb actually served.
//!
//! The graph records what was written and never recorded what was read, so
//! every read-side decision (ranking, retirement, promotion evidence) rests
//! on intuition. This module logs IDs only, per session, append-only, into
//! the graph it describes. Nothing reads the output in any check path, and
//! no code path may fail because a write here failed.
//!
//! See `docs/proposals/read-path-first-steps.md`.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Per-graph directory holding the session logs. Gitignored.
const TELEMETRY_DIR: &str = ".telemetry";

/// Any non-empty value disables collection (the `NO_COLOR` convention).
const OPT_OUT_ENV: &str = "KOS_NO_TELEMETRY";

/// Groups one agent session's reads into one file across invocations.
const SESSION_ENV: &str = "KOS_SESSION";

/// Session ids reach the filesystem, and filenames cap at 255 bytes on the
/// platforms kos ships to.
const SESSION_ID_MAX: usize = 64;

/// How a read was served, decided at log time.
///
/// The circulation hypothesis (what share of nodes is never read after
/// creation) can only be tested against consultation reads: an unfiltered
/// orient marks essentially the whole graph as read by construction, so
/// counting it would answer the question before asking it. Pre-registered
/// before collection began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadClass {
    /// The served set was narrowed by what the caller asked for.
    Consultation,
    /// The served set is the graph (or a whole tier of it) by construction.
    Bulk,
}

impl ReadClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Consultation => "consultation",
            Self::Bulk => "bulk",
        }
    }
}

/// One invocation of a read verb: what it served, never what it contained.
#[derive(Debug)]
pub struct ReadEvent<'a> {
    /// The verb as the user typed it (`orient`, `orient-ready`).
    pub verb: &'a str,
    /// Target repo the read was scoped to.
    pub target: &'a str,
    pub read_class: ReadClass,
    /// Whether the caller asked for `--json` (agent reads vs human reads).
    pub json_output: bool,
    /// Graph node IDs served. Charter items, probes, ideas, and RD artifacts
    /// are excluded: the hypothesis is about nodes.
    pub node_ids: Vec<&'a str>,
    /// Finding IDs served.
    pub finding_ids: Vec<&'a str>,
}

/// Whether collection is on. On by default; checked at the call site.
pub fn enabled() -> bool {
    enabled_from(std::env::var_os(OPT_OUT_ENV).as_deref())
}

fn enabled_from(opt_out: Option<&OsStr>) -> bool {
    !opt_out.is_some_and(|value| !value.is_empty())
}

/// This process's session id: `KOS_SESSION` when set, else `<pid>-<unix secs>`.
///
/// The derived form is fixed on first use so every read in one invocation
/// lands in one file even if the process straddles a second boundary.
pub fn session_id() -> String {
    if let Some(explicit) = std::env::var_os(SESSION_ENV) {
        let sanitized = sanitize(&explicit.to_string_lossy());
        if !sanitized.is_empty() {
            return sanitized;
        }
    }

    static DERIVED: OnceLock<String> = OnceLock::new();
    DERIVED
        .get_or_init(|| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since_epoch| since_epoch.as_secs());
            format!("{}-{secs}", std::process::id())
        })
        .clone()
}

/// Reduce a session id to something safe to use as a filename. The value
/// arrives from the environment, so a path separator or `..` would otherwise
/// steer the log out of the graph.
fn sanitize(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .take(SESSION_ID_MAX)
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();

    if cleaned.chars().all(|c| c == '.') {
        String::new()
    } else {
        cleaned
    }
}

fn log_path(graph_dir: &Path, session: &str) -> PathBuf {
    graph_dir
        .join(TELEMETRY_DIR)
        .join(format!("reads-{session}.jsonl"))
}

/// Append one JSONL line describing `event` to this session's log.
///
/// Callers treat the error as advisory and keep going: telemetry is
/// diagnostic and must never fail the verb it observes.
pub fn record_reads(graph_dir: &Path, event: &ReadEvent<'_>) -> std::io::Result<()> {
    record_reads_as(graph_dir, &session_id(), event)
}

/// The session id is a parameter here so tests can pin it. Setting the
/// environment from a test is not available under edition 2024 without
/// `unsafe`, which this crate forbids.
fn record_reads_as(graph_dir: &Path, session: &str, event: &ReadEvent<'_>) -> std::io::Result<()> {
    std::fs::create_dir_all(graph_dir.join(TELEMETRY_DIR))?;

    let entry = serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "session": session,
        "verb": event.verb,
        "target": event.target,
        "read_class": event.read_class.as_str(),
        "json_output": event.json_output,
        "node_ids": event.node_ids,
        "finding_ids": event.finding_ids,
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path(graph_dir, session))?;
    writeln!(file, "{entry}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> ReadEvent<'static> {
        ReadEvent {
            verb: "orient",
            target: "kos",
            read_class: ReadClass::Bulk,
            json_output: false,
            node_ids: vec!["question-read-telemetry-decision-value", "elem-node-schema"],
            finding_ids: vec!["finding-044"],
        }
    }

    fn read_lines(graph_dir: &Path, session: &str) -> Vec<String> {
        std::fs::read_to_string(log_path(graph_dir, session))
            .unwrap()
            .lines()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn record_writes_valid_jsonl() {
        let graph = tempfile::tempdir().unwrap();
        record_reads_as(graph.path(), "sess-1", &sample_event()).unwrap();

        let lines = read_lines(graph.path(), "sess-1");
        assert_eq!(lines.len(), 1);

        let parsed: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(parsed["session"], "sess-1");
        assert_eq!(parsed["verb"], "orient");
        assert_eq!(parsed["target"], "kos");
        assert_eq!(parsed["read_class"], "bulk");
        assert_eq!(parsed["json_output"], false);
        assert_eq!(parsed["node_ids"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["finding_ids"][0], "finding-044");
        assert!(parsed["ts"].as_str().unwrap().starts_with("20"));
    }

    #[test]
    fn record_appends_rather_than_truncating() {
        let graph = tempfile::tempdir().unwrap();
        record_reads_as(graph.path(), "sess-1", &sample_event()).unwrap();
        record_reads_as(graph.path(), "sess-1", &sample_event()).unwrap();

        assert_eq!(read_lines(graph.path(), "sess-1").len(), 2);
    }

    #[test]
    fn distinct_sessions_write_distinct_files() {
        let graph = tempfile::tempdir().unwrap();
        record_reads_as(graph.path(), "sess-1", &sample_event()).unwrap();
        record_reads_as(graph.path(), "sess-2", &sample_event()).unwrap();

        assert_eq!(read_lines(graph.path(), "sess-1").len(), 1);
        assert_eq!(read_lines(graph.path(), "sess-2").len(), 1);
    }

    #[test]
    fn record_surfaces_the_error_on_an_unwritable_dir() {
        // A file where the graph directory should be: the caller, not this
        // module, decides that a failure here is survivable.
        let parent = tempfile::tempdir().unwrap();
        let graph = parent.path().join("not-a-directory");
        std::fs::write(&graph, "").unwrap();

        assert!(record_reads_as(&graph, "sess-1", &sample_event()).is_err());
    }

    #[test]
    fn opt_out_skips_the_write() {
        let graph = tempfile::tempdir().unwrap();

        // Mirrors the call site: the gate decides, the writer stays pure.
        if enabled_from(Some(OsStr::new("1"))) {
            record_reads_as(graph.path(), "sess-1", &sample_event()).unwrap();
        }

        assert!(!graph.path().join(TELEMETRY_DIR).exists());
    }

    #[test]
    fn opt_out_is_off_when_unset_or_empty() {
        assert!(enabled_from(None));
        assert!(enabled_from(Some(OsStr::new(""))));
        assert!(!enabled_from(Some(OsStr::new("0"))));
    }

    #[test]
    fn session_id_stays_inside_the_telemetry_directory() {
        assert_eq!(sanitize("../../etc/passwd"), "..-..-etc-passwd");
        assert_eq!(sanitize(".."), "");
        assert_eq!(sanitize("agent session 3"), "agent-session-3");
        assert_eq!(sanitize(&"x".repeat(200)).len(), SESSION_ID_MAX);
    }
}
