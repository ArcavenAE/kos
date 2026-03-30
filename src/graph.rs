use std::collections::HashMap;
use std::path::Path;

use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;

use crate::error::{KosError, Result};
use crate::model::{Confidence, Node};

/// Output format for graph rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFormat {
    Mermaid,
    Dot,
}

/// Run the graph subcommand — render the node graph.
pub fn run(kos_root: &Path, format: GraphFormat) -> Result<()> {
    let nodes = load_nodes(&kos_root.join("nodes"))?;
    let graph = build_graph(&nodes);

    match format {
        GraphFormat::Mermaid => print_mermaid(&graph, &nodes),
        GraphFormat::Dot => print_dot(&graph, &nodes),
    }

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

/// Build a petgraph DiGraph from the nodes.
/// Returns the graph with node indices mapped to node IDs.
fn build_graph(nodes: &[Node]) -> DiGraph<String, String> {
    let mut graph = DiGraph::new();
    let mut id_to_idx: HashMap<&str, petgraph::graph::NodeIndex> = HashMap::new();

    // Add all nodes
    for node in nodes {
        let idx = graph.add_node(node.id.clone());
        id_to_idx.insert(&node.id, idx);
    }

    // Add edges (both depends_on and edges)
    for node in nodes {
        if let Some(&source_idx) = id_to_idx.get(node.id.as_str()) {
            for edge in node.all_edges() {
                if let Some(&target_idx) = id_to_idx.get(edge.target.as_str()) {
                    let label = match edge.signal {
                        Some(ref s) => format!("{} ({})", edge.edge_type, s),
                        None => edge.edge_type.to_string(),
                    };
                    graph.add_edge(source_idx, target_idx, label);
                }
                // Edges pointing outside nodes/ are silently skipped in the graph
            }
        }
    }

    graph
}

fn confidence_style_mermaid(conf: &Confidence) -> &'static str {
    match conf {
        Confidence::Bedrock => ":::bedrock",
        Confidence::Frontier => ":::frontier",
        Confidence::Placeholder => ":::placeholder",
        Confidence::Graveyard => ":::graveyard",
    }
}

fn print_mermaid(graph: &DiGraph<String, String>, nodes: &[Node]) {
    let id_to_node: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    println!("graph TD");

    // Style definitions
    println!("    classDef bedrock fill:#4a9,stroke:#333,color:#fff");
    println!("    classDef frontier fill:#da5,stroke:#333,color:#fff");
    println!("    classDef placeholder fill:#aaa,stroke:#333,color:#fff,stroke-dasharray: 5 5");
    println!("    classDef graveyard fill:#a55,stroke:#333,color:#fff");
    println!();

    // Nodes
    for idx in graph.node_indices() {
        let id = &graph[idx];
        let style = id_to_node
            .get(id.as_str())
            .map(|n| confidence_style_mermaid(&n.confidence))
            .unwrap_or("");
        // Sanitize id for mermaid (replace hyphens with underscores for node IDs,
        // keep original as label)
        let safe_id = id.replace('-', "_");
        println!("    {safe_id}[\"{id}\"]{style}");
    }
    println!();

    // Edges
    for edge_ref in graph.edge_references() {
        let source = graph[edge_ref.source()].replace('-', "_");
        let target = graph[edge_ref.target()].replace('-', "_");
        let label = edge_ref.weight();
        println!("    {source} -->|{label}| {target}");
    }
}

fn print_dot(graph: &DiGraph<String, String>, nodes: &[Node]) {
    let id_to_node: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    println!("digraph kos {{");
    println!("    rankdir=TD;");
    println!("    node [shape=box, style=filled, fontname=\"Helvetica\"];");
    println!();

    // Nodes with colors
    for idx in graph.node_indices() {
        let id = &graph[idx];
        let color = id_to_node
            .get(id.as_str())
            .map(|n| match n.confidence {
                Confidence::Bedrock => "#4a9",
                Confidence::Frontier => "#da5",
                Confidence::Placeholder => "#aaa",
                Confidence::Graveyard => "#a55",
            })
            .unwrap_or("#fff");
        println!("    \"{id}\" [fillcolor=\"{color}\", fontcolor=\"white\"];");
    }
    println!();

    // Edges
    for edge_ref in graph.edge_references() {
        let source = &graph[edge_ref.source()];
        let target = &graph[edge_ref.target()];
        let label = edge_ref.weight();
        println!("    \"{source}\" -> \"{target}\" [label=\"{label}\"];");
    }

    println!("}}");
}
