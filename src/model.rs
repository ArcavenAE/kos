use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A kos graph node, deserialized from nodes/**/*.yaml.
/// Handles both v0.1 (depends_on) and v0.2+ (edges) formats.
#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub confidence: Confidence,
    pub title: String,
    pub content: String,
    /// v0.2+ typed edges
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// v0.1 compatibility — flat list of node IDs (treated as derives edges)
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Type-specific graveyard section
    #[serde(default)]
    pub graveyard: Option<GraveyardSection>,
    #[serde(default)]
    pub provenance: Option<Provenance>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
    /// Source file path (populated after loading, not in YAML)
    #[serde(skip)]
    pub source_path: PathBuf,
}

impl Node {
    /// Return all edges, unifying depends_on (v0.1) and edges (v0.2+).
    pub fn all_edges(&self) -> Vec<Edge> {
        let mut result = self.edges.clone();
        for dep in &self.depends_on {
            result.push(Edge {
                target: dep.clone(),
                edge_type: EdgeType::Derives,
                signal: None,
                note: None,
            });
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NodeType {
    Value,
    NonGoal,
    Question,
    Brief,
    Finding,
    Element,
    Graveyard,
    Correspondence,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Value => write!(f, "value"),
            NodeType::NonGoal => write!(f, "non-goal"),
            NodeType::Question => write!(f, "question"),
            NodeType::Brief => write!(f, "brief"),
            NodeType::Finding => write!(f, "finding"),
            NodeType::Element => write!(f, "element"),
            NodeType::Graveyard => write!(f, "graveyard"),
            NodeType::Correspondence => write!(f, "correspondence"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Confidence {
    Bedrock,
    Frontier,
    Placeholder,
    Graveyard,
}

impl Confidence {
    /// The directory name this confidence maps to.
    pub fn directory(&self) -> &str {
        match self {
            Confidence::Bedrock => "bedrock",
            Confidence::Frontier => "frontier",
            Confidence::Placeholder => "placeholder",
            Confidence::Graveyard => "graveyard",
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.directory())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Edge {
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    #[serde(default)]
    pub signal: Option<SignalType>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EdgeType {
    Derives,
    Implements,
    Contradicts,
    Supersedes,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Derives => write!(f, "derives"),
            EdgeType::Implements => write!(f, "implements"),
            EdgeType::Contradicts => write!(f, "contradicts"),
            EdgeType::Supersedes => write!(f, "supersedes"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SignalType {
    Error,
    Evolution,
    Drift,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Error => write!(f, "error"),
            SignalType::Evolution => write!(f, "evolution"),
            SignalType::Drift => write!(f, "drift"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraveyardSection {
    #[serde(default)]
    pub approach: Option<String>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub finding: Option<String>,
    #[serde(default)]
    pub ruling: Option<String>,
    #[serde(default)]
    pub reopener: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Provenance {
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub derived_from: Vec<String>,
    #[serde(default)]
    pub reviewed_by: Option<String>,
}

// ── Types used by orient (lighter, for findings/charter) ─────

/// A kos finding, deserialized from findings/finding-NNN-*.yaml.
/// Uses String types for fields that orient only displays, not validates.
#[derive(Debug, Deserialize)]
pub struct Finding {
    pub id: String,
    pub confidence: Confidence,
    pub title: String,
    pub content: String,
    #[serde(skip)]
    pub source_path: PathBuf,
}

/// A charter section extracted from markdown.
#[derive(Debug)]
pub struct CharterItem {
    pub id: String,
    pub section: CharterSection,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharterSection {
    Bedrock,
    Frontier,
    Graveyard,
}

impl std::fmt::Display for CharterSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharterSection::Bedrock => write!(f, "bedrock"),
            CharterSection::Frontier => write!(f, "frontier"),
            CharterSection::Graveyard => write!(f, "graveyard"),
        }
    }
}

/// An RD brief header, extracted from sprint/rd/*.md.
#[derive(Debug)]
pub struct RdBrief {
    pub slug: String,
    pub question: String,
    pub frontier: Option<String>,
    pub status: Option<String>,
    pub path: PathBuf,
}

// ── Graph manifest and discovery types ──────────────────────

/// The kos.yaml manifest that identifies a knowledge graph.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphManifest {
    pub graph_id: String,
    pub scope: GraphScope,
    #[serde(default)]
    pub description: Option<String>,
    pub schema_version: String,
    #[serde(default)]
    pub includes: Vec<GraphInclude>,
}

/// A graph scope — orchestrator (composes subrepo graphs) or repo (standalone).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum GraphScope {
    Orchestrator,
    Repo,
}

impl std::fmt::Display for GraphScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphScope::Orchestrator => write!(f, "orchestrator"),
            GraphScope::Repo => write!(f, "repo"),
        }
    }
}

/// A reference to an included subrepo graph in an orchestrator manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphInclude {
    pub path: String,
}

/// A discovered graph source on the filesystem.
#[derive(Debug, Clone)]
pub struct GraphSource {
    /// The graph_id from kos.yaml.
    pub graph_id: String,
    /// Absolute path to the _kos/ directory.
    pub path: PathBuf,
    /// The graph scope.
    pub scope: GraphScope,
    /// The full parsed manifest.
    pub manifest: GraphManifest,
}
