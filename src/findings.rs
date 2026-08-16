//! Shared finding loader.
//!
//! A kos "finding" is a node type (schema/node.schema.yaml). On disk, findings
//! in `_kos/findings/` take one of three shapes, and every reader (ask, orient,
//! validate) must accept all three so no finding is invisible:
//!
//! 1. Pure YAML: a `.yaml` node file with id/type/confidence/title/content/
//!    edges/provenance. kos's own findings are this shape.
//! 2. Markdown with YAML frontmatter: a `.md` file opening with a `---` fenced
//!    yaml block (id, type, confidence, date/created_at, edges), then markdown
//!    prose. The first-class prose-finding shape.
//! 3. Legacy bare markdown: a `.md` file with no frontmatter, just an H1 title,
//!    a `**Date:** YYYY-MM-DD` line, and body. Hundreds of these exist across
//!    the fleet graphs; they degrade gracefully rather than being dropped.
//!
//! No file is renamed; `.md` findings keep their `.md` names. This module is
//! the one place the three shapes are decoded. Read paths call
//! [`load_finding_nodes`]; validate calls [`load_findings`] for the per-file
//! bookkeeping it needs to flag duplicate ids and unloadable files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{KosError, Result};
use crate::model::{Confidence, Edge, EdgeType, Node, NodeType, Provenance};

/// Best-effort edge extraction from bare markdown stops after this many
/// distinct targets. A finding that names hundreds of references is not more
/// connected for the purpose of proximity ranking.
const EDGE_CAP: usize = 64;

/// A finding file that loaded into a searchable node, plus the filename stem so
/// validate can flag id/stem drift.
pub struct LoadedFinding {
    pub node: Node,
    /// The filename without its extension (e.g. `finding-136-chat-as-probe`).
    pub stem: String,
}

/// The outcome of reading one file in a findings/ directory.
pub enum FindingLoad {
    Loaded(Box<LoadedFinding>),
    /// A `.yaml`/`.yml` file that parsed as neither a Node nor the legacy
    /// finding-block shape. Kept, not discarded, so validate can warn instead
    /// of the file vanishing silently.
    Unloadable {
        path: PathBuf,
        error: String,
    },
}

/// Load every finding file in `dir`, across all three on-disk shapes. Returns
/// one entry per file (loaded or unloadable). Files without a `.yaml`/`.yml`/
/// `.md` extension are skipped. Directory absence is not an error.
pub fn load_findings(dir: &Path) -> Result<Vec<FindingLoad>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        let Some(ext) = ext else { continue };
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let text = std::fs::read_to_string(path).map_err(KosError::Io)?;
        let load = match ext {
            "yaml" | "yml" => yaml_finding(path, &stem, &text),
            "md" => FindingLoad::Loaded(Box::new(md_finding(path, &stem, &text))),
            _ => continue,
        };
        out.push(load);
    }
    Ok(out)
}

/// Convenience for the read paths (ask, orient): the finding nodes only,
/// unloadable files dropped. A malformed finding is not the read path's error
/// to report; validate is where that surfaces.
pub fn load_finding_nodes(dir: &Path) -> Result<Vec<Node>> {
    Ok(load_findings(dir)?
        .into_iter()
        .filter_map(|f| match f {
            FindingLoad::Loaded(l) => Some(l.node),
            FindingLoad::Unloadable { .. } => None,
        })
        .collect())
}

/// The canonical finding number (`finding-123`) used to detect two files that
/// claim the same finding, even when their slugs differ. Two distinct findings
/// sharing a number is the collision validate must catch. Falls back to the
/// full id, lowercased, when the id is not a numbered finding.
pub fn finding_key(id: &str) -> String {
    let lower = id.to_lowercase();
    if let Some(rest) = lower.strip_prefix("finding-") {
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            return format!("finding-{digits}");
        }
    }
    lower
}

// ── YAML shape ───────────────────────────────────────────────

fn yaml_finding(path: &Path, _stem: &str, text: &str) -> FindingLoad {
    match serde_yaml::from_str::<Node>(text) {
        Ok(mut node) => {
            node.source_path = path.to_path_buf();
            loaded(node, path)
        }
        Err(e) => {
            if let Some(mut node) = finding_node_fallback(text) {
                node.source_path = path.to_path_buf();
                loaded(node, path)
            } else {
                FindingLoad::Unloadable {
                    path: path.to_path_buf(),
                    error: e.to_string(),
                }
            }
        }
    }
}

/// Build a searchable node from a finding whose prose lives under a `finding:`
/// block instead of a top-level `content` field. Pulls id, title, and
/// confidence from the top level and flattens the `finding` block into content
/// so the text is still matchable. Preserved from ask's original loader.
fn finding_node_fallback(text: &str) -> Option<Node> {
    let value: serde_yaml::Value = serde_yaml::from_str(text).ok()?;
    let id = value.get("id")?.as_str()?.to_string();
    let title = value
        .get("title")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("")
        .to_string();
    let content = value
        .get("finding")
        .and_then(|f| serde_yaml::to_string(f).ok())
        .unwrap_or_default();

    Some(finding_node(
        id,
        NodeType::Finding,
        Confidence::Frontier,
        title,
        content,
        Vec::new(),
        None,
    ))
}

// ── Markdown shapes ──────────────────────────────────────────

/// Frontmatter fields we read. Everything is optional; a `.md` finding may
/// carry all, some, or none of these.
#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    id: Option<String>,
    #[serde(rename = "type")]
    node_type: Option<NodeType>,
    confidence: Option<Confidence>,
    title: Option<String>,
    created_at: Option<String>,
    date: Option<String>,
    #[serde(default)]
    edges: Vec<Edge>,
}

/// Decode a `.md` finding. If it opens with a parseable `---` frontmatter
/// block, use that; otherwise (no fence, or a fence whose yaml will not parse)
/// degrade to the bare-markdown derivation so the finding stays visible.
fn md_finding(path: &Path, stem: &str, text: &str) -> LoadedFinding {
    if let Some((fm_text, body)) = split_frontmatter(text) {
        match serde_yaml::from_str::<Frontmatter>(fm_text) {
            Ok(fm) => return frontmatter_node(path, stem, fm, body),
            // Fence present but malformed: fall through to bare-md over the body
            // so the prose is still searchable and the file is not dropped.
            Err(_) => return bare_md_node(path, stem, body),
        }
    }
    bare_md_node(path, stem, text)
}

fn frontmatter_node(path: &Path, stem: &str, fm: Frontmatter, body: &str) -> LoadedFinding {
    let id = fm.id.unwrap_or_else(|| stem.to_string());
    let node_type = fm.node_type.unwrap_or(NodeType::Finding);
    let confidence = fm.confidence.unwrap_or(Confidence::Frontier);
    let title = fm
        .title
        .or_else(|| first_h1(body))
        .unwrap_or_else(|| stem.to_string());
    let created_at = fm.created_at.or(fm.date);

    let mut node = finding_node(
        id,
        node_type,
        confidence,
        title,
        body.to_string(),
        fm.edges,
        created_at,
    );
    node.source_path = path.to_path_buf();
    LoadedFinding {
        node,
        stem: stem.to_string(),
    }
}

fn bare_md_node(path: &Path, stem: &str, content: &str) -> LoadedFinding {
    let title = first_h1(content).unwrap_or_else(|| stem.to_string());
    let created_at = date_line(content);
    let edges = parse_md_edges(content, stem);

    let mut node = finding_node(
        stem.to_string(),
        NodeType::Finding,
        Confidence::Frontier,
        title,
        content.to_string(),
        edges,
        created_at,
    );
    node.source_path = path.to_path_buf();
    LoadedFinding {
        node,
        stem: stem.to_string(),
    }
}

/// Split a leading `---` fenced yaml block from the markdown body. Returns
/// `(frontmatter_text, body)` when the file opens with a `---` line and a later
/// `---` or `...` closing line is found; otherwise `None`.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let mut lines = text.lines();
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }
    // Byte offset where the frontmatter content begins (after the first line).
    let fm_start = first.len() + 1;
    let mut offset = fm_start;
    for line in lines {
        let line_start = offset;
        // +1 for the '\n' that `lines()` stripped. The final line of a file
        // without a trailing newline over-counts by one, but it can never be a
        // closing fence followed by a body, so the body slice is unaffected.
        offset += line.len() + 1;
        let trimmed = line.trim_end();
        if trimmed == "---" || trimmed == "..." {
            let fm_text = &text[fm_start..line_start];
            let body = if offset <= text.len() {
                &text[offset..]
            } else {
                ""
            };
            return Some((fm_text, body));
        }
    }
    None
}

/// The first `# ` H1 heading text, trimmed. `## ` and deeper are not H1s.
fn first_h1(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

/// The date from the first `**Date:** YYYY-MM-DD` line, if present. Lenient:
/// takes the first whitespace-delimited token after the marker.
fn date_line(content: &str) -> Option<String> {
    let idx = content.find("**Date:**")?;
    let rest = &content[idx + "**Date:**".len()..];
    let line = rest.lines().next().unwrap_or("");
    line.split_whitespace().next().map(str::to_string)
}

/// Best-effort citation edges from bare markdown: `[[wikilinks]]` and bare
/// `finding-…` / `aae-orc-…` slugs. All typed `derives`; the point is to keep
/// a bare-md finding's references in the graph the ranker walks, not to
/// reconstruct authored edge semantics. Self-references and duplicates dropped.
fn parse_md_edges(content: &str, self_id: &str) -> Vec<Edge> {
    let mut targets: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let self_lower = self_id.to_lowercase();

    // Wikilinks: [[target]] or [[target|alias]].
    let mut rest = content;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let inner = &after[..end];
        let target = inner.split('|').next().unwrap_or(inner).trim();
        push_target(target, &self_lower, &mut seen, &mut targets);
        rest = &after[end + 2..];
    }

    // Bare slug references.
    for tok in content.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let t = tok.trim_matches('-');
        let tl = t.to_lowercase();
        if tl.starts_with("finding-") || tl.starts_with("aae-orc-") {
            push_target(t, &self_lower, &mut seen, &mut targets);
        }
    }

    targets
        .into_iter()
        .take(EDGE_CAP)
        .map(|target| Edge {
            target,
            edge_type: EdgeType::Derives,
            signal: None,
            note: None,
        })
        .collect()
}

fn push_target(target: &str, self_lower: &str, seen: &mut HashSet<String>, out: &mut Vec<String>) {
    let target = target.trim();
    if target.len() < 3 || target.contains(char::is_whitespace) {
        return;
    }
    let lower = target.to_lowercase();
    if lower == self_lower {
        return;
    }
    if seen.insert(lower) {
        out.push(target.to_string());
    }
}

// ── Node construction ────────────────────────────────────────

/// Assemble a finding `Node` with the fields a finding carries and defaults for
/// the rest. `created_at`, when present, rides in `provenance` so the read
/// paths can display a finding's date.
#[allow(clippy::too_many_arguments)]
fn finding_node(
    id: String,
    node_type: NodeType,
    confidence: Confidence,
    title: String,
    content: String,
    edges: Vec<Edge>,
    created_at: Option<String>,
) -> Node {
    let provenance = created_at.map(|c| Provenance {
        created_by: None,
        session: None,
        created_at: Some(c),
        derived_from: Vec::new(),
        reviewed_by: None,
    });
    Node {
        id,
        node_type,
        confidence,
        title,
        content,
        edges,
        depends_on: Vec::new(),
        graveyard: None,
        brief: None,
        finding: None,
        compaction: None,
        provenance,
        tags: Vec::new(),
        notes: None,
        source_path: PathBuf::new(),
    }
}

fn loaded(node: Node, path: &Path) -> FindingLoad {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    FindingLoad::Loaded(Box::new(LoadedFinding { node, stem }))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write(dir: &Path, name: &str, text: &str) {
        fs::write(dir.join(name), text).unwrap();
    }

    fn only_node(dir: &Path, name: &str) -> Node {
        let path = dir.join(name);
        let text = fs::read_to_string(&path).unwrap();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap();
        match ext {
            "yaml" | "yml" => match yaml_finding(&path, stem, &text) {
                FindingLoad::Loaded(l) => l.node,
                FindingLoad::Unloadable { error, .. } => panic!("unloadable: {error}"),
            },
            _ => md_finding(&path, stem, &text).node,
        }
    }

    #[test]
    fn finding_key_extracts_number_and_ignores_slug() {
        assert_eq!(finding_key("finding-123-harness-invocation"), "finding-123");
        assert_eq!(finding_key("finding-123-org-owned-fork"), "finding-123");
        assert_eq!(finding_key("finding-009-terminal"), "finding-009");
        // Non-numbered ids fall back to the whole id, lowercased.
        assert_eq!(finding_key("Elem-Foo"), "elem-foo");
    }

    #[test]
    fn pure_yaml_finding_loads_as_node() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-001-tmux.yaml",
            "id: finding-001-tmux\ntype: finding\nconfidence: bedrock\ntitle: tmux works\ncontent: the body\n",
        );
        let node = only_node(dir.path(), "finding-001-tmux.yaml");
        assert_eq!(node.id, "finding-001-tmux");
        assert_eq!(node.node_type, NodeType::Finding);
        assert_eq!(node.confidence, Confidence::Bedrock);
        assert_eq!(node.title, "tmux works");
        assert_eq!(node.content, "the body");
    }

    #[test]
    fn md_frontmatter_splits_meta_from_content() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-140-prose.md",
            "---\nid: finding-140-prose\ntype: finding\nconfidence: bedrock\ncreated_at: 2026-08-16\n---\n# The prose title\n\nBody paragraph here.\n",
        );
        let node = only_node(dir.path(), "finding-140-prose.md");
        assert_eq!(node.id, "finding-140-prose");
        assert_eq!(node.confidence, Confidence::Bedrock);
        assert_eq!(node.node_type, NodeType::Finding);
        assert_eq!(
            node.provenance.and_then(|p| p.created_at),
            Some("2026-08-16".to_string())
        );
        assert!(node.content.starts_with("# The prose title"));
        assert!(!node.content.contains("confidence: bedrock"));
    }

    #[test]
    fn md_frontmatter_accepts_date_alias_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        // No type, no confidence, `date:` rather than `created_at:`, no title.
        write(
            dir.path(),
            "finding-141-defaults.md",
            "---\nid: finding-141-defaults\ndate: 2026-01-02\n---\n# H1 becomes the title\n\nbody\n",
        );
        let node = only_node(dir.path(), "finding-141-defaults.md");
        assert_eq!(node.node_type, NodeType::Finding);
        assert_eq!(node.confidence, Confidence::Frontier);
        assert_eq!(node.title, "H1 becomes the title");
        assert_eq!(
            node.provenance.and_then(|p| p.created_at),
            Some("2026-01-02".to_string())
        );
    }

    #[test]
    fn bare_md_derives_id_title_and_date() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-136-chat-as-probe-surface.md",
            "# finding-136: operator chat is a probe surface\n\n**Date:** 2026-08-16\n**Status:** OBSERVED\n\nThe finding body.\n",
        );
        let node = only_node(dir.path(), "finding-136-chat-as-probe-surface.md");
        // The stem is the id, even though the H1 carries a different phrasing.
        assert_eq!(node.id, "finding-136-chat-as-probe-surface");
        assert_eq!(node.title, "finding-136: operator chat is a probe surface");
        assert_eq!(node.confidence, Confidence::Frontier);
        assert_eq!(node.node_type, NodeType::Finding);
        assert_eq!(
            node.provenance.and_then(|p| p.created_at),
            Some("2026-08-16".to_string())
        );
        assert!(node.content.contains("The finding body."));
    }

    #[test]
    fn bare_md_without_h1_or_date_falls_back_to_stem() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-200-no-header.md",
            "just some prose with no heading and no date line at all\n",
        );
        let node = only_node(dir.path(), "finding-200-no-header.md");
        assert_eq!(node.id, "finding-200-no-header");
        assert_eq!(node.title, "finding-200-no-header");
        assert!(node.provenance.is_none());
    }

    #[test]
    fn bare_md_extracts_wikilink_and_slug_edges() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-201-refs.md",
            "# refs\n\nSee [[question-charter-management]] and finding-018-charter-inflation, tracked as aae-orc-3bxt.\n",
        );
        let node = only_node(dir.path(), "finding-201-refs.md");
        let targets: Vec<&str> = node.edges.iter().map(|e| e.target.as_str()).collect();
        assert!(targets.contains(&"question-charter-management"));
        assert!(targets.contains(&"finding-018-charter-inflation"));
        assert!(targets.contains(&"aae-orc-3bxt"));
        assert!(node.edges.iter().all(|e| e.edge_type == EdgeType::Derives));
    }

    #[test]
    fn malformed_frontmatter_degrades_to_bare_md() {
        let dir = tempfile::tempdir().unwrap();
        // The frontmatter block is not valid yaml (unclosed bracket), but the
        // file must not disappear.
        write(
            dir.path(),
            "finding-202-broken-fm.md",
            "---\nid: [unterminated\n---\n# Still visible\n\nbody text\n",
        );
        let node = only_node(dir.path(), "finding-202-broken-fm.md");
        assert_eq!(node.id, "finding-202-broken-fm");
        assert_eq!(node.title, "Still visible");
    }

    #[test]
    fn unparseable_yaml_finding_is_reported_unloadable() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "finding-203-bad.yaml", "id: [unterminated\n");
        let loads = load_findings(dir.path()).unwrap();
        assert_eq!(loads.len(), 1);
        assert!(matches!(loads[0], FindingLoad::Unloadable { .. }));
    }

    #[test]
    fn legacy_finding_block_yaml_loads_via_fallback() {
        let dir = tempfile::tempdir().unwrap();
        // No top-level `content`; prose lives under a `finding:` block. The Node
        // parse fails on the missing content field; the fallback rescues it.
        write(
            dir.path(),
            "finding-204-block.yaml",
            "id: finding-204-block\ntitle: block finding\nfinding:\n  summary: a summary\n  detail: the detail\n",
        );
        let node = only_node(dir.path(), "finding-204-block.yaml");
        assert_eq!(node.id, "finding-204-block");
        assert!(node.content.contains("summary"));
    }

    #[test]
    fn load_finding_nodes_reads_all_three_shapes() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "finding-001-yaml.yaml",
            "id: finding-001-yaml\ntype: finding\nconfidence: frontier\ntitle: y\ncontent: c\n",
        );
        write(
            dir.path(),
            "finding-002-fm.md",
            "---\nid: finding-002-fm\n---\n# fm title\nbody\n",
        );
        write(
            dir.path(),
            "finding-003-bare.md",
            "# bare title\n\n**Date:** 2026-08-16\n\nbody\n",
        );
        let mut ids: Vec<String> = load_finding_nodes(dir.path())
            .unwrap()
            .into_iter()
            .map(|n| n.id)
            .collect();
        ids.sort();
        assert_eq!(
            ids,
            vec!["finding-001-yaml", "finding-002-fm", "finding-003-bare"]
        );
    }

    #[test]
    fn missing_directory_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        assert!(load_findings(&missing).unwrap().is_empty());
        assert!(load_finding_nodes(&missing).unwrap().is_empty());
    }
}
