use std::path::Path;
use std::time::Instant;

use crate::bridge::RdFinding;
use crate::error::{KosError, Result};
use crate::model::{CharterItem, CharterSection, Finding, Node, RdBrief};
use crate::workspace::Workspace;

/// Collected orientation data for a target repo.
#[derive(Debug)]
pub struct Orientation {
    pub target: String,
    pub charter_items: Vec<CharterItem>,
    pub findings: Vec<Finding>,
    pub rd_briefs: Vec<RdBrief>,
    pub rd_findings: Vec<RdFinding>,
    pub frontier_questions: Vec<Node>,
}

/// Run the orient subcommand.
pub fn run(workspace: &Workspace, target: &str, json: bool, log: bool) -> Result<()> {
    let start = Instant::now();
    let orientation = gather(workspace, target)?;
    let duration = start.elapsed();

    if json {
        print_jsonl(&orientation);
    } else {
        print_human(&orientation);
    }

    if log {
        if let Err(e) = append_usage_log(&orientation, duration, json) {
            eprintln!("warning: could not write usage log: {e}");
        }
    }

    Ok(())
}

/// Gather all orientation data for a target repo.
fn gather(workspace: &Workspace, target: &str) -> Result<Orientation> {
    let charter_items = load_charter_items(&workspace.root.join("charter.md"), target)?;
    let findings = load_findings(&workspace.kos_root.join("findings"), target)?;
    let rd_briefs = load_rd_briefs(&workspace.root.join("sprint/rd"), target)?;
    let rd_findings = load_rd_findings(&workspace.root.join("sprint/rd"), target)?;
    let frontier_questions =
        load_frontier_questions(&workspace.kos_root.join("nodes/frontier"), target)?;

    Ok(Orientation {
        target: target.to_string(),
        charter_items,
        findings,
        rd_briefs,
        rd_findings,
        frontier_questions,
    })
}

// ── Charter parsing ──────────────────────────────────────────

fn load_charter_items(charter_path: &Path, target: &str) -> Result<Vec<CharterItem>> {
    if !charter_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(charter_path).map_err(KosError::Io)?;
    let target_lower = target.to_lowercase();
    let mut items = Vec::new();
    let mut current_section: Option<CharterSection> = None;
    let mut current_id = String::new();
    let mut current_title = String::new();
    let mut current_body = String::new();

    for line in content.lines() {
        // Detect section headers
        if line.starts_with("## Bedrock") {
            flush_item(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
                &target_lower,
            );
            current_section = Some(CharterSection::Bedrock);
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }
        if line.starts_with("## Frontier") {
            flush_item(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
                &target_lower,
            );
            current_section = Some(CharterSection::Frontier);
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }
        if line.starts_with("## Graveyard") {
            flush_item(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
                &target_lower,
            );
            current_section = Some(CharterSection::Graveyard);
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }
        // Other ## headers end the current section
        if line.starts_with("## ") && current_section.is_some() && !line.starts_with("## Research")
        {
            flush_item(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
                &target_lower,
            );
            current_section = None;
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }

        if current_section.is_none() {
            continue;
        }

        // Detect item headers (### B1: ..., ### F1: ..., ### G1: ...)
        if let Some(header) = line.strip_prefix("### ") {
            flush_item(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
                &target_lower,
            );
            if let Some((id, title)) = header.split_once(':') {
                current_id = id.trim().to_string();
                current_title = title.trim().to_string();
            } else {
                current_id = header.trim().to_string();
                current_title = header.trim().to_string();
            }
            current_body.clear();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    // Flush last item
    flush_item(
        &mut items,
        &current_section,
        &current_id,
        &current_title,
        &current_body,
        &target_lower,
    );

    Ok(items)
}

fn flush_item(
    items: &mut Vec<CharterItem>,
    section: &Option<CharterSection>,
    id: &str,
    title: &str,
    body: &str,
    target: &str,
) {
    if id.is_empty() {
        return;
    }
    let Some(section) = section else { return };

    let full_text = format!("{title} {body}").to_lowercase();
    if full_text.contains(target) {
        items.push(CharterItem {
            id: id.to_string(),
            section: section.clone(),
            title: title.to_string(),
            body: body.trim().to_string(),
        });
    }
}

// ── Finding loading ──────────────────────────────────────────

fn load_findings(findings_dir: &Path, target: &str) -> Result<Vec<Finding>> {
    if !findings_dir.exists() {
        return Ok(vec![]);
    }

    let target_lower = target.to_lowercase();
    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(findings_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;

        // Check if the finding mentions the target before full parse
        if !content.to_lowercase().contains(&target_lower) {
            continue;
        }

        match serde_yaml::from_str::<Finding>(&content) {
            Ok(mut finding) => {
                finding.source_path = path.to_path_buf();
                results.push(finding);
            }
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", path.display());
            }
        }
    }

    Ok(results)
}

// ── Frontier question loading ────────────────────────────────

fn load_frontier_questions(frontier_dir: &Path, target: &str) -> Result<Vec<Node>> {
    if !frontier_dir.exists() {
        return Ok(vec![]);
    }

    let target_lower = target.to_lowercase();
    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(frontier_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;

        // Only include questions (not all frontier nodes)
        if !content.contains("type: question") {
            continue;
        }

        // Include if it mentions the target or if target is "kos" (all questions are kos-relevant)
        if target_lower == "kos" || content.to_lowercase().contains(&target_lower) {
            match serde_yaml::from_str::<Node>(&content) {
                Ok(mut node) => {
                    node.source_path = path.to_path_buf();
                    results.push(node);
                }
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", path.display());
                }
            }
        }
    }

    Ok(results)
}

// ── RD brief loading ─────────────────────────────────────────

fn load_rd_briefs(rd_dir: &Path, target: &str) -> Result<Vec<RdBrief>> {
    if !rd_dir.exists() {
        return Ok(vec![]);
    }

    let target_lower = target.to_lowercase();
    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(rd_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;

        // Check if the brief mentions the target
        if !content.to_lowercase().contains(&target_lower) {
            continue;
        }

        // Parse the lightweight RD brief frontmatter
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let question = extract_field(&content, "question:")
            .unwrap_or_default()
            .to_string();
        let frontier = extract_field(&content, "frontier:").map(ToString::to_string);
        let status = extract_field(&content, "status:").map(ToString::to_string);

        results.push(RdBrief {
            slug,
            question,
            frontier,
            status,
            path: path.to_path_buf(),
        });
    }

    Ok(results)
}

/// Extract a single-line field value from markdown frontmatter-style content.
fn extract_field<'a>(content: &'a str, field: &str) -> Option<&'a str> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

// ── RD finding loading (via bridge) ──────────────────────────

fn load_rd_findings(rd_dir: &Path, target: &str) -> Result<Vec<RdFinding>> {
    if !rd_dir.exists() {
        return Ok(vec![]);
    }

    let target_lower = target.to_lowercase();

    // Use bridge extraction, then filter by target repo
    let mut all_findings = Vec::new();
    for entry in walkdir::WalkDir::new(rd_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let source_repo = crate::bridge::infer_repo(&slug);
        let findings = crate::bridge::extract_findings(&content, &slug, &source_repo);
        all_findings.extend(findings);
    }

    // Filter: include findings whose source_repo matches, or whose content mentions the target
    let filtered: Vec<RdFinding> = all_findings
        .into_iter()
        .filter(|f| {
            f.source_repo.to_lowercase() == target_lower
                || f.title.to_lowercase().contains(&target_lower)
                || f.body.to_lowercase().contains(&target_lower)
        })
        .collect();

    Ok(filtered)
}

// ── Output formatting ────────────────────────────────────────

fn print_human(o: &Orientation) {
    println!("=== kos orient: {} ===\n", o.target);

    if !o.charter_items.is_empty() {
        println!("## Charter items mentioning {}\n", o.target);
        for item in &o.charter_items {
            println!("  [{}/{}] {}", item.section, item.id, item.title);
        }
        println!();
    }

    if !o.rd_briefs.is_empty() {
        println!("## RD briefs involving {}\n", o.target);
        for brief in &o.rd_briefs {
            print!("  [{}]", brief.slug);
            if let Some(ref f) = brief.frontier {
                print!(" ({f})");
            }
            println!();
            if !brief.question.is_empty() {
                println!("    Q: {}", brief.question);
            }
            if let Some(ref s) = brief.status {
                println!("    Status: {s}");
            }
        }
        println!();
    }

    if !o.rd_findings.is_empty() {
        println!("## RD findings for {}\n", o.target);
        for f in &o.rd_findings {
            print!("  [{}] {}", f.id, f.title);
            if let Some(ref conf) = f.confidence {
                print!(" ({conf})");
            }
            println!("  ← {}", f.source_brief);
        }
        println!();
    }

    if !o.findings.is_empty() {
        println!("## kos findings mentioning {}\n", o.target);
        for finding in &o.findings {
            println!(
                "  [{}] {} ({})",
                finding.id, finding.title, finding.confidence
            );
        }
        println!();
    }

    if !o.frontier_questions.is_empty() {
        println!("## Open questions\n");
        for q in &o.frontier_questions {
            println!("  [{}] {}", q.id, q.title);
        }
        println!();
    }

    let total = o.charter_items.len()
        + o.findings.len()
        + o.rd_briefs.len()
        + o.rd_findings.len()
        + o.frontier_questions.len();
    if total == 0 {
        println!("  (no items found mentioning '{}')", o.target);
        println!("  Try: kos orient kos | kos orient marvel | kos orient aclaude");
    }
}

fn print_jsonl(o: &Orientation) {
    for item in &o.charter_items {
        let json = serde_json::json!({
            "type": "charter",
            "target": o.target,
            "section": item.section.to_string(),
            "id": item.id,
            "title": item.title,
        });
        println!("{json}");
    }

    for brief in &o.rd_briefs {
        let json = serde_json::json!({
            "type": "rd_brief",
            "target": o.target,
            "slug": brief.slug,
            "question": brief.question,
            "frontier": brief.frontier,
            "status": brief.status,
            "path": brief.path.display().to_string(),
        });
        println!("{json}");
    }

    for f in &o.rd_findings {
        let json = serde_json::json!({
            "type": "rd_finding",
            "target": o.target,
            "id": f.id,
            "title": f.title,
            "confidence": f.confidence,
            "source_brief": f.source_brief,
            "source_repo": f.source_repo,
        });
        println!("{json}");
    }

    for finding in &o.findings {
        let json = serde_json::json!({
            "type": "finding",
            "target": o.target,
            "id": finding.id,
            "title": finding.title,
            "confidence": finding.confidence.to_string(),
        });
        println!("{json}");
    }

    for q in &o.frontier_questions {
        let json = serde_json::json!({
            "type": "question",
            "target": o.target,
            "id": q.id,
            "title": q.title,
            "confidence": q.confidence.to_string(),
        });
        println!("{json}");
    }
}

// ── Usage logging (opt-in via --log) ─────────────────────────

/// Append a single JSONL line to the local usage log.
/// No content or file paths — just the shape of what was surfaced.
fn append_usage_log(
    o: &Orientation,
    duration: std::time::Duration,
    json_output: bool,
) -> std::io::Result<()> {
    use std::io::Write;

    let log_dir = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("kos");
    std::fs::create_dir_all(&log_dir)?;
    let log_path = log_dir.join("orient.jsonl");

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "target": o.target,
        "charter_items": o.charter_items.len(),
        "findings": o.findings.len(),
        "rd_briefs": o.rd_briefs.len(),
        "rd_findings": o.rd_findings.len(),
        "questions": o.frontier_questions.len(),
        "duration_ms": duration.as_millis(),
        "json_output": json_output,
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    writeln!(file, "{entry}")?;

    Ok(())
}
