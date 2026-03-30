use std::collections::HashSet;
use std::path::Path;

use crate::error::{KosError, Result};
use crate::model::{Node, NodeType};

#[derive(Debug)]
pub struct ValidationResult {
    pub node_id: String,
    pub path: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn passed(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Run the validate subcommand against all nodes in the kos root.
pub fn run(kos_root: &Path) -> Result<()> {
    let nodes_dir = kos_root.join("nodes");
    if !nodes_dir.exists() {
        println!("no nodes/ directory found at {}", kos_root.display());
        return Ok(());
    }

    // First pass: load all nodes and collect IDs
    let (nodes, known_ids) = load_all_nodes(&nodes_dir)?;

    // Second pass: validate each node
    let mut results: Vec<ValidationResult> = Vec::new();
    for (node, rel_path) in &nodes {
        results.push(validate_node(node, rel_path, &known_ids));
    }

    // Output
    let mut pass_count = 0;
    let mut warn_count = 0;
    let mut fail_count = 0;

    for r in &results {
        if r.errors.is_empty() && r.warnings.is_empty() {
            pass_count += 1;
            println!("  PASS  {}", r.node_id);
        } else if r.errors.is_empty() {
            warn_count += 1;
            println!("  WARN  {}", r.node_id);
            for w in &r.warnings {
                println!("        ⚠ {w}");
            }
        } else {
            fail_count += 1;
            println!("  FAIL  {}", r.node_id);
            for e in &r.errors {
                println!("        ✗ {e}");
            }
            for w in &r.warnings {
                println!("        ⚠ {w}");
            }
        }
    }

    println!();
    println!(
        "{} nodes: {} passed, {} warnings, {} failed",
        results.len(),
        pass_count,
        warn_count,
        fail_count
    );

    if fail_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

type NodeWithPath = (Node, String);

/// Load all YAML files from nodes/**/*.yaml, returning (Node, relative_path) pairs.
fn load_all_nodes(nodes_dir: &Path) -> Result<(Vec<NodeWithPath>, HashSet<String>)> {
    let mut nodes = Vec::new();
    let mut known_ids = HashSet::new();

    for entry in walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let rel_path = path
            .strip_prefix(nodes_dir)
            .unwrap_or(path)
            .display()
            .to_string();

        let content = std::fs::read_to_string(path).map_err(KosError::Io)?;
        match serde_yaml::from_str::<Node>(&content) {
            Ok(mut node) => {
                node.source_path = path.to_path_buf();
                known_ids.insert(node.id.clone());
                nodes.push((node, rel_path));
            }
            Err(e) => {
                // Can't parse — report as a failed node
                nodes.push((
                    Node {
                        id: rel_path.clone(),
                        node_type: NodeType::Element,
                        confidence: crate::model::Confidence::Frontier,
                        title: String::new(),
                        content: String::new(),
                        edges: vec![],
                        depends_on: vec![],
                        graveyard: None,
                        provenance: None,
                        tags: vec![],
                        notes: None,
                        source_path: path.to_path_buf(),
                    },
                    rel_path.clone(),
                ));
                eprintln!("  PARSE ERROR  {rel_path}: {e}");
            }
        }
    }

    Ok((nodes, known_ids))
}

fn validate_node(node: &Node, rel_path: &str, known_ids: &HashSet<String>) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Filename matches id
    let expected_filename = format!("{}.yaml", node.id);
    if let Some(actual_filename) = Path::new(rel_path).file_name().and_then(|f| f.to_str()) {
        if actual_filename != expected_filename {
            errors.push(format!(
                "filename '{actual_filename}' does not match id '{}'",
                node.id
            ));
        }
    }

    // 2. File is in correct confidence directory
    let expected_dir = node.confidence.directory();
    if let Some(parent) = Path::new(rel_path).parent().and_then(|p| p.to_str()) {
        if parent != expected_dir {
            errors.push(format!(
                "in directory '{parent}' but confidence is '{}' (expected '{expected_dir}/')",
                node.confidence
            ));
        }
    }

    // 3. Edge targets reference known node IDs (warn, don't fail)
    for edge in node.all_edges() {
        if !known_ids.contains(&edge.target) {
            warnings.push(format!(
                "edge target '{}' not found in nodes/ (may be a finding or probe)",
                edge.target
            ));
        }
    }

    // 4. Graveyard type-specific: should have graveyard section
    if node.node_type == NodeType::Graveyard && node.graveyard.is_none() {
        warnings.push("type is 'graveyard' but missing graveyard section (approach, context, finding, ruling, reopener)".to_string());
    }

    ValidationResult {
        node_id: node.id.clone(),
        path: rel_path.to_string(),
        errors,
        warnings,
    }
}
