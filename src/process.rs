#![forbid(unsafe_code)]

//! Process subcommands: idea, question, finding, probe.
//! These scaffold the kos process cycle artifacts in the right
//! directories with the right schema, lowering the ceremony floor.

use std::path::Path;

use crate::error::{KosError, Result};
use crate::workspace::{KOS_DIR, MANIFEST_FILE, Workspace};

/// Resolve the graph root for process commands.
///
/// Process commands CREATE content, so they target the nearest _kos/
/// relative to cwd — not the workspace's kos_root (which is for the
/// kos subrepo's legacy layout).
///
/// Resolution order:
/// 1. nearest_graph() from cwd (handles orchestrator vs subrepo)
/// 2. cwd/_kos/ if it has a manifest (standalone)
/// 3. Fall back to workspace.node_root() (kos repo legacy layout)
fn resolve_graph_root(workspace: &Workspace, cwd: &Path) -> std::path::PathBuf {
    // Try nearest_graph first — handles orchestrator and subrepo graphs
    if let Some(graph) = workspace.nearest_graph(cwd) {
        return graph.path.clone();
    }

    // Check for _kos/ in cwd directly (standalone not yet in graphs list)
    let local_kos = cwd.join(KOS_DIR);
    if local_kos.join(MANIFEST_FILE).exists() {
        return local_kos;
    }

    // Fall back to workspace node_root (kos repo legacy layout)
    workspace.node_root()
}

/// Create an idea file in _kos/ideas/{slug}.md.
///
/// Ideas are pre-hypothesis brainstorming — generative, possibly
/// contradictory, no commitment. Lowest ceremony level.
pub fn idea(workspace: &Workspace, cwd: &Path, slug: &str, title: Option<&str>) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let ideas_dir = graph_root.join("ideas");
    ensure_dir(&ideas_dir)?;

    let filename = format!("{slug}.md");
    let path = ideas_dir.join(&filename);

    if path.exists() {
        return Err(KosError::Init {
            message: format!("idea already exists: {}", path.display()),
        });
    }

    let display_title = title.unwrap_or(slug);
    let content = format!(
        "# {display_title}\n\
         \n\
         *Idea — pre-hypothesis brainstorming. Generative, uncommitted.*\n\
         \n\
         ## The observation\n\
         \n\
         \n\
         \n\
         ## Tensions\n\
         \n\
         \n"
    );

    std::fs::write(&path, content).map_err(KosError::Io)?;

    let rel = relative_display(&path, &workspace.root);
    println!("Created idea: {rel}");
    println!("  Edit the file to capture your thinking.");
    println!("  When it crystallizes, extract into: kos question <slug>");

    Ok(())
}

/// Create a question node in _kos/nodes/frontier/question-{slug}.yaml.
///
/// Questions are the unit of work in kos — an open unknown that
/// drives a probe. Schema-valid from creation.
pub fn question(workspace: &Workspace, cwd: &Path, slug: &str, title: &str) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let frontier_dir = graph_root.join("nodes").join("frontier");
    ensure_dir(&frontier_dir)?;

    let node_id = if slug.starts_with("question-") {
        slug.to_string()
    } else {
        format!("question-{slug}")
    };
    let filename = format!("{node_id}.yaml");
    let path = frontier_dir.join(&filename);

    if path.exists() {
        return Err(KosError::Init {
            message: format!("question node already exists: {}", path.display()),
        });
    }

    let today = today_iso();
    let content = format!(
        "\
id: {node_id}
type: question
confidence: frontier
title: \"{title}\"
content: |

edges: []
provenance:
  created_by: human
  session: null
  created_at: \"{today}\"
"
    );

    std::fs::write(&path, content).map_err(KosError::Io)?;

    let rel = relative_display(&path, &workspace.root);
    println!("Created question: {rel}");
    println!("  id: {node_id}");
    println!("  Edit the content field, add edges to what this derives from.");
    println!("  When ready to investigate: kos probe {slug}");

    Ok(())
}

/// Create a finding in _kos/findings/finding-{NNN}-{slug}.yaml.
///
/// Findings are probe results — evidence, not opinion. Auto-numbered
/// by scanning existing findings.
pub fn finding(workspace: &Workspace, cwd: &Path, slug: &str, title: &str) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let findings_dir = graph_root.join("findings");
    ensure_dir(&findings_dir)?;

    let next_num = next_finding_number(&findings_dir)?;
    let finding_id = format!("finding-{next_num:03}-{slug}");
    let filename = format!("{finding_id}.yaml");
    let path = findings_dir.join(&filename);

    let today = today_iso();
    let content = format!(
        "\
id: {finding_id}
type: finding
confidence: frontier
title: \"{title}\"
content: |
  ## Summary

  ## Findings

  ## Assessment

edges: []
provenance:
  created_by: human
  session: null
  created_at: \"{today}\"
finding:
  probe: null
  result: partial
  surprise_magnitude: null
"
    );

    std::fs::write(&path, content).map_err(KosError::Io)?;

    let rel = relative_display(&path, &workspace.root);
    println!("Created finding: {rel}");
    println!("  id: {finding_id}");
    println!("  Fill in the content, link to the probe brief, set result.");
    println!("  Then harvest: update affected nodes, charter if bedrock changed.");

    Ok(())
}

/// Create an exploration brief in _kos/probes/brief-{slug}.yaml.
///
/// Briefs are the plan for a probe — one question, one timebox,
/// explicit success signals. Includes predicted_confidence for
/// the predictor calibration instrument.
pub fn probe(workspace: &Workspace, cwd: &Path, slug: &str, title: &str) -> Result<()> {
    let graph_root = resolve_graph_root(workspace, cwd);
    let probes_dir = graph_root.join("probes");
    ensure_dir(&probes_dir)?;

    let brief_id = if slug.starts_with("brief-") {
        slug.to_string()
    } else {
        format!("brief-{slug}")
    };
    let filename = format!("{brief_id}.yaml");
    let path = probes_dir.join(&filename);

    if path.exists() {
        return Err(KosError::Init {
            message: format!("probe brief already exists: {}", path.display()),
        });
    }

    let today = today_iso();
    let content = format!(
        "\
id: {brief_id}
type: brief
confidence: frontier
title: \"{title}\"
content: |
  ## Question

  ## Approach

  ## Excluded scope

edges: []
provenance:
  created_by: human
  session: null
  created_at: \"{today}\"
brief:
  hypothesis: \"\"
  excluded_scope: \"\"
  success_signal: \"\"
  timebox: \"\"
  predicted_confidence: null
"
    );

    std::fs::write(&path, content).map_err(KosError::Io)?;

    let rel = relative_display(&path, &workspace.root);
    println!("Created probe brief: {rel}");
    println!("  id: {brief_id}");
    println!("  Fill in hypothesis, success_signal, timebox.");
    println!("  Set predicted_confidence (0.0-1.0) before running the probe.");
    println!("  When done: kos finding {slug} \"<title>\"");

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────

fn ensure_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(KosError::Io)?;
    }
    Ok(())
}

fn today_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Convert to date: this is a simple conversion, not timezone-aware
    let days = secs / 86400;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn next_finding_number(findings_dir: &Path) -> Result<u32> {
    if !findings_dir.exists() {
        return Ok(1);
    }

    let mut max = 0u32;
    for entry in std::fs::read_dir(findings_dir).map_err(KosError::Io)? {
        let entry = entry.map_err(KosError::Io)?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Match finding-NNN-*.yaml
        if let Some(rest) = name.strip_prefix("finding-") {
            if let Some(num_str) = rest.split('-').next() {
                if let Ok(n) = num_str.parse::<u32>() {
                    max = max.max(n);
                }
            }
        }
    }

    Ok(max + 1)
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_to_date_epoch() {
        assert_eq!(days_to_date(0), (1970, 1, 1));
    }

    #[test]
    fn days_to_date_known_date() {
        // 2026-04-05 = day 20548 since epoch
        assert_eq!(days_to_date(20548), (2026, 4, 5));
    }

    #[test]
    fn next_finding_number_empty_dir() {
        let dir = std::env::temp_dir().join("kos-test-empty-findings");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(next_finding_number(&dir).unwrap(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn next_finding_number_with_existing() {
        let dir = std::env::temp_dir().join("kos-test-findings");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("finding-001-foo.yaml"), "").unwrap();
        std::fs::write(dir.join("finding-005-bar.yaml"), "").unwrap();
        std::fs::write(dir.join("finding-003-baz.yaml"), "").unwrap();
        assert_eq!(next_finding_number(&dir).unwrap(), 6);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn next_finding_number_nonexistent_dir() {
        let dir = std::env::temp_dir().join("kos-test-no-such-findings");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(next_finding_number(&dir).unwrap(), 1);
    }
}
