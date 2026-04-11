#![forbid(unsafe_code)]

use std::path::Path;

use regex::Regex;
use serde::Serialize;

use crate::error::{KosError, Result};
use crate::workspace::Workspace;

/// An RD finding extracted from a sprint/rd/ brief.
#[derive(Debug, Serialize)]
pub struct RdFinding {
    /// e.g. "F-01" or "F1"
    pub id: String,
    /// The finding title (without the date suffix)
    pub title: String,
    /// The body text following the title
    pub body: String,
    /// Confidence if stated in the finding text
    pub confidence: Option<String>,
    /// Source brief slug (e.g. "marvel-mvp-probe")
    pub source_brief: String,
    /// Primary repo this finding relates to
    pub source_repo: String,
    /// Date if present in the finding header
    pub date: Option<String>,
}

/// Result of running bridge extraction across all briefs.
#[derive(Debug, Serialize)]
pub struct BridgeResult {
    pub briefs_scanned: usize,
    pub briefs_with_findings: usize,
    pub findings: Vec<RdFinding>,
}

/// Run the bridge subcommand.
pub fn run(workspace: &Workspace, json: bool) -> Result<()> {
    let rd_dir = workspace.root.join("sprint/rd");
    let result = extract_all(&rd_dir)?;

    if json {
        print_jsonl(&result);
    } else {
        print_human(&result);
    }

    Ok(())
}

/// Extract all RD findings from sprint/rd/*.md.
fn extract_all(rd_dir: &Path) -> Result<BridgeResult> {
    if !rd_dir.exists() {
        return Ok(BridgeResult {
            briefs_scanned: 0,
            briefs_with_findings: 0,
            findings: vec![],
        });
    }

    let mut briefs_scanned = 0;
    let mut briefs_with_findings = 0;
    let mut findings = Vec::new();

    let mut entries: Vec<_> = walkdir::WalkDir::new(rd_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|e| e.to_str()) == Some("md")
        })
        .collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let path = entry.path();
        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let source_repo = infer_repo(&slug);
        let brief_findings = extract_findings(&content, &slug, &source_repo);

        briefs_scanned += 1;
        if !brief_findings.is_empty() {
            briefs_with_findings += 1;
        }
        findings.extend(brief_findings);
    }

    Ok(BridgeResult {
        briefs_scanned,
        briefs_with_findings,
        findings,
    })
}

/// Extract numbered findings from a single brief's markdown content.
///
/// Matches two patterns:
/// 1. `**F-01: Title.**` (marvel-mvp-probe style, dash in ID)
/// 2. `**F1: Title (date)**` (forestage-distribution style, no dash)
pub fn extract_findings(content: &str, slug: &str, default_repo: &str) -> Vec<RdFinding> {
    // Match **F-01: ...** or **F1: ...** at start of line
    let header_re =
        Regex::new(r"(?m)^\*\*(?P<id>F-?\d+):\s*(?P<rest>[^*]+)\*\*").expect("valid regex");

    let lines: Vec<&str> = content.lines().collect();
    let mut findings = Vec::new();

    // Find all finding headers and their line positions
    let mut headers: Vec<(usize, String, String)> = Vec::new(); // (line_idx, id, rest)

    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = header_re.captures(line) {
            let id = caps["id"].to_string();
            let rest = caps["rest"].trim().to_string();
            headers.push((i, id, rest));
        }
    }

    for (h_idx, (line_idx, id, rest)) in headers.iter().enumerate() {
        // Parse title and date from the rest
        let (title, date) = parse_title_and_date(rest);

        // Collect body: lines after the header until the next finding header or section header
        let body_start = line_idx + 1;
        let body_end = if h_idx + 1 < headers.len() {
            headers[h_idx + 1].0
        } else {
            // Scan forward until a section header (### or ##) or end of file
            lines[body_start..]
                .iter()
                .position(|l| l.starts_with("### ") || l.starts_with("## "))
                .map_or(lines.len(), |pos| body_start + pos)
        };

        let body = lines[body_start..body_end]
            .to_vec()
            .join("\n")
            .trim()
            .to_string();

        // Extract confidence from body text
        let confidence = extract_confidence(&body);

        // Infer repo from finding content if possible
        let source_repo = infer_finding_repo(&title, &body, default_repo);

        findings.push(RdFinding {
            id: id.clone(),
            title,
            body,
            confidence,
            source_brief: slug.to_string(),
            source_repo,
            date,
        });
    }

    findings
}

/// Parse title and optional date from the finding header rest text.
/// Input: "bun compile works (2026-03-17)" or "Simulator-as-separate-binary validates BYOA."
/// Output: (title, optional date)
fn parse_title_and_date(rest: &str) -> (String, Option<String>) {
    let date_re = Regex::new(r"\((\d{4}-\d{2}-\d{2})\)\s*$").expect("valid regex");
    if let Some(caps) = date_re.captures(rest) {
        let date = caps[1].to_string();
        let title = rest[..caps.get(0).expect("matched group").start()]
            .trim()
            .trim_end_matches('.')
            .to_string();
        (title, Some(date))
    } else {
        let title = rest.trim().trim_end_matches('.').to_string();
        (title, None)
    }
}

/// Extract confidence level from finding body text.
/// Looks for "Confidence: frontier" or "Confidence: bedrock" patterns.
fn extract_confidence(body: &str) -> Option<String> {
    let conf_re = Regex::new(r"(?i)confidence:\s*(bedrock|frontier|placeholder|graveyard)")
        .expect("valid regex");
    conf_re.captures(body).map(|caps| caps[1].to_lowercase())
}

/// Infer the primary repo from brief slug.
pub fn infer_repo(slug: &str) -> String {
    if slug.starts_with("marvel") {
        "marvel".to_string()
    } else if slug.starts_with("forestage") {
        "forestage".to_string()
    } else if slug.starts_with("switchboard") {
        "switchboard".to_string()
    } else if slug.starts_with("spectacle") {
        "spectacle".to_string()
    } else if slug.starts_with("kos") {
        "kos".to_string()
    } else if slug.starts_with("director") {
        "director".to_string()
    } else {
        "aae-orc".to_string()
    }
}

/// Refine repo attribution from finding content.
/// Some findings in a brief about one repo actually relate to another.
fn infer_finding_repo(title: &str, body: &str, default: &str) -> String {
    let text = format!("{title} {body}").to_lowercase();

    // F-08 in marvel-mvp-probe explicitly mentions kos
    if text.contains("this is kos's problem") || text.contains("this is kos") {
        return "kos".to_string();
    }

    default.to_string()
}

// ── Output formatting ──────────────────────────────────────

fn print_human(result: &BridgeResult) {
    println!("=== kos bridge ===\n");
    println!(
        "Scanned {} briefs, {} with findings, {} findings total\n",
        result.briefs_scanned,
        result.briefs_with_findings,
        result.findings.len()
    );

    // Group by source brief
    let mut current_brief = String::new();
    for f in &result.findings {
        if f.source_brief != current_brief {
            current_brief.clone_from(&f.source_brief);
            println!("## {current_brief}\n");
        }

        print!("  [{}] {}", f.id, f.title);
        if let Some(ref conf) = f.confidence {
            print!(" ({conf})");
        }
        if f.source_repo != infer_repo(&f.source_brief) {
            print!(" → {}", f.source_repo);
        }
        println!();
    }
}

fn print_jsonl(result: &BridgeResult) {
    for f in &result.findings {
        let json = serde_json::json!({
            "type": "rd_finding",
            "id": f.id,
            "title": f.title,
            "confidence": f.confidence,
            "source_brief": f.source_brief,
            "source_repo": f.source_repo,
            "date": f.date,
        });
        println!("{json}");
    }
    // Summary line
    let summary = serde_json::json!({
        "type": "bridge_summary",
        "briefs_scanned": result.briefs_scanned,
        "briefs_with_findings": result.briefs_with_findings,
        "findings_total": result.findings.len(),
    });
    println!("{summary}");
}
