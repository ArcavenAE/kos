use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::bridge::RdFinding;
use crate::error::{KosError, Result};
use crate::model::{CharterItem, CharterSection, Confidence, Finding, Node, RdBrief};
use crate::workspace::{KOS_DIR, Workspace};

/// Collected orientation data for a target repo.
#[derive(Debug)]
pub struct Orientation {
    pub target: String,
    pub charter_items: Vec<CharterItem>,
    pub findings: Vec<Finding>,
    pub rd_briefs: Vec<RdBrief>,
    pub rd_findings: Vec<RdFinding>,
    pub frontier_questions: Vec<Node>,
    /// Standalone-specific: bedrock nodes from `_kos/nodes/bedrock/`.
    pub bedrock_nodes: Vec<Node>,
    /// Standalone-specific: graveyard nodes from `_kos/nodes/graveyard/`.
    pub graveyard_nodes: Vec<Node>,
    /// Standalone-specific: active probes from `_kos/probes/`.
    pub probes: Vec<ProbeEntry>,
    /// Standalone-specific: idea filenames from `_kos/ideas/`.
    pub ideas: Vec<String>,
    /// Whether this orientation is for a standalone repo (no orchestrator).
    pub is_standalone: bool,
}

/// A probe entry loaded from `_kos/probes/`.
#[derive(Debug)]
pub struct ProbeEntry {
    pub slug: String,
    pub title: Option<String>,
    pub path: PathBuf,
}

/// Run the orient subcommand.
pub fn run(workspace: &Workspace, target: &str, json: bool, log: bool, ready: bool) -> Result<()> {
    let start = Instant::now();
    let cwd = std::env::current_dir().map_err(KosError::Io)?;

    if ready {
        return run_ready(workspace, &cwd, json);
    }

    let orientation = if workspace.is_standalone() {
        gather_standalone(workspace, target)?
    } else {
        gather(workspace, &cwd, target)?
    };
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

/// A frontier question classified as ready or blocked.
#[derive(Debug)]
pub struct ReadyQuestion {
    pub node: Node,
    pub status: ReadyStatus,
    /// Number of other nodes that derive from this question (impact score).
    pub dependents: usize,
}

#[derive(Debug)]
pub enum ReadyStatus {
    /// All blocking dependencies are resolved (bedrock or have findings).
    Ready,
    /// One or more blocking dependencies are unresolved frontier questions.
    Blocked { blockers: Vec<String> },
}

/// Run the --ready computation: which frontier questions are actionable?
fn run_ready(workspace: &Workspace, cwd: &Path, json: bool) -> Result<()> {
    let nearest = workspace.nearest_graph(cwd);
    let graph_root = if let Some(graph) = nearest {
        graph.path.clone()
    } else {
        workspace.node_root()
    };

    let nodes_dir = graph_root.join("nodes");
    if !nodes_dir.exists() {
        println!("No nodes/ directory found.");
        return Ok(());
    }

    // Load ALL nodes (all confidence tiers)
    let all_nodes = load_all_nodes(&nodes_dir)?;

    // Build ID sets for resolution checking
    let bedrock_ids: std::collections::HashSet<&str> = all_nodes
        .iter()
        .filter(|n| n.confidence == Confidence::Bedrock)
        .map(|n| n.id.as_str())
        .collect();

    let graveyard_ids: std::collections::HashSet<&str> = all_nodes
        .iter()
        .filter(|n| n.confidence == Confidence::Graveyard)
        .map(|n| n.id.as_str())
        .collect();

    // Load findings to check which questions have findings
    let findings_dir = graph_root.join("findings");
    let finding_ids = load_finding_ids(&findings_dir);

    // Count dependents (how many nodes have blocking edges TO each node)
    let mut dependent_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for node in &all_nodes {
        for edge in node.all_edges() {
            if edge.edge_type.is_blocking() {
                *dependent_counts.entry(edge.target.clone()).or_default() += 1;
            }
        }
    }

    // Classify frontier questions
    let mut ready_questions: Vec<ReadyQuestion> = Vec::new();

    for node in &all_nodes {
        if node.confidence != Confidence::Frontier {
            continue;
        }
        if node.node_type != crate::model::NodeType::Question {
            continue;
        }

        let all_edges_owned = node.all_edges();
        let blocking: Vec<&crate::model::Edge> = all_edges_owned
            .iter()
            .filter(|e| e.edge_type.is_blocking())
            .collect();

        let mut blockers = Vec::new();
        for edge in &blocking {
            let target = &edge.target;
            // Resolved if: target is bedrock, graveyard, or has a finding
            let is_resolved = bedrock_ids.contains(target.as_str())
                || graveyard_ids.contains(target.as_str())
                || finding_ids.contains(target.as_str());
            if !is_resolved {
                blockers.push(target.clone());
            }
        }

        let dependents = dependent_counts.get(&node.id).copied().unwrap_or(0);

        let status = if blockers.is_empty() {
            ReadyStatus::Ready
        } else {
            ReadyStatus::Blocked { blockers }
        };

        ready_questions.push(ReadyQuestion {
            node: Node {
                id: node.id.clone(),
                node_type: node.node_type.clone(),
                confidence: node.confidence.clone(),
                title: node.title.clone(),
                content: String::new(), // Don't carry full content
                edges: node.edges.clone(),
                depends_on: node.depends_on.clone(),
                graveyard: None,
                brief: None,
                finding: None,
                compaction: None,
                provenance: None,
                tags: vec![],
                notes: None,
                source_path: node.source_path.clone(),
            },
            status,
            dependents,
        });
    }

    // Sort: ready first, then by dependents descending
    ready_questions.sort_by(|a, b| {
        let a_ready = matches!(a.status, ReadyStatus::Ready);
        let b_ready = matches!(b.status, ReadyStatus::Ready);
        b_ready.cmp(&a_ready).then(b.dependents.cmp(&a.dependents))
    });

    if json {
        print_ready_jsonl(&ready_questions);
    } else {
        print_ready_human(&ready_questions);
    }

    Ok(())
}

fn load_all_nodes(nodes_dir: &Path) -> Result<Vec<Node>> {
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;
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
    Ok(results)
}

fn load_finding_ids(findings_dir: &Path) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();
    if !findings_dir.exists() {
        return ids;
    }
    if let Ok(entries) = std::fs::read_dir(findings_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".yaml") || name.ends_with(".yml") {
                // Extract finding ID from filename
                if let Some(stem) = name.strip_suffix(".yaml").or(name.strip_suffix(".yml")) {
                    ids.insert(stem.to_string());
                }
            }
        }
    }
    ids
}

fn print_ready_human(questions: &[ReadyQuestion]) {
    let ready_count = questions
        .iter()
        .filter(|q| matches!(q.status, ReadyStatus::Ready))
        .count();
    let blocked_count = questions.len() - ready_count;

    println!("=== kos orient --ready ===\n");

    if ready_count > 0 {
        println!("## Ready to probe ({ready_count})\n");
        for q in questions
            .iter()
            .filter(|q| matches!(q.status, ReadyStatus::Ready))
        {
            print!("  [{}] {}", q.node.id, q.node.title);
            if q.dependents > 0 {
                print!("  ({} dependents)", q.dependents);
            }
            println!();
        }
        println!();
    }

    if blocked_count > 0 {
        println!("## Blocked ({blocked_count})\n");
        for q in questions
            .iter()
            .filter(|q| matches!(q.status, ReadyStatus::Blocked { .. }))
        {
            println!("  [{}] {}", q.node.id, q.node.title);
            if let ReadyStatus::Blocked { ref blockers } = q.status {
                for b in blockers {
                    println!("    ← blocked by: {b}");
                }
            }
        }
        println!();
    }

    if questions.is_empty() {
        println!("  (no frontier questions found)");
    }
}

fn print_ready_jsonl(questions: &[ReadyQuestion]) {
    for q in questions {
        let (status, blockers) = match &q.status {
            ReadyStatus::Ready => ("ready", vec![]),
            ReadyStatus::Blocked { blockers } => ("blocked", blockers.clone()),
        };
        let json = serde_json::json!({
            "type": "ready_question",
            "id": q.node.id,
            "title": q.node.title,
            "status": status,
            "blockers": blockers,
            "dependents": q.dependents,
        });
        println!("{json}");
    }
}

/// Gather all orientation data for a target repo.
///
/// Loads charter items, RD briefs/findings (orchestrator-level), PLUS
/// _kos/ graph content from the nearest graph to cwd (nodes by tier,
/// findings, probes, ideas).
fn gather(workspace: &Workspace, cwd: &Path, target: &str) -> Result<Orientation> {
    let charter_items = load_charter_items(&workspace.root.join("charter.md"), target)?;
    let rd_briefs = load_rd_briefs(&workspace.root.join("sprint/rd"), target)?;
    let rd_findings = load_rd_findings(&workspace.root.join("sprint/rd"), target)?;

    // Load _kos/ graph content from nearest graph to cwd
    let nearest = workspace.nearest_graph(cwd);
    let (bedrock_nodes, frontier_questions, graveyard_nodes, graph_findings, probes, ideas) =
        if let Some(graph) = nearest {
            let nodes_dir = graph.path.join("nodes");
            let bedrock = load_nodes_by_confidence(&nodes_dir, &Confidence::Bedrock)?;
            let frontier = load_nodes_by_confidence(&nodes_dir, &Confidence::Frontier)?;
            let graveyard = load_nodes_by_confidence(&nodes_dir, &Confidence::Graveyard)?;
            let findings = load_findings_unfiltered(&graph.path.join("findings"))?;
            let probes = load_probes(&graph.path.join("probes"))?;
            let ideas = load_ideas(&graph.path.join("ideas"))?;
            (bedrock, frontier, graveyard, findings, probes, ideas)
        } else {
            // Fall back to legacy kos subrepo layout (findings/ and nodes/ at kos_root)
            let findings = load_findings(&workspace.kos_root.join("findings"), target)?;
            let frontier =
                load_frontier_questions(&workspace.kos_root.join("nodes/frontier"), target)?;
            (vec![], frontier, vec![], findings, vec![], vec![])
        };

    Ok(Orientation {
        target: target.to_string(),
        charter_items,
        findings: graph_findings,
        rd_briefs,
        rd_findings,
        frontier_questions,
        bedrock_nodes,
        graveyard_nodes,
        probes,
        ideas,
        is_standalone: false,
    })
}

/// Gather orientation data for a standalone repo (no orchestrator).
///
/// In standalone mode, everything lives under `_kos/`:
/// - Charter items from a local charter file (unfiltered — all are relevant)
/// - Nodes from `_kos/nodes/{bedrock,frontier,graveyard,placeholder}/`
/// - Findings from `_kos/findings/`
/// - Probes from `_kos/probes/`
/// - Ideas from `_kos/ideas/`
///
/// No RD briefs or RD findings (those are aae-orc orchestrator artifacts).
fn gather_standalone(workspace: &Workspace, target: &str) -> Result<Orientation> {
    let kos_dir = workspace.root.join(KOS_DIR);

    // Charter: check local charter files (unfiltered — all items are relevant)
    let charter_items = load_standalone_charter(&workspace.root)?;

    // Findings from _kos/findings/
    let findings = load_findings_unfiltered(&kos_dir.join("findings"))?;

    // Nodes by confidence tier
    let nodes_dir = kos_dir.join("nodes");
    let frontier_questions = load_nodes_by_confidence(&nodes_dir, &Confidence::Frontier)?;
    let bedrock_nodes = load_nodes_by_confidence(&nodes_dir, &Confidence::Bedrock)?;
    let graveyard_nodes = load_nodes_by_confidence(&nodes_dir, &Confidence::Graveyard)?;

    // Probes from _kos/probes/
    let probes = load_probes(&kos_dir.join("probes"))?;

    // Ideas from _kos/ideas/
    let ideas = load_ideas(&kos_dir.join("ideas"))?;

    Ok(Orientation {
        target: target.to_string(),
        charter_items,
        findings,
        rd_briefs: vec![],
        rd_findings: vec![],
        frontier_questions,
        bedrock_nodes,
        graveyard_nodes,
        probes,
        ideas,
        is_standalone: true,
    })
}

/// Load charter items from a standalone repo's charter file, unfiltered.
///
/// Checks for `KOS-charter.md` first (kos repo convention), then `charter.md`.
fn load_standalone_charter(root: &Path) -> Result<Vec<CharterItem>> {
    // Try KOS-charter.md first (the kos repo itself), then charter.md
    let charter_path = if root.join("KOS-charter.md").exists() {
        root.join("KOS-charter.md")
    } else if root.join("charter.md").exists() {
        root.join("charter.md")
    } else {
        return Ok(vec![]);
    };

    load_charter_items_unfiltered(&charter_path)
}

/// Load charter items without filtering by target — all items are relevant.
fn load_charter_items_unfiltered(charter_path: &Path) -> Result<Vec<CharterItem>> {
    if !charter_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(charter_path).map_err(KosError::Io)?;
    let mut items = Vec::new();
    let mut current_section: Option<CharterSection> = None;
    let mut current_id = String::new();
    let mut current_title = String::new();
    let mut current_body = String::new();

    for line in content.lines() {
        // Detect section headers
        if line.starts_with("## Bedrock") {
            flush_item_unfiltered(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
            );
            current_section = Some(CharterSection::Bedrock);
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }
        if line.starts_with("## Frontier") {
            flush_item_unfiltered(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
            );
            current_section = Some(CharterSection::Frontier);
            current_id.clear();
            current_title.clear();
            current_body.clear();
            continue;
        }
        if line.starts_with("## Graveyard") {
            flush_item_unfiltered(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
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
            flush_item_unfiltered(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
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
            flush_item_unfiltered(
                &mut items,
                &current_section,
                &current_id,
                &current_title,
                &current_body,
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
    flush_item_unfiltered(
        &mut items,
        &current_section,
        &current_id,
        &current_title,
        &current_body,
    );

    Ok(items)
}

fn flush_item_unfiltered(
    items: &mut Vec<CharterItem>,
    section: &Option<CharterSection>,
    id: &str,
    title: &str,
    body: &str,
) {
    if id.is_empty() {
        return;
    }
    let Some(section) = section else { return };

    items.push(CharterItem {
        id: id.to_string(),
        section: section.clone(),
        title: title.to_string(),
        body: body.trim().to_string(),
    });
}

/// Load findings without filtering by target — all findings are relevant.
fn load_findings_unfiltered(findings_dir: &Path) -> Result<Vec<Finding>> {
    if !findings_dir.exists() {
        return Ok(vec![]);
    }

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

/// Load all nodes from a specific confidence tier directory.
fn load_nodes_by_confidence(nodes_dir: &Path, confidence: &Confidence) -> Result<Vec<Node>> {
    let tier_dir = nodes_dir.join(confidence.directory());
    if !tier_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(&tier_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;

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

    Ok(results)
}

/// Load probe entries from `_kos/probes/`.
fn load_probes(probes_dir: &Path) -> Result<Vec<ProbeEntry>> {
    if !probes_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(probes_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") && ext != Some("md") {
            continue;
        }

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to extract a title from the file
        let title = std::fs::read_to_string(path)
            .ok()
            .and_then(|content| extract_probe_title(&content));

        results.push(ProbeEntry {
            slug,
            title,
            path: path.to_path_buf(),
        });
    }

    Ok(results)
}

/// Extract a title from a probe file.
/// For YAML: looks for a `title:` field.
/// For Markdown: looks for a `# ` heading.
fn extract_probe_title(content: &str) -> Option<String> {
    for line in content.lines() {
        // YAML title field
        if let Some(rest) = line.strip_prefix("title:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        // Markdown heading
        if let Some(rest) = line.strip_prefix("# ") {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Load idea filenames from `_kos/ideas/`.
fn load_ideas(ideas_dir: &Path) -> Result<Vec<String>> {
    if !ideas_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(ideas_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        results.push(name);
    }

    Ok(results)
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
    if o.is_standalone {
        println!("=== kos orient: {} (standalone) ===\n", o.target);
    } else {
        println!("=== kos orient: {} ===\n", o.target);
    }

    if !o.charter_items.is_empty() {
        if o.is_standalone {
            println!("## Charter\n");
        } else {
            println!("## Charter items mentioning {}\n", o.target);
        }
        for item in &o.charter_items {
            println!("  [{}/{}] {}", item.section, item.id, item.title);
        }
        println!();
    }

    if !o.bedrock_nodes.is_empty() {
        println!("## Bedrock nodes\n");
        for node in &o.bedrock_nodes {
            println!("  [{}] {} ({})", node.id, node.title, node.node_type);
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
        if o.is_standalone {
            println!("## Findings\n");
        } else {
            println!("## kos findings mentioning {}\n", o.target);
        }
        for finding in &o.findings {
            println!(
                "  [{}] {} ({})",
                finding.id, finding.title, finding.confidence
            );
        }
        println!();
    }

    if !o.frontier_questions.is_empty() {
        println!("## Open questions (frontier)\n");
        for q in &o.frontier_questions {
            println!("  [{}] {}", q.id, q.title);
        }
        println!();
    }

    if !o.probes.is_empty() {
        println!("## Active probes\n");
        for probe in &o.probes {
            if let Some(ref title) = probe.title {
                println!("  [{}] {title}", probe.slug);
            } else {
                println!("  [{}]", probe.slug);
            }
        }
        println!();
    }

    if !o.ideas.is_empty() {
        println!("## Ideas\n");
        for idea in &o.ideas {
            println!("  - {idea}");
        }
        println!();
    }

    if !o.graveyard_nodes.is_empty() {
        println!("## Graveyard nodes\n");
        for node in &o.graveyard_nodes {
            println!("  [{}] {} ({})", node.id, node.title, node.node_type);
        }
        println!();
    }

    let total = o.charter_items.len()
        + o.findings.len()
        + o.rd_briefs.len()
        + o.rd_findings.len()
        + o.frontier_questions.len()
        + o.bedrock_nodes.len()
        + o.graveyard_nodes.len()
        + o.probes.len()
        + o.ideas.len();
    if total == 0 {
        println!("  (no items found)");
        if o.is_standalone {
            println!("  Hint: add nodes to _kos/nodes/, findings to _kos/findings/,");
            println!("        or create a charter file (charter.md or KOS-charter.md).");
        } else {
            println!("  Try: kos orient kos | kos orient marvel | kos orient forestage");
        }
    }
}

fn print_jsonl(o: &Orientation) {
    // Emit standalone marker as first line when applicable
    if o.is_standalone {
        let json = serde_json::json!({
            "type": "orient_meta",
            "target": o.target,
            "standalone": true,
        });
        println!("{json}");
    }

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

    for node in &o.bedrock_nodes {
        let json = serde_json::json!({
            "type": "node",
            "target": o.target,
            "id": node.id,
            "title": node.title,
            "node_type": node.node_type.to_string(),
            "confidence": node.confidence.to_string(),
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

    for probe in &o.probes {
        let json = serde_json::json!({
            "type": "probe",
            "target": o.target,
            "slug": probe.slug,
            "title": probe.title,
            "path": probe.path.display().to_string(),
        });
        println!("{json}");
    }

    for idea in &o.ideas {
        let json = serde_json::json!({
            "type": "idea",
            "target": o.target,
            "name": idea,
        });
        println!("{json}");
    }

    for node in &o.graveyard_nodes {
        let json = serde_json::json!({
            "type": "node",
            "target": o.target,
            "id": node.id,
            "title": node.title,
            "node_type": node.node_type.to_string(),
            "confidence": node.confidence.to_string(),
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
        "standalone": o.is_standalone,
        "charter_items": o.charter_items.len(),
        "findings": o.findings.len(),
        "rd_briefs": o.rd_briefs.len(),
        "rd_findings": o.rd_findings.len(),
        "questions": o.frontier_questions.len(),
        "bedrock_nodes": o.bedrock_nodes.len(),
        "graveyard_nodes": o.graveyard_nodes.len(),
        "probes": o.probes.len(),
        "ideas": o.ideas.len(),
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
