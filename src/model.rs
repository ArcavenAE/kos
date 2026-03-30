use std::path::PathBuf;

use serde::Deserialize;

/// A kos graph node, deserialized from nodes/**/*.yaml.
/// Fields are optional where the schema allows absence.
#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub confidence: Confidence,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Source file path (not in YAML — populated after loading)
    #[serde(skip)]
    pub source_path: PathBuf,
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

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::Bedrock => write!(f, "bedrock"),
            Confidence::Frontier => write!(f, "frontier"),
            Confidence::Placeholder => write!(f, "placeholder"),
            Confidence::Graveyard => write!(f, "graveyard"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    #[serde(default)]
    pub signal: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// A kos finding, deserialized from findings/finding-NNN-*.yaml.
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
/// Not deserialized from YAML — parsed from charter.md.
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

/// An RD brief header, extracted from sprint/rd/*.md frontmatter.
#[derive(Debug)]
pub struct RdBrief {
    pub slug: String,
    pub question: String,
    pub frontier: Option<String>,
    pub status: Option<String>,
    pub path: PathBuf,
}
