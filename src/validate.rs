use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::error::{KosError, Result};
use crate::findings::{self, FindingLoad};
use crate::model::{LEGAL_EDGE_TYPES, Node, NodeType};

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

/// Aggregate outcome of validating one graph. The caller decides the
/// process exit code; `run` never exits, so multi-graph validation
/// can accumulate results across graphs (kos#54 / aae-orc-z67m).
#[derive(Debug, Default, Clone, Copy)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub warnings: usize,
    pub failed: usize,
    pub parse_errors: usize,
    /// Findings examined across all three on-disk shapes.
    pub findings_total: usize,
    /// Findings that share a finding number with another file (structural
    /// failure; two files claiming the same finding).
    pub findings_failed: usize,
    /// Findings with an id/filename mismatch, or files that would not load.
    pub findings_warnings: usize,
}

impl Summary {
    pub fn clean(&self) -> bool {
        self.failed == 0 && self.parse_errors == 0 && self.findings_failed == 0
    }

    pub fn merge(&mut self, other: &Summary) {
        self.total += other.total;
        self.passed += other.passed;
        self.warnings += other.warnings;
        self.failed += other.failed;
        self.parse_errors += other.parse_errors;
        self.findings_total += other.findings_total;
        self.findings_failed += other.findings_failed;
        self.findings_warnings += other.findings_warnings;
    }
}

/// Run the validate subcommand against all nodes in the kos root.
pub fn run(kos_root: &Path) -> Result<Summary> {
    let nodes_dir = kos_root.join("nodes");
    if !nodes_dir.exists() {
        println!("no nodes/ directory found at {}", kos_root.display());
        return Ok(Summary::default());
    }

    // First pass: load all nodes and collect IDs
    let (nodes, known_ids) = load_all_nodes(&nodes_dir)?;

    // Second pass: validate parsed nodes, skip parse errors
    let mut results: Vec<ValidationResult> = Vec::new();
    let mut parse_error_count = 0;

    for loaded in &nodes {
        match loaded {
            LoadedNode::Parsed(node, rel_path) => {
                results.push(validate_node(node, rel_path, &known_ids));
            }
            LoadedNode::ParseError => {
                parse_error_count += 1;
            }
        }
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

    let total = results.len() + parse_error_count;
    println!();
    println!(
        "{total} nodes: {pass_count} passed, {warn_count} warnings, {fail_count} failed, {parse_error_count} parse errors",
    );

    // Findings pass; a second section over _kos/findings/. Findings are not
    // under nodes/, so the node passes above never saw them; the collision
    // between two files claiming the same finding number went unnoticed.
    let findings = validate_findings(&kos_root.join("findings"));

    Ok(Summary {
        total,
        passed: pass_count,
        warnings: warn_count,
        failed: fail_count,
        parse_errors: parse_error_count,
        findings_total: findings.total,
        findings_failed: findings.failed,
        findings_warnings: findings.warnings,
    })
}

/// Counts from the findings pass, folded into the returned `Summary`.
#[derive(Default)]
struct FindingsReport {
    total: usize,
    failed: usize,
    warnings: usize,
}

/// Validate findings: fail on two files claiming the same finding number, warn
/// on id/filename drift and on files that will not load. Shape is never an
/// error; pure yaml, md+frontmatter, and legacy bare md are all valid. A
/// duplicate id is structural well-formedness (may gate per ADR-007), not a
/// health metric.
fn validate_findings(findings_dir: &Path) -> FindingsReport {
    let mut report = FindingsReport::default();
    if !findings_dir.exists() {
        return report;
    }

    let loads = match findings::load_findings(findings_dir) {
        Ok(l) => l,
        Err(e) => {
            println!();
            println!("=== findings ===");
            println!("  could not read findings/: {e}");
            return report;
        }
    };

    // Group loaded findings by finding number so two files claiming the same
    // number are visible; carry an id/stem-drift warning per file.
    let mut by_key: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut drift: Vec<(String, String)> = Vec::new();
    let mut unloadable: Vec<(String, String)> = Vec::new();

    for load in loads {
        match load {
            FindingLoad::Loaded(l) => {
                report.total += 1;
                let key = findings::finding_key(&l.node.id);
                by_key
                    .entry(key)
                    .or_default()
                    .push((l.node.id.clone(), l.stem.clone()));
                if l.node.id != l.stem {
                    drift.push((l.node.id.clone(), l.stem.clone()));
                }
            }
            FindingLoad::Unloadable { path, error } => {
                report.total += 1;
                let name = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("?")
                    .to_string();
                unloadable.push((name, error));
            }
        }
    }

    println!();
    println!("=== findings ===");

    // Duplicate finding numbers; the structural failure.
    for (key, members) in &by_key {
        if members.len() > 1 {
            report.failed += members.len();
            println!(
                "  FAIL  duplicate finding number '{key}' across {} files:",
                members.len()
            );
            for (id, stem) in members {
                if id == stem {
                    println!("        ✗ {stem}");
                } else {
                    println!("        ✗ {stem} (id '{id}')");
                }
            }
        }
    }

    // id/filename drift; a warning, not a failure.
    for (id, stem) in &drift {
        report.warnings += 1;
        println!("  WARN  id '{id}' does not match filename stem '{stem}'");
    }

    // Files that would not load at all; kept visible as a warning.
    for (name, error) in &unloadable {
        report.warnings += 1;
        println!("  WARN  could not load {name}: {error}");
    }

    println!();
    println!(
        "{} findings: {} duplicate-id failures, {} warnings",
        report.total, report.failed, report.warnings
    );

    report
}

/// A loaded node: either successfully parsed or failed to parse.
enum LoadedNode {
    Parsed(Box<Node>, String),
    ParseError,
}

/// Load all YAML files from nodes/**/*.yaml.
fn load_all_nodes(nodes_dir: &Path) -> Result<(Vec<LoadedNode>, HashSet<String>)> {
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
                nodes.push(LoadedNode::Parsed(Box::new(node), rel_path));
            }
            Err(e) => {
                // Parse error is the complete report; no further validation
                println!("  PARSE ERROR  {rel_path}: {e}");
                nodes.push(LoadedNode::ParseError);
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

        // 3b. Edge type is in the schema vocabulary. The loader is lenient so
        // one bad edge never hides a node from readers; the gate lives here,
        // at authorship, where the author can still fix it.
        if let Some(raw) = edge.edge_type.unknown_value() {
            errors.push(format!(
                "edge to '{}' has unknown type '{raw}' (expected one of: {})",
                edge.target,
                LEGAL_EDGE_TYPES.join(", ")
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    /// A minimal but valid kos root: one well-formed bedrock node, plus a
    /// findings/ dir the caller populates. Findings validation only runs when
    /// nodes/ exists, so the node is required scaffolding.
    fn scaffold() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let bedrock = root.join("nodes").join("bedrock");
        fs::create_dir_all(&bedrock).unwrap();
        fs::write(
            bedrock.join("elem-anchor.yaml"),
            "id: elem-anchor\ntype: element\nconfidence: bedrock\ntitle: anchor\ncontent: a body\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("findings")).unwrap();
        (dir, root)
    }

    fn write_finding(root: &Path, name: &str, text: &str) {
        fs::write(root.join("findings").join(name), text).unwrap();
    }

    #[test]
    fn duplicate_finding_number_fails() {
        let (_guard, root) = scaffold();
        // Two distinct findings, different slugs, SAME number; the collision.
        write_finding(
            &root,
            "finding-123-harness-invocation.md",
            "# harness invocation\n\n**Date:** 2026-08-01\n\nbody one\n",
        );
        write_finding(
            &root,
            "finding-123-org-owned-fork.md",
            "# org owned fork\n\n**Date:** 2026-08-02\n\nbody two\n",
        );

        let summary = run(&root).unwrap();
        assert!(
            !summary.clean(),
            "a duplicate finding number must fail the graph"
        );
        assert_eq!(summary.findings_total, 2);
        assert_eq!(summary.findings_failed, 2);
    }

    #[test]
    fn distinct_finding_numbers_pass() {
        let (_guard, root) = scaffold();
        write_finding(
            &root,
            "finding-201-alpha.md",
            "# alpha\n\n**Date:** 2026-08-01\n\nbody\n",
        );
        write_finding(
            &root,
            "finding-202-beta.yaml",
            "id: finding-202-beta\ntype: finding\nconfidence: frontier\ntitle: beta\ncontent: body\n",
        );

        let summary = run(&root).unwrap();
        assert!(summary.clean(), "distinct finding numbers must pass");
        assert_eq!(summary.findings_total, 2);
        assert_eq!(summary.findings_failed, 0);
    }

    #[test]
    fn id_filename_drift_warns_without_failing() {
        let (_guard, root) = scaffold();
        // Frontmatter id disagrees with the filename stem: a drift warning.
        write_finding(
            &root,
            "finding-210-on-disk.md",
            "---\nid: finding-210-declared-differently\ntype: finding\nconfidence: frontier\n---\n# t\nbody\n",
        );

        let summary = run(&root).unwrap();
        assert!(summary.clean(), "id/stem drift is a warning, not a failure");
        assert_eq!(summary.findings_warnings, 1);
    }

    #[test]
    fn unloadable_yaml_finding_warns_but_does_not_fail() {
        let (_guard, root) = scaffold();
        write_finding(&root, "finding-220-broken.yaml", "id: [unterminated\n");

        let summary = run(&root).unwrap();
        assert!(summary.clean(), "an unloadable finding warns, never fails");
        assert_eq!(summary.findings_total, 1);
        assert_eq!(summary.findings_warnings, 1);
    }
}
