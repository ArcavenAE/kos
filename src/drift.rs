#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use petgraph::visit::EdgeRef;
use sha2::{Digest, Sha256};

use crate::error::{KosError, Result};
use crate::model::{EdgeType, Node};

/// Snapshot of node content hashes, stored as JSON for diffability.
type Snapshot = BTreeMap<String, String>;

/// A node flagged as potentially stale.
#[derive(Debug)]
struct StaleNode {
    /// The node that is stale.
    id: String,
    /// Why it's stale — which upstream node changed.
    changed_upstream: Vec<String>,
}

/// A node that changed since the last snapshot.
#[derive(Debug)]
struct ChangedNode {
    id: String,
    old_hash: Option<String>,
    new_hash: String,
}

/// Summary of drift analysis.
#[derive(Debug)]
struct DriftReport {
    total_nodes: usize,
    /// Nodes reachable via edges (have at least one edge in or out).
    connected_nodes: usize,
    /// Nodes with no edges at all.
    isolated_nodes: Vec<String>,
    /// Nodes whose content changed since last snapshot.
    changed: Vec<ChangedNode>,
    /// Nodes downstream of changes, flagged as potentially stale.
    stale: Vec<StaleNode>,
    /// Whether this is the first run (no previous snapshot).
    baseline: bool,
}

/// Run the drift subcommand.
pub fn run(kos_root: &Path) -> Result<()> {
    let nodes = load_nodes(&kos_root.join("nodes"))?;
    let snapshot_path = kos_root.join(".drift-snapshot.json");

    let old_snapshot = load_snapshot(&snapshot_path);
    let new_snapshot = compute_snapshot(&nodes);
    let report = analyze(&nodes, &old_snapshot, &new_snapshot);

    print_report(&report);

    // Save new snapshot
    save_snapshot(&snapshot_path, &new_snapshot)?;

    Ok(())
}

fn load_nodes(nodes_dir: &Path) -> Result<Vec<Node>> {
    if !nodes_dir.exists() {
        return Ok(vec![]);
    }

    let mut nodes = Vec::new();
    for entry in walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;
        match serde_yaml::from_str::<Node>(&content) {
            Ok(mut node) => {
                node.source_path = path.to_path_buf();
                nodes.push(node);
            }
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", path.display());
            }
        }
    }

    Ok(nodes)
}

fn compute_snapshot(nodes: &[Node]) -> Snapshot {
    let mut snapshot = BTreeMap::new();
    for node in nodes {
        let hash = content_hash(&node.content);
        snapshot.insert(node.id.clone(), hash);
    }
    snapshot
}

/// Hash the content field of a node. Normalizes trailing whitespace
/// to reduce false positives from formatting-only changes.
fn content_hash(content: &str) -> String {
    let normalized = content.trim();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn load_snapshot(path: &Path) -> Option<Snapshot> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_snapshot(path: &Path, snapshot: &Snapshot) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot).map_err(|e| KosError::Init {
        message: format!("serialize snapshot: {e}"),
    })?;
    std::fs::write(path, json).map_err(KosError::Io)?;
    Ok(())
}

fn analyze(nodes: &[Node], old: &Option<Snapshot>, new_snapshot: &Snapshot) -> DriftReport {
    let baseline = old.is_none();
    let empty = BTreeMap::new();
    let old = old.as_ref().unwrap_or(&empty);

    // Build petgraph for edge traversal
    let mut graph = petgraph::graph::DiGraph::<String, EdgeType>::new();
    let mut id_to_idx: HashMap<&str, petgraph::graph::NodeIndex> = HashMap::new();

    for node in nodes {
        let idx = graph.add_node(node.id.clone());
        id_to_idx.insert(&node.id, idx);
    }

    for node in nodes {
        if let Some(&source_idx) = id_to_idx.get(node.id.as_str()) {
            for edge in node.all_edges() {
                if let Some(&target_idx) = id_to_idx.get(edge.target.as_str()) {
                    graph.add_edge(source_idx, target_idx, edge.edge_type.clone());
                }
            }
        }
    }

    // Find connected vs isolated nodes
    let mut has_edge: HashSet<&str> = HashSet::new();
    for edge_ref in graph.edge_references() {
        has_edge.insert(&graph[edge_ref.source()]);
        has_edge.insert(&graph[edge_ref.target()]);
    }

    let isolated_nodes: Vec<String> = nodes
        .iter()
        .filter(|n| !has_edge.contains(n.id.as_str()))
        .map(|n| n.id.clone())
        .collect();

    let connected_nodes = nodes.len() - isolated_nodes.len();

    // Detect changed nodes
    let mut changed: Vec<ChangedNode> = Vec::new();
    let mut changed_ids: HashSet<String> = HashSet::new();

    for (id, new_hash) in new_snapshot {
        let old_hash = old.get(id);
        if old_hash != Some(new_hash) {
            changed_ids.insert(id.clone());
            changed.push(ChangedNode {
                id: id.clone(),
                old_hash: old_hash.cloned(),
                new_hash: new_hash.clone(),
            });
        }
    }

    // Also detect removed nodes (in old but not in new)
    for id in old.keys() {
        if !new_snapshot.contains_key(id) {
            changed_ids.insert(id.clone());
            changed.push(ChangedNode {
                id: id.clone(),
                old_hash: old.get(id).cloned(),
                new_hash: "(removed)".to_string(),
            });
        }
    }

    // Walk edges downstream from changed nodes to find stale dependents
    let mut stale_map: HashMap<String, Vec<String>> = HashMap::new();

    for changed_id in &changed_ids {
        if let Some(&idx) = id_to_idx.get(changed_id.as_str()) {
            // Walk all nodes that derive FROM this changed node.
            // In our graph, edges go source --> target where source "derives from" target.
            // So if node A derives from node B, and B changed, A is stale.
            // That means we need to find nodes that point TO the changed node (incoming edges)
            // and then walk transitively.
            propagate_staleness(&graph, idx, changed_id, &changed_ids, &mut stale_map);
        }
    }

    let stale: Vec<StaleNode> = stale_map
        .into_iter()
        .map(|(id, mut changed_upstream)| {
            changed_upstream.sort();
            changed_upstream.dedup();
            StaleNode {
                id,
                changed_upstream,
            }
        })
        .collect();

    DriftReport {
        total_nodes: nodes.len(),
        connected_nodes,
        isolated_nodes,
        changed,
        stale,
        baseline,
    }
}

/// Propagate staleness from a changed node to its dependents.
///
/// Edge direction in kos: `A --derives--> B` means "A derives from B."
/// If B changed, A is stale. So we look for INCOMING edges to the changed node
/// (nodes that derive from it) and propagate upstream.
fn propagate_staleness(
    graph: &petgraph::graph::DiGraph<String, EdgeType>,
    changed_idx: petgraph::graph::NodeIndex,
    changed_id: &str,
    already_changed: &HashSet<String>,
    stale_map: &mut HashMap<String, Vec<String>>,
) {
    // BFS from the changed node, following incoming edges (reverse direction)
    let mut queue = std::collections::VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back(changed_idx);
    visited.insert(changed_idx);

    while let Some(current) = queue.pop_front() {
        // Find all nodes that point TO current (incoming edges)
        for edge_ref in graph.edges_directed(current, petgraph::Direction::Incoming) {
            let dependent_idx = edge_ref.source();
            let dependent_id = &graph[dependent_idx];
            let edge_type = edge_ref.weight();

            // Only propagate along derives and contradicts edges
            match edge_type {
                EdgeType::Derives | EdgeType::Contradicts => {}
                _ => continue,
            }

            // Don't flag nodes that are themselves changed (they'll be in the changed list)
            if already_changed.contains(dependent_id) {
                continue;
            }

            // Flag as stale
            stale_map
                .entry(dependent_id.clone())
                .or_default()
                .push(changed_id.to_string());

            // Continue propagation
            if visited.insert(dependent_idx) {
                queue.push_back(dependent_idx);
            }
        }
    }
}

fn print_report(report: &DriftReport) {
    println!("=== kos drift ===\n");

    // Coverage
    println!(
        "Graph: {} nodes, {} connected, {} isolated",
        report.total_nodes,
        report.connected_nodes,
        report.isolated_nodes.len()
    );
    if !report.isolated_nodes.is_empty() {
        for id in &report.isolated_nodes {
            println!("  isolated: {id}");
        }
    }
    let coverage = if report.total_nodes > 0 {
        (report.connected_nodes as f64 / report.total_nodes as f64) * 100.0
    } else {
        0.0
    };
    println!("Coverage: {coverage:.0}% of nodes reachable via edges\n");

    if report.baseline {
        println!("First run — baseline snapshot established.");
        println!("Run again after modifying nodes to detect drift.");
        return;
    }

    // Changes
    if report.changed.is_empty() && report.stale.is_empty() {
        println!("No drift detected. All nodes match snapshot.");
        return;
    }

    if !report.changed.is_empty() {
        println!("## Changed nodes\n");
        for c in &report.changed {
            if c.old_hash.is_none() {
                println!("  + {} (new)", c.id);
            } else if c.new_hash == "(removed)" {
                println!("  - {} (removed)", c.id);
            } else {
                println!("  ~ {} (content changed)", c.id);
            }
        }
        println!();
    }

    if !report.stale.is_empty() {
        println!("## Potentially stale (upstream changed)\n");
        for s in &report.stale {
            println!(
                "  ? {}  ← depends on: {}",
                s.id,
                s.changed_upstream.join(", ")
            );
        }
        println!();
    }

    let total_drift = report.changed.len() + report.stale.len();
    println!(
        "Summary: {} changed, {} potentially stale, {} total drift signals",
        report.changed.len(),
        report.stale.len(),
        total_drift
    );
}
