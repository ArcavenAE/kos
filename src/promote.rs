#![forbid(unsafe_code)]

//! `kos promote <node-id> <tier>`: move a node up a confidence tier.
//!
//! Promotion is the step where a graph commits to a claim, so the move is
//! guarded. The command finds the node, refuses any move that is not upward
//! (placeholder, then frontier, then bedrock), rewrites only the node's
//! top-level `confidence:` line, runs the `kos validate` node checks on the
//! node as it would sit at its new path before anything is written, and prints
//! the commit line from the kos commit conventions. A refused move changes no
//! file. Dissent and evidence prompts are not part of this command yet.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{KosError, Result};
use crate::model::Node;
use crate::validate::validate_node;
use crate::workspace::{KOS_DIR, MANIFEST_FILE, Workspace};

/// Every tier directory a node can sit in.
const TIERS: [&str; 4] = ["placeholder", "frontier", "bedrock", "graveyard"];

/// The kos action-commit separator (an em dash, written as an escape so the
/// source stays free of the character).
const SEP: &str = "\u{2014}";

fn refuse<T>(message: String) -> Result<T> {
    Err(KosError::Promote { message })
}

/// Rank of a tier an upward move may start from or end at. The graveyard has
/// none: it is never the start or end of a promotion.
fn rank(tier: &str) -> Option<u8> {
    match tier {
        "placeholder" => Some(0),
        "frontier" => Some(1),
        "bedrock" => Some(2),
        _ => None,
    }
}

/// The graph root `kos promote` works in: the same resolution the other
/// process commands use, so `--dir` means the same thing everywhere.
fn graph_root(workspace: &Workspace, cwd: &Path) -> PathBuf {
    if let Some(graph) = workspace.nearest_graph(cwd) {
        return graph.path.clone();
    }
    let local = cwd.join(KOS_DIR);
    if local.join(MANIFEST_FILE).exists() {
        return local;
    }
    workspace.node_root()
}

/// Replace the value of the first top-level `confidence:` line, keeping the
/// rest of the file, line endings and any trailing comment byte for byte. A
/// line inside a block scalar is indented, so it never matches.
fn rewrite_confidence(content: &str, to: &str) -> Option<String> {
    let mut out = String::with_capacity(content.len());
    let mut done = false;
    for line in content.split_inclusive('\n') {
        if !done && line.starts_with("confidence:") {
            let (body, eol) = match line.strip_suffix("\r\n") {
                Some(b) => (b, "\r\n"),
                None => match line.strip_suffix('\n') {
                    Some(b) => (b, "\n"),
                    None => (line, ""),
                },
            };
            let comment = body.find(" #").map_or("", |i| &body[i..]);
            out.push_str("confidence: ");
            out.push_str(to);
            out.push_str(comment);
            out.push_str(eol);
            done = true;
        } else {
            out.push_str(line);
        }
    }
    done.then_some(out)
}

/// Every node id in the graph, for the edge-target check. A file that does not
/// parse is skipped; `kos validate` reports those.
fn known_ids(nodes_dir: &Path) -> HashSet<String> {
    walkdir::WalkDir::new(nodes_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|x| x.to_str()) == Some("yaml")
        })
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|text| crate::yaml::from_str::<Node>(&text).ok())
        .map(|n| n.id)
        .collect()
}

/// Move `id` to the tier named `to`, printing what happened.
pub fn promote(workspace: &Workspace, cwd: &Path, id: &str, to: &str) -> Result<()> {
    if !TIERS.contains(&to) {
        return refuse(format!(
            "unknown tier '{to}' (expected one of: placeholder, frontier, bedrock)"
        ));
    }
    let nodes_dir = graph_root(workspace, cwd).join("nodes");

    let found: Vec<(&str, PathBuf)> = TIERS
        .iter()
        .map(|t| (*t, nodes_dir.join(t).join(format!("{id}.yaml"))))
        .filter(|(_, p)| p.is_file())
        .collect();
    let (from, source) = match found.as_slice() {
        [] => {
            return refuse(format!(
                "no node '{id}' in {} (looked in every tier)",
                nodes_dir.display()
            ));
        }
        [one] => (one.0, one.1.clone()),
        many => {
            let tiers: Vec<&str> = many.iter().map(|(t, _)| *t).collect();
            return refuse(format!(
                "'{id}' is claimed by more than one file (tiers: {}); fix that before moving it",
                tiers.join(", ")
            ));
        }
    };

    // Upward moves only; the tier-scope question for anything wider is open
    // (kos#147), so every other request is refused and writes nothing.
    if from == "graveyard" {
        return refuse(format!(
            "'{id}' is in graveyard; kos promote does not move a node out of the graveyard"
        ));
    }
    if to == "graveyard" {
        return refuse(format!(
            "kos promote does not move '{id}' to graveyard; that is a demotion, not a promotion"
        ));
    }
    let (Some(from_rank), Some(to_rank)) = (rank(from), rank(to)) else {
        return refuse(format!("cannot rank {from} and {to}"));
    };
    if from_rank == to_rank {
        return refuse(format!("'{id}' is already in {to}"));
    }
    if to_rank < from_rank {
        return refuse(format!(
            "moving '{id}' from {from} to {to} is a demotion; kos promote only moves a node up"
        ));
    }

    let target = nodes_dir.join(to).join(format!("{id}.yaml"));
    let content = std::fs::read_to_string(&source)?;
    let Some(moved) = rewrite_confidence(&content, to) else {
        return refuse(format!(
            "{} has no top-level 'confidence:' line to rewrite",
            source.display()
        ));
    };

    // The same node checks `kos validate` runs, on the node as it would sit
    // at its new path. Errors refuse the move; warnings are shown and the
    // move goes ahead.
    let node = crate::yaml::from_str::<Node>(&moved).map_err(|e| KosError::Promote {
        message: format!("{} would not parse after the move: {e}", source.display()),
    })?;
    let result = validate_node(&node, &format!("{to}/{id}.yaml"), &known_ids(&nodes_dir));
    for w in &result.warnings {
        eprintln!("warning: {id}: {w}");
    }
    if !result.errors.is_empty() {
        return refuse(format!(
            "'{id}' fails validation at {to}, so nothing was moved:\n  {}",
            result.errors.join("\n  ")
        ));
    }

    std::fs::write(&target, &moved)?;
    if let Err(e) = std::fs::remove_file(&source) {
        // Leave one file, not two: undo the copy before reporting.
        let _ = std::fs::remove_file(&target);
        return Err(e.into());
    }

    println!("{id}: {from} -> {to}");
    println!("  {} -> {}", source.display(), target.display());
    println!();
    println!("commit line (add the evidence to the body):");
    println!("  promote: {id} {SEP} move from {from} to {to}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_top_level_line_changes() {
        let text = "id: a\nconfidence: frontier\nbody: |\n  confidence: frontier\nconfidence: x\n";
        assert_eq!(
            rewrite_confidence(text, "bedrock").unwrap(),
            "id: a\nconfidence: bedrock\nbody: |\n  confidence: frontier\nconfidence: x\n"
        );
    }

    #[test]
    fn a_trailing_comment_and_crlf_are_kept() {
        assert_eq!(
            rewrite_confidence("confidence: frontier # why\r\nid: a\r\n", "bedrock").unwrap(),
            "confidence: bedrock # why\r\nid: a\r\n"
        );
    }

    #[test]
    fn a_last_line_with_no_newline_still_changes() {
        assert_eq!(
            rewrite_confidence("id: a\nconfidence: frontier", "bedrock").unwrap(),
            "id: a\nconfidence: bedrock"
        );
    }

    #[test]
    fn no_top_level_line_is_none() {
        assert!(rewrite_confidence("id: a\n  confidence: frontier\n", "bedrock").is_none());
    }

    #[test]
    fn graveyard_has_no_rank() {
        assert_eq!(rank("graveyard"), None);
        assert!(rank("placeholder") < rank("frontier") && rank("frontier") < rank("bedrock"));
    }
}
