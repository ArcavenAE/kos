#![forbid(unsafe_code)]

//! Manual compaction for kos nodes.
//!
//! Compaction is a human-directed operation: you point at a node whose
//! knowledge has been absorbed elsewhere (promoted to bedrock, superseded,
//! conclusions captured in other nodes) and provide a summary. This module
//! handles the mechanics: snapshot preservation, content replacement,
//! metadata update, size guard.
//!
//! `kos compact` (no flags) lists nodes by content size for awareness.
//! `kos compact --node <id> --apply --summary <file>` performs compaction.
//!
//! No age-based scanning. Old learning is still valid learning. The trigger
//! for compaction is human judgment, not a calendar.

use std::path::{Path, PathBuf};

use crate::error::{KosError, Result};
use crate::model::Node;
use crate::workspace::Workspace;

/// A node's size info for the overview listing.
#[derive(Debug)]
pub struct NodeSizeEntry {
    pub node_id: String,
    pub title: String,
    pub confidence: String,
    pub content_size: usize,
    pub compaction_level: u8,
}

/// List all nodes by content size (largest first) for awareness.
pub fn list_by_size(workspace: &Workspace, cwd: &Path) -> Result<Vec<NodeSizeEntry>> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let nodes_dir = graph_root.join("nodes");

    if !nodes_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();

    for entry in walkdir::WalkDir::new(&nodes_dir)
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
        let node: Node = match serde_yaml::from_str(&content) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", path.display());
                continue;
            }
        };

        let compaction_level = node.compaction.as_ref().map(|c| c.level).unwrap_or(0);

        entries.push(NodeSizeEntry {
            node_id: node.id.clone(),
            title: node.title.clone(),
            confidence: node.confidence.to_string(),
            content_size: node.content.len(),
            compaction_level,
        });
    }

    // Sort by content size descending (largest first)
    entries.sort_by_key(|e| std::cmp::Reverse(e.content_size));

    Ok(entries)
}

/// Apply compaction to a specific node: replace content with summary,
/// save original to snapshot.
pub fn apply_compaction(
    workspace: &Workspace,
    cwd: &Path,
    node_id: &str,
    summary: &str,
) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);

    // Find the node file
    let node_path = find_node_file(&graph_root, node_id)?;
    let original_content = std::fs::read_to_string(&node_path).map_err(KosError::Io)?;

    let node: Node = serde_yaml::from_str(&original_content).map_err(|e| KosError::Yaml {
        path: node_path.display().to_string(),
        source: e,
    })?;

    // Size guard: don't compact if summary >= original content
    if summary.len() >= node.content.len() {
        println!(
            "Skipped: summary ({} bytes) >= original ({} bytes). No space savings.",
            summary.len(),
            node.content.len()
        );
        return Ok(());
    }

    // Save snapshot
    let snapshots_dir = graph_root.join("snapshots");
    if !snapshots_dir.exists() {
        std::fs::create_dir_all(&snapshots_dir).map_err(KosError::Io)?;
    }
    let snapshot_path = snapshots_dir.join(format!("{node_id}.yaml"));
    std::fs::write(&snapshot_path, &original_content).map_err(KosError::Io)?;

    // Build updated YAML with compacted content
    let today = today_iso();
    let snapshot_rel = format!("snapshots/{node_id}.yaml");
    let original_size = node.content.len();

    let updated = update_node_yaml(
        &original_content,
        summary,
        &today,
        original_size,
        &snapshot_rel,
    );

    std::fs::write(&node_path, updated).map_err(KosError::Io)?;

    let savings = original_size.saturating_sub(summary.len());
    let pct = if original_size > 0 {
        (savings as f64 / original_size as f64 * 100.0) as u64
    } else {
        0
    };

    println!("Compacted: {node_id}");
    println!("  original: {original_size} bytes");
    println!("  summary:  {} bytes ({pct}% reduction)", summary.len());
    println!("  snapshot: {snapshot_rel}");

    Ok(())
}

/// Print node size listing.
pub fn print_size_listing(entries: &[NodeSizeEntry]) {
    if entries.is_empty() {
        println!("No nodes found.");
        return;
    }

    let total_bytes: usize = entries.iter().map(|e| e.content_size).sum();
    let compacted_count = entries.iter().filter(|e| e.compaction_level > 0).count();

    println!(
        "## Node content sizes ({} nodes, {} bytes total)\n",
        entries.len(),
        total_bytes
    );
    println!(
        "  {:<40} {:>10} {:>10} {:>5}",
        "NODE", "CONFIDENCE", "SIZE", "LEVEL"
    );
    for e in entries {
        let level_str = if e.compaction_level > 0 {
            format!("L{}", e.compaction_level)
        } else {
            "-".to_string()
        };
        println!(
            "  {:<40} {:>10} {:>8}B {:>5}",
            e.node_id, e.confidence, e.content_size, level_str
        );
    }

    if compacted_count > 0 {
        println!("\n  {compacted_count} nodes already compacted.");
    }

    println!("\n  To compact a node whose knowledge has been absorbed elsewhere:");
    println!("    kos compact --node <id> --apply --summary <file>");
    println!("  Original content is preserved in _kos/snapshots/<id>.yaml.");
}

// ── Helpers ───────────────────────────────────────────────────

fn resolve_graph_root(workspace: &Workspace, cwd: &Path) -> PathBuf {
    if let Some(graph) = workspace.nearest_graph(cwd) {
        return graph.path.clone();
    }
    let local_kos = cwd.join(crate::workspace::KOS_DIR);
    if local_kos.join(crate::workspace::MANIFEST_FILE).exists() {
        return local_kos;
    }
    workspace.node_root()
}

fn find_node_file(graph_root: &Path, node_id: &str) -> Result<PathBuf> {
    let nodes_dir = graph_root.join("nodes");
    for tier in &["bedrock", "frontier", "graveyard", "placeholder"] {
        let path = nodes_dir.join(tier).join(format!("{node_id}.yaml"));
        if path.exists() {
            return Ok(path);
        }
    }
    Err(KosError::Init {
        message: format!("node not found: {node_id}"),
    })
}

fn update_node_yaml(
    original: &str,
    summary: &str,
    today: &str,
    original_size: usize,
    snapshot_rel: &str,
) -> String {
    let mut result = String::new();
    let mut in_content = false;
    let mut content_done = false;
    let mut has_compaction = false;

    for line in original.lines() {
        if line.starts_with("content:") && !content_done {
            in_content = true;
            result.push_str("content: |\n");
            result.push_str("  [COMPACTED] Original preserved in snapshot.\n");
            for summary_line in summary.lines() {
                result.push_str("  ");
                result.push_str(summary_line);
                result.push('\n');
            }
            continue;
        }

        if in_content {
            if line.starts_with("  ") || line.is_empty() {
                continue;
            }
            in_content = false;
            content_done = true;
        }

        if line.starts_with("compaction:") {
            has_compaction = true;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !has_compaction {
        result.push_str("compaction:\n");
        result.push_str("  level: 1\n");
        result.push_str(&format!("  compacted_at: \"{today}\"\n"));
        result.push_str(&format!("  original_size: {original_size}\n"));
        result.push_str(&format!("  snapshot: \"{snapshot_rel}\"\n"));
    }

    result
}

fn today_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days = now.as_secs() / 86400;
    let (year, month, day) = crate::process::days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}")
}
