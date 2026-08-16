//! Ask: ranked, scoped retrieval over the knowledge graph.
//!
//! `grep` hands back a location and cannot order what it finds; an unfiltered
//! `orient` hands back the whole graph and makes the reader triage it. `ask`
//! sits between them: it ranks nodes and findings by lexical match strength,
//! boosts an item that cites or is cited by a strong hit (so a match the plain
//! word search would have missed can still surface), and carries provenance
//! (id, type, confidence tier, date) and freshness (superseded and ruled-out
//! items are shown as such, never silently returned as current) on every row.
//!
//! Phase 0 substrate is lexical matching plus graph proximity, nothing heavier.
//! The verb is the durable contract; the substrate is meant to change under it.
//! See `docs/proposals/read-path-first-steps.md` and the frontier node
//! `question-kos-ask-retrieval-value`.

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::model::{Confidence, EdgeType, Node, NodeType};
use crate::orient;
use crate::telemetry::{self, ReadClass, ReadEvent};
use crate::workspace::{KOS_DIR, Workspace};

// ── Ranking weights ──────────────────────────────────────────
//
// Curated ids and titles are strong signal; body text is weak and capped so a
// long node that happens to repeat a word cannot outweigh a titled match.

/// A query term found in a node id.
const W_ID: f64 = 6.0;
/// A query term found in a node title.
const W_TITLE: f64 = 4.0;
/// Each occurrence of a query term in body content, up to `CONTENT_CAP`.
const W_CONTENT: f64 = 0.6;
/// Content occurrences past this add nothing (a long node is not more relevant
/// for saying the same word ten times).
const CONTENT_CAP: usize = 4;
/// Fraction of a neighbor's lexical score that flows to an adjacent node.
const PROX_FACTOR: f64 = 0.3;
/// Multiplier applied to a node another node supersedes.
const SUPERSEDED_MULT: f64 = 0.75;
/// Scores at or below this are treated as no match.
const EPS: f64 = 1e-9;

// Snippet window sizes, in bytes (clamped to char boundaries before slicing).
const WIN_BEFORE: usize = 48;
const WIN_AFTER: usize = 132;
const WIN_HEAD: usize = 140;

/// Grammatical words that carry no retrieval signal. Dropped before scoring so
/// "what was ruled out about pack management" ranks on "ruled out pack
/// management", not on "what" and "was". Kept deliberately small: only pure
/// function words, never content words like "out" or "not".
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "for", "and", "or", "is", "are", "was", "were", "be",
    "been", "being", "what", "how", "does", "do", "did", "with", "that", "this", "about", "i",
    "we", "it", "as", "at", "by", "from", "can", "will", "would", "should",
];

/// How current a result is, relative to the rest of the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    /// Live knowledge.
    Current,
    /// Another node supersedes this one.
    Superseded,
    /// Ruled out (graveyard tier). Returned, not hidden, so a stale answer is
    /// visibly stale.
    RuledOut,
}

impl Freshness {
    fn as_str(self) -> &'static str {
        match self {
            Freshness::Current => "current",
            Freshness::Superseded => "superseded",
            Freshness::RuledOut => "ruled_out",
        }
    }

    /// Sort key: current before superseded before ruled-out, at equal score.
    fn rank(self) -> u8 {
        match self {
            Freshness::Current => 0,
            Freshness::Superseded => 1,
            Freshness::RuledOut => 2,
        }
    }
}

/// One ranked result, with the provenance and freshness a bare grep line lacks.
#[derive(Debug)]
pub struct AskResult {
    pub id: String,
    pub node_type: NodeType,
    pub confidence: Confidence,
    pub title: String,
    /// The final relevance score (lexical, proximity-boosted, tier-weighted).
    pub score: f64,
    /// A short slice of matching body text, whitespace collapsed.
    pub snippet: String,
    /// Creation date when the node records one (findings almost always do).
    pub date: Option<String>,
    pub freshness: Freshness,
    /// Set only when the item had no lexical hit of its own and surfaced
    /// through graph proximity: the strongest neighbor that pulled it in. This
    /// is the case where proximity did work grep could not have done.
    pub via: Option<String>,
    pub is_finding: bool,
}

/// Run the ask subcommand: resolve the graph, rank the corpus, print, log.
pub fn run(
    workspace: &Workspace,
    cwd: &Path,
    target: Option<&str>,
    query: &str,
    limit: usize,
    json: bool,
) -> Result<()> {
    let (graph_dir, label) = resolve_graph(workspace, cwd, target);
    let corpus = load_corpus(&graph_dir)?;
    let total = corpus.len();
    let results = rank(&corpus, query, limit);

    if json {
        print_json(&results);
    } else {
        print_human(query, &label, total, &results);
    }

    // Consultation-class read: the served set is narrowed by the question, so
    // it can evidence adoption in the read telemetry. Fail-open exactly like
    // orient; a read verb must never fail because its own telemetry failed.
    if telemetry::enabled() {
        let event = ReadEvent {
            verb: "ask",
            target: &label,
            read_class: ReadClass::Consultation,
            json_output: json,
            node_ids: results
                .iter()
                .filter(|r| !r.is_finding)
                .map(|r| r.id.as_str())
                .collect(),
            finding_ids: results
                .iter()
                .filter(|r| r.is_finding)
                .map(|r| r.id.as_str())
                .collect(),
        };
        if let Err(e) = telemetry::record_reads(&graph_dir, &event) {
            eprintln!("warning: could not write read telemetry: {e}");
        }
    }

    Ok(())
}

/// Rank a corpus of nodes against a query. Pure: no IO, deterministic order.
///
/// Three passes: lexical base score per item; a graph-proximity boost that
/// lets an item near a strong hit surface even with no lexical match of its
/// own; a tier and freshness weighting so bedrock outranks a ruled-out node at
/// equal lexical strength and superseded items are penalized but still shown.
pub fn rank(corpus: &[Node], query: &str, limit: usize) -> Vec<AskResult> {
    let terms = tokenize(query);
    if terms.is_empty() || corpus.is_empty() {
        return Vec::new();
    }

    // id → index, for resolving edge targets to corpus positions.
    let mut index: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::with_capacity(corpus.len());
    for (i, node) in corpus.iter().enumerate() {
        index.insert(node.id.as_str(), i);
    }

    // Nodes some other node supersedes. `supersedes` points newer → older, so
    // the edge target is the stale one.
    let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
    for node in corpus {
        for edge in node.all_edges() {
            if matches!(edge.edge_type, EdgeType::Supersedes) {
                superseded.insert(edge.target);
            }
        }
    }

    let base: Vec<f64> = corpus.iter().map(|n| lexical_score(n, &terms)).collect();

    // Undirected adjacency for proximity: a node ranks up whether it cites a
    // strong hit or is cited by one.
    let mut neighbors: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); corpus.len()];
    for (i, node) in corpus.iter().enumerate() {
        for edge in node.all_edges() {
            if let Some(&j) = index.get(edge.target.as_str()) {
                if j != i {
                    neighbors[i].insert(j);
                    neighbors[j].insert(i);
                }
            }
        }
    }

    let mut results: Vec<AskResult> = Vec::new();
    for (i, node) in corpus.iter().enumerate() {
        let mut boost = 0.0;
        let mut best_via: Option<(f64, &str)> = None;
        for &j in &neighbors[i] {
            if base[j] > EPS {
                boost += PROX_FACTOR * base[j];
                if best_via.is_none_or(|(b, _)| base[j] > b) {
                    best_via = Some((base[j], corpus[j].id.as_str()));
                }
            }
        }

        let combined = base[i] + boost;
        if combined <= EPS {
            continue;
        }

        let is_superseded = superseded.contains(&node.id);
        let mut score = combined * tier_mult(&node.confidence);
        if is_superseded {
            score *= SUPERSEDED_MULT;
        }

        let freshness = if node.confidence == Confidence::Graveyard {
            Freshness::RuledOut
        } else if is_superseded {
            Freshness::Superseded
        } else {
            Freshness::Current
        };

        let via = if base[i] <= EPS {
            best_via.map(|(_, id)| id.to_string())
        } else {
            None
        };

        results.push(AskResult {
            id: node.id.clone(),
            node_type: node.node_type.clone(),
            confidence: node.confidence.clone(),
            title: node.title.clone(),
            score,
            snippet: snippet(&node.content, &terms),
            date: node.provenance.as_ref().and_then(|p| p.created_at.clone()),
            freshness,
            via,
            is_finding: node.node_type == NodeType::Finding,
        });
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.freshness.rank().cmp(&b.freshness.rank()))
            .then_with(|| a.id.cmp(&b.id))
    });
    results.truncate(limit);
    results
}

/// Split a query into scored terms: alphanumeric words of two or more chars,
/// lowercased, function words dropped, de-duplicated. If the query is nothing
/// but stopwords, fall back to the raw tokens rather than searching for
/// nothing.
fn tokenize(query: &str) -> Vec<String> {
    let raw: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 2)
        .map(str::to_lowercase)
        .collect();

    let mut kept: Vec<String> = raw
        .iter()
        .filter(|w| !STOPWORDS.contains(&w.as_str()))
        .cloned()
        .collect();

    let source = if kept.is_empty() {
        raw
    } else {
        std::mem::take(&mut kept)
    };

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    source
        .into_iter()
        .filter(|w| seen.insert(w.clone()))
        .collect()
}

/// Lexical relevance of one node to the query terms. Title and id hits weigh
/// heavily; body hits are weak and capped; matching more distinct query terms
/// (coverage) lifts the score so a node hitting every term beats one hitting a
/// single term repeatedly.
fn lexical_score(node: &Node, terms: &[String]) -> f64 {
    let id = node.id.to_lowercase();
    let title = node.title.to_lowercase();
    let content = node.content.to_lowercase();

    let mut score = 0.0;
    let mut hits = 0usize;
    for term in terms {
        let mut hit = false;
        if id.contains(term.as_str()) {
            score += W_ID;
            hit = true;
        }
        if title.contains(term.as_str()) {
            score += W_TITLE;
            hit = true;
        }
        let occ = content.matches(term.as_str()).count();
        if occ > 0 {
            score += W_CONTENT * occ.min(CONTENT_CAP) as f64;
            hit = true;
        }
        if hit {
            hits += 1;
        }
    }

    if hits == 0 {
        return 0.0;
    }
    let coverage = hits as f64 / terms.len() as f64;
    score * (0.5 + 0.5 * coverage)
}

/// Freshness weighting by confidence tier: bedrock outranks a ruled-out node at
/// equal lexical strength, but graveyard nodes still score above zero so they
/// are surfaced (with their status) rather than hidden.
fn tier_mult(confidence: &Confidence) -> f64 {
    match confidence {
        Confidence::Bedrock => 1.2,
        Confidence::Frontier => 1.0,
        Confidence::Placeholder => 0.85,
        Confidence::Graveyard => 0.55,
    }
}

/// A short body slice around the first query-term hit, whitespace collapsed.
/// When nothing matched in the body (a title, id, or proximity-only result),
/// show the head of the content instead.
fn snippet(content: &str, terms: &[String]) -> String {
    if content.trim().is_empty() {
        return String::new();
    }
    let lc = content.to_lowercase();
    let first = terms.iter().filter_map(|t| lc.find(t.as_str())).min();

    let window = match first {
        Some(pos) => {
            let start = floor_boundary(content, pos.saturating_sub(WIN_BEFORE));
            let end = floor_boundary(content, (pos + WIN_AFTER).min(content.len()));
            let mut s = String::new();
            if start > 0 {
                s.push_str("...");
            }
            s.push_str(&content[start..end]);
            if end < content.len() {
                s.push_str("...");
            }
            s
        }
        None => {
            let end = floor_boundary(content, WIN_HEAD.min(content.len()));
            let mut s = content[..end].to_string();
            if end < content.len() {
                s.push_str("...");
            }
            s
        }
    };

    window.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Largest char boundary at or below `i`, so a byte-index window never slices
/// through a multi-byte character.
fn floor_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

// ── Corpus loading ───────────────────────────────────────────

/// Load the search corpus for a graph: every node (all tiers) plus every
/// finding. Findings are read as nodes so their citation edges and dates join
/// the graph the ranking walks. Node loading reuses `orient`'s loader; finding
/// loading goes through the shared `findings` module, which accepts all three
/// on-disk finding shapes (pure yaml, md+frontmatter, legacy bare md).
pub fn load_corpus(graph_dir: &Path) -> Result<Vec<Node>> {
    let mut corpus = orient::load_all_nodes(&graph_dir.join("nodes"))?;
    corpus.extend(crate::findings::load_finding_nodes(
        &graph_dir.join("findings"),
    )?);
    Ok(corpus)
}

/// Resolve which graph to search and the label to log it under. An explicit
/// `--target` naming a discovered graph wins; otherwise mirror the resolution
/// `orient` uses for where it logs a read.
fn resolve_graph(workspace: &Workspace, cwd: &Path, target: Option<&str>) -> (PathBuf, String) {
    if let Some(t) = target {
        if let Some(graph) = workspace.graphs.iter().find(|g| g.graph_id == t) {
            return (graph.path.clone(), graph.graph_id.clone());
        }
    }

    let (dir, default_label) = if workspace.is_standalone() {
        let dir = workspace.root.join(KOS_DIR);
        let label = workspace
            .graphs
            .iter()
            .find(|g| g.path == dir)
            .map(|g| g.graph_id.clone())
            .unwrap_or_else(|| "kos".to_string());
        (dir, label)
    } else if let Some(graph) = workspace.nearest_graph(cwd) {
        (graph.path.clone(), graph.graph_id.clone())
    } else {
        (workspace.node_root(), "kos".to_string())
    };

    let label = target.map(str::to_string).unwrap_or(default_label);
    (dir, label)
}

// ── Output ───────────────────────────────────────────────────

fn print_human(query: &str, label: &str, total: usize, results: &[AskResult]) {
    println!("=== kos ask: \"{query}\" ({label}) ===\n");

    if results.is_empty() {
        println!("  no matches among {total} nodes and findings searched");
        println!("  try broader terms, or `kos orient` for the full graph");
        return;
    }

    println!(
        "{} result(s), ranked (of {total} searched)\n",
        results.len()
    );
    for (rank, r) in results.iter().enumerate() {
        println!("  {}. {}", rank + 1, r.id);

        let mut meta = format!("score {:.1} | {} | {}", r.score, r.node_type, r.confidence);
        if let Some(ref date) = r.date {
            meta.push_str(" | ");
            meta.push_str(date);
        }
        match r.freshness {
            Freshness::RuledOut => meta.push_str(" | RULED OUT"),
            Freshness::Superseded => meta.push_str(" | SUPERSEDED"),
            Freshness::Current => {}
        }
        println!("     {meta}");
        println!("     {}", r.title);
        if let Some(ref via) = r.via {
            println!("     via graph proximity from {via}");
        }
        if !r.snippet.is_empty() {
            println!("     > {}", r.snippet);
        }
        println!();
    }
}

fn print_json(results: &[AskResult]) {
    let arr: Vec<serde_json::Value> = results
        .iter()
        .enumerate()
        .map(|(rank, r)| {
            serde_json::json!({
                "rank": rank + 1,
                "id": r.id,
                "type": r.node_type.to_string(),
                "confidence": r.confidence.to_string(),
                "score": (r.score * 1000.0).round() / 1000.0,
                "title": r.title,
                "snippet": r.snippet,
                "date": r.date,
                "freshness": r.freshness.as_str(),
                "via": r.via,
                "is_finding": r.is_finding,
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".to_string());
    println!("{json}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Edge;

    fn node(
        id: &str,
        tier: Confidence,
        node_type: NodeType,
        title: &str,
        content: &str,
        edges: &[(&str, EdgeType)],
    ) -> Node {
        Node {
            id: id.to_string(),
            node_type,
            confidence: tier,
            title: title.to_string(),
            content: content.to_string(),
            edges: edges
                .iter()
                .map(|(target, edge_type)| Edge {
                    target: (*target).to_string(),
                    edge_type: edge_type.clone(),
                    signal: None,
                    note: None,
                })
                .collect(),
            depends_on: Vec::new(),
            graveyard: None,
            brief: None,
            finding: None,
            compaction: None,
            provenance: None,
            tags: Vec::new(),
            notes: None,
            source_path: PathBuf::new(),
        }
    }

    #[test]
    fn tokenize_drops_stopwords_keeps_content_words() {
        assert_eq!(
            tokenize("what was ruled out about pack management"),
            vec!["ruled", "out", "pack", "management"]
        );
    }

    #[test]
    fn tokenize_dedupes_and_falls_back_when_all_stopwords() {
        assert_eq!(tokenize("the the of to"), vec!["the", "of", "to"]);
        assert_eq!(tokenize("pack pack management"), vec!["pack", "management"]);
    }

    #[test]
    fn title_hit_outranks_content_only_hit() {
        let corpus = vec![
            node(
                "elem-a",
                Confidence::Frontier,
                NodeType::Element,
                "telemetry decision",
                "unrelated body",
                &[],
            ),
            node(
                "elem-b",
                Confidence::Frontier,
                NodeType::Element,
                "unrelated title",
                "a passing mention of telemetry in the body",
                &[],
            ),
        ];
        let results = rank(&corpus, "telemetry", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "elem-a");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn bedrock_outranks_graveyard_at_equal_lexical() {
        let corpus = vec![
            node(
                "elem-grave",
                Confidence::Graveyard,
                NodeType::Element,
                "pack management approach",
                "",
                &[],
            ),
            node(
                "elem-rock",
                Confidence::Bedrock,
                NodeType::Element,
                "pack management approach",
                "",
                &[],
            ),
        ];
        let results = rank(&corpus, "pack management", 10);
        assert_eq!(results[0].id, "elem-rock");
        assert_eq!(results[0].freshness, Freshness::Current);
    }

    #[test]
    fn graveyard_is_surfaced_not_hidden() {
        let corpus = vec![node(
            "grv-monolith",
            Confidence::Graveyard,
            NodeType::Graveyard,
            "monolith approach for pack management",
            "ruled out",
            &[],
        )];
        let results = rank(&corpus, "pack management", 10);
        assert_eq!(results.len(), 1, "a ruled-out match must still be returned");
        assert_eq!(results[0].freshness, Freshness::RuledOut);
    }

    #[test]
    fn proximity_surfaces_a_node_with_no_lexical_hit() {
        // elem-b matches nothing lexically, but cites the strong hit elem-a.
        let with_edge = vec![
            node(
                "elem-a",
                Confidence::Frontier,
                NodeType::Element,
                "telemetry read path",
                "",
                &[],
            ),
            node(
                "elem-b",
                Confidence::Frontier,
                NodeType::Element,
                "unrelated",
                "nothing here",
                &[("elem-a", EdgeType::Derives)],
            ),
        ];
        let results = rank(&with_edge, "telemetry", 10);
        let b = results.iter().find(|r| r.id == "elem-b");
        assert!(b.is_some(), "proximity should surface the citing node");
        assert_eq!(b.unwrap().via.as_deref(), Some("elem-a"));

        // Remove the edge and elem-b must disappear: the edge, not the text,
        // put it in the result set.
        let without_edge = vec![
            node(
                "elem-a",
                Confidence::Frontier,
                NodeType::Element,
                "telemetry read path",
                "",
                &[],
            ),
            node(
                "elem-b",
                Confidence::Frontier,
                NodeType::Element,
                "unrelated",
                "nothing here",
                &[],
            ),
        ];
        let results = rank(&without_edge, "telemetry", 10);
        assert!(results.iter().all(|r| r.id != "elem-b"));
    }

    #[test]
    fn superseded_node_is_flagged() {
        let corpus = vec![
            node(
                "elem-old",
                Confidence::Frontier,
                NodeType::Element,
                "pack management v1",
                "",
                &[],
            ),
            node(
                "elem-new",
                Confidence::Frontier,
                NodeType::Element,
                "pack management v2",
                "",
                &[("elem-old", EdgeType::Supersedes)],
            ),
        ];
        let results = rank(&corpus, "pack management", 10);
        let old = results.iter().find(|r| r.id == "elem-old").unwrap();
        assert_eq!(old.freshness, Freshness::Superseded);
        let new = results.iter().find(|r| r.id == "elem-new").unwrap();
        assert_eq!(new.freshness, Freshness::Current);
    }

    #[test]
    fn empty_query_returns_nothing() {
        let corpus = vec![node(
            "elem-a",
            Confidence::Frontier,
            NodeType::Element,
            "title",
            "body",
            &[],
        )];
        assert!(rank(&corpus, "   ", 10).is_empty());
    }

    #[test]
    fn limit_truncates_results() {
        let corpus: Vec<Node> = (0..5)
            .map(|i| {
                node(
                    &format!("elem-{i}"),
                    Confidence::Frontier,
                    NodeType::Element,
                    "pack management",
                    "",
                    &[],
                )
            })
            .collect();
        assert_eq!(rank(&corpus, "pack management", 3).len(), 3);
    }
}
