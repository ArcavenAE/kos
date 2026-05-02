use std::path::Path;

use crate::error::{KosError, Result};
use crate::model::{Confidence, GraphScope, Node};
use crate::workspace::Workspace;

const HEADER: &str = "<!--
This file is rendered by `kos charter render`.
Do not hand-edit projection sections — edit the underlying nodes
in `_kos/nodes/{bedrock,frontier,graveyard}/*.yaml`.
Backdrop blocks (between `<!-- backdrop:start -->` and
`<!-- backdrop:end -->` markers) are preserved verbatim.
-->\n\n";

/// Render the orchestrator charter from the kos graph.
///
/// First-pass implementation per `_kos/probes/brief-charter-as-projection-renderer.md`:
/// - Bedrock: full content per node, sorted by id
/// - Frontier: title only, sorted by id
/// - Graveyard: title only, sorted by id
/// - No backdrop-marker handling yet (phase-1 extraction must complete first)
/// - No "current_state" field projection yet (sub-question A on the brief)
pub fn render(workspace: &Workspace) -> Result<String> {
    // Find the orchestrator graph (the orc's own _kos/, not a subrepo's).
    let orc_graph = workspace
        .graphs
        .iter()
        .find(|g| g.scope == GraphScope::Orchestrator)
        .ok_or_else(|| {
            KosError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no orchestrator-scoped graph found in workspace".to_string(),
            ))
        })?;
    let nodes_dir = orc_graph.path.join("nodes");
    if !nodes_dir.exists() {
        return Err(KosError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("nodes dir not found: {}", nodes_dir.display()),
        )));
    }

    let mut bedrock = load_tier(&nodes_dir, &Confidence::Bedrock)?;
    let mut frontier = load_tier(&nodes_dir, &Confidence::Frontier)?;
    let mut graveyard = load_tier(&nodes_dir, &Confidence::Graveyard)?;

    bedrock.sort_by(|a, b| a.id.cmp(&b.id));
    frontier.sort_by(|a, b| a.id.cmp(&b.id));
    graveyard.sort_by(|a, b| a.id.cmp(&b.id));

    let mut out = String::new();
    out.push_str(HEADER);
    out.push_str("# aae-orc Charter (Rendered)\n\n");
    out.push_str(
        "> Projection of the orc knowledge graph. \
         Source: `_kos/nodes/`. \
         Regenerate with `kos charter render --write`.\n\n",
    );

    render_bedrock_section(&mut out, &bedrock);
    render_frontier_section(&mut out, &frontier);
    render_graveyard_section(&mut out, &graveyard);

    Ok(out)
}

fn render_bedrock_section(out: &mut String, nodes: &[Node]) {
    out.push_str(&format!("## Bedrock ({})\n\n", nodes.len()));
    out.push_str("*Established. Evidence-based or decided with rationale.*\n\n");
    for node in nodes {
        out.push_str(&format!("### {} — {}\n\n", node.id, node.title));
        let body = node.content.trim_end();
        if !body.is_empty() {
            out.push_str(body);
            out.push_str("\n\n");
        }
    }
}

fn render_frontier_section(out: &mut String, nodes: &[Node]) {
    out.push_str(&format!("## Frontier ({})\n\n", nodes.len()));
    out.push_str("*Open. Each links to its full body in `_kos/nodes/frontier/`.*\n\n");
    for node in nodes {
        let rel = relative_node_path(node);
        out.push_str(&format!(
            "- **{}** — {} ([detail]({}))\n",
            node.id, node.title, rel
        ));
    }
    out.push('\n');
}

fn render_graveyard_section(out: &mut String, nodes: &[Node]) {
    out.push_str(&format!("## Graveyard ({})\n\n", nodes.len()));
    out.push_str("*Ruled out. Kept for the reasoning.*\n\n");
    for node in nodes {
        let rel = relative_node_path(node);
        out.push_str(&format!(
            "- **{}** — {} ([detail]({}))\n",
            node.id, node.title, rel
        ));
    }
    out.push('\n');
}

/// Diff the rendered output against the current charter.md.
/// Returns the unified diff as a string, plus a boolean indicating whether they differ.
/// Does not require any external diff binary — uses similar's text diffing.
pub fn diff(workspace: &Workspace) -> Result<(String, bool)> {
    let rendered = render(workspace)?;
    let charter_path = workspace.root.join("charter.md");
    let current = std::fs::read_to_string(&charter_path)
        .unwrap_or_else(|_| String::from("(charter.md does not exist yet)\n"));

    if rendered == current {
        return Ok((String::new(), false));
    }

    // Plain line-by-line diff without an extra dependency.
    // Format: lines unchanged → " ", removed → "-", added → "+".
    let rendered_lines: Vec<&str> = rendered.lines().collect();
    let current_lines: Vec<&str> = current.lines().collect();
    let mut out = String::new();
    out.push_str(&format!(
        "--- charter.md (current, {} lines)\n+++ rendered (from graph, {} lines)\n",
        current_lines.len(),
        rendered_lines.len()
    ));
    // Simple side-by-side counts plus first-N divergent lines.
    let max = rendered_lines.len().max(current_lines.len());
    let mut shown = 0;
    let limit = 200; // cap output so the diff is reviewable
    for i in 0..max {
        let r = rendered_lines.get(i).copied().unwrap_or("");
        let c = current_lines.get(i).copied().unwrap_or("");
        if r != c {
            if shown < limit {
                if !c.is_empty() {
                    out.push_str(&format!("- {c}\n"));
                }
                if !r.is_empty() {
                    out.push_str(&format!("+ {r}\n"));
                }
                shown += 2;
            } else if shown == limit {
                out.push_str(&format!(
                    "... (diff truncated at {limit} lines; use --full to see all)\n"
                ));
                shown += 1;
            }
        }
    }
    Ok((out, true))
}

fn relative_node_path(node: &Node) -> String {
    // Best-effort: trim everything before `_kos/`.
    let s = node.source_path.display().to_string();
    if let Some(idx) = s.find("_kos/") {
        s[idx..].to_string()
    } else {
        s
    }
}

fn load_tier(nodes_dir: &Path, confidence: &Confidence) -> Result<Vec<Node>> {
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
