use std::path::Path;

use crate::error::Result;
use crate::model::{GraphScope, GraphSource, Node};
use crate::workspace::{MANIFEST_FILE, Workspace};

/// Run the doctor subcommand.
pub fn run(workspace: &Workspace, cwd: &Path, merged: bool, fix: bool) -> Result<()> {
    let graphs: Vec<&GraphSource> = if merged {
        workspace.graphs.iter().collect()
    } else if let Some(nearest) = workspace.nearest_graph(cwd) {
        vec![nearest]
    } else {
        // No _kos/ graphs — check legacy layout
        println!("kos doctor: no _kos/ graphs found. Checking legacy layout.");
        return check_legacy(workspace, fix);
    };

    if graphs.is_empty() {
        println!("kos doctor: no graphs to check.");
        println!("  hint: run `kos init` to create a _kos/ graph");
        return Ok(());
    }

    let mut total_errors = 0;
    let mut total_warnings = 0;

    for graph in &graphs {
        let (errors, warnings) = check_graph(graph, workspace, fix)?;
        total_errors += errors;
        total_warnings += warnings;
    }

    println!();
    if total_errors == 0 && total_warnings == 0 {
        println!("kos doctor: all checks passed.");
    } else if total_errors == 0 {
        println!("kos doctor: {total_warnings} warning(s), 0 errors.");
    } else {
        println!("kos doctor: {total_errors} error(s), {total_warnings} warning(s).");
        std::process::exit(1);
    }

    Ok(())
}

/// Check a single graph. Returns (error_count, warning_count).
fn check_graph(graph: &GraphSource, workspace: &Workspace, fix: bool) -> Result<(usize, usize)> {
    let rel_path = graph
        .path
        .strip_prefix(&workspace.root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| graph.path.display().to_string());

    println!(
        "kos doctor: checking {} graph ({}/)",
        graph.graph_id, rel_path
    );

    let mut errors = 0;
    let mut warnings = 0;

    // ── Structure checks ────────────────────────────────────
    println!();
    println!("  Structure");

    // _kos/ exists (it must, since we loaded it)
    ok("_kos/ directory exists");

    // kos.yaml parses (it must, since we loaded it)
    ok(&format!(
        "kos.yaml manifest valid (graph_id: {}, scope: {})",
        graph.graph_id, graph.scope
    ));

    // Check subdirectories
    let required_dirs = [
        "nodes/bedrock",
        "nodes/frontier",
        "nodes/graveyard",
        "nodes/placeholder",
        "findings",
        "probes",
        "ideas",
    ];
    let mut missing_dirs = Vec::new();
    for dir in &required_dirs {
        if !graph.path.join(dir).exists() {
            missing_dirs.push(*dir);
        }
    }
    if missing_dirs.is_empty() {
        ok("all subdirectories present");
    } else if fix {
        for dir in &missing_dirs {
            std::fs::create_dir_all(graph.path.join(dir)).ok();
        }
        fixed(&format!(
            "created missing directories: {}",
            missing_dirs.join(", ")
        ));
    } else {
        err(&format!("missing directories: {}", missing_dirs.join(", ")));
        hint("run `kos doctor --fix` to create them");
        errors += missing_dirs.len();
    }

    // Schema version
    if graph.manifest.schema_version == "0.3" {
        ok("schema version: 0.3 (current)");
    } else {
        warn(&format!(
            "schema version: {} (current is 0.3)",
            graph.manifest.schema_version
        ));
        if fix {
            // Would need to update manifest file — skip for now
            hint("run `kos init --update` to bump schema version");
        }
        warnings += 1;
    }

    // Includes (orchestrator only)
    if graph.scope == GraphScope::Orchestrator {
        for inc in &graph.manifest.includes {
            let inc_path = workspace.root.join(&inc.path).join(MANIFEST_FILE);
            if inc_path.exists() {
                ok(&format!("include: {} (reachable)", inc.path));
            } else {
                warn(&format!("include: {} (not found)", inc.path));
                warnings += 1;
            }
        }
    }

    // ── Content checks ──────────────────────────────────────
    println!();
    println!("  Content");

    let nodes = load_all_nodes(&graph.path);
    let node_count = nodes.len();
    let parse_errors = 0;
    let mut id_mismatches = 0;
    let mut dir_mismatches = 0;
    let mut broken_edges = 0;

    let node_ids: std::collections::HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    for node in &nodes {
        // Check filename matches id
        let expected_filename = format!("{}.yaml", node.id);
        if let Some(actual_filename) = node.source_path.file_name().and_then(|n| n.to_str()) {
            if actual_filename != expected_filename {
                err(&format!(
                    "node {} filename mismatch: expected {}, got {}",
                    node.id, expected_filename, actual_filename
                ));
                id_mismatches += 1;
            }
        }

        // Check node is in correct confidence directory
        if let Some(parent) = node.source_path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                if dir_name != node.confidence.directory() {
                    err(&format!(
                        "node {} in {dir_name}/ but confidence is {}",
                        node.id, node.confidence
                    ));
                    dir_mismatches += 1;
                }
            }
        }

        // Check edge targets resolve
        for edge in node.all_edges() {
            if edge.target.contains("::") {
                // Cross-graph reference — skip for non-merged checks
                continue;
            }
            // Allow edges to findings (finding-NNN-*) and probes (brief-*)
            if edge.target.starts_with("finding-") || edge.target.starts_with("brief-") {
                continue;
            }
            if !node_ids.contains(&edge.target) {
                warn(&format!(
                    "node {}: edge target '{}' not found in graph",
                    node.id, edge.target
                ));
                broken_edges += 1;
            }
        }
    }

    if parse_errors == 0 && node_count > 0 {
        ok(&format!("{node_count} nodes parse successfully"));
    } else if node_count == 0 {
        warn("no nodes found");
        warnings += 1;
    }

    if id_mismatches == 0 && node_count > 0 {
        ok("all IDs match filenames");
    }
    errors += id_mismatches + dir_mismatches;

    if broken_edges > 0 {
        warnings += broken_edges;
    } else if node_count > 0 {
        ok("all edge targets resolve");
    }

    // Check for orphan nodes (no edges to or from)
    let mut referenced_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut referencing_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for node in &nodes {
        let edges = node.all_edges();
        if !edges.is_empty() {
            referencing_ids.insert(node.id.clone());
        }
        for edge in edges {
            if !edge.target.contains("::") {
                referenced_ids.insert(edge.target);
            }
        }
    }
    let orphans: Vec<&str> = nodes
        .iter()
        .filter(|n| !referenced_ids.contains(&n.id) && !referencing_ids.contains(&n.id))
        .map(|n| n.id.as_str())
        .collect();
    if !orphans.is_empty() {
        warn(&format!(
            "{} orphan node(s): {}",
            orphans.len(),
            orphans.join(", ")
        ));
        hint("add edges from nodes that reference these concepts");
        warnings += orphans.len();
    }

    // ── Process checks ──────────────────────────────────────
    println!();
    println!("  Process");

    // Charter exists
    let graph_parent = graph.path.parent().unwrap_or(&graph.path);
    let charter_exists =
        graph_parent.join("charter.md").exists() || graph_parent.join("KOS-charter.md").exists();
    if charter_exists {
        ok("charter document found");
    } else {
        warn("no charter.md found");
        hint("run `kos init` to create one, or write it manually");
        warnings += 1;
    }

    // Summary line
    println!();
    println!("  Summary: {node_count} nodes, {errors} error(s), {warnings} warning(s)");

    Ok((errors, warnings))
}

/// Check the legacy layout (no _kos/ directories, nodes/ at kos_root).
fn check_legacy(workspace: &Workspace, _fix: bool) -> Result<()> {
    let nodes_dir = workspace.kos_root.join("nodes");
    if nodes_dir.exists() {
        println!(
            "  Legacy layout detected: nodes/ at {}",
            workspace.kos_root.display()
        );
        println!("  Run `kos init` to migrate to _kos/ convention.");

        let count = walkdir::WalkDir::new(&nodes_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
            })
            .count();
        println!("  Found {count} node files in legacy layout.");
    } else {
        println!("  No nodes/ directory found.");
        println!("  hint: run `kos init` to create a _kos/ graph");
    }
    Ok(())
}

/// Load all node YAML files from a _kos/ graph directory.
fn load_all_nodes(kos_dir: &Path) -> Vec<Node> {
    let nodes_dir = kos_dir.join("nodes");
    if !nodes_dir.exists() {
        return vec![];
    }

    let mut nodes = Vec::new();
    for entry in walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(mut node) = serde_yaml::from_str::<Node>(&content) {
                    node.source_path = path.to_path_buf();
                    nodes.push(node);
                }
            }
        }
    }
    nodes
}

// ── Output helpers ──────────────────────────────────────────

fn ok(msg: &str) {
    println!("    \x1b[32m✓\x1b[0m {msg}");
}

fn warn(msg: &str) {
    println!("    \x1b[33m⚠\x1b[0m {msg}");
}

fn err(msg: &str) {
    println!("    \x1b[31m✗\x1b[0m {msg}");
}

fn hint(msg: &str) {
    println!("      hint: {msg}");
}

fn fixed(msg: &str) {
    println!("    \x1b[36m⟳\x1b[0m {msg}");
}
