#![forbid(unsafe_code)]

//! `kos reflect` — session-close retrospection audit.
//!
//! Surfaces what a session did in the graph's vocabulary so the operator
//! can tell whether a session converged without external validation.
//! Read-only; never modifies the graph.
//!
//! See `_kos/probes/brief-kos-reflect.yaml` for the hypothesis this
//! subcommand exists to test.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::Result;
use crate::model::{Confidence, Node};
use crate::workspace::Workspace;

// ── Report types ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct Reflection {
    since: String,
    head: String,
    commits: Vec<CommitInfo>,
    added: Vec<NodeChange>,
    moved: Vec<NodeMove>,
    deleted: Vec<NodeChange>,
    findings: Vec<FindingChange>,
    briefs_added: Vec<ArtifactRef>,
    briefs_in_flight: Vec<ArtifactRef>,
    ideas_added: Vec<ArtifactRef>,
    charter_deltas: Vec<CharterDelta>,
    un_updated: Vec<UnUpdated>,
    calibration: CalibrationStats,
}

#[derive(Debug, Serialize)]
struct CommitInfo {
    sha: String,
    subject: String,
}

#[derive(Debug, Serialize)]
struct NodeChange {
    id: String,
    confidence: String,
    title: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct NodeMove {
    id: String,
    from: String,
    to: String,
    kind: String, // "promote" | "demote" | "graveyard" | "other"
    title: String,
}

#[derive(Debug, Serialize)]
struct FindingChange {
    id: String,
    title: String,
    probe: Option<String>,
    result: Option<String>,
    surprise: Option<String>,
    predicted_confidence: Option<f64>,
    actual_confidence: String,
    delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactRef {
    id: String,
    title: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct CharterDelta {
    path: String,
    lines_before: Option<usize>,
    lines_after: usize,
    delta: Option<i64>,
}

#[derive(Debug, Serialize)]
struct UnUpdated {
    dependent_id: String,
    dependent_title: String,
    changed_upstream: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
struct CalibrationStats {
    pairs: usize,
    mean_delta: Option<f64>,
    mean_surprise_score: Option<f64>,
    entries: Vec<CalibrationEntry>,
}

#[derive(Debug, Serialize)]
struct CalibrationEntry {
    brief: String,
    finding: String,
    predicted: f64,
    actual: f64,
    delta: f64,
    surprise: Option<String>,
}

// ── Entry point ──────────────────────────────────────────────

pub fn run(workspace: &Workspace, cwd: &Path, since: Option<&str>, json: bool) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let repo_root = infer_repo_root(&graph_root, &workspace.root);

    let since_ref = resolve_since(&repo_root, since)?;
    let head_ref = run_git(&repo_root, &["rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|| "HEAD".to_string())
        .trim()
        .to_string();

    let reflection = build_reflection(&graph_root, &repo_root, &since_ref, &head_ref, workspace)?;

    if json {
        match serde_json::to_string_pretty(&reflection) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("json serialization failed: {e}"),
        }
    } else {
        render_markdown(&reflection);
    }

    Ok(())
}

// ── Scope resolution ─────────────────────────────────────────

fn resolve_graph_root(workspace: &Workspace, cwd: &Path) -> PathBuf {
    if let Some(graph) = workspace.nearest_graph(cwd) {
        return graph.path.clone();
    }
    workspace.node_root()
}

/// Infer the git repo root that contains this graph.
/// For orc graphs, graph_root = <orc>/_kos, repo = <orc>.
/// For subrepo graphs, graph_root = <subrepo>/_kos, repo = <subrepo>.
fn infer_repo_root(graph_root: &Path, workspace_root: &Path) -> PathBuf {
    graph_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| workspace_root.to_path_buf())
}

// ── Session boundary detection ───────────────────────────────

fn resolve_since(repo_root: &Path, explicit: Option<&str>) -> Result<String> {
    if let Some(r) = explicit {
        return Ok(r.to_string());
    }
    if let Some(sha) = find_last_boundary_commit(repo_root) {
        return Ok(sha);
    }
    // Fallback: previous commit. If that fails too (single-commit repo),
    // fall back to the empty tree so everything shows as added.
    if run_git(repo_root, &["rev-parse", "HEAD^"]).is_some() {
        Ok("HEAD^".to_string())
    } else {
        Ok(EMPTY_TREE.to_string())
    }
}

/// Git's canonical empty tree SHA. Safe fallback for first-commit repos.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

fn find_last_boundary_commit(repo_root: &Path) -> Option<String> {
    // Skip HEAD itself — if HEAD is a harvest, we want the previous one.
    let out = run_git(
        repo_root,
        &["log", "--pretty=%H|%s", "--skip=1", "-200", "HEAD"],
    )?;
    for line in out.lines() {
        let (sha, subject) = line.split_once('|')?;
        let trimmed = subject.trim_start();
        if is_boundary_subject(trimmed) {
            return Some(sha.to_string());
        }
    }
    None
}

fn is_boundary_subject(subject: &str) -> bool {
    // `harvest(scope): ...` or `charter(scope): ...` marks a session close.
    subject.starts_with("harvest(") || subject.starts_with("charter(")
}

// ── Build reflection ─────────────────────────────────────────

fn build_reflection(
    graph_root: &Path,
    repo_root: &Path,
    since: &str,
    head: &str,
    _workspace: &Workspace,
) -> Result<Reflection> {
    let commits = git_commits_in_range(repo_root, since);
    let graph_rel = graph_root
        .strip_prefix(repo_root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| graph_root.display().to_string());

    let diff_entries = git_diff_name_status(repo_root, since, &graph_rel);

    let (added, moved, deleted, briefs_added, findings_added, ideas_added, _charter_edits) =
        classify_changes(graph_root, repo_root, &diff_entries);

    let mut changed_ids: HashSet<String> = HashSet::new();
    for c in &added {
        changed_ids.insert(c.id.clone());
    }
    for m in &moved {
        changed_ids.insert(m.id.clone());
    }
    for d in &deleted {
        changed_ids.insert(d.id.clone());
    }
    for b in &briefs_added {
        changed_ids.insert(b.id.clone());
    }
    for f in &findings_added {
        changed_ids.insert(f.id.clone());
    }

    let all_nodes = load_all_nodes(graph_root);
    let un_updated = detect_un_updated(&all_nodes, &changed_ids);

    let briefs_by_id: HashMap<String, &Node> =
        all_nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let findings = findings_with_calibration(&findings_added, graph_root, &briefs_by_id);

    let briefs_in_flight = detect_in_flight(graph_root, &all_nodes);

    let charter_deltas = charter_line_deltas(repo_root, since);

    let calibration = running_calibration(graph_root, &all_nodes);

    Ok(Reflection {
        since: since.to_string(),
        head: head.to_string(),
        commits,
        added,
        moved,
        deleted,
        findings,
        briefs_added,
        briefs_in_flight,
        ideas_added,
        charter_deltas,
        un_updated,
        calibration,
    })
}

// ── Git helpers ──────────────────────────────────────────────

fn run_git(root: &Path, args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
}

fn git_commits_in_range(repo_root: &Path, since: &str) -> Vec<CommitInfo> {
    let range = format!("{since}..HEAD");
    let out = match run_git(repo_root, &["log", "--pretty=%h|%s", &range]) {
        Some(s) => s,
        None => return vec![],
    };
    out.lines()
        .filter_map(|line| {
            line.split_once('|').map(|(sha, sub)| CommitInfo {
                sha: sha.to_string(),
                subject: sub.to_string(),
            })
        })
        .collect()
}

#[derive(Debug)]
struct DiffEntry {
    status: char,
    old_path: Option<String>,
    new_path: String,
}

fn git_diff_name_status(repo_root: &Path, since: &str, scope_path: &str) -> Vec<DiffEntry> {
    let range = format!("{since}..HEAD");
    let out = match run_git(
        repo_root,
        &["diff", "--name-status", "-M80", &range, "--", scope_path],
    ) {
        Some(s) => s,
        None => return vec![],
    };

    let mut entries = Vec::new();
    for line in out.lines() {
        let mut parts = line.split('\t');
        let status = parts.next().unwrap_or("");
        let first_char = status.chars().next().unwrap_or('?');
        let p1 = parts.next();
        let p2 = parts.next();
        match (first_char, p1, p2) {
            ('R', Some(old), Some(new)) => entries.push(DiffEntry {
                status: 'R',
                old_path: Some(old.to_string()),
                new_path: new.to_string(),
            }),
            (ch, Some(path), _) => entries.push(DiffEntry {
                status: ch,
                old_path: None,
                new_path: path.to_string(),
            }),
            _ => continue,
        }
    }
    entries
}

// ── Classification ───────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn classify_changes(
    graph_root: &Path,
    repo_root: &Path,
    diff: &[DiffEntry],
) -> (
    Vec<NodeChange>,
    Vec<NodeMove>,
    Vec<NodeChange>,
    Vec<ArtifactRef>,
    Vec<ArtifactRef>,
    Vec<ArtifactRef>,
    Vec<String>,
) {
    let mut added = Vec::new();
    let mut moved = Vec::new();
    let mut deleted = Vec::new();
    let mut briefs_added = Vec::new();
    let mut findings_added = Vec::new();
    let mut ideas_added = Vec::new();
    let mut charter_edits = Vec::new();

    for entry in diff {
        let p = &entry.new_path;
        let kind = classify_path(p);

        match (entry.status, kind) {
            ('R', PathKind::Node) => {
                // Rename within nodes/ — usually a confidence move.
                let Some(old_path) = entry.old_path.as_deref() else {
                    continue;
                };
                let old_conf = confidence_from_path(old_path);
                let new_conf = confidence_from_path(p);
                let move_kind = classify_move(old_conf.as_deref(), new_conf.as_deref());
                let node_opt = load_node_at(&repo_root.join(p));
                let (id, title) = node_opt
                    .as_ref()
                    .map(|n| (n.id.clone(), n.title.clone()))
                    .unwrap_or_else(|| (stem(p).to_string(), stem(p).to_string()));
                moved.push(NodeMove {
                    id,
                    title,
                    from: old_conf.unwrap_or_else(|| "?".into()),
                    to: new_conf.unwrap_or_else(|| "?".into()),
                    kind: move_kind.into(),
                });
            }
            ('A', PathKind::Node) | ('M', PathKind::Node) if entry.status == 'A' => {
                push_node_change(&mut added, repo_root, p, graph_root);
            }
            ('A', PathKind::Node) => {
                push_node_change(&mut added, repo_root, p, graph_root);
            }
            ('D', PathKind::Node) => {
                let id = stem(p).to_string();
                let confidence = confidence_from_path(p).unwrap_or_else(|| "?".into());
                deleted.push(NodeChange {
                    id,
                    confidence,
                    title: String::new(),
                    path: p.clone(),
                });
            }
            ('A', PathKind::Finding) => {
                if let Some(refer) = load_artifact_ref(repo_root, p) {
                    findings_added.push(refer);
                }
            }
            ('A', PathKind::Brief) => {
                if let Some(refer) = load_artifact_ref(repo_root, p) {
                    briefs_added.push(refer);
                }
            }
            ('A', PathKind::Idea) => {
                let title = idea_title(&repo_root.join(p));
                ideas_added.push(ArtifactRef {
                    id: stem(p).to_string(),
                    title,
                    path: p.clone(),
                });
            }
            ('M' | 'A', PathKind::Charter) => {
                charter_edits.push(p.clone());
            }
            _ => {}
        }
    }

    (
        added,
        moved,
        deleted,
        briefs_added,
        findings_added,
        ideas_added,
        charter_edits,
    )
}

#[derive(Debug, PartialEq, Eq)]
enum PathKind {
    Node,
    Finding,
    Brief,
    Idea,
    Charter,
    Other,
}

fn classify_path(path: &str) -> PathKind {
    if path.contains("/_kos/nodes/") || path.contains("_kos/nodes/") {
        PathKind::Node
    } else if path.contains("/_kos/findings/") || path.contains("_kos/findings/") {
        PathKind::Finding
    } else if path.contains("/_kos/probes/") || path.contains("_kos/probes/") {
        PathKind::Brief
    } else if path.contains("/_kos/ideas/") || path.contains("_kos/ideas/") {
        PathKind::Idea
    } else if path.ends_with("charter.md") || path.ends_with("KOS-charter.md") {
        PathKind::Charter
    } else {
        PathKind::Other
    }
}

fn confidence_from_path(path: &str) -> Option<String> {
    // _kos/nodes/<confidence>/<slug>.yaml
    let tokens: Vec<&str> = path.split('/').collect();
    for (i, tok) in tokens.iter().enumerate() {
        if *tok == "nodes" {
            if let Some(conf) = tokens.get(i + 1) {
                return Some((*conf).to_string());
            }
        }
    }
    None
}

fn classify_move(from: Option<&str>, to: Option<&str>) -> &'static str {
    let order = |c: &str| match c {
        "graveyard" => 0,
        "placeholder" => 1,
        "frontier" => 2,
        "bedrock" => 3,
        _ => -1,
    };
    let (Some(f), Some(t)) = (from, to) else {
        return "other";
    };
    if t == "graveyard" {
        return "graveyard";
    }
    let fo = order(f);
    let to = order(t);
    if fo < 0 || to < 0 {
        return "other";
    }
    if to > fo {
        "promote"
    } else if to < fo {
        "demote"
    } else {
        "other"
    }
}

fn push_node_change(
    out: &mut Vec<NodeChange>,
    repo_root: &Path,
    rel_path: &str,
    _graph_root: &Path,
) {
    let node = load_node_at(&repo_root.join(rel_path));
    let (id, confidence, title) = match node {
        Some(n) => (n.id.clone(), n.confidence.to_string(), n.title.clone()),
        None => (
            stem(rel_path).to_string(),
            confidence_from_path(rel_path).unwrap_or_else(|| "?".into()),
            String::new(),
        ),
    };
    out.push(NodeChange {
        id,
        confidence,
        title,
        path: rel_path.to_string(),
    });
}

fn stem(path: &str) -> &str {
    path.rsplit('/')
        .next()
        .unwrap_or(path)
        .strip_suffix(".yaml")
        .or_else(|| path.rsplit('/').next().and_then(|s| s.strip_suffix(".md")))
        .unwrap_or(path)
}

// ── Node loading ─────────────────────────────────────────────

fn load_node_at(path: &Path) -> Option<Node> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut node: Node = serde_yaml::from_str(&content).ok()?;
    node.source_path = path.to_path_buf();
    Some(node)
}

fn load_all_nodes(graph_root: &Path) -> Vec<Node> {
    let mut all = Vec::new();
    for sub in ["nodes", "findings", "probes"] {
        let dir = graph_root.join(sub);
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            if let Some(node) = load_node_at(p) {
                all.push(node);
            }
        }
    }
    all
}

fn load_artifact_ref(repo_root: &Path, rel_path: &str) -> Option<ArtifactRef> {
    let node = load_node_at(&repo_root.join(rel_path))?;
    Some(ArtifactRef {
        id: node.id,
        title: node.title,
        path: rel_path.to_string(),
    })
}

fn idea_title(path: &Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    for line in content.lines().take(10) {
        if let Some(rest) = line.strip_prefix("# ") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

// ── Calibration (per-finding) ────────────────────────────────

fn findings_with_calibration(
    findings_added: &[ArtifactRef],
    graph_root: &Path,
    nodes_by_id: &HashMap<String, &Node>,
) -> Vec<FindingChange> {
    let mut out = Vec::new();
    let findings_dir = graph_root.join("findings");
    for f in findings_added {
        // The artifact path is relative to repo root; re-derive absolute.
        let abs = findings_dir.join(
            Path::new(&f.path)
                .file_name()
                .unwrap_or_else(|| f.path.as_ref()),
        );
        let node = match load_node_at(&abs) {
            Some(n) => n,
            None => {
                // Fallback: try loading via id match among loaded nodes.
                match nodes_by_id.get(&f.id) {
                    Some(n) => Node {
                        id: n.id.clone(),
                        node_type: n.node_type.clone(),
                        confidence: n.confidence.clone(),
                        title: n.title.clone(),
                        content: n.content.clone(),
                        edges: n.edges.clone(),
                        depends_on: n.depends_on.clone(),
                        graveyard: n.graveyard.clone(),
                        brief: n.brief.clone(),
                        finding: n.finding.clone(),
                        compaction: n.compaction.clone(),
                        provenance: n.provenance.clone(),
                        tags: n.tags.clone(),
                        notes: n.notes.clone(),
                        source_path: n.source_path.clone(),
                    },
                    None => continue,
                }
            }
        };

        let finding_section = node.finding.as_ref();
        let probe_id = finding_section.and_then(|s| s.probe.clone());
        let result = finding_section.and_then(|s| s.result.clone());
        let surprise = finding_section.and_then(|s| s.surprise_magnitude.clone());

        let predicted = probe_id
            .as_ref()
            .and_then(|pid| nodes_by_id.get(pid))
            .and_then(|brief_node| brief_node.brief.as_ref())
            .and_then(|b| b.predicted_confidence);

        let actual_score = confidence_score(&node.confidence);
        let delta = predicted.map(|p| actual_score - p);

        out.push(FindingChange {
            id: node.id.clone(),
            title: node.title.clone(),
            probe: probe_id,
            result,
            surprise,
            predicted_confidence: predicted,
            actual_confidence: node.confidence.to_string(),
            delta,
        });
    }
    out
}

fn confidence_score(c: &Confidence) -> f64 {
    match c {
        Confidence::Bedrock => 1.0,
        Confidence::Frontier => 0.5,
        Confidence::Placeholder => 0.25,
        Confidence::Graveyard => 0.0,
    }
}

// ── In-flight probes ─────────────────────────────────────────

fn detect_in_flight(graph_root: &Path, all_nodes: &[Node]) -> Vec<ArtifactRef> {
    // Collect all brief IDs.
    let briefs: Vec<&Node> = all_nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::model::NodeType::Brief))
        .collect();

    // Collect probe references from findings.
    let referenced: HashSet<String> = all_nodes
        .iter()
        .filter_map(|n| n.finding.as_ref().and_then(|f| f.probe.clone()))
        .collect();

    briefs
        .into_iter()
        .filter(|b| !referenced.contains(&b.id))
        .map(|b| ArtifactRef {
            id: b.id.clone(),
            title: b.title.clone(),
            path: b
                .source_path
                .strip_prefix(graph_root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| b.source_path.display().to_string()),
        })
        .collect()
}

// ── Un-updated dependents ────────────────────────────────────

fn detect_un_updated(all_nodes: &[Node], changed_ids: &HashSet<String>) -> Vec<UnUpdated> {
    let mut flagged: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for node in all_nodes {
        if changed_ids.contains(&node.id) {
            continue;
        }
        for edge in node.all_edges() {
            if changed_ids.contains(&edge.target) {
                flagged
                    .entry(node.id.clone())
                    .or_insert_with(|| (node.title.clone(), Vec::new()))
                    .1
                    .push(edge.target);
            }
        }
    }
    flagged
        .into_iter()
        .map(|(id, (title, mut upstream))| {
            upstream.sort();
            upstream.dedup();
            UnUpdated {
                dependent_id: id,
                dependent_title: title,
                changed_upstream: upstream,
            }
        })
        .collect()
}

// ── Charter deltas ───────────────────────────────────────────

fn charter_line_deltas(repo_root: &Path, since: &str) -> Vec<CharterDelta> {
    let candidates = ["charter.md", "KOS-charter.md"];
    let mut out = Vec::new();
    for name in candidates {
        let path = repo_root.join(name);
        if !path.exists() {
            continue;
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        let lines_after = current.lines().count();
        let before = run_git(repo_root, &["show", &format!("{since}:{name}")]);
        let (lines_before, delta) = match before {
            Some(s) => {
                let lb = s.lines().count();
                (Some(lb), Some(lines_after as i64 - lb as i64))
            }
            None => (None, None),
        };
        out.push(CharterDelta {
            path: name.to_string(),
            lines_before,
            lines_after,
            delta,
        });
    }
    out
}

// ── Running calibration (historic) ───────────────────────────

fn running_calibration(_graph_root: &Path, all_nodes: &[Node]) -> CalibrationStats {
    let briefs_by_id: HashMap<&str, &Node> = all_nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::model::NodeType::Brief))
        .map(|n| (n.id.as_str(), n))
        .collect();

    let mut entries = Vec::new();
    for finding in all_nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::model::NodeType::Finding))
    {
        let fs = match finding.finding.as_ref() {
            Some(fs) => fs,
            None => continue,
        };
        let probe_id = match fs.probe.as_deref() {
            Some(p) => p,
            None => continue,
        };
        let brief = match briefs_by_id.get(probe_id) {
            Some(b) => b,
            None => continue,
        };
        let predicted = match brief.brief.as_ref().and_then(|b| b.predicted_confidence) {
            Some(p) => p,
            None => continue,
        };
        let actual = confidence_score(&finding.confidence);
        entries.push(CalibrationEntry {
            brief: brief.id.clone(),
            finding: finding.id.clone(),
            predicted,
            actual,
            delta: actual - predicted,
            surprise: fs.surprise_magnitude.clone(),
        });
    }

    let pairs = entries.len();
    let mean_delta = if pairs == 0 {
        None
    } else {
        Some(entries.iter().map(|e| e.delta).sum::<f64>() / pairs as f64)
    };
    let surprise_scores: Vec<f64> = entries
        .iter()
        .filter_map(|e| e.surprise.as_deref().map(surprise_score))
        .collect();
    let mean_surprise_score = if surprise_scores.is_empty() {
        None
    } else {
        Some(surprise_scores.iter().sum::<f64>() / surprise_scores.len() as f64)
    };

    CalibrationStats {
        pairs,
        mean_delta,
        mean_surprise_score,
        entries,
    }
}

fn surprise_score(s: &str) -> f64 {
    match s.to_ascii_lowercase().as_str() {
        "none" => 0.0,
        "low" => 0.25,
        "moderate" => 0.5,
        "high" => 1.0,
        _ => 0.0,
    }
}

// ── Rendering ────────────────────────────────────────────────

fn render_markdown(r: &Reflection) {
    println!("=== kos reflect ===\n");
    println!("Range: {}..{}", short(&r.since), short(&r.head));
    println!(
        "Commits: {} ({} shown)",
        r.commits.len(),
        r.commits.len().min(20)
    );
    for c in r.commits.iter().take(20) {
        println!("  {} {}", c.sha, c.subject);
    }
    if r.commits.len() > 20 {
        println!("  ... {} more", r.commits.len() - 20);
    }
    println!();

    section("Confidence changes", || {
        let mut any = false;
        if !r.added.is_empty() {
            any = true;
            for n in &r.added {
                println!("  + [{}] {} — {}", n.confidence, n.id, n.title);
            }
        }
        if !r.moved.is_empty() {
            any = true;
            for m in &r.moved {
                let glyph = match m.kind.as_str() {
                    "promote" => "↑",
                    "demote" => "↓",
                    "graveyard" => "✕",
                    _ => "·",
                };
                println!(
                    "  {glyph} {} ({} → {}) [{}] — {}",
                    m.id, m.from, m.to, m.kind, m.title
                );
            }
        }
        if !r.deleted.is_empty() {
            any = true;
            for d in &r.deleted {
                println!("  - [{}] {} (deleted)", d.confidence, d.id);
            }
        }
        if !any {
            println!("  (none)");
        }
    });

    section("Findings produced", || {
        if r.findings.is_empty() {
            println!("  (none)");
            return;
        }
        for f in &r.findings {
            println!("  • {} — {}", f.id, f.title);
            if let Some(p) = &f.probe {
                println!("      probe:   {p}");
            }
            if let Some(res) = &f.result {
                println!("      result:  {res}");
            }
            match (f.predicted_confidence, &f.delta) {
                (Some(p), Some(d)) => println!(
                    "      calibration: predicted {:.2} → actual {} (delta {:+.2})",
                    p, f.actual_confidence, d
                ),
                (Some(p), None) => println!(
                    "      calibration: predicted {p:.2}, actual {}",
                    f.actual_confidence
                ),
                _ => println!("      calibration: (no predicted_confidence on brief)"),
            }
            if let Some(s) = &f.surprise {
                println!("      surprise: {s}");
            }
        }
    });

    section("Probes / briefs created this session", || {
        if r.briefs_added.is_empty() {
            println!("  (none)");
            return;
        }
        for b in &r.briefs_added {
            println!("  • {} — {}", b.id, b.title);
        }
    });

    section("Ideas created this session", || {
        if r.ideas_added.is_empty() {
            println!("  (none)");
            return;
        }
        for i in &r.ideas_added {
            let t = if i.title.is_empty() { &i.id } else { &i.title };
            println!("  • {} — {}", i.id, t);
        }
    });

    section("In-flight probes (harvest debt, graph-wide)", || {
        if r.briefs_in_flight.is_empty() {
            println!("  (none — every brief has a finding)");
            return;
        }
        for b in &r.briefs_in_flight {
            println!("  ⧗ {} — {}", b.id, b.title);
        }
    });

    section("Charter delta", || {
        if r.charter_deltas.is_empty() {
            println!("  (no charter files found)");
            return;
        }
        for c in &r.charter_deltas {
            match (c.lines_before, c.delta) {
                (Some(lb), Some(d)) => {
                    let flag = if d > 0 && r.added.is_empty() && r.moved.is_empty() {
                        "  ⚠ grew without new bedrock/move"
                    } else {
                        ""
                    };
                    println!("  {}: {} → {} ({:+}){flag}", c.path, lb, c.lines_after, d);
                }
                _ => println!("  {}: {} (no baseline)", c.path, c.lines_after),
            }
        }
    });

    section(
        "Un-updated dependents (self-healing at session scope)",
        || {
            if r.un_updated.is_empty() {
                println!("  (none)");
                return;
            }
            for u in &r.un_updated {
                println!(
                    "  ? {} — depends on: {}",
                    u.dependent_id,
                    u.changed_upstream.join(", ")
                );
            }
        },
    );

    section("Running calibration (historic, all sessions)", || {
        if r.calibration.pairs == 0 {
            println!("  (no brief↔finding pairs yet)");
            return;
        }
        println!("  Pairs:         {}", r.calibration.pairs);
        if let Some(md) = r.calibration.mean_delta {
            println!("  Mean delta:    {md:+.2}  (actual - predicted, -1..+1)");
        }
        if let Some(ms) = r.calibration.mean_surprise_score {
            println!("  Mean surprise: {ms:.2}  (0=none, 1=high)");
        }
        if r.calibration.pairs < 3 {
            println!("  (need >=3 pairs before trend claims are meaningful)");
        }
    });
}

fn section(title: &str, body: impl FnOnce()) {
    println!("## {title}\n");
    body();
    println!();
}

fn short(s: &str) -> &str {
    // `HEAD^`, refs, or short SHAs — leave as-is; truncate long SHAs.
    if s.len() > 12 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        &s[..12]
    } else {
        s
    }
}
